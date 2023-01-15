use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Socket {
    stream: TcpStream,
}

impl From<TcpStream> for Socket {
    fn from(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl Socket {
    pub fn connect(host: &str, port: u16) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", host, port);
        Ok(
            Self {
                stream: TcpStream::connect(addr)?,
            }
        )
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.stream.write(data)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        self.stream.read_exact(buf)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.stream.read(buf)
    }

    pub fn close(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}
