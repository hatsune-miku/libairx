use std::io;
use crate::network::discovery_service;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use crate::service::text_service::TextService;
use crate::util::shared_mutable::SharedMutable;

fn on_text_received(text: String, _: &SocketAddr) {
    if let Ok(mut clip) = arboard::Clipboard::new() {
        let _ = clip.set_text(text);

        // TODO: Should skip next broadcast here.
    }
}

pub struct AirXServiceConfig<'a> {
    discovery_service_server_port: u16,
    discovery_service_client_port: u16,
    text_service_listen_addr: &'a str,
    text_service_listen_port: u16,
}

impl Clone for AirXServiceConfig<'_> {
    fn clone(&self) -> Self {
        Self {
            discovery_service_server_port: self.discovery_service_server_port,
            discovery_service_client_port: self.discovery_service_client_port,
            text_service_listen_addr: self.text_service_listen_addr,
            text_service_listen_port: self.text_service_listen_port,
        }
    }
}

pub struct AirXService<'a> {
    config: AirXServiceConfig<'a>,
    text_service: SharedMutable<TextService>,
}

impl<'a> AirXService<'a> {
    pub fn new(config: &AirXServiceConfig<'a>) -> Result<Self, io::Error> {
        // Create services.
        let discovery_service = discovery_service::DiscoveryService::new(
            config.discovery_service_server_port,
            config.discovery_service_client_port,
        )?;

        let text_service = TextService::create_and_listen(
            config.text_service_listen_addr.to_string(),
            config.text_service_listen_port,
            on_text_received,
        )?;
        let text_service = SharedMutable::new(text_service);

        // Create service pointers.
        let discover_srv_ref = SharedMutable::new(discovery_service);

        // Start discovery service.
        discover_srv_ref
            .lock_and_get()
            .unwrap()
            .start()
            .expect("Failed to start service.");
        println!("Discovery service started.");

        Ok(Self {
            config: config.clone(),
            text_service,
        })
    } // run
}
