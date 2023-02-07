extern crate core;

use lib_util::string_from_lengthen_ptr;

use crate::lib_util::PointerWrapper;
use crate::network::peer::Peer;
use crate::service::airx_service::AirXService;
use crate::service::discovery_service::DiscoveryService;
use crate::service::text_service::TextService;
use std::os::raw::c_char;
use std::ptr::copy;
use std::time::Duration;
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Logger, Root};
use log::{info, LevelFilter};

pub mod lib_util;
pub mod network;
pub mod service;
pub mod packet;
pub mod util;
pub mod compatibility;

static mut FIRST_RUN: bool = true;
static mut AIRX_SERVICE: *mut AirXService = std::ptr::null_mut();

#[export_name = "airx_version"]
pub extern "C" fn airx_version() -> i32 {
    20230206
}

#[export_name = "airx_is_first_run"]
pub extern "C" fn is_first_run() -> bool {
    unsafe {
        if FIRST_RUN {
            FIRST_RUN = false;
            true
        } else {
            false
        }
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
    // Init logger.
    if let Ok(logger_config) = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(
            ConsoleAppender::builder().build()
        )))
        .logger(Logger::builder().build("libairx", LevelFilter::Trace))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace)) {
        let _ = log4rs::init_config(logger_config);
    }

    let addr = string_from_lengthen_ptr(text_service_listen_addr, text_service_listen_addr_len);

    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port,
        discovery_service_client_port,
        text_service_listen_addr: addr.clone(),
        text_service_listen_port,
        group_identity,
    };
    let airx = AirXService::new(&config);
    AIRX_SERVICE = match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    };

    info!("lib: AirX service created (addr={}:{},gid={})",
          addr, text_service_listen_port, group_identity);

    AIRX_SERVICE
}

#[export_name = "airx_restore"]
pub unsafe extern "C" fn airx_restore_service() -> *mut AirXService {
    AIRX_SERVICE
}

#[export_name = "airx_lan_discovery_service"]
pub extern "C" fn airx_lan_discovery_service(
    airx_ptr: *mut AirXService,
    should_interrupt: extern "C" fn() -> bool,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let service_disc = airx.discovery_service();
    let service_disc = service_disc.access();
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
}

// `&'static` mut is actually a pointer type.
#[export_name = "airx_lan_discovery_service_async"]
pub extern "C" fn airx_lan_discovery_service_async(
    airx_ptr: &'static mut AirXService,
    should_interrupt: extern "C" fn() -> bool,
) {
    let wrapper = PointerWrapper::new(airx_ptr);
    std::thread::spawn(move || {
        let airx_ptr = wrapper.get();
        let airx_ptr = unsafe { &mut *airx_ptr };
        airx_lan_discovery_service(airx_ptr, should_interrupt);
    });
}

#[export_name = "airx_text_service"]
pub extern "C" fn airx_text_service(
    airx_ptr: *mut AirXService,
    callback: extern "C" fn(*const c_char, u32),
    should_interrupt: extern "C" fn() -> bool,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let service_text = airx.text_service();
    let mut service_text = service_text.access();

    service_text.subscribe(Box::new(move |msg, _| {
        let c_str_ptr = msg.as_ptr();
        callback(c_str_ptr as *const c_char, msg.len() as u32); // u8 to i8
    }));

    let subscribers_ptr = service_text.subscribers();
    drop(service_text);

    info!("lib: Text service starting (addr={},port={})",
          config.text_service_listen_addr, config.text_service_listen_port);

    let _ = TextService::run(
        config.text_service_listen_addr.as_str(),
        config.text_service_listen_port,
        Box::new(move || should_interrupt()),
        subscribers_ptr,
    );
}

#[export_name = "airx_text_service_async"]
pub extern "C" fn airx_text_service_async(
    airx_ptr: &'static mut AirXService,
    callback: extern "C" fn(*const c_char, u32),
    should_interrupt: extern "C" fn() -> bool,
) {
    let wrapper = PointerWrapper::new(airx_ptr);
    std::thread::spawn(move || {
        let airx_ptr = wrapper.get();
        airx_text_service(airx_ptr, callback, should_interrupt);
    });
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

#[allow(deprecated)]
#[deprecated]
#[export_name = "airx_start_auto_broadcast"]
pub extern "C" fn airx_start_auto_broadcast(airx_ptr: &'static mut AirXService) {
    let airx = &mut *airx_ptr;
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));
        airx_lan_broadcast(airx);
    });
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
    let service_text = airx.text_service();
    let text = string_from_lengthen_ptr(text, text_len);
    let host = string_from_lengthen_ptr(host, host_len);

    info!("lib: Sending text to (addr={}:{})",
        host, config.text_service_listen_port);

    if let Ok(locked) = service_text.clone().lock() {
        let _ = locked.send(
            &Peer::new(&host, config.text_service_listen_port),
            config.text_service_listen_port,
            &text,
            Duration::from_millis(500),
        );
    }
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
    let service_text = airx.text_service();
    let service_disc = airx.discovery_service();

    let text = string_from_lengthen_ptr(text, len);

    if let Ok(locked) = service_disc.clone().lock() {
        if let Ok(peers_ptr) = locked.peers().lock() {
            for peer in peers_ptr.iter() {
                let thread_service_text = service_text.clone();
                let thread_peer = peer.clone();
                let thread_config = config.clone();
                let thread_text = text.clone();
                std::thread::spawn(move || {
                    if let Ok(locked) = thread_service_text.lock() {
                        info!("lib: Sending text to (addr={}:{})",
                            thread_peer.host(), thread_config.text_service_listen_port);
                        let _ = locked.send(
                            &thread_peer,
                            thread_config.text_service_listen_port,
                            &thread_text,
                            Duration::from_millis(500),
                        );
                    }
                });
            }
        }
    }
}
