use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::net::TcpStream;

const TCP_STREAM_TRY_TIMES: u64 = 5;

pub struct Socket {
    stream: TcpStream,
}

impl From<TcpStream> for Socket {
    fn from(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[allow(dead_code)]
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

    pub fn send_with_retry(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        let mut size_sent_total = 0;
        let mut retry = 0;
        let data_len = data.len();
        while size_sent_total < data_len {
            let size_sent = match self.stream.write(&data[size_sent_total..]) {
                Ok(x) => x,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        if retry < TCP_STREAM_TRY_TIMES {
                            retry += 1;
                            continue;
                        } else {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                }
            };
            size_sent_total += size_sent;
        }
        Ok(size_sent_total)
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
