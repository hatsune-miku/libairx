use crate::service::discovery_service::DiscoveryService;
use crate::service::data_service::DataService;
use crate::util::shared_mutable::SharedMutable;
use std::io;

pub struct AirXServiceConfig {
    pub discovery_service_server_port: u16,
    pub discovery_service_client_port: u16,
    pub text_service_listen_addr: String,
    pub data_service_listen_port: u16,
    pub group_identifier: u32,
}

impl Clone for AirXServiceConfig {
    fn clone(&self) -> Self {
        Self {
            discovery_service_server_port: self.discovery_service_server_port,
            discovery_service_client_port: self.discovery_service_client_port,
            text_service_listen_addr: self.text_service_listen_addr.clone(),
            data_service_listen_port: self.data_service_listen_port,
            group_identifier: self.group_identifier,
        }
    }
}

#[allow(dead_code)]
pub struct AirXService {
    config: AirXServiceConfig,
    text_service: SharedMutable<DataService>,
    discovery_service: SharedMutable<DiscoveryService>,
}

#[allow(dead_code)]
impl AirXService {
    pub fn new(config: &AirXServiceConfig) -> Result<Self, io::Error> {
        // Create services.
        let discovery_service = DiscoveryService::new();
        let text_service = DataService::new();

        Ok(Self {
            config: config.clone(),
            text_service: SharedMutable::new(text_service),
            discovery_service: SharedMutable::new(discovery_service),
        })
    } // run

    pub fn data_service(&self) -> SharedMutable<DataService> {
        self.text_service.clone()
    }

    pub fn discovery_service(&self) -> SharedMutable<DiscoveryService> {
        self.discovery_service.clone()
    }

    pub fn config(&self) -> AirXServiceConfig {
        self.config.clone()
    }
}
