use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::local::file_sending_packet::FileSendingPacket;
use crate::packet::data::text_packet::TextPacket;
use crate::service::data_service::OnPacketReceivedFunctionType;
use crate::service::ShouldInterruptFunctionType;

pub struct DataServiceContext {
    host: String,
    port: u16,
    should_interrupt: ShouldInterruptFunctionType,
    text_callback: OnPacketReceivedFunctionType<TextPacket>,
    file_coming_callback: OnPacketReceivedFunctionType<FileComingPacket>,
    file_sending_callback: OnPacketReceivedFunctionType<FileSendingPacket>,
    file_part_callback: OnPacketReceivedFunctionType<FilePartPacket>,
}

impl DataServiceContext {
    pub fn new(
        host: String,
        port: u16,
        should_interrupt: ShouldInterruptFunctionType,
        text_callback: OnPacketReceivedFunctionType<TextPacket>,
        file_coming_callback: OnPacketReceivedFunctionType<FileComingPacket>,
        file_sending_callback: OnPacketReceivedFunctionType<FileSendingPacket>,
        file_part_callback: OnPacketReceivedFunctionType<FilePartPacket>,
    ) -> Self {
        Self {
            host,
            port,
            should_interrupt,
            file_coming_callback,
            text_callback,
            file_sending_callback,
            file_part_callback,
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

    pub fn file_sending_callback(&self) -> &OnPacketReceivedFunctionType<FileSendingPacket> {
        &self.file_sending_callback
    }

    pub fn file_part_callback(&self) -> &OnPacketReceivedFunctionType<FilePartPacket> {
        &self.file_part_callback
    }
}
