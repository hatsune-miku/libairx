use std::net::{TcpListener, TcpStream};

pub struct TcpServer {
    listener: Option<TcpListener>
}

impl TcpServer {
    pub fn new() -> Self {
        Self { listener: None }
    }

    pub fn listen(&mut self, host: &str, port: u16) -> Result<(), String> {
        self.listener = match TcpListener::bind(
            format!("{}:{}", host, port)) {
            Ok(l) => Some(l),
            Err(e) => return Err(e.to_string())
        };
        Ok(())
    }

    pub fn accept(&mut self) -> Result<TcpStream, String> {
        let listener: &TcpListener = &self.listener.as_ref().unwrap();
        match listener.accept() {
            Ok((stream, _)) => Ok(stream),
            Err(e) => Err(e.to_string())
        }
    }

}

