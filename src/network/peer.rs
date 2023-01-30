use std::fmt;
use std::hash::Hash;
use std::net::SocketAddr;

#[derive(Eq, Clone)]
pub struct Peer {
    host: String,
    port: u16,
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.host)
    }
}

impl Hash for Peer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.host.hash(state);
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
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

    pub fn new(host: &String, port: u16) -> Self {
        Self {
            host: host.clone(),
            port,
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
