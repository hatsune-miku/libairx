extern crate core;

use std::net::SocketAddr;
use lib_util::string_from_lengthen_ptr;

use crate::network::peer::Peer;
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
use crate::packet::data::file_coming_packet::FileComingPacket;
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

#[export_name = "airx_version"]
pub extern "C" fn airx_version() -> i32 {
    20230624
}

#[export_name = "airx_compatibility_number"]
pub extern "C" fn airx_compatibility_number() -> i32 {
    1
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
        info!("libairx initialized.");
    }
}

#[export_name = "airx_create"]
pub unsafe extern "C" fn airx_create_service(
    discovery_service_server_port: u16,
    discovery_service_client_port: u16,
    text_service_listen_addr: *mut c_char,
    text_service_listen_addr_len: u32,
    text_service_listen_port: u16,
    group_identity: u8,
) -> *mut AirXService {
    let addr = string_from_lengthen_ptr(
        text_service_listen_addr, text_service_listen_addr_len);

    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port,
        discovery_service_client_port,
        text_service_listen_addr: addr.clone(),
        text_service_listen_port,
        group_identity,
    };
    let airx = AirXService::new(&config);
    let airx = match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    };

    info!("lib: AirX service created (addr={}:{},gid={})",
          addr, text_service_listen_port, group_identity);

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
    let service_disc = service_disc.lock().unwrap();
    let peers_ptr = service_disc.peers();
    drop(service_disc);

    info!("lib: Discovery service starting (cp={},sp={},gid={})",
          config.discovery_service_client_port,
          config.discovery_service_server_port,
          config.group_identity);

    let _ = DiscoveryService::run(
        config.discovery_service_client_port,
        config.discovery_service_server_port,
        peers_ptr,
        Box::new(move || should_interrupt()),
        config.group_identity,
    );

    info!("lib: Discovery service stopped.");
}

#[export_name = "airx_text_service"]
pub extern "C" fn airx_text_service(
    airx_ptr: *mut AirXService,
    text_callback_c: extern "C" fn(
        *const c_char, /* text */
        u32, /* text_len */
        *const c_char, /* socket_addr */
        u32, /* socket_addr_len */
    ),
    file_coming_callback_c: extern "C" fn(
        u32, /* file_size */
        *const c_char, /* file_name */
        u32, /* file_name_len */
        *const c_char, /* socket_addr */
        u32, /* socket_addr_len */
    ),
    should_interrupt: extern "C" fn() -> bool,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let text_callback = move |text_packet: &TextPacket, socket_addr: &SocketAddr| {
        let text_cstr = text_packet.text().as_ptr();
        let socket_addr_str = socket_addr.to_string();
        let socket_addr_cstr = socket_addr_str.as_ptr();
        text_callback_c(
            text_cstr as *const c_char,
            text_packet.text().len() as u32,
            socket_addr_cstr as *const c_char,
            socket_addr_str.len() as u32,
        );
    };

    let file_coming_callback = move |file_coming_packet: &FileComingPacket, socket_addr: &SocketAddr| {
        let file_name_cstr = file_coming_packet.file_name().as_ptr();
        let socket_addr_str = socket_addr.to_string();
        let socket_addr_cstr = socket_addr_str.as_ptr();
        file_coming_callback_c(
            file_coming_packet.file_size(),
            file_name_cstr as *const c_char,
            file_coming_packet.file_name().len() as u32,
            socket_addr_cstr as *const c_char,
            socket_addr_str.len() as u32,
        );
    };

    let context = DataServiceContext::new(
        config.text_service_listen_addr.to_string(),
        config.text_service_listen_port,
        Box::new(move || should_interrupt()),
        Box::new(text_callback),
        Box::new(file_coming_callback),
    );

    info!("lib: Text service starting (addr={},port={})",
          config.text_service_listen_addr, config.text_service_listen_port);

    let _ = DataService::run(context);

    info!("lib: Text service stopped");
}

#[deprecated]
#[export_name = "airx_lan_broadcast"]
pub extern "C" fn airx_lan_broadcast(airx_ptr: *mut AirXService) -> bool {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    DiscoveryService::broadcast_discovery_request(
        config.discovery_service_client_port,
        config.discovery_service_server_port,
        config.group_identity,
    ).is_ok()
}

#[export_name = "airx_get_peers"]
pub extern "C" fn airx_get_peers(
    airx_ptr: *mut AirXService,
    buffer: *mut c_char,
) -> u32 {
    let airx = unsafe { &mut *airx_ptr };
    let service_disc = airx.discovery_service();

    if let Ok(locked) = service_disc.lock() {
        if let Ok(peers_ptr) = locked.peers().lock() {
            let joined = peers_ptr
                .iter()
                .map(|peer| peer.to_string())
                .collect::<Vec<String>>()
                .join(",");
            let bytes = joined.as_bytes();

            unsafe {
                copy(bytes.as_ptr(), buffer as *mut u8, bytes.len());
                *buffer.offset(bytes.len() as isize) = 0;
            }
            return bytes.len() as u32;
        }
    }
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
        host, config.text_service_listen_port);

    let text_packet = match TextPacket::new(text) {
        Ok(packet) => packet,
        Err(err) => {
            error!("lib: Failed to create text packet: {}", err);
            return;
        }
    };

    let _ = DataService::send_data(
        &Peer::new(&host, config.text_service_listen_port),
        config.text_service_listen_port,
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

    if let Ok(locked) = service_disc.clone().lock() {
        if let Ok(peers_ptr) = locked.peers().lock() {
            for peer in peers_ptr.iter() {
                let thread_peer = peer.clone();
                let thread_config = config.clone();
                let thread_text_serialized = text_serialized.clone();
                std::thread::spawn(move || {
                    info!("lib: Sending text to (addr={}:{})",
                            thread_peer.host(), thread_config.text_service_listen_port);
                    let _ = DataService::send_data(
                        &thread_peer,
                        thread_config.text_service_listen_port,
                        MagicNumbers::Text,
                        &thread_text_serialized,
                        Duration::from_millis(500),
                    );
                });
            }
        }
    }
}

pub extern "C" fn airx_send_file(
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

    info!("lib: Sending file {} to (addr={}:{})",
        file_path, host, config.text_service_listen_port);

}
