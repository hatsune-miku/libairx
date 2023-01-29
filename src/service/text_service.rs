use std::borrow::Borrow;
use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::transmission;
use crate::transmission::protocol::text_transmission::{ReadText, SendText};
use std::io;
use std::net::SocketAddr;

pub type OnReceiveType = fn(text: String, source: &SocketAddr);

static SYNC_PREFIX: &'static str = "SYNC:";

pub struct TextService {
    host: String,
    port: u16,
    subscribers: Vec<OnReceiveType>,
}

impl TextService {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, subscriber: OnReceiveType) {
        self.subscribers.push(subscriber);
    }

    pub fn run(
        &self,
    ) -> Result<(), io::Error> {
        let mut server_socket = TcpServer::create_and_listen(
            self.host.as_str(), self.port)?;
        while let Ok((stream, socket_addr)) = server_socket.accept() {
            let mut socket = Socket::from(stream);
            let mut tt = transmission::text::TextTransmission::from(&mut socket);
            if let Ok(s) = tt.read_text() {
                if !s.starts_with(SYNC_PREFIX) {
                    continue;
                }

                let text = String::from(&s[SYNC_PREFIX.len()..]);
                for subscriber in &self.subscribers {
                    subscriber(text.clone(), &socket_addr);
                }
            }
        }
        Ok(())
    }

    /// Connect, send and close.
    pub fn send(
        &self,
        peer: &Peer,
        port: u16,
        text: &String,
        connect_timeout: core::time::Duration,
    ) -> Result<(), io::Error> {
        let mut socket = Socket::connect(peer.host(), port, connect_timeout)?;
        let mut tt = transmission::text::TextTransmission::from(&mut socket);
        tt.send_text(String::from(format!("{}{}", SYNC_PREFIX, text)).as_str())?;
        socket.close()?;
        Ok(())
    }
}
