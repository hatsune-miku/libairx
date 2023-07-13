use log::{info, trace, warn};
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::handler::context::{ConnectionControl, HandlerContext};

pub fn handle(context: HandlerContext) -> ConnectionControl {
    let packet = match FilePartPacket::deserialize(context.packet().data()) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize file part packet ({:?}).", e);
            return ConnectionControl::Default;
        },
    };

    trace!("Received file part packet from {} (offset={}, length={}).", context.socket_addr(), packet.offset(), packet.length());
    let should_interrupt = (context.data_service_context().file_part_callback())(&packet, None);
    if should_interrupt {
        info!("File part callback requested to interrupt connection.");
        return ConnectionControl::CloseConnection;
    }

    ConnectionControl::Default
}
