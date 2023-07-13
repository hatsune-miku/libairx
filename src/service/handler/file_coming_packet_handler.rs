use log::{info, warn};
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

    info!("Received file coming packet from {}.", context.socket_addr());
    (context.data_service_context().file_coming_callback())(&packet, &context.socket_addr());

    ConnectionControl::CloseConnection
}
