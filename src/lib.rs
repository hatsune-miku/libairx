extern crate core;

use lib_util::string_from_lengthed_ptr;

use crate::lib_util::PointerWrapper;
use crate::network::peer::Peer;
use crate::service::airx_service::AirXService;
use crate::service::discovery_service::DiscoveryService;
use crate::service::text_service::TextService;
use std::os::raw::c_char;
use std::ptr::copy;
use std::time::Duration;

mod lib_util;
mod network;
mod service;
mod transmission;
mod util;

static mut FIRST_RUN: bool = true;
static mut AIRX_SERVICE: *mut AirXService = std::ptr::null_mut();

#[allow(dead_code)]
#[export_name = "airx_version"]
pub extern "C" fn airx_version() -> i32 {
    20230129
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[export_name = "airx_create"]
pub unsafe extern "C" fn airx_create_service(
    discovery_service_server_port: u16,
    discovery_service_client_port: u16,
    text_service_listen_addr: *mut c_char,
    text_service_listen_addr_len: u32,
    text_service_listen_port: u16,
) -> *mut AirXService {
    let addr = string_from_lengthed_ptr(text_service_listen_addr, text_service_listen_addr_len);

    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port,
        discovery_service_client_port,
        text_service_listen_addr: addr,
        text_service_listen_port,
    };
    let airx = AirXService::new(&config);
    AIRX_SERVICE = match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    };
    AIRX_SERVICE
}

#[allow(dead_code)]
#[export_name = "airx_restore"]
pub unsafe extern "C" fn airx_restore_service() -> *mut AirXService {
    AIRX_SERVICE
}

#[allow(dead_code)]
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

    let _ = DiscoveryService::run(
        config.discovery_service_server_port,
        peers_ptr,
        Box::new(move || should_interrupt()),
    );
}

// `&'static` mut is actually a pointer type.
#[allow(dead_code)]
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

#[allow(dead_code)]
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

    let _ = TextService::run(
        config.text_service_listen_addr.as_str(),
        config.text_service_listen_port,
        Box::new(move || should_interrupt()),
        subscribers_ptr,
    );
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[export_name = "airx_lan_broadcast"]
pub extern "C" fn airx_lan_broadcast(airx_ptr: *mut AirXService) -> bool {
    let airx = unsafe { &mut *airx_ptr };
    let service_disc = airx.discovery_service();
    let x = if let Ok(locked) = service_disc.lock() {
        locked.broadcast_discovery_request().is_ok()
    } else {
        false
    };
    x
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[export_name = "airx_start_auto_broadcast"]
pub extern "C" fn airx_start_auto_broadcast(airx_ptr: &'static mut AirXService) {
    let airx = &mut *airx_ptr;
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));
        airx_lan_broadcast(airx);
    });
}

#[allow(dead_code)]
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
    let text = string_from_lengthed_ptr(text, text_len);
    let host = string_from_lengthed_ptr(host, host_len);

    if let Ok(locked) = service_text.clone().lock() {
        let _ = locked.send(
            &Peer::new(&host, config.text_service_listen_port),
            config.text_service_listen_port,
            &text,
            Duration::from_secs(1),
        );
    }
}

#[allow(dead_code)]
#[export_name = "airx_broadcast_text"]
pub extern "C" fn airx_broadcast_text(
    airx_ptr: *mut AirXService,
    text: *mut c_char,
    len: u32,
) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();
    let service_text = airx.text_service();
    let service_disc = airx.discovery_service();

    let text = string_from_lengthed_ptr(text, len);

    if let Ok(locked) = service_disc.clone().lock() {
        if let Ok(peers_ptr) = locked.peers().lock() {
            for peer in peers_ptr.iter() {
                if let Ok(locked) = service_text.lock() {
                    let _ = locked.send(
                        peer,
                        config.text_service_listen_port,
                        &text,
                        Duration::from_secs(1),
                    );
                }
            }
        }
    }
}
