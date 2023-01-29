use std::io;
use crate::network::discovery_service;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use crate::network::discovery_service::DiscoveryService;
use crate::service::text_service::TextService;
use crate::util::shared_mutable::SharedMutable;

pub struct AirXServiceConfig<'a> {
    pub discovery_service_server_port: u16,
    pub discovery_service_client_port: u16,
    pub text_service_listen_addr: &'a str,
    pub text_service_listen_port: u16,
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
    discovery_service: SharedMutable<DiscoveryService>,
}

impl<'a> AirXService<'a> {
    pub fn new(config: &AirXServiceConfig<'a>) -> Result<Self, io::Error> {
        // Create services.
        let mut discovery_service = DiscoveryService::new(
            config.discovery_service_server_port,
            config.discovery_service_client_port,
        )?;

        let mut text_service = TextService::new(
            config.text_service_listen_addr.to_string(),
            config.text_service_listen_port,
        );

        Ok(Self {
            config: config.clone(),
            text_service: SharedMutable::new(text_service),
            discovery_service: SharedMutable::new(discovery_service),
        })
    } // run

    pub fn text_service(&self) -> SharedMutable<TextService> {
        self.text_service.clone()
    }

    pub fn discovery_service(&self) -> SharedMutable<DiscoveryService> {
        self.discovery_service.clone()
    }

    pub fn config(&self) -> AirXServiceConfig<'a> {
        self.config.clone()
    }
}
