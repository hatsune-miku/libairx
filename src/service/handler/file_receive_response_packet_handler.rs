use std::fs::File;
use std::io;
use std::io::{Read};
use std::net::SocketAddr;
use std::time::Duration;
use log::{error, info, warn};
use crate::network::peer::Peer;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::file_receive_response_packet::FileReceiveResponsePacket;
use crate::packet::data::local::file_sending_packet::{FileSendingPacket, FileSendingStatus};
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data_packet::DataPacket;
use crate::packet::data_transmission::DataTransmission;
use crate::packet::protocol::data::{SendDataWithRetry};
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;
use crate::service::data_service::DataService;

const BUFFER_SIZE: usize = 64 * 1024;
const TIMEOUT_MILLIS: u64 = 1000;
const DATA_SESSION_RECONNECT_TRIES: u32 = 3;

pub fn handle(
    packet: &DataPacket,
    socket_addr: &SocketAddr,
    context: &DataServiceContext,
) {
    let packet = match FileReceiveResponsePacket::deserialize(packet.data()) {
        Ok(p) => p,
        Err(e) => return warn!(
            "Failed to deserialize file receive response packet ({:?}).", e),
    };

    info!("Received file receive response packet from {}.", socket_addr);

    let update_status = |status: FileSendingStatus| {
        (context.file_sending_callback())(&FileSendingPacket::new(
            packet.file_id(),
            0,
            packet.file_size(),
            status,
        ), socket_addr);
    };

    // Update status!
    update_status(FileSendingStatus::Requested);

    let ipv4addr = match socket_addr {
        SocketAddr::V4(addr) => addr.ip(),
        SocketAddr::V6(_) => {
            warn!("Received file receive response packet from IPv6 address.");
            update_status(FileSendingStatus::Error);
            return;
        }
    };

    if !packet.accepted() {
        info!("File receive request rejected by peer.");
        update_status(FileSendingStatus::Rejected);
        return;
    }

    info!("File receive request accepted by peer.");
    update_status(FileSendingStatus::Accepted);

    let filename = packet.file_name();

    let peer = Peer::from(&ipv4addr, context.port(), None);

    // Connect to peer, start data transmission and close connection.
    let mut session = |dt: &mut DataTransmission| -> Result<(), io::Error> {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to open file ({}).", e);
                update_status(FileSendingStatus::Error);
                return Err(e);
            }
        };
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut offset = 0;

        loop {
            // Read a chunk of data from file.
            let bytes_read = match file.read(&mut buffer) {
                Ok(n) => n,
                Err(e) => {
                    warn!("Failed to read file ({}).", e);
                    update_status(FileSendingStatus::Error);
                    return Err(e);
                }
            };

            // Read to end?
            if bytes_read == 0 {
                break;
            }

            // Create file part packet.
            let file_part_packet = FilePartPacket::new(
                packet.file_id(), offset, bytes_read as u32, buffer[..bytes_read].to_vec(),
            );

            // Wrap to generic data packet.
            let data_packet = DataPacket::new(MagicNumbers::FilePart.value(), &file_part_packet.serialize());

            // Send.
            if let Err(e) = dt.send_data_with_retry(&data_packet.serialize()) {
                error!("Failed to send file part packet ({}).", e);
                update_status(FileSendingStatus::Error);
                return Err(e);
            }

            info!("Sent file part packet ({} bytes).", bytes_read);

            // Create local notification packet, update status and notify.
            let local_packet = FileSendingPacket::new(
                packet.file_id(),
                offset as u64,
                packet.file_size(),
                FileSendingStatus::InProgress,
            );
            (context.file_sending_callback())(&local_packet, socket_addr);
            offset += bytes_read as u32;
        }
        Ok(())
    };

    if let Err(e) = DataService::data_session(
        &peer, context.port(),
        Duration::from_millis(TIMEOUT_MILLIS),
        &mut session,
        DATA_SESSION_RECONNECT_TRIES
    ) {
        error!("Failed to send file part packet ({}).", e);
        update_status(FileSendingStatus::Error);
        return;
    }

    update_status(FileSendingStatus::Completed);
}
