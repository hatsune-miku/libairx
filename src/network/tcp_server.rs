use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};

pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    pub fn create_and_listen(host: &str, port: u16) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr)?;
        Ok(Self { listener })
    }

    pub fn accept(&mut self) -> Result<(TcpStream, SocketAddr), io::Error> {
        self.listener.accept()
    }
}
