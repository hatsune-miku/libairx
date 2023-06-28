use std::{io, usize};
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use log::warn;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::data;

const PACKET_TRY_TIMES: u64 = 3;
const TCP_ACCEPT_TRY_WAIT_MILLISECONDS: u64 = 10;

pub struct DataTransmission {
    stream: TcpStream,
}

impl DataTransmission {
    pub fn from(stream: TcpStream) -> Self {
        Self { stream }
    }
    pub fn close(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}

const SIZE_SIZE: usize = size_of::<u32>();

impl data::SendDataWithRetry for DataTransmission {
    // Try to send data for TCP_ACCEPT_TRY_TIMES times.
    fn send_data_with_retry(&mut self, data: &Vec<u8>) -> Result<(), io::Error> {
        // Strings are already utf8 encoded.
        let data_len = data.len() as u32;
        let mut buf = vec![0u8; SIZE_SIZE + data_len as usize];

        buf[0..SIZE_SIZE].copy_from_slice(&data_len.to_bytes());
        buf[SIZE_SIZE..].copy_from_slice(data);

        let mut remaining_tries = PACKET_TRY_TIMES;
        let mut error: io::Error = io::Error::new(io::ErrorKind::Other, "Failed to send data.");

        while remaining_tries > 0 {
            match self.stream.write_all(&buf) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error = e;
                    remaining_tries -= 1;
                    warn!("Failed to send data ({}), remaining tries: {}.", error, remaining_tries);
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            }
        }
        Err(error)
    }
}

impl data::ReadDataWithRetry for DataTransmission {
    // Try to read data for TCP_ACCEPT_TRY_TIMES times.
    fn read_data_with_retry(&mut self) -> Result<Vec<u8>, io::Error> {
        let mut size_buf: [u8; SIZE_SIZE] = [0; SIZE_SIZE];
        let mut tries = PACKET_TRY_TIMES;
        let mut error: io::Error = io::Error::new(io::ErrorKind::Other, "Failed to read data.");

        while tries > 0 {
            match {
                if let Err(e) = self.stream.read_exact(&mut size_buf) {
                    return Err(e);
                }
                let size = u32::from_bytes(size_buf);
                let mut buf = vec![0u8; size as usize];

                match self.stream.read_exact(&mut buf) {
                    Ok(_) => Ok(buf),
                    Err(e) => Err(e),
                }
            } {
                Ok(buf) => {
                    let _ = self.stream.flush();
                    return Ok(buf);
                },
                Err(e) => {
                    error = e;
                    tries -= 1;
                    warn!("Failed to read data ({}), remaining tries: {}.", error, tries);
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            }
        }

        Err(error)
    }
}
