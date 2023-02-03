use crate::service::discovery_service::DiscoveryService;
use crate::service::text_service::TextService;
use crate::util::shared_mutable::SharedMutable;
use std::io;

pub struct AirXServiceConfig {
    pub discovery_service_server_port: u16,
    pub discovery_service_client_port: u16,
    pub text_service_listen_addr: String,
    pub text_service_listen_port: u16,
}

impl Clone for AirXServiceConfig {
    fn clone(&self) -> Self {
        Self {
            discovery_service_server_port: self.discovery_service_server_port,
            discovery_service_client_port: self.discovery_service_client_port,
            text_service_listen_addr: self.text_service_listen_addr.clone(),
            text_service_listen_port: self.text_service_listen_port,
        }
    }
}

pub struct AirXService {
    config: AirXServiceConfig,
    text_service: SharedMutable<TextService>,
    discovery_service: SharedMutable<DiscoveryService>,
}

impl AirXService {
    pub fn new(config: &AirXServiceConfig) -> Result<Self, io::Error> {
        // Create services.
        let discovery_service = DiscoveryService::new()?;
        let text_service = TextService::new();

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

    pub fn config(&self) -> AirXServiceConfig {
        self.config.clone()
    }
}
