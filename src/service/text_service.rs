use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::service::ShouldInterruptType;
use crate::packet::protocol::text_transmission::{ReadText, SendText};
use crate::packet::text::TextTransmission;
use crate::util::shared_mutable::SharedMutable;
use std::io;
use std::io::ErrorKind::{TimedOut, WouldBlock};
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use log::{error, info, warn};

pub type OnReceiveType = Box<dyn Fn(String, &SocketAddr) + Send + Sync>;
pub type SubscriberType = SharedMutable<Vec<OnReceiveType>>;

const TCP_ACCEPT_WAIT_MILLIS: u64 = 10;
const TCP_ACCEPT_TIMEOUT_COUNT: u64 = 100;
const TCP_ACCEPT_TRY_TIMES: u64 = 5;
const TCP_ACCEPT_TRY_WAIT_MILLISECONDS: u64 = 10;

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
            info!("A new warrior has entered the ring!");
            locked.push(callback);
        } else {
            error!("Failed to subscribe to text service.");
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
        tt.send_text(text.clone())?;
        socket.close()?;
        Ok(())
    }

    #[allow(unused_assignments)]
    pub fn run(
        host: &str,
        port: u16,
        should_interrupt: ShouldInterruptType,
        subscribers: SubscriberType,
    ) -> Result<(), io::Error> {
        let server_socket = TcpServer::create_and_listen(host, port)?;
        let mut timeout_counter = 0;

        for stream in server_socket.incoming() {
            match stream {
                Ok(stream) => {
                    let socket_addr = match stream.peer_addr() {
                        Ok(addr) => addr,
                        Err(_) => {
                            warn!("Failed to get peer address.");
                            continue;
                        }
                    };
                    let mut socket = Socket::from(stream);
                    let mut tt = TextTransmission::from(&mut socket);
                    let mut tries = TCP_ACCEPT_TRY_TIMES;
                    while tries > 0 {
                        match tt.read_text() {
                            Ok(s) => {
                                if let Ok(locked) = subscribers.lock() {
                                    for subscriber in locked.iter() {
                                        subscriber(s.clone(), &socket_addr);
                                    }
                                    break;
                                }
                            }
                            Err(e) => {
                                tries -= 1;
                                sleep(Duration::from_millis(TCP_ACCEPT_TRY_WAIT_MILLISECONDS));
                                warn!("Failed to read text ({}). Tries remaining: {}", e, tries);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                    // Check if interrupted.
                    sleep(Duration::from_millis(TCP_ACCEPT_WAIT_MILLIS));

                    // Check if timeout.
                    if timeout_counter > TCP_ACCEPT_TIMEOUT_COUNT {
                        timeout_counter = 0;
                        if should_interrupt() {
                            info!("Text service is interrupted by caller.");
                            break;
                        }
                    }

                    timeout_counter += 1;
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
