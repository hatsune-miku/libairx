use crate::network::peer::Peer;
use crate::service::ShouldInterruptFunctionType;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::collections::HashSet;
use std::io;
use std::io::ErrorKind::{TimedOut, WouldBlock};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{error, info};
use protobuf::Message;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::proto::discovery_packet::DiscoveryPacket;
use crate::util::os::OSUtil;
use crate::extension::ip_to_u32::ToU32;

const DISCOVERY_TIMEOUT_MILLIS: u64 = 1000;

pub type PeerCollectionType = Arc<Mutex<HashSet<Peer>>>;

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

        Ipv4Addr::from(ip_octets)
    }
}

fn scan_local_addresses() -> Result<HashSet<Ipv4Addr>, network_interface::Error> {
    Ok(NetworkInterface::show()?
        .iter()
        .filter(|i| i.addr.map_or(false, |a| a.ip().is_ipv4()))
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
    peer_set_ptr: PeerCollectionType,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            peer_set_ptr: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn create_broadcast_socket(port: u16) -> Result<UdpSocket, io::Error> {
        match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(s) => {
                s.set_read_timeout(Some(Duration::from_millis(DISCOVERY_TIMEOUT_MILLIS)))?;
                s.set_broadcast(true)?;
                Ok(s)
            }
            Err(e) => {
                error!("Failed to bind UDP socket: {}", e);
                Err(e)
            }
        }
    }

    pub fn peers(&self) -> PeerCollectionType {
        self.peer_set_ptr.clone()
    }

    pub fn peer_lookup(&self, socker_address: &SocketAddr) -> Option<Peer> {
        if let Ok(locked) = self.peer_set_ptr.lock() {
            for peer in locked.iter() {
                if *peer.host() == socker_address.ip().to_string() {
                    return Some(peer.clone());
                }
            }
        }
        None
    }

    // Suppress: `std::` can't be omitted but IDEA thinks it can.
    #[allow(unused_qualifications)]
    pub fn handle_new_peer(
        local_addresses: HashSet<Ipv4Addr>,
        server_socket: &UdpSocket,
        peers: PeerCollectionType,
        packet: DiscoveryPacket,
        group_identifier: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sender_address = packet.address();
        let sender_address_ipv4 = Ipv4Addr::from(sender_address);
        if local_addresses.contains(&sender_address_ipv4) {
            return Err("Received packet from self".into());
        }

        if packet.group_identifier() != group_identifier {
            // Group identity mismatch.
            info!("Dropped packet from different group (mine={}, theirs={}).",
                group_identifier, packet.group_identifier());
            return Err("Group identity mismatch".into());
        }

        info!("Received discovery packet from {} - {}", packet.host_name(), sender_address);

        if packet.need_response() {
            // Respond to our new friend on behalf of each local address.
            info!("Responding to discovery request from {}", sender_address);
            let self_hostname = OSUtil::hostname();
            for local_addr_ipv4 in local_addresses {
                let mut response_packet = DiscoveryPacket::new();
                response_packet.set_address(local_addr_ipv4.into());
                response_packet.set_server_port(packet.server_port());
                response_packet.set_group_identifier(group_identifier);
                response_packet.set_need_response(false);
                response_packet.set_host_name(self_hostname.clone());

                let serialized = match response_packet.write_to_bytes() {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Failed to serialize response packet: {}", e);
                        return Err(e.into());
                    }
                };

                let size = serialized.len() as u32;
                let _ = server_socket.send_to(
                    &size.to_bytes(),
                    SocketAddrV4::new(sender_address_ipv4, packet.server_port() as u16),
                );

                match server_socket.send_to(
                    serialized.as_slice(),
                    SocketAddrV4::new(sender_address_ipv4, packet.server_port() as u16),
                ) {
                    Ok(_) => {
                        info!("Successfully response packet to {}", sender_address);
                    }
                    Err(e) => {
                        error!("Failed to send response packet: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }

        info!("Adding peer {} to peer set.", sender_address);
        if let Ok(mut locked) = peers.lock() {
            locked.insert(Peer::from(
                &sender_address_ipv4,
                packet.server_port() as u16,
                Some(&packet.host_name().to_string())
            ));
            info!("Added peer {} to peer set.", sender_address);
        }

        Ok(())
    }

    pub fn broadcast_discovery_request(client_port: u16, server_port: u16, group_identifier: u32) -> Result<(), io::Error> {
        let client_socket = Self::create_broadcast_socket(client_port)?;
        let broadcast_addresses = match scan_broadcast_addresses() {
            Ok(x) => x,
            Err(e) => {
                error!("Failed to get broadcast addresses: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get broadcast addresses: {}", e),
                ));
            }
        };
        let local_addresses = match scan_local_addresses() {
            Ok(x) => x,
            Err(e) => {
                error!("Failed to get local addresses: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get local addresses: {}", e),
                ));
            }
        };

        let self_hostname = OSUtil::hostname();
        let mut broadcast_packet = DiscoveryPacket::new();
        broadcast_packet.set_server_port(server_port as u32);
        broadcast_packet.set_group_identifier(group_identifier);
        broadcast_packet.set_need_response(true);
        broadcast_packet.set_host_name(self_hostname.clone());

        for broadcast_addr_ipv4 in &broadcast_addresses {
            for local_addr_ipv4 in &local_addresses {
                broadcast_packet.set_address(local_addr_ipv4.clone().to_u32());

                let broadcast_packet_bytes = match broadcast_packet.write_to_bytes() {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Failed to serialize broadcast packet: {}", e);
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to serialize broadcast packet: {}", e),
                        ));
                    }
                };

                let size = broadcast_packet_bytes.len() as u32;
                let result1 = client_socket.send_to(
                    &size.to_bytes(),
                    SocketAddrV4::new(*broadcast_addr_ipv4, server_port),
                );

                let result2 = client_socket.send_to(
                    broadcast_packet_bytes.as_slice(),
                    SocketAddrV4::new(*broadcast_addr_ipv4, server_port),
                );

                if result1.is_ok() && result2.is_ok() {
                    info!("Successfully broadcast discovery packet to {}", broadcast_addr_ipv4);
                } else {
                    error!("Failed to broadcast discovery packet to {}", broadcast_addr_ipv4);
                }
            }
        }
        Ok(())
    }

    pub fn run(
        client_port: u16,
        server_port: u16,
        peer_set_ptr: PeerCollectionType,
        should_interrupt: ShouldInterruptFunctionType,
        group_identifier: u32,
    ) -> Result<(), io::Error> {
        let server_socket = Self::create_broadcast_socket(server_port)?;
        let mut size_buffer = [0u8; 4];

        // Broadcast discovery request twice to ensure that we are discovered.
        for _ in 0..2 {
            let _ = Self::broadcast_discovery_request(client_port, server_port, group_identifier);
        }

        info!("Discovery service online and ready for connections.");

        loop {
            let packet_size = match server_socket.recv(&mut size_buffer) {
                Ok(_) => u32::from_bytes(size_buffer),
                Err(e) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                    if should_interrupt() {
                        info!("Discovery service interrupted by caller.");
                        break;
                    }
                    continue;
                }
                Err(e) => {
                    error!("Failed to receive packet size ({})", e);

                    // Broadcast another one to ensure that we are discovered.
                    let _ = Self::broadcast_discovery_request(client_port, server_port, group_identifier);
                    continue;
                }
            };

            let mut buf = vec![0u8; packet_size as usize];
            match server_socket.recv(&mut buf) {
                Ok(_) => {
                    if let Ok(local_addresses) = scan_local_addresses() {
                        let packet = match DiscoveryPacket::parse_from_bytes(buf.as_slice()) {
                            Ok(x) => x,
                            Err(e) => {
                                error!("Failed to parse discovery packet ({})", e);
                                continue;
                            }
                        };
                        let _ = Self::handle_new_peer(
                            local_addresses,
                            &server_socket,
                            peer_set_ptr.clone(),
                            packet,
                            group_identifier,
                        );
                    }
                }
                Err(ref e) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                    if should_interrupt() {
                        info!("Discovery service interrupted by caller.");
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to receive packet ({})", e);
                    continue;
                }
            }
        }

        Ok(())
    }
}
