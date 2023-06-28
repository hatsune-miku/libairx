use std::{io, usize};
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use log::warn;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::data;

const PACKET_TRY_TIMES: u64 = 5;
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

impl DataTransmission {
    pub fn send_data_progress_with_retry<F>(&mut self, data: &Vec<u8>, on_progress: F) -> Result<(), io::Error> where F: Fn(u64) {
        // Strings are already utf8 encoded.
        let data_len = data.len() as u32;
        let mut buf = vec![0u8; SIZE_SIZE + data_len as usize];

        buf[0..SIZE_SIZE].copy_from_slice(&data_len.to_bytes());
        buf[SIZE_SIZE..].copy_from_slice(data);

        let mut remaining_tries = PACKET_TRY_TIMES;
        let mut bytes_written_total = 0;
        let mut error: io::Error = io::Error::new(io::ErrorKind::Other, "Failed to send data.");

        while remaining_tries > 0 {
            let bytes_written = match self.stream.write(&buf[bytes_written_total..]) {
                Ok(n) => n,
                Err(e) => {
                    error = e;
                    remaining_tries -= 1;
                    warn!("Failed to send data ({}), remaining tries: {}.", error, remaining_tries);
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            };
            bytes_written_total += bytes_written;
            on_progress(bytes_written_total as u64);
            if bytes_written_total >= buf.len() {
                return Ok(());
            }
        }
        Err(error)
    }

    pub fn read_data_progress_with_retry<F>(&mut self, on_progress: F) -> Result<Vec<u8>, io::Error> where F: Fn(f32) {
        let mut remaining_tries = PACKET_TRY_TIMES;
        let mut last_error: io::Error = io::Error::new(io::ErrorKind::Other, "Failed to read data.");
        let mut size_buf: [u8; SIZE_SIZE] = [0u8; SIZE_SIZE];
        let mut packet_size = 0;
        let mut bytes_read_total = 0;

        while remaining_tries > 0 {
            packet_size = match self.stream.read_exact(&mut size_buf) {
                Ok(_) => u32::from_bytes(size_buf),
                Err(e) => {
                    last_error = e;
                    remaining_tries -= 1;
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            };
            break;
        }

        remaining_tries = PACKET_TRY_TIMES;
        let mut data_buf = vec![0u8; packet_size as usize];

        while remaining_tries > 0 {
            let bytes_read = match self.stream.read(&mut data_buf[bytes_read_total..]) {
                Ok(n) => n,
                Err(e) => {
                    last_error = e;
                    remaining_tries -= 1;
                    warn!("Failed to read payload data ({}), remaining tries: {}.", last_error, remaining_tries);
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            };
            on_progress(bytes_read as f32 / packet_size as f32);
            bytes_read_total += bytes_read;
            if bytes_read_total >= packet_size as usize {
                return Ok(data_buf);
            }
        }

        Err(last_error)
    }
}

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
                Ok(buf) => return Ok(buf),
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
