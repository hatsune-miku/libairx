use std::hash::Hash;
use std::net::{Ipv4Addr};
use std::string::ToString;

const DEFAULT_HOSTNAME: &str = "<empty>";

#[derive(Eq, Clone)]
pub struct Peer {
    host: String,
    port: u16,
    host_name: String,
}

impl Default for Peer {
    fn default() -> Self {
        Self {
            host: String::from("0.0.0.0"),
            port: 0,
            host_name: DEFAULT_HOSTNAME.to_string(),
        }
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

impl ToString for Peer {
    fn to_string(&self) -> String {
        format!(
            "{}@{}:{}",
            &self.host_name,
            self.host,
            self.port,
        )
    }
}

impl Peer {
    pub fn from(socket_addr: &Ipv4Addr, port: u16, host_name: Option<&String>) -> Self {
        Self {
            host: socket_addr.to_string(),
            port,
            host_name: match host_name {
                Some(name) => name.clone(),
                None => DEFAULT_HOSTNAME.to_string(),
            },
        }
    }

    pub fn new(host: &String, port: u16, host_name: Option<&String>) -> Self {
        Self {
            host: host.clone(),
            port,
            host_name: match host_name {
                Some(name) => name.clone(),
                None => DEFAULT_HOSTNAME.to_string(),
            },
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host_name(&self) -> &String {
        &self.host_name
    }
}
