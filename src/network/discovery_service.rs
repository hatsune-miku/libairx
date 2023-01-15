use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::{io, time};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::ops::Add;
use std::thread::sleep;
use std::time::Duration;
use chrono;
use interfaces::{Address, Interface, InterfaceFlags};

const BUF_SIZE: usize = 512;
const THREAD_MAX: usize = 8;
const HANDSHAKE_MESSAGE: &'static str = "Hi There! 👋 \\^O^/";

pub type ClientHandler = fn(&SocketAddr);

trait ToIpV4Addr {
    fn to_ipv4_addr(&self) -> Ipv4Addr;
}

trait Broadcast {
    fn to_broadcast_address(&self) -> Ipv4Addr;
}

impl ToIpV4Addr for SocketAddr {
    fn to_ipv4_addr(&self) -> Ipv4Addr {
        let address_default = Ipv4Addr::new(255, 255, 255, 255);
        match self.ip() {
            IpAddr::V4(x) => x,
            _ => address_default,
        }
    }
}

impl Broadcast for Address {
    fn to_broadcast_address(&self) -> Ipv4Addr {
        let address_default = Ipv4Addr::new(255, 255, 255, 255);
        let ipv4_address = match self.addr {
            Some(x) => x.to_ipv4_addr(),
            _ => address_default,
        };
        let mask_address = match self.mask {
            Some(x) => x.to_ipv4_addr(),
            _ => address_default,
        };

        let mut ip_octets = ipv4_address.octets();
        let mut mask_octets = mask_address.octets();

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
        Err(
            io::Error::new(
                io::ErrorKind::Other,
                "Local address is not an IPv4 address.",
            )
        )
    }
}

fn get_broadcast_addresses() -> Result<Vec<Ipv4Addr>, interfaces::InterfacesError> {
    Ok(
        Interface::get_all()?
            .iter()
            .filter(|i| i.flags.contains(InterfaceFlags::IFF_BROADCAST))
            .map(|i| i.addresses.clone())
            .flatten()
            .map(|a| a.to_broadcast_address())
            .collect::<Vec<Ipv4Addr>>()
    )
}

fn server_routine(
    server_socket: &UdpSocket,
    on_new_client: ClientHandler,
) -> Result<(), io::Error> {
    let mut buf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

    loop {
        let (size, peer_addr) = match server_socket.recv_from(&mut buf) {
            Ok((x, y)) => (x, y),
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        println!(
            "{} - Received {} bytes from {}: {}",
            chrono::Local::now().format("%H:%M:%S").to_string(),
            size, peer_addr,
            String::from_utf8(Vec::from(buf)).unwrap()
        );

        // From self?
        if peer_addr.ip() == server_socket.local_addr()?.ip() {
            continue;
        }

        // Not handshake message?
        if let Ok(s) = String::from_utf8(Vec::from(buf)) {
            if s != HANDSHAKE_MESSAGE {
                continue;
            }
        } else {
            continue;
        }

        on_new_client(&peer_addr);
    }
    Ok(())
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

    let local_ip = get_local_ipv4_address(client_socket)?;

    /*
    let broadcast_address = IpAddr::V4(
        Ipv4Addr::new(
            local_ip.octets()[0] | 0b11111111,
            local_ip.octets()[1] | 0b11111111,
            local_ip.octets()[2] | 0b11111111,
            local_ip.octets()[3] | 0b11111111,
        )
    );
    let target_address = SocketAddr::new(broadcast_address, server_port as u16);
    */

    loop {
        broadcast_addresses.iter().for_each(|addr| {
            let broadcast_addr = SocketAddr::new(IpAddr::from(addr.octets()), server_port as u16);
            let _ = client_socket.send_to(handshake_string_bytes, broadcast_addr);
        });

        sleep(Duration::from_secs(1));
    }
    Ok(())
}

pub struct DiscoveryService {
    server_socket: UdpSocket,
    client_socket: UdpSocket,
    server_port: i16,
    thread_pool: threadpool::ThreadPool,
    is_started: bool,
}

impl DiscoveryService {
    pub fn new(server_port: i16, client_port: i16) -> Result<Self, io::Error> {
        let server_socket = UdpSocket::bind(
            format!("0.0.0.0:{}", server_port))?;
        let client_socket = UdpSocket::bind(
            format!("0.0.0.0:{}", client_port))?;

        client_socket.set_broadcast(true)?;
        server_socket.set_broadcast(true)?;

        let thread_pool = threadpool::Builder::new()
            .thread_name(String::from("DiscoveryDispatch"))
            .num_threads(THREAD_MAX)
            .build();

        Ok(Self {
            server_socket,
            client_socket,
            server_port,
            thread_pool,
            is_started: false,
        })
    }

    pub fn start(&mut self, client_handler: ClientHandler) -> Result<(), io::Error> {
        if self.is_started {
            return Err(io::Error::new(io::ErrorKind::Other, "Already started"));
        }
        self.is_started = true;

        // Clone sockets.
        let server_socket = self.server_socket.try_clone()?;
        let client_socket = self.client_socket.try_clone()?;

        let server_port = self.server_port;

        self.thread_pool.execute(move || {
            let _ = server_routine(&server_socket, client_handler);
        });
        self.thread_pool.execute(move || {
            let _ = client_routine(&client_socket, server_port);
        });

        Ok(())
    }
}
