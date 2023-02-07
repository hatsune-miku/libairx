use crate::network::peer::Peer;
use crate::service::ShouldInterruptType;
use crate::util::shared_mutable::SharedMutable;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::collections::HashSet;
use std::io;
use std::io::ErrorKind::{TimedOut, WouldBlock};
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;
use crate::packet::discovery_packet;
use crate::packet::discovery_packet::DiscoveryPacket;
use crate::packet::protocol::serialize::Serialize;

const DISCOVERY_TIMEOUT_MILLIS: u64 = 1000;

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

fn scan_local_addresses() -> Result<HashSet<Ipv4Addr>, network_interface::Error> {
    Ok(NetworkInterface::show()?
        .iter()
        .filter(|i| i.addr.is_some() && i.addr.unwrap().ip().is_ipv4())
        .map(|i| i.addr.unwrap().ip().to_ipv4_addr().unwrap())
        .filter(|ip| ip.is_private())
        .collect::<HashSet<Ipv4Addr>>()
    )
}

fn scan_broadcast_addresses() -> Result<HashSet<Ipv4Addr>, network_interface::Error> {
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
        .collect::<HashSet<Ipv4Addr>>())
}

pub struct DiscoveryService {
    peer_set_ptr: PeerSetType,
}

impl DiscoveryService {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            peer_set_ptr: SharedMutable::new(HashSet::new()),
        })
    }

    pub fn create_broadcast_socket(port: u16) -> Result<UdpSocket, io::Error> {
        match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(s) => {
                s.set_read_timeout(Some(Duration::from_millis(DISCOVERY_TIMEOUT_MILLIS)))?;
                s.set_broadcast(true)?;
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    pub fn peers(&self) -> PeerSetType {
        self.peer_set_ptr.clone()
    }

    pub fn handle_new_peer(
        local_addresses: HashSet<Ipv4Addr>,
        server_socket: &UdpSocket,
        peer_set: PeerSetType,
        buf: [u8; discovery_packet::PACKET_SIZE],
        group_identity: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // From self?
        // Deserialize packet.
        // Not handshake message?
        let packet = DiscoveryPacket::deserialize(buf)?;
        let sender_address = packet.sender_address();
        if local_addresses.contains(&sender_address) {
            return Err("Received packet from self".into());
        }

        if packet.group_identity() != group_identity {
            // Group identity mismatch.
            return Err("Group identity mismatch".into());
        }

        if packet.need_response() {
            // Respond to our new friend on behalf of each local address.
            for local_addr_ipv4 in local_addresses {
                let response_packet = DiscoveryPacket::new(
                    local_addr_ipv4,
                    packet.server_port(),
                    group_identity,
                    false,
                );
                let _ = server_socket.send_to(
                    &response_packet.serialize(),
                    SocketAddrV4::new(packet.sender_address(), packet.server_port()),
                );
            }
        }

        if let Ok(mut locked) = peer_set.lock() {
            locked.insert(Peer::from(&sender_address, packet.server_port()));
        }

        Ok(())
    }

    pub fn broadcast_discovery_request(client_port: u16, server_port: u16, group_identity: u8) -> Result<(), io::Error> {
        let client_socket = Self::create_broadcast_socket(client_port)?;
        let broadcast_addresses = match scan_broadcast_addresses() {
            Ok(x) => x,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get broadcast addresses: {}", e),
                ));
            }
        };
        let local_addresses = match scan_local_addresses() {
            Ok(x) => x,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get local addresses: {}", e),
                ));
            }
        };

        for broadcast_addr_ipv4 in &broadcast_addresses {
            for local_addr_ipv4 in &local_addresses {
                let broadcast_packet = DiscoveryPacket::new(
                    local_addr_ipv4.clone(),
                    server_port,
                    group_identity,
                    true,
                );
                let broadcast_packet_bytes = broadcast_packet.serialize();
                let _ = client_socket.send_to(
                    &broadcast_packet_bytes,
                    SocketAddrV4::new(*broadcast_addr_ipv4, server_port),
                );
            }
        }

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

        Ok(())
    }

    pub fn run(
        client_port: u16,
        server_port: u16,
        peer_set_ptr: PeerSetType,
        should_interrupt: ShouldInterruptType,
        group_identity: u8,
    ) -> Result<(), io::Error> {
        let server_socket = Self::create_broadcast_socket(server_port)?;
        let mut buf: [u8; discovery_packet::PACKET_SIZE] = [0u8; discovery_packet::PACKET_SIZE];

        let _ = Self::broadcast_discovery_request(client_port, server_port, group_identity);

        loop {
            match server_socket.recv(&mut buf) {
                Ok(_) => {
                    if let Ok(local_addresses) = scan_local_addresses() {
                        let _ = Self::handle_new_peer(
                            local_addresses,
                            &server_socket,
                            peer_set_ptr.clone(),
                            buf,
                            group_identity,
                        );
                    }
                }
                Err(ref e) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                    // Check if interrupted.
                    // Calling should_interrupt() is
                    // expensive for some frontends like electron.
                    if should_interrupt() {
                        break;
                    }
                    continue;
                }
                Err(_) => {
                    break;
                }
            }
        }

        Ok(())
    }
}
