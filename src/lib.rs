extern crate core;

use std::fs::File;
use lib_util::string_from_lengthen_ptr;

use crate::network::peer::{Peer};
use crate::service::airx_service::{AirXService};
use crate::service::discovery_service::DiscoveryService;
use std::os::raw::c_char;
use std::ptr::copy;
use std::sync::Arc;
use std::time::Duration;
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Logger, Root};
use log::{error, info, LevelFilter};
use crate::network::peer;
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::file_receive_response_packet::FileReceiveResponsePacket;
use crate::packet::data::local::file_sending_packet::FileSendingPacket;
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data::text_packet::TextPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;
use crate::service::data_service::{DataService};

pub mod lib_util;
pub mod network;
pub mod service;
pub mod packet;
pub mod util;
pub mod compatibility;
pub mod proto;
pub mod extension;

#[export_name = "airx_version"]
pub extern "C" fn airx_version() -> i32 {
    20230715
}

#[export_name = "airx_compatibility_number"]
pub extern "C" fn airx_compatibility_number() -> i32 {
    3
}

#[export_name = "airx_version_string"]
pub extern "C" fn airx_version_string(buffer: *mut c_char) -> u64 {
    let version = "🐮";
    let version = version.as_bytes();
    let len = version.len();
    unsafe {
        copy(version.as_ptr(), buffer as *mut u8, len);
        buffer.offset(len as isize).write(0);
    }
    len as u64
}

#[export_name = "airx_init"]
pub extern "C" fn airx_init() {
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

#[export_name = "airx_create"]
pub unsafe extern "C" fn airx_create_service(
    discovery_service_server_port: u16,
    discovery_service_client_port: u16,
    text_service_listen_addr: *mut c_char,
    text_service_listen_addr_len: u32,
    text_service_listen_port: u16,
    group_identifier: u32,
) -> *mut AirXService {
    let addr = string_from_lengthen_ptr(
        text_service_listen_addr, text_service_listen_addr_len);

    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port,
        discovery_service_client_port,
        text_service_listen_addr: addr.clone(),
        data_service_listen_port: text_service_listen_port,
        group_identifier,
    };
    let airx = AirXService::new(&config);
    let airx = match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    };

    info!("lib: AirX config created (addr={}:{},gid={})",
          addr, text_service_listen_port, group_identifier);

    airx
}

#[export_name = "airx_lan_discovery_service"]
pub extern "C" fn airx_lan_discovery_service(
    airx_ptr: *mut AirXService,
    should_interrupt: extern "C" fn() -> bool,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let service_disc = airx.discovery_service();
    let peers_ptr = service_disc.peers();
    drop(service_disc);

    info!("lib: Discovery service starting (cp={},sp={},gid={})",
          config.discovery_service_client_port,
          config.discovery_service_server_port,
          config.group_identifier);

    let _ = DiscoveryService::run(
        config.discovery_service_client_port,
        config.discovery_service_server_port,
        peers_ptr,
        Box::new(move || should_interrupt()),
        config.group_identifier,
    );

    info!("lib: Discovery service stopped.");
}

#[export_name = "airx_data_service"]
pub extern "C" fn airx_data_service(
    airx_ptr: *mut AirXService,
    text_callback_c: extern "C" fn(
        *const c_char, /* text */
        u32, /* text_len */
        *const c_char, /* socket_addr */
        u32, /* socket_addr_len */
    ),
    file_coming_callback_c: extern "C" fn(
        u64, /* file_size */
        *const c_char, /* file_name */
        u32, /* file_name_len */
        *const c_char, /* socket_addr */
        u32, /* socket_addr_len */
    ),
    file_sending_callback_c: extern "C" fn(
        u8, /* file_id */
        u64, /* progress */
        u64, /* total */
        u8, /* status */
    ),
    file_part_callback_c: extern "C" fn(
        u8, /* file_id */
        u64, /* offset */
        u64, /* length */
        *const u8, /* data */
    ) -> bool,
    should_interrupt: extern "C" fn() -> bool,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let should_interrupt_callback = move || should_interrupt();

    let text_callback = move |text_packet: &TextPacket, peer: Option<&Peer>| {
        let text_cstr = text_packet.text().as_ptr();
        let socket_addr_str = match peer {
            Some(p) => p.to_string(),
            None => peer::DEFAULT_HOSTNAME.to_string(),
        };
        let socket_addr_cstr = socket_addr_str.as_ptr();
        text_callback_c(
            text_cstr as *const c_char,
            text_packet.text().len() as u32,
            socket_addr_cstr as *const c_char,
            socket_addr_str.len() as u32,
        );
    };

    let file_coming_callback = move |file_coming_packet: &FileComingPacket, peer: Option<&Peer>| {
        let file_name_cstr = file_coming_packet.file_name().as_ptr();
        let socket_addr_str = match peer {
            Some(p) => p.to_string(),
            None => peer::DEFAULT_HOSTNAME.to_string(),
        };
        let socket_addr_cstr = socket_addr_str.as_ptr();
        file_coming_callback_c(
            file_coming_packet.file_size(),
            file_name_cstr as *const c_char,
            file_coming_packet.file_name().len() as u32,
            socket_addr_cstr as *const c_char,
            socket_addr_str.len() as u32,
        );
    };

    let file_sending_callback = move |file_sending_packet: &FileSendingPacket, _: Option<&Peer>| {
        file_sending_callback_c(
            file_sending_packet.file_id(),
            file_sending_packet.progress(),
            file_sending_packet.total(),
            file_sending_packet.status().to_u8(),
        );
    };

    let file_part_callback = move |file_part_packet: &FilePartPacket, _: Option<&Peer>| -> bool {
        let data = file_part_packet.data().clone();
        let data_cstr = data.as_ptr();

        file_part_callback_c(
            file_part_packet.file_id(),
            file_part_packet.offset(),
            file_part_packet.length(),
            data_cstr,
        )
    };

    let context = DataServiceContext::new(
        config.text_service_listen_addr.to_string(),
        config.data_service_listen_port,
        Arc::new(Box::new(text_callback)),
        Arc::new(Box::new(file_coming_callback)),
        Arc::new(Box::new(file_sending_callback)),
        Arc::new(Box::new(file_part_callback)),
        airx.discovery_service().clone(),
    );

    info!("lib: Data service starting (addr={},port={})",
          config.text_service_listen_addr, config.data_service_listen_port);

    let _ = DataService::run(context, Box::new(should_interrupt_callback));

    info!("lib: Data service stopped");
}

