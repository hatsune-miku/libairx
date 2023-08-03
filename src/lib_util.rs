use std::fs::File;
use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Duration;
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Logger, Root};
use log::{error, info, LevelFilter};
use crate::network::peer::Peer;
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data::file_receive_response_packet::FileReceiveResponsePacket;
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data::text_packet::TextPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::airx_service::AirXServiceConfig;
use crate::service::context::data_service_context::DataServiceContext;
use crate::service::data_service::DataService;
use crate::service::discovery_service::DiscoveryService;
use crate::service::ShouldInterruptFunctionType;

pub const CONNECTION_TIMEOUT_MILLIS: u64 = 3000;
pub const AIRX_VERSION: i32 = 20230802;
pub const AIRX_COMPATIBLE_NUMBER: i32 = 4;

pub fn shared_airx_version_code() -> String {
    String::from("\\^O^/")
}

pub fn shared_airx_broadcast_text(text: String, service_disc: Arc<DiscoveryService>, config: &AirXServiceConfig) {
    let packet = match TextPacket::new(text) {
        Ok(packet) => packet,
        Err(err) => {
            error!("lib: Failed to create text packet: {}", err);
            return;
        }
    };
    let text_serialized = Arc::new(packet.serialize());

    if let Ok(peers_ptr) = service_disc.peers().lock() {
        for peer in peers_ptr.iter() {
            let thread_peer = peer.clone();
            let thread_config = config.clone();
            let thread_text_serialized = text_serialized.clone();
            std::thread::spawn(move || {
                info!("lib: Sending text to (addr={}:{})",
                            thread_peer.host(), thread_config.data_service_listen_port);
                if let Err(e) = DataService::send_once_with_retry(
                    &thread_peer,
                    thread_config.data_service_listen_port,
                    MagicNumbers::Text,
                    &thread_text_serialized,
                    Duration::from_millis(CONNECTION_TIMEOUT_MILLIS),
                ) {
                    error!(
                        "lib: Failed to send text to (addr={}:{}): {}",
                        thread_peer.host(), thread_config.data_service_listen_port, e);
                }
            });
        }
    }
}

pub fn shared_airx_try_send_file(host: String, file_path: String, config: &AirXServiceConfig) {
    info!("lib: Sending file info {} to (addr={}:{})",
        file_path, host, config.data_service_listen_port);

    info!("lib: Reading file info {}", file_path);
    let file_info = match File::open(file_path.clone()) {
        Ok(f) => f,
        Err(e) => {
            error!("lib: Failed to open file {}: {}", file_path, e);
            return;
        }
    };

    info!("lib: Reading metadata of file {}", file_path);
    let metadata = match file_info.metadata() {
        Ok(d) => d,
        Err(e) => {
            error!("lib: Failed to read metadata of file {}: {}", file_path, e);
            return;
        }
    };

    info!("lib: Sending file info {} to (addr={}:{})",
        file_path, host, config.data_service_listen_port);
    let packet = FileComingPacket::new(metadata.len(), file_path.clone());
    match DataService::send_once_with_retry(
        &Peer::new(&host, config.data_service_listen_port, None),
        config.data_service_listen_port,
        MagicNumbers::FileComing,
        &packet.serialize(),
        Duration::from_millis(CONNECTION_TIMEOUT_MILLIS),
    ) {
        Ok(_) => {
            info!("lib: File info {} sent to (addr={}:{})",
                file_path, host, config.data_service_listen_port);
        }
        Err(e) => {
            error!("lib: Failed to send file info {}: {}", file_path, e);
        }
    }
}

pub fn shared_airx_data_service(context: DataServiceContext, config: &AirXServiceConfig, should_interrupt: ShouldInterruptFunctionType) {
    info!("lib: Data service starting (addr={},port={})",
          config.text_service_listen_addr, config.data_service_listen_port);

    let _ = DataService::run(context, should_interrupt);

    info!("lib: Data service stopped");
}

pub fn shared_airx_respond_to_file(host: String, file_id: u8, file_size: u64, file_path: String, accept: bool, config: &AirXServiceConfig) {
    let packet = FileReceiveResponsePacket::new(
        file_id,
        file_size,
        file_path,
        accept,
    );
    match DataService::send_once_with_retry(
        &Peer::new(&host, config.data_service_listen_port, None),
        config.data_service_listen_port,
        MagicNumbers::FileReceiveResponse,
        &packet.serialize(),
        Duration::from_millis(CONNECTION_TIMEOUT_MILLIS),
    ) {
        Ok(_) => {
            info!("lib: Successfully sent file response to (addr={}:{})",
                host, config.data_service_listen_port);
        }
        Err(e) => {
            error!("lib: Failed to send file response to (addr={}:{}): {}",
                host, config.data_service_listen_port, e);
        }
    }
}

pub fn shared_airx_init() {
    // Init logger.
    if let Ok(logger_config) = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(
            ConsoleAppender::builder().build()
        )))
        .logger(Logger::builder().build("libairx", LevelFilter::Trace))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace)) {
        let _ = log4rs::init_config(logger_config);
        info!("lib: Initialized.");
    }
}

pub fn shared_string_from_lengthen_ptr(ptr: *const c_char, len: u32) -> String {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(slice).to_string()
}
