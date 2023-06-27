use std::fmt;
use std::hash::Hash;
use std::net::{Ipv4Addr};

#[derive(Eq, Clone)]
#[allow(dead_code)]
pub struct Peer {
    host: String,
    port: u16,
    host_name: Option<String>,
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

#[allow(dead_code)]
impl Peer {
    pub fn from(socket_addr: &Ipv4Addr, port: u16, host_name: Option<&String>) -> Self {
        Self {
            host: socket_addr.to_string(),
            port,
            host_name: match host_name {
                Some(name) => Some(name.clone()),
                None => None,
            },
        }
    }

    pub fn new(host: &String, port: u16, host_name: Option<&String>) -> Self {
        Self {
            host: host.clone(),
            port,
            host_name: match host_name {
                Some(name) => Some(name.clone()),
                None => None,
            },
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host_name(&self) -> Option<&String> {
        match &self.host_name {
            Some(name) => Some(name),
            None => None,
        }
    }
}
