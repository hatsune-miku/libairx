use std::fmt;
use std::net::SocketAddr;

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Peer {
    ttl: i8,
    host: String,
    port: u16,
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Peer {}:{}]", self.host, self.port)
    }
}

impl Peer {
    pub fn from(ttl: i8, socket_addr: &SocketAddr) -> Self {
        Self {
            ttl,
            host: socket_addr.ip().to_string().into(),
            port: socket_addr.port(),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.ttl <= 0
    }

    pub fn increment_ttl(&mut self) {
        self.ttl += 1;
    }

    pub fn decrement_ttl(&mut self) {
        self.ttl -= 1;
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
