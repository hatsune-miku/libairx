use std::net::SocketAddr;
use log::{trace, warn};
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;

pub fn handle(
    packet: &DataPacket,
    socket_addr: &SocketAddr,
    context: &DataServiceContext
) {
    let packet = match FilePartPacket::deserialize(packet.data()) {
        Ok(p) => p,
        Err(e) => return warn!(
            "Failed to deserialize file part packet ({:?}).", e),
    };

    trace!("Received file part packet from {} (offset={}, length={}).", socket_addr, packet.offset(), packet.length());
    (context.file_part_callback())(&packet, socket_addr);
}
