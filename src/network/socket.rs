use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};

pub struct Socket {
    stream: Option<TcpStream>,
}

impl From<TcpStream> for Socket {
    fn from(tcp_stream: TcpStream) -> Self {
        Self { stream: Some(tcp_stream) }
    }
}

impl Socket {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn connect(&mut self, host: &str, port: u16) -> Result<(), String> {
        let addr = format!("{}:{}", host, port);
        self.stream = match TcpStream::connect(addr) {
            Ok(s) => Some(s),
            Err(e) => return Err(e.to_string())
        };
        Ok(())
    }

    pub fn send(&self, data: &[u8]) -> Result<usize, String> {
        self.stream.as_ref().unwrap().write(data).map_err(|e| e.to_string())
            /*
        match &self.stream {
            Some(&mut s) => {
                s.write(data).map_err(|e| e.to_string())
            },
            None => Err(String::from("No live connection."))
        }

             */
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        match &mut self.stream {
            Some(s) => {
                match s.read_exact(buf) {
                    Ok(()) => Ok(buf.len()),
                    Err(e) => Err(e.to_string())
                }
            }
            None => Err(String::from("No live connection."))
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        match &mut self.stream {
            Some(s) => {
                match s.read(buf) {
                    Ok(size) => Ok(size),
                    Err(e) => Err(e.to_string())
                }
            }
            None => Err(String::from("No live connection."))
        }
    }

    pub fn close(&mut self) -> Result<(), String> {
        match &self.stream {
            Some(s) => {
                match s.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string())
                }
            },
            None => Err(String::from("No live connection."))
        }
    }
}
