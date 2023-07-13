use log::{info, warn};
use crate::packet::data::text_packet::TextPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::handler::context::{HandlerContext, ConnectionControl};

pub fn handle(context: HandlerContext) -> ConnectionControl {
    let packet = match TextPacket::deserialize(context.packet().data()) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize text packet ({:?}).", e);
            return ConnectionControl::CloseConnection;
        },
    };

    info!("Received text packet from {}.", context.socket_addr());
    (context.data_service_context().text_callback())(&packet, &context.socket_addr());

    ConnectionControl::CloseConnection
}
