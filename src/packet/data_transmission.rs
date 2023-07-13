use std::{io, usize};
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use log::warn;
use crate::compatibility::unified_endian::UnifiedEndian;

const PACKET_TRY_TIMES: u64 = 10;
const TCP_ACCEPT_TRY_WAIT_MILLISECONDS: u64 = 100;

pub struct DataTransmit {
    stream: TcpStream,
}

impl DataTransmit {
    pub fn from(stream: TcpStream) -> Self {
        Self { stream }
    }
    pub fn close(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}

const SIZE_SIZE: usize = size_of::<u32>();

impl DataTransmit {
    pub fn send_data_progress_with_retry<F>(&mut self, data: &Vec<u8>, mut on_progress: F) -> Result<(), io::Error> where F: FnMut(u64) {
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
                    if e.kind() != io::ErrorKind::WouldBlock {
                        error = e;
                        remaining_tries -= 1;
                        warn!("Failed to send data ({}), remaining tries: {}.", error, remaining_tries);
                    }
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

    /// Read size N and read N bytes of data with retry and progress reporting.
    pub fn read_data_progress_with_retry<F>(&mut self, on_progress: F) -> Result<Vec<u8>, io::Error> where F: Fn(f32) {
        let mut remaining_tries = PACKET_TRY_TIMES;
        let mut last_error: io::Error = io::Error::new(io::ErrorKind::Other, "Failed to read data.");
        let mut size_buf: [u8; SIZE_SIZE] = [0u8; SIZE_SIZE];
        let mut packet_size = 0;
        let mut bytes_read_total = 0;

        // Read size.
        while remaining_tries > 0 {
            packet_size = match self.stream.read_exact(&mut size_buf) {
                Ok(_) => u32::from_bytes(size_buf),
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        last_error = e;
                        remaining_tries -= 1;
                    }
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            };
            break;
        }

        // Failed to read size?
        if packet_size == 0 {
            return Err(last_error);
        }

        // Allocate buffer.
        remaining_tries = PACKET_TRY_TIMES;
        let mut data_buf = vec![0u8; packet_size as usize];

        // Read data.
        while remaining_tries > 0 {
            let bytes_read = match self.stream.read(&mut data_buf[bytes_read_total..]) {
                Ok(n) => n,
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        last_error = e;
                        remaining_tries -= 1;
                        warn!("Failed to read payload data ({}), remaining tries: {}.", last_error, remaining_tries);
                    }
                    sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                    continue;
                }
            };

            bytes_read_total += bytes_read;

            // Report progress.
            on_progress(bytes_read_total as f32 / packet_size as f32);

            // Read everything?
            if bytes_read_total >= packet_size as usize {
                return Ok(data_buf);
            }
        }

        Err(last_error)
    }
}
