use std::io;
use crate::network::discovery_service;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use crate::service::text_service::TextService;
use crate::util::shared_mutable::SharedMutable;

fn on_text_received(text: String, _: &SocketAddr) {
    println!("Received text: {}", text);
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
        let mut discovery_service = discovery_service::DiscoveryService::new(
            config.discovery_service_server_port,
            config.discovery_service_client_port,
        )?;

        let mut text_service = TextService::new(
            config.text_service_listen_addr.to_string(),
            config.text_service_listen_port,
        );
        text_service.subscribe(on_text_received);

        // Start discovery service.
        discovery_service.run()?;

        Ok(Self {
            config: config.clone(),
            text_service: SharedMutable::new(text_service),
        })
    } // run

    pub fn run_text_service_sync(&self) -> Result<(), io::Error> {
        match self.text_service.lock_and_get() {
            Ok(locked) => locked.run(),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Failed to lock text service.")),
        }
    }
}
