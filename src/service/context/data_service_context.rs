use std::sync::Arc;
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::local::file_sending_packet::FileSendingPacket;
use crate::packet::data::text_packet::TextPacket;
use crate::service::data_service::OnPacketReceivedFunctionType;
use crate::service::discovery_service::DiscoveryService;

pub struct DataServiceContext {
    host: String,
    port: u16,
    text_callback: OnPacketReceivedFunctionType<TextPacket, ()>,
    file_coming_callback: OnPacketReceivedFunctionType<FileComingPacket, ()>,
    file_sending_callback: OnPacketReceivedFunctionType<FileSendingPacket, ()>,
    file_part_callback: OnPacketReceivedFunctionType<FilePartPacket, bool>,
    discovery_service: Arc<DiscoveryService>,
}

impl DataServiceContext {
    pub fn new(
        host: String,
        port: u16,
        text_callback: OnPacketReceivedFunctionType<TextPacket, ()>,
        file_coming_callback: OnPacketReceivedFunctionType<FileComingPacket, ()>,
        file_sending_callback: OnPacketReceivedFunctionType<FileSendingPacket, ()>,
        file_part_callback: OnPacketReceivedFunctionType<FilePartPacket, bool>,
        discovery_service: Arc<DiscoveryService>,
    ) -> Self {
        Self {
            host,
            port,
            text_callback,
            file_coming_callback,
            file_sending_callback,
            file_part_callback,
            discovery_service,
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn file_coming_callback(&self) -> OnPacketReceivedFunctionType<FileComingPacket, ()> {
        self.file_coming_callback.clone()
    }

    pub fn text_callback(&self) -> OnPacketReceivedFunctionType<TextPacket, ()> {
        self.text_callback.clone()
    }

    pub fn file_sending_callback(&self) -> OnPacketReceivedFunctionType<FileSendingPacket, ()> {
        self.file_sending_callback.clone()
    }

    pub fn file_part_callback(&self) -> OnPacketReceivedFunctionType<FilePartPacket, bool> {
        self.file_part_callback.clone()
    }

    pub fn discovery_service(&self) -> Arc<DiscoveryService> {
        self.discovery_service.clone()
    }
}

impl Clone for DataServiceContext {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            port: self.port,
            text_callback: self.text_callback.clone(),
            file_coming_callback: self.file_coming_callback.clone(),
            file_sending_callback: self.file_sending_callback.clone(),
            file_part_callback: self.file_part_callback.clone(),
            discovery_service: self.discovery_service.clone(),
        }
    }
}
