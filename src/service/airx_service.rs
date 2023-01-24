use crate::hack::global::GLOBAL;
use crate::network::discovery_service;
use crate::network::text_service;
use crate::service::clipboard_service;
use crate::service::clipboard_service::{DiscoveryServiceType, TextServiceType};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

fn on_text_received(text: String, _: &SocketAddr) {
    if let Ok(mut clip) = arboard::Clipboard::new() {
        let _ = clip.set_text(text);

        // Should skip next broadcast.
        unsafe {
            GLOBAL.skip_next_send = true;
        }
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
    // 我用了呀（无辜）
    // rust analyzer说我没用（指unused）
    // 可是我用了呀（不敢相信）
    #[allow(dead_code)]
    pub fn run(&self, args: Vec<String>) {
        // Create services.
        let discovery_service = discovery_service::DiscoveryService::new(
            self.discovery_service_server_port,
            self.discovery_service_client_port,
        )
        .expect("Failed to create discovery service");
        let text_service = text_service::TextService::create_and_listen(
            self.text_service_listen_addr.to_string(),
            self.text_service_listen_port,
            on_text_received,
        )
        .expect("Failed to create text service");

        // Create service pointers.
        let discover_srv_ref: DiscoveryServiceType = Arc::new(Mutex::new(discovery_service));
        let text_srv_ref: TextServiceType = Arc::new(Mutex::new(text_service));

        // Create clipboard handler.
        clipboard_service::ClipboardService::start(
            discover_srv_ref.clone(),
            text_srv_ref.clone(),
            self.text_service_listen_port,
        );
        println!("Clipboard service started.");

        // Start discovery service.
        discover_srv_ref
            .lock()
            .unwrap()
            .start()
            .expect("Failed to start service.");
        println!("Discovery service started.");

        loop {
            sleep(Duration::from_secs(1));

            // print peers.
            if args.contains(&String::from("--verbose")) {
                if let Ok(locked) = discover_srv_ref.lock() {
                    if let Ok(peer_list) = locked.get_peer_list() {
                        println!(
                            "[{}] Peers: {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            peer_list
                                .iter()
                                .map(|p| p.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        );
                    }
                }
            } // if
        } // loop
    } // run
}
