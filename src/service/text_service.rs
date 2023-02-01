use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::transmission;
use crate::transmission::protocol::text_transmission::{ReadText, SendText};
use std::io;
use std::net::SocketAddr;
use crate::util::shared_mutable::SharedMutable;

pub type OnReceiveType = Box<dyn Fn(String, &SocketAddr) + Send + Sync>;
pub type SubscriberType = SharedMutable<Vec<OnReceiveType>>;

static SYNC_PREFIX: &'static str = "SYNC:";

pub struct TextService {
    subscribers_ptr: SubscriberType,
}

impl TextService {
    pub fn new() -> Self {
        Self {
            subscribers_ptr: SharedMutable::new(Vec::new()),
        }
    }

    pub fn subscribe(&mut self, callback: OnReceiveType) {
        if let Ok(mut locked) = self.subscribers_ptr.lock() {
            locked.push(callback);
        }
    }

    pub fn subscribers(&self) -> SubscriberType {
        self.subscribers_ptr.clone()
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

    pub fn run(host: &str, port: u16, subscribers: SubscriberType) -> Result<(), io::Error> {
        let mut server_socket = TcpServer::create_and_listen(
            host, port)?;
        while let Ok((stream, socket_addr)) = server_socket.accept() {
            let mut socket = Socket::from(stream);
            let mut tt = transmission::text::TextTransmission::from(&mut socket);
            if let Ok(s) = tt.read_text() {
                if !s.starts_with(SYNC_PREFIX) {
                    continue;
                }

                let text = String::from(&s[SYNC_PREFIX.len()..]);
                if let Ok(locked) = subscribers.lock() {
                    for subscriber in locked.iter() {
                        subscriber(text.clone(), &socket_addr);
                    }
                }
            }
        }
        Ok(())
    }
}
