use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::SocketAddr;
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
    pub fn connect(
        host: &str,
        port: u16,
        timeout: core::time::Duration,
    ) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", host, port);
        let socket_addr = match addr.parse::<SocketAddr>() {
            Ok(x) => x,
            Err(_) => return Err(io::Error::new(ErrorKind::Other, "Address parse failed")),
        };
        Ok(Self {
            stream: TcpStream::connect_timeout(&socket_addr, timeout)?,
        })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.stream.write(data)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        self.stream.read_exact(buf)
    }

    #[allow(dead_code)]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.stream.read(buf)
    }

    pub fn close(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}
