use std::net::SocketAddr;
use log::{info, warn};
use crate::packet::data::text_packet::TextPacket;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;

pub fn handle(
    packet: &DataPacket, 
    socket_addr: &SocketAddr,
    context: &DataServiceContext
) {
    let packet = match TextPacket::deserialize(packet.data()) {
        Ok(p) => p,
        Err(e) => return warn!("Failed to deserialize text packet ({:?}).", e),
    };

    info!("Received text packet from {}.", socket_addr);
    (context.text_callback())(&packet, &socket_addr);
}
