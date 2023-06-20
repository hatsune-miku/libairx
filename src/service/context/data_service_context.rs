use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data::text_packet::TextPacket;
use crate::service::data_service::OnPacketReceivedFunctionType;
use crate::service::ShouldInterruptFunctionType;

pub struct DataServiceContext {
    host: String,
    port: u16,
    should_interrupt: ShouldInterruptFunctionType,
    text_callback: OnPacketReceivedFunctionType<TextPacket>,
    file_coming_callback: OnPacketReceivedFunctionType<FileComingPacket>,
}

impl DataServiceContext {
    pub fn new(
        host: String,
        port: u16,
        should_interrupt: ShouldInterruptFunctionType,
        text_callback: OnPacketReceivedFunctionType<TextPacket>,
        file_coming_callback: OnPacketReceivedFunctionType<FileComingPacket>,
    ) -> Self {
        Self {
            host,
            port,
            should_interrupt,
            file_coming_callback,
            text_callback,
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn should_interrupt(&self) -> &ShouldInterruptFunctionType {
        &self.should_interrupt
    }

    pub fn file_coming_callback(&self) -> &OnPacketReceivedFunctionType<FileComingPacket> {
        &self.file_coming_callback
    }

    pub fn text_callback(&self) -> &OnPacketReceivedFunctionType<TextPacket> {
        &self.text_callback
    }
}
