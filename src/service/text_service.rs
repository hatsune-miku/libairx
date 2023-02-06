use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::service::ShouldInterruptType;
use crate::packet::protocol::text_transmission::{ReadText, SendText};
use crate::packet::text::TextTransmission;
use crate::util::shared_mutable::SharedMutable;
use std::io;
use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;

pub type OnReceiveType = Box<dyn Fn(String, &SocketAddr) + Send + Sync>;
pub type SubscriberType = SharedMutable<Vec<OnReceiveType>>;

static SYNC_PREFIX: &'static str = "SYNC:";

#[allow(dead_code)]
pub struct TextService {
    subscribers_ptr: SubscriberType,
}

#[allow(dead_code)]
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
        connect_timeout: Duration,
    ) -> Result<(), io::Error> {
        let mut socket = Socket::connect(peer.host(), port, connect_timeout)?;
        let mut tt = TextTransmission::from(&mut socket);
        tt.send_text(String::from(format!("{}{}", SYNC_PREFIX, text)))?;
        socket.close()?;
        Ok(())
    }

    pub fn run(
        host: &str,
        port: u16,
        should_interrupt: ShouldInterruptType,
        subscribers: SubscriberType,
    ) -> Result<(), io::Error> {
        let server_socket = TcpServer::create_and_listen(host, port)?;

        for stream in server_socket.incoming() {
            match stream {
                Ok(stream) => {
                    let socket_addr = match stream.peer_addr() {
                        Ok(addr) => addr,
                        Err(_) => continue,
                    };
                    let mut socket = Socket::from(stream);
                    let mut tt = TextTransmission::from(&mut socket);
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
                Err(ref e) if e.kind() == WouldBlock => {
                    // Check if interrupted.
                    if should_interrupt() {
                        break;
                    }
                    sleep(Duration::from_millis(10));
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
