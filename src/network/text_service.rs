use std::io;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::transmission;
use crate::transmission::protocol::text_transmission::{ReadText, SendText};

pub type OnReceiveType = fn(text: String, source: &SocketAddr);

fn server_routine(host: &str, port: u16, on_receive: OnReceiveType) -> Result<(), io::Error> {
    let mut server_socket = TcpServer::create_and_listen(host, port)?;
    while let Ok((stream, socket_addr)) = server_socket.accept() {
        let mut socket = Socket::from(stream);
        let mut tt = transmission::text::TextTransmission::from(&mut socket);
        if let Ok(s) = tt.read_text() {
            if !s.starts_with("SYNC:") {
                continue;
            }
            let text = s.split(":").collect::<Vec<&str>>()[1..].join(":");
            on_receive(text, &socket_addr);
        }
    };
    Ok(())
}

pub struct TextService {
    server_thread: JoinHandle<Result<(), io::Error>>,
}

impl TextService {
    pub fn new(host: String, port: u16, on_receive: OnReceiveType) -> Result<Self, io::Error> {
        let host_clone = host.clone();
        Ok(
            Self {
                server_thread: std::thread::spawn(move || {
                    server_routine(host_clone.as_str(), port, on_receive)
                })
            }
        )
    }

    /// Connect, send and close.
    pub fn send(&self, peer: &Peer, port: u16, text: &String, connect_timeout: core::time::Duration) -> Result<(), io::Error> {
        let mut socket = Socket::connect(peer.host(), port, connect_timeout)?;
        let mut tt = transmission::text::TextTransmission::from(&mut socket);
        tt.send_text(&*String::from(format!("SYNC:{}", text)))?;
        socket.close()?;
        Ok(())
    }
}
