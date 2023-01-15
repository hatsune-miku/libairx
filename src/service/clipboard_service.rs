use std::io;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::network::discovery_service;
use std::thread::sleep;
use std::time::Duration;
use clipboard_master::{CallbackResult, ClipboardHandler};
use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::text_service;
use crate::transmission::protocol::text_transmission::{ReadText};

pub type DiscoveryServiceType = Arc<Mutex<discovery_service::DiscoveryService>>;
pub type TextServiceType = Arc<Mutex<text_service::TextService>>;

struct ClipboardBridge {
    pub clipboard: arboard::Clipboard,
    pub discovery_service: DiscoveryServiceType,
    pub text_service: TextServiceType,
    pub last_text: String,
    pub listen_port: u16,
}

impl ClipboardHandler for ClipboardBridge {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let text = match self.clipboard.get_text() {
            Ok(s) => s,
            Err(_) => return CallbackResult::Next,
        };

        if text == self.last_text {
            return CallbackResult::Next;
        }
        self.last_text = text.clone();
        println!("Clipboard changed: {}", text);

        if let Ok(disc_srv_locked) = self.discovery_service.lock() {
            if let Ok(list) = disc_srv_locked.get_peer_list() {
                list.iter().for_each(|peer| {
                    println!("Sending to {}", peer);

                    if let Ok(text_srv_locked) = self.text_service.lock() {
                        if let Err(e) = text_srv_locked.send(
                            peer,
                            self.listen_port,
                            &text,
                            Duration::from_secs(2),
                        ) {
                            println!("Error sending text: {}", e);
                        }
                    }
                });
            }
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: Error) -> CallbackResult {
        println!("Error: {}", error);
        CallbackResult::Next
    }
}

impl ClipboardBridge {
    pub fn new(
        discovery_service: DiscoveryServiceType,
        text_service: TextServiceType,
        text_service_listen_port: u16,
    ) -> Result<Self, arboard::Error> {
        Ok(
            Self {
                clipboard: arboard::Clipboard::new()?,
                last_text: String::new(),
                listen_port: text_service_listen_port,
                discovery_service,
                text_service,
            }
        )
    }
}

pub struct ClipboardService {}

impl ClipboardService {
    pub fn start(
        discovery_service: DiscoveryServiceType,
        text_service: TextServiceType,
        text_service_listen_port: u16,
    ) {
        std::thread::spawn(move || {
            let bridge = ClipboardBridge::new(discovery_service, text_service, text_service_listen_port)
                .expect("Failed to create clipboard bridge");
            let mut master = clipboard_master::Master::new(bridge);
            master.run().expect("Failed to run clipboard master");
        });
    }
}

