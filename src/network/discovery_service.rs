use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::io;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::thread::sleep;
use std::time::Duration;

const BUF_SIZE: usize = 512;
const THREAD_MAX: usize = 8;
const HANDSHAKE_MESSAGE: &'static str = "Hi There! 👋 \\^O^/";

pub type ClientHandler = fn(&SocketAddr);

fn server_routine(
    server_socket: &UdpSocket,
    on_new_client: ClientHandler,
) -> Result<(), io::Error> {
    let mut buf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

    loop {
        let (_, peer_addr) = match server_socket.recv_from(&mut buf) {
            Ok((x, y)) => (x, y),
            Err(_) => {
                continue;
            }
        };

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


fn client_routine(client_socket: &UdpSocket, server_port: i16) -> Result<(), String> {
    let handshake_string_bytes = HANDSHAKE_MESSAGE.as_bytes();
    loop {
        let _ = client_socket.send_to(handshake_string_bytes,
                                           format!("255.255.255.255:{}", server_port));
        sleep(Duration::from_secs(6));
    }
    Ok(())
}

pub struct DiscoveryService {
    server_socket: UdpSocket,
    client_socket: UdpSocket,
    server_port: i16,
    client_port: i16,
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
            is_started: false,
        })
    }

    pub fn start(&mut self, client_handler: ClientHandler) -> Result<(), String> {
        if self.is_started {
            return Err(String::from("Already started."));
        }
        self.is_started = true;

        // Clone sockets.
        let server_socket = match self.server_socket.try_clone() {
            Ok(s) => s,
            Err(_) => {
                return Err(String::from("Failed to clone server socket."));
            }
        };
        let client_socket = match self.client_socket.try_clone() {
            Ok(s) => s,
            Err(_) => {
                return Err(String::from("Failed to clone client socket."));
            }
        };

        let server_port = self.server_port;

        self.thread_pool.execute(move || {
            server_routine(&server_socket, client_handler)
                .unwrap();
        });
        self.thread_pool.execute(move || {
            client_routine(&client_socket, server_port).unwrap();
        });

        Ok(())
    }
}

