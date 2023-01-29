use crate::network::peer::Peer;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use crate::util::shared_mutable::SharedMutable;

const BUF_SIZE: usize = 512;
const THREAD_MAX: usize = 8;
const DISCOVERY_RATE: u64 = 1;
const DISCOVERY_DROP_RATE: u64 = 2;
const HANDSHAKE_MESSAGE: &'static str = "Hi There! 👋 \\^O^/";
const PEER_INITIAL_TTL: i8 = 3;

pub type PeerSetType = SharedMutable<Vec<Peer>>;


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
        .collect::<Vec<Ipv4Addr>>())
}

fn server_routine(server_socket: &UdpSocket, peers: PeerSetType, should_reconnect: SharedMutable<bool>) -> Result<(), io::Error> {
    let mut buf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

    loop {
        if let Ok(locked) = should_reconnect.lock_and_get() {
            if *locked {
                return Ok(());
            }
        }

        let (size, peer_addr) = match server_socket.recv_from(&mut buf) {
            Ok((x, y)) => (x, y),
            Err(e) => {
                return Err(e);
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

        if let Ok(mut locked) = peers.lock_and_get() {
            match locked.iter_mut().find(|peer| {
                peer.host() == &peer_addr.ip().to_string() && peer.port() == peer_addr.port()
            }) {
                Some(peer) => {
                    // Already in list.
                    peer.increment_ttl();
                }
                None => {
                    // Add to list.
                    locked.push(Peer::from(PEER_INITIAL_TTL, &peer_addr));
                }
            }
        }
    }
}

fn peer_ttl_decrement(peers: PeerSetType, should_reconnect: SharedMutable<bool>) -> Result<(), io::Error> {
    loop {
        if let Ok(locked) = should_reconnect.lock_and_get() {
            if *locked {
                return Ok(());
            }
        }

        if let Ok(mut locked) = peers.lock_and_get() {
            for peer in locked.iter_mut() {
                peer.decrement_ttl();
            }
            locked.retain(|peer| peer.is_alive());
        }
        sleep(Duration::from_secs(DISCOVERY_DROP_RATE));
    }
}

fn client_routine(client_socket: &UdpSocket, server_port: u16, should_reconnect: SharedMutable<bool>) -> Result<(), io::Error> {
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
    loop {
        if let Ok(locked) = should_reconnect.lock_and_get() {
            if *locked {
                return Ok(());
            }
        }

        for addr in broadcast_addresses.iter() {
            let broadcast_addr = SocketAddr::new(IpAddr::from(addr.octets()), server_port);
            if let Err(e) = client_socket.send_to(handshake_string_bytes, broadcast_addr) {
                return Err(e);
            }
        }

        sleep(Duration::from_secs(DISCOVERY_RATE));
    }
}

pub struct DiscoveryService {
    server_socket: UdpSocket,
    client_socket: UdpSocket,
    server_port: u16,
    client_port: u16,
    thread_pool: threadpool::ThreadPool,
    is_started: bool,
    peer_list: PeerSetType,
    should_reconnect: SharedMutable<bool>,
}

impl DiscoveryService {
    pub fn new(server_port: u16, client_port: u16) -> Result<Self, io::Error> {
        let server_socket = Self::create_broadcast_socket(server_port)?;
        let client_socket = Self::create_broadcast_socket(client_port)?;

        let thread_pool = threadpool::Builder::new()
            .thread_name(String::from("DiscoveryDispatch"))
            .num_threads(THREAD_MAX)
            .build();

        Ok(Self {
            server_socket,
            client_socket,
            server_port,
            client_port,
            thread_pool,
            peer_list: SharedMutable::new(Vec::new()),
            is_started: false,
            should_reconnect: SharedMutable::new(false),
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

    pub fn reconnect(&mut self, server_port: u16, client_port: u16) -> Result<(), io::Error> {
        self.server_socket = Self::create_broadcast_socket(server_port)?;
        self.client_socket = Self::create_broadcast_socket(client_port)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        if self.is_started {
            return Err(io::Error::new(io::ErrorKind::Other, "Already started"));
        }
        self.is_started = true;

        loop {
            // Clone sockets.
            let server_socket = self.server_socket.try_clone()?;
            let client_socket = self.client_socket.try_clone()?;

            let server_port = self.server_port;

            let should_reconnect_a = self.should_reconnect.clone();
            let should_reconnect_b = self.should_reconnect.clone();
            let should_reconnect_c = self.should_reconnect.clone();
            let should_reconnect_d = self.should_reconnect.clone();

            let peer_list_a = self.peer_list.clone();
            let peer_list_b = self.peer_list.clone();

            self.thread_pool.execute(move || {
                let _ = server_routine(&server_socket, peer_list_a, should_reconnect_a.clone());
                if let Ok(mut locked) = should_reconnect_a.clone().lock_and_get() {
                    *locked = true;
                }
            });
            self.thread_pool.execute(move || {
                let _ = peer_ttl_decrement(peer_list_b, should_reconnect_b.clone());
                if let Ok(mut locked) = should_reconnect_b.clone().lock_and_get() {
                    *locked = true;
                }
            });
            self.thread_pool.execute(move || {
                let _ = client_routine(&client_socket, server_port, should_reconnect_c.clone());
                if let Ok(mut locked) = should_reconnect_c.clone().lock_and_get() {
                    *locked = true;
                }
            });

            self.thread_pool.join();

            if let Ok(mut locked) = should_reconnect_d.lock_and_get() {
                *locked = false;
            }
            self.reconnect(self.server_port, self.client_port)?;
        }

        Ok(())
    }

    pub fn get_peer_list(&self) -> Result<Vec<Peer>, io::Error> {
        if let Ok(locked) = self.peer_list.lock_and_get() {
            Ok(locked.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to lock peer list.",
            ))
        }
    }
}