#[deprecated]
#[export_name = "airx_lan_broadcast"]
pub extern "C" fn airx_lan_broadcast(airx_ptr: *mut AirXService) -> bool {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    DiscoveryService::broadcast_discovery_request(
        config.discovery_service_client_port,
        config.discovery_service_server_port,
        config.group_identifier,
    ).is_ok()
}

#[export_name = "airx_get_peers"]
pub extern "C" fn airx_get_peers(
    airx_ptr: *mut AirXService,
    buffer: *mut c_char,
) -> u32 {
    let airx = unsafe { &mut *airx_ptr };
    let service_disc = airx.discovery_service();

    if let Ok(peers_ptr) = service_disc.peers().lock() {
        let joined = peers_ptr
            .iter()
            .map(|peer| peer.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let bytes = joined.as_bytes();

        info!("lib: Get peers (peers={})", joined);

        unsafe {
            copy(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            *buffer.offset(bytes.len() as isize) = 0;
        }
        return bytes.len() as u32;
    }
    error!("lib: Failed to get peers");
    0
}

#[export_name = "airx_send_text"]
pub extern "C" fn airx_send_text(
    airx_ptr: *mut AirXService,
    host: *const c_char,
    host_len: u32,
    text: *mut c_char,
    text_len: u32,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    let text = string_from_lengthen_ptr(text, text_len);
    let host = string_from_lengthen_ptr(host, host_len);

    info!("lib: Sending text to (addr={}:{})",
        host, config.data_service_listen_port);

    let text_packet = match TextPacket::new(text) {
        Ok(packet) => packet,
        Err(err) => {
            error!("lib: Failed to create text packet: {}", err);
            return;
        }
    };

    let _ = DataService::send_once_with_retry(
        &Peer::new(&host, config.data_service_listen_port, None),
        config.data_service_listen_port,
        MagicNumbers::Text,
        &text_packet.serialize(),
        Duration::from_millis(500),
    );
}

#[export_name = "airx_broadcast_text"]
pub extern "C" fn airx_broadcast_text(
    airx_ptr: *mut AirXService,
    text: *mut c_char,
    len: u32,
) {
    if text == std::ptr::null_mut() || len < 1 {
        return;
    }

    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    let service_disc = airx.discovery_service();

    let text = string_from_lengthen_ptr(text, len);
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
                let _ = DataService::send_once_with_retry(
                    &thread_peer,
                    thread_config.data_service_listen_port,
                    MagicNumbers::Text,
                    &thread_text_serialized,
                    Duration::from_millis(500),
                );
            });
        }
    }
}

#[export_name = "airx_try_send_file"]
pub extern "C" fn airx_try_send_file(
    airx_ptr: *mut AirXService,
    host: *const c_char,
    host_len: u32,
    file_path: *const c_char,
    file_path_len: u32,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    let host = string_from_lengthen_ptr(host, host_len);
    let file_path = string_from_lengthen_ptr(file_path, file_path_len);

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
    let _ = DataService::send_once_with_retry(
        &Peer::new(&host, config.data_service_listen_port, None),
        config.data_service_listen_port,
        MagicNumbers::FileComing,
        &packet.serialize(),
        Duration::from_millis(500),
    );
}

#[export_name = "airx_respond_to_file"]
pub extern "C" fn airx_respond_to_file(
    airx_ptr: *mut AirXService,
    host: *const c_char,
    host_len: u32,
    file_id: u8,
    file_size: u64,
    file_path: *const c_char,
    file_path_len: u32,
    accept: bool,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    let host = string_from_lengthen_ptr(host, host_len);
    let file_path = string_from_lengthen_ptr(file_path, file_path_len);

    let packet = FileReceiveResponsePacket::new(
        file_id, file_size, file_path, accept);
    let _ = DataService::send_once_with_retry(
        &Peer::new(&host, config.data_service_listen_port, None),
        config.data_service_listen_port,
        MagicNumbers::FileReceiveResponse,
        &packet.serialize(),
        Duration::from_millis(500),
    );
}
