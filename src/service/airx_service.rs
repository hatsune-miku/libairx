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
use crate::network::tcp_server::TcpServer;
use crate::network::text_service;
use crate::service::clipboard_service;
use crate::service::clipboard_service::{DiscoveryServiceType, TextServiceType};
use crate::transmission::protocol::text_transmission::{ReadText};

fn on_text_received(text: String, _: &SocketAddr) {
    if let Ok(mut clip) = arboard::Clipboard::new() {
        let _ = clip.set_text(text);
    }
}

pub struct AirXService<'a> {
    discovery_service_server_port: u16,
    discovery_service_client_port: u16,
    text_service_listen_addr: &'a str,
    text_service_listen_port: u16,
}

impl Default for AirXService<'_> {
    fn default() -> Self {
        Self {
            discovery_service_server_port: 9818,
            discovery_service_client_port: 0,
            text_service_listen_addr: "0.0.0.0",
            text_service_listen_port: 9819,
        }
    }
}

impl<'a> AirXService<'a> {
    fn new(
        discovery_service_server_port: u16,
        discovery_service_client_port: u16,
        text_service_listen_addr: &'a str,
        text_service_listen_port: u16,
    ) -> Self {
        Self {
            discovery_service_server_port,
            discovery_service_client_port,
            text_service_listen_addr,
            text_service_listen_port,
        }
    }
    pub fn run(&self) {
        // Create services.
        let mut discovery_service = discovery_service::DiscoveryService::new(
            self.discovery_service_server_port, self.discovery_service_client_port)
            .expect("Failed to create discovery service");
        let mut text_service = text_service::TextService::new(
            self.text_service_listen_addr.to_string(), self.text_service_listen_port, on_text_received)
            .expect("Failed to create text service");

        // Create service pointers.
        let discover_srv_ref: DiscoveryServiceType = Arc::new(Mutex::new(discovery_service));
        let text_srv_ref: TextServiceType = Arc::new(Mutex::new(text_service));

        // Create clipboard handler.
        clipboard_service::ClipboardService::start(discover_srv_ref.clone(), text_srv_ref.clone(), self.text_service_listen_port);
        println!("Clipboard service started.");

        // Start discovery service.
        discover_srv_ref.lock().unwrap().start().expect("Failed to start service.");
        println!("Discovery service started.");

        loop {
            sleep(Duration::from_secs(1));
            // Print peer list.
            if let Ok(disc_srv_locked) = discover_srv_ref.lock() {
                if let Ok(list) = disc_srv_locked.get_peer_list() {
                    println!("Peer list:");
                    list.iter().for_each(|peer| {
                        println!("  {}", peer);
                    });
                }
            }
        }
    }
}
