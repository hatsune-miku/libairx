extern crate core;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::copy;
use crate::network::discovery_service::DiscoveryService;
use crate::service::airx_service::AirXService;
use crate::service::text_service::TextService;

mod network;
mod service;
mod transmission;
mod util;

#[export_name = "airx_version"]
pub extern "C" fn airx_version() -> i32 {
    1
}

#[export_name = "airx_create"]
pub extern "C" fn airx_create_service(
    discovery_service_server_port: u16,
    discovery_service_client_port: u16,
    text_service_listen_addr: *mut c_char,
    text_service_listen_port: u16,
) -> *mut AirXService<'static> {
    let addr = unsafe { CStr::from_ptr(text_service_listen_addr) };

    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port,
        discovery_service_client_port,
        text_service_listen_addr: addr.to_str().unwrap(),
        text_service_listen_port,
    };
    let airx = AirXService::new(&config);
    match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[export_name = "airx_lan_discovery_service"]
pub extern "C" fn airx_lan_discovery_service(airx_ptr: *mut AirXService) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let service_disc = airx.discovery_service();
    let service_disc = service_disc.access();
    let peers_ptr = service_disc.peers();

    drop(service_disc);

    let _ = DiscoveryService::run(
        config.discovery_service_server_port,
        peers_ptr,
    );
}

#[export_name = "airx_text_service"]
pub extern "C" fn airx_text_service(airx_ptr: *mut AirXService) {
    let airx = unsafe { &mut *airx_ptr };
    let config = airx.config();

    let service_text = airx.text_service();
    let service_text = service_text.access();
    let subscribers_ptr = service_text.subscribers();

    drop(service_text);

    let _ = TextService::run(
        config.text_service_listen_addr,
        config.text_service_listen_port,
        subscribers_ptr,
    );
}

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

#[export_name = "airx_get_peers"]
pub extern "C" fn airx_get_peers(airx_ptr: *mut AirXService, buffer: *mut c_char) -> usize {
    let airx = unsafe { &mut *airx_ptr };
    let service_disc = airx.discovery_service();

    if let Ok(locked) = service_disc.lock() {
        if let Ok(peers_ptr) = locked.peers().lock() {
            let joined = peers_ptr.iter().map(|peer| peer.to_string())
                .collect::<Vec<String>>()
                .join(",");
            let bytes = joined.as_bytes();

            unsafe {
                copy(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            }
            return bytes.len();
        }
    }
    0
}
