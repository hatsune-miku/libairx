use std::collections::HashSet;
use crate::network::peer::Peer;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::util::shared_mutable::SharedMutable;

const BUF_SIZE: usize = 512;
const HANDSHAKE_MESSAGE: &'static str = "Hi There! 👋 \\^O^/";

pub type PeerSetType = SharedMutable<HashSet<Peer>>;


trait Broadcast {
    fn to_broadcast_addr(&self) -> Ipv4Addr;
}

trait ToIpV4Addr {
    fn to_ipv4_addr(&self) -> Option<Ipv4Addr>;
}

impl ToIpV4Addr for IpAddr {
    fn to_ipv4_addr(&self) -> Option<Ipv4Addr> {
        match self {
            IpAddr::V4(ip) => Some(*ip),
            IpAddr::V6(_) => None,
        }
    }
}

impl Broadcast for Addr {
    /// Fallback method that calculates
    /// broadcast address from IP address and netmask.
    fn to_broadcast_addr(&self) -> Ipv4Addr {
        let addr_fallback = Ipv4Addr::new(255, 255, 255, 255);
        let ipv4_addr = match self.ip() {
            IpAddr::V4(x) => x,
            _ => addr_fallback,
        };
        let mask_addr = match self.netmask() {
            Some(IpAddr::V4(x)) => x,
            _ => addr_fallback,
        };

        let mut ip_octets = ipv4_addr.octets();
        let mask_octets = mask_addr.octets();

        for i in 0..4 {
            ip_octets[i] |= !mask_octets[i];
        }

        Ipv4Addr::new(ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3])
    }
}

fn get_broadcast_addresses() -> Result<Vec<Ipv4Addr>, network_interface::Error> {
    let fallback = Ipv4Addr::new(255, 255, 255, 255);
    Ok(NetworkInterface::show()?
        .iter()
        .map(|i| match i.addr {
            Some(addr) => match addr.broadcast() {
                Some(b) => match b.to_ipv4_addr() {
                    Some(ip) => ip,
                    None => fallback,
                },
                None => addr.to_broadcast_addr(),
            },
            None => fallback,
        })
        .filter(|ip| !ip.is_loopback())
        .collect::<Vec<Ipv4Addr>>())
}

pub struct DiscoveryService {
    server_socket: UdpSocket,
    server_port: u16,
    client_port: u16,
    is_started: bool,
    peer_set: PeerSetType,
}

impl DiscoveryService {
    pub fn new(server_port: u16, client_port: u16) -> Result<Self, io::Error> {
        let server_socket = Self::create_broadcast_socket(server_port)?;

        Ok(Self {
            server_socket,
            server_port,
            client_port,
            peer_set: SharedMutable::new(HashSet::new()),
            is_started: false,
        })
    }

    pub fn create_broadcast_socket(port: u16) -> Result<UdpSocket, io::Error> {
        match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(s) => {
                s.set_broadcast(true)?;
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    pub fn reconnect(&mut self, server_port: u16) -> Result<(), io::Error> {
        self.server_socket = Self::create_broadcast_socket(server_port)?;
        Ok(())
    }


    pub fn run(&mut self) -> Result<(), io::Error> {
        if self.is_started {
            return Err(io::Error::new(io::ErrorKind::Other, "Already started"));
        }
        self.is_started = true;

        loop {
            // Clone sockets.
            let server_socket = &self.server_socket;

            let mut buf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];
            let (size, peer_addr) = match server_socket.recv_from(&mut buf) {
                Ok((x, y)) => (x, y),
                Err(_) => {
                    break;
                }
            };

            // From self?
            if peer_addr.ip() == server_socket.local_addr()?.ip() {
                continue;
            }

            // Not handshake message?
            let message = String::from_utf8(Vec::from(&buf[..size]));
            match message {
                Ok(x) => {
                    if &x != HANDSHAKE_MESSAGE {
                        continue;
                    }
                }
                Err(e) => {
                    // Client sent invalid message.
                    continue;
                }
            }

            if let Ok(mut locked) = self.peer_set.lock() {
                locked.insert(Peer::from(&peer_addr));
            }
        }

        Ok(())
    }

    pub fn broadcast_discovery_request(&self) -> Result<(), io::Error> {
        let client_socket = Self::create_broadcast_socket(self.client_port)?;
        let handshake_string_bytes = HANDSHAKE_MESSAGE.as_bytes();
        let broadcast_addresses = match get_broadcast_addresses() {
            Ok(x) => x,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get broadcast addresses: {}", e),
                ));
            }
        };

        //
        //     ==================================
        //     For current network interface only
        //     ==================================
        //
        //      fn get_local_ipv4_address(socket: &UdpSocket) -> Result<Ipv4Addr, io::Error> {
        //          let ip = socket.local_addr()?.ip();
        //          if let IpAddr::V4(ip) = ip {
        //              Ok(ip)
        //          } else {
        //              Err(io::Error::new(
        //                  io::ErrorKind::Other,
        //                  "Local address is not an IPv4 address.",
        //              ))
        //          }
        //      }
        //
        //     let local_ip = get_local_ipv4_address(client_socket)?;
        //     let broadcast_address = IpAddr::V4(
        //         Ipv4Addr::new(
        //             local_ip.octets()[0] | 0b11111111,
        //             local_ip.octets()[1] | 0b11111111,
        //             local_ip.octets()[2] | 0b11111111,
        //             local_ip.octets()[3] | 0b11111111,
        //         )
        //     );
        //     let target_address = SocketAddr::new(broadcast_address, server_port as u16);
        //

        if let Ok(mut locked) = self.peer_set.lock() {
            locked.clear();
        }
        for addr in broadcast_addresses.iter() {
            let broadcast_addr = SocketAddr::new(IpAddr::from(addr.octets()), self.server_port);
            if let Err(_) = client_socket.send_to(handshake_string_bytes, broadcast_addr) {
                continue;
            }
        }

        Ok(())
    }

    pub fn get_peer_list(&self) -> Result<HashSet<Peer>, io::Error> {
        if let Ok(locked) = self.peer_set.lock() {
            Ok(locked.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to lock peer list.",
            ))
        }
    }
}
