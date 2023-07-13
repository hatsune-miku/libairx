use std::fs::File;
use std::io;
use std::io::{Read, Seek};
use std::net::SocketAddr;
use std::time::Duration;
use log::{error, info, warn};
use crate::network::peer::Peer;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::file_receive_response_packet::FileReceiveResponsePacket;
use crate::packet::data::local::file_sending_packet::{FileSendingPacket, FileSendingStatus};
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data_packet::DataPacket;
use crate::packet::data_transmission::DataTransmit;
use crate::packet::protocol::serialize::Serialize;
use crate::service::data_service::DataService;
use crate::service::handler::context::{ConnectionControl, HandlerContext};

const BUFFER_SIZE: usize = 8 * 1024 * 1024;
const TIMEOUT_MILLIS: u64 = 1000;
const DATA_SESSION_RECONNECT_TRIES: u32 = 3;

struct TransmissionState {
    bytes_sent_total: u64,
}

pub fn handle(context: HandlerContext) -> ConnectionControl {
    let packet = match FileReceiveResponsePacket::deserialize(context.packet().data()) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize file receive response packet ({:?}).", e);
            return ConnectionControl::Default;
        },
    };

    info!("Received file receive response packet from {}.", context.socket_addr());

    let update_status = |status: FileSendingStatus| {
        (context.data_service_context().file_sending_callback())(&FileSendingPacket::new(
            packet.file_id(),
            0,
            packet.file_size(),
            status,
        ), &context.socket_addr());
    };

    // Update status!
    update_status(FileSendingStatus::Requested);

    let ipv4addr = match context.socket_addr() {
        SocketAddr::V4(addr) => addr.ip().clone(),
        SocketAddr::V6(_) => {
            warn!("Received file receive response packet from IPv6 address.");
            update_status(FileSendingStatus::Error);
            return ConnectionControl::Default;
        }
    };

    if !packet.accepted() {
        info!("File receive request rejected by peer.");
        update_status(FileSendingStatus::Rejected);
        return ConnectionControl::Default;
    }

    info!("File receive request accepted by peer.");
    update_status(FileSendingStatus::Accepted);

    let filename = packet.file_name();
    let peer = Peer::from(&ipv4addr, context.data_service_context().port(), None);

    // Connect to peer, start data transmission and close connection.
    let mut buffer = vec![0u8; BUFFER_SIZE];

    // Log on every 100th iteration.
    let mut log_counter = 0;

    let mut session = |dt: &mut DataTransmit,
                       state: &mut TransmissionState| -> Result<(), io::Error> {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to open file ({}).", e);
                update_status(FileSendingStatus::Error);
                return Err(e);
            }
        };
        let mut offset = state.bytes_sent_total;
        match file.seek(io::SeekFrom::Start(offset)) {
            Ok(n) => {
                if n != offset {
                    let error = io::Error::new(
                        io::ErrorKind::Other,
                        "Wrong seek position.",
                    );
                    warn!("Failed to seek file ({}).", error);
                    update_status(FileSendingStatus::Error);
                    return Err(error);
                }
                info!("Seeked file to {}.", offset);
            }
            Err(e) => {
                warn!("Failed to seek file ({}).", e);
                update_status(FileSendingStatus::Error);
                return Err(e);
            }
        }

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
                packet.file_id(), offset, bytes_read as u64, buffer[..bytes_read].to_vec(),
            );

            // Wrap to generic data packet.
            let data_packet = DataPacket::new(MagicNumbers::FilePart.value(), &file_part_packet.serialize());

            // Send.
            if let Err(e) = dt.send_data_progress_with_retry(&data_packet.serialize(), |bytes_written_total| {
                state.bytes_sent_total += bytes_written_total;
            }) {
                error!("Failed to send file part packet ({}).", e);
                update_status(FileSendingStatus::Error);
                return Err(e);
            }

            // Report on every 100th packet.
            if log_counter >= 100 {
                log_counter = 0;
                info!("File part status: (fid={}, progress={}/{}).", packet.file_id(), offset, packet.file_size());

                // Create local notification packet, update status and notify.
                let local_packet = FileSendingPacket::new(
                    packet.file_id(),
                    offset,
                    packet.file_size(),
                    FileSendingStatus::InProgress,
                );
                (context.data_service_context().file_sending_callback())(&local_packet, &context.socket_addr());
            }

            offset += bytes_read as u64;
        }
        Ok(())
    };

    let state = TransmissionState {
        bytes_sent_total: 0,
    };

    if let Err(e) = DataService::data_session(
        &peer, context.data_service_context().port(),
        Duration::from_millis(TIMEOUT_MILLIS),
        &mut session,
        DATA_SESSION_RECONNECT_TRIES,
        state,
    ) {
        error!("Failed to send file part packet ({}).", e);
        update_status(FileSendingStatus::Error);
        return ConnectionControl::Default;
    }

    update_status(FileSendingStatus::Completed);
    ConnectionControl::Default
}
