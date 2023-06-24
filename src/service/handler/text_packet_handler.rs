use std::net::SocketAddr;
use log::warn;
use crate::packet::data::text_packet::TextPacket;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;

pub fn handle(
    packet: &DataPacket, 
    socket_addr: &SocketAddr,
    context: &DataServiceContext
) {
    match TextPacket::deserialize(packet.data()) {
        Ok(p) => (context.text_callback())(&p, &socket_addr),
        Err(e) => warn!("Failed to deserialize text packet ({:?}).", e),
    };
}
