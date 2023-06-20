use std::io;
use std::net::{Incoming, TcpListener};

pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    /// Create a new TcpServer with non-blocking mode.
    pub fn create_and_listen(host: &String, port: u16) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }

    pub fn incoming(&self) -> Incoming<'_> {
        self.listener.incoming()
    }
}
