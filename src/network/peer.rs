use std::fmt;
use std::net::SocketAddr;

#[derive(Eq, Hash, Clone)]
pub struct Peer {
    host: String,
    port: u16,
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", self.host, self.port)
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host && self.port == other.port
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Peer {
    pub fn from(socket_addr: &SocketAddr) -> Self {
        Self {
            host: socket_addr.ip().to_string().into(),
            port: socket_addr.port(),
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
