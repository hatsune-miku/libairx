use std::net::SocketAddr;
use log::{info, warn};
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;

pub fn handle(
    packet: &DataPacket,
    socket_addr: &SocketAddr,
    context: &DataServiceContext
) {
    let packet = match FileComingPacket::deserialize(packet.data()) {
        Ok(p) => p,
        Err(e) => return warn!("Failed to deserialize file coming packet ({:?}).", e),
    };

    info!("Received file coming packet from {}.", socket_addr);
    (context.file_coming_callback())(&packet, &socket_addr);
}
