use log::{info, warn};
use crate::network::peer::Peer;
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::handler::context::{ConnectionControl, HandlerContext};

pub fn handle(context: HandlerContext) -> ConnectionControl {
    let packet = match FileComingPacket::deserialize(context.packet().data()) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize file coming packet ({:?}).", e);
            return ConnectionControl::CloseConnection;
        },
    };

    let peer = context
        .data_service_context()
        .discovery_service()
        .peer_lookup(&context.socket_addr());
    let peer = match peer {
        Some(p) => p,
        None => Peer::new(&context.socket_addr().ip().to_string(), context.socket_addr().port(), None),
    };

    info!("Received file coming packet from {} ({}).", peer.host_name(), context.socket_addr());
    (context.data_service_context().file_coming_callback())(&packet, Some(&peer));

    ConnectionControl::CloseConnection
}
