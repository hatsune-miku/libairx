use std::net::SocketAddr;
use log::warn;
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;

pub fn handle(
    packet: &DataPacket,
    socket_addr: &SocketAddr,
    context: &DataServiceContext
) {
    match FileComingPacket::deserialize(packet.data()) {
        Ok(p) => (context.file_coming_callback())(&p, &socket_addr),
        Err(e) => warn!("Failed to deserialize file coming packet ({:?}).", e),
    };
}
