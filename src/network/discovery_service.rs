use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::collections::HashSet;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const BUF_SIZE: usize = 512;
const THREAD_MAX: usize = 8;
const HANDSHAKE_MESSAGE: &'static str = "Hi There! 👋 \\^O^/";

// Arc (Atomically Reference Counted) convinces the compiler
// that it's safe to share between threads.
// Mutex explicitly uses a lock to convince the compiler
// that it's safe to mutate.
// The peer list itself is a HashSet.
// TODO: maybe better solution?
type PeerSetType = Arc<Mutex<HashSet<SocketAddr>>>;

trait Broadcast {
    fn to_broadcast_addr(&self) -> Ipv4Addr;
}

impl Broadcast for Addr {
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

fn get_local_ipv4_address(socket: &UdpSocket) -> Result<Ipv4Addr, io::Error> {
    let ip = socket.local_addr()?.ip();
    if let IpAddr::V4(ip) = ip {
        Ok(ip)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Local address is not an IPv4 address.",
        ))
    }
}

fn get_broadcast_addresses() -> Result<Vec<Ipv4Addr>, network_interface::Error> {
    Ok(
        NetworkInterface::show()?
            .iter()
            .map(|i| {
                if let Some(addr) = i.addr {
                    addr.to_broadcast_addr()
                } else {
                    Ipv4Addr::new(255, 255, 255, 255)
                }
            })
            .collect::<Vec<Ipv4Addr>>()
    )
}

fn server_routine(server_socket: &UdpSocket, peers: PeerSetType) -> Result<(), io::Error> {
    let mut buf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

    loop {
        let (size, peer_addr) = match server_socket.recv_from(&mut buf) {
            Ok((x, y)) => (x, y),
            Err(e) => {
                println!("Error: {}", e);
                continue;
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
                println!("Error: {}", e);
                continue;
            }
        }

        if let Ok(mut locked) = peers.lock() {
            locked.insert(peer_addr.clone());
        }
    }
}

fn client_routine(client_socket: &UdpSocket, server_port: i16) -> Result<(), io::Error> {
    let handshake_string_bytes = HANDSHAKE_MESSAGE.as_bytes();
    let broadcast_addresses = match get_broadcast_addresses() {
        Ok(x) => x,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };

    ///
    ///     For current network interface only:
    ///
    ///     let local_ip = get_local_ipv4_address(client_socket)?;
    ///     let broadcast_address = IpAddr::V4(
    ///         Ipv4Addr::new(
    ///             local_ip.octets()[0] | 0b11111111,
    ///             local_ip.octets()[1] | 0b11111111,
    ///             local_ip.octets()[2] | 0b11111111,
    ///             local_ip.octets()[3] | 0b11111111,
    ///         )
    ///     );
    ///     let target_address = SocketAddr::new(broadcast_address, server_port as u16);
    ///
    loop {
        broadcast_addresses.iter().for_each(|addr| {
            let broadcast_addr = SocketAddr::new(
                IpAddr::from(addr.octets()), server_port as u16);
            let _ = client_socket.send_to(handshake_string_bytes, broadcast_addr);
        });

        sleep(Duration::from_secs(1));
    }
}

pub struct DiscoveryService {
    server_socket: UdpSocket,
    client_socket: UdpSocket,
    server_port: i16,
    thread_pool: threadpool::ThreadPool,
    is_started: bool,
    peer_list: PeerSetType,
}

impl DiscoveryService {
    pub fn new(server_port: i16, client_port: i16) -> Result<Self, io::Error> {
        let server_socket = UdpSocket::bind(format!("0.0.0.0:{}", server_port))?;
        let client_socket = UdpSocket::bind(format!("0.0.0.0:{}", client_port))?;

        client_socket.set_broadcast(true)?;
        server_socket.set_broadcast(true)?;

        let thread_pool = threadpool::Builder::new()
            .thread_name(String::from("DiscoveryDispatch"))
            .num_threads(THREAD_MAX)
            .build();

        Ok(
            Self {
                server_socket,
                client_socket,
                server_port,
                thread_pool,
                peer_list: Arc::new(Mutex::new(HashSet::new())),
                is_started: false,
            }
        )
    }

    pub fn start(&mut self) -> Result<(), io::Error> {
        if self.is_started {
            return Err(io::Error::new(io::ErrorKind::Other, "Already started"));
        }
        self.is_started = true;

        // Clone sockets.
        let server_socket = self.server_socket.try_clone()?;
        let client_socket = self.client_socket.try_clone()?;

        let server_port = self.server_port;
        let peer_list = self.peer_list.clone();

        self.thread_pool.execute(move || {
            let _ = server_routine(&server_socket, peer_list);
        });
        self.thread_pool.execute(move || {
            let _ = client_routine(&client_socket, server_port);
        });

        Ok(())
    }

    pub fn get_peer_list(&self) -> Result<HashSet<SocketAddr>, io::Error> {
        if let Ok(locked) = self.peer_list.lock() {
            Ok(locked.clone())
        } else {
            Err(
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to lock peer list.",
                )
            )
        }
    }
}
