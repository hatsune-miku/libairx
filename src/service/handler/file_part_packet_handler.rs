use std::net::SocketAddr;
use log::warn;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data_packet::DataPacket;
use crate::packet::data_transmission::DataTransmission;
use crate::packet::protocol::data::SendDataWithRetry;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;

pub fn handle(
    tt: &mut DataTransmission,
    packet: &DataPacket,
    socket_addr: &SocketAddr,
    context: &DataServiceContext
) {
    let packet = match FilePartPacket::deserialize(packet.data()) {
        Ok(p) => p,
        Err(e) => return warn!(
            "Failed to deserialize file part packet ({:?}).", e),
    };

    let response_body = vec![1, 1, 4, 5, 1, 4];
    let response = DataPacket::new(MagicNumbers::FilePartResponse.value(), &response_body);
    let _ = tt.send_data_with_retry(&response.serialize());

    (context.file_part_callback())(&packet, socket_addr);
}
