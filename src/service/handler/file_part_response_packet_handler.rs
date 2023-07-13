use log::{trace, warn};
use crate::packet::data::file_part_response_packet::{FilePartResponsePacket, ResponseKind};
use crate::packet::protocol::serialize::Serialize;
use crate::service::handler::context::{ConnectionControl, HandlerContext};

pub fn handle(context: HandlerContext) -> ConnectionControl {
    let packet = match FilePartResponsePacket::deserialize(context.packet().data()) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize file part response packet ({:?}).", e);
            return ConnectionControl::Default;
        },
    };

    trace!("Received file part response packet from {}.", context.socket_addr());

    match packet.response_kind() {
        ResponseKind::StopReceiving | ResponseKind::StopSending => ConnectionControl::CloseConnection,
    }
}
