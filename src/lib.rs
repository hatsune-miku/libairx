extern crate core;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::copy;
use std::thread::Thread;
use std::time::Duration;
use crate::network::discovery_service::DiscoveryService;
use crate::service::airx_service::AirXService;
use crate::service::text_service::TextService;

mod network;
mod service;
mod transmission;
mod util;

static mut FIRST_RUN: bool = true;
static mut AIRX_SERVICE: *mut AirXService = std::ptr::null_mut();

struct PointerWrapper<T> {
    ptr: *mut T,
}

unsafe impl<T> Send for PointerWrapper<T> {}

unsafe impl<T> Sync for PointerWrapper<T> {}

impl<T> Clone for PointerWrapper<T> {
    fn clone(&self) -> Self {
        PointerWrapper { ptr: self.ptr }
    }
}

impl<T> PointerWrapper<T> {
    fn new(ptr: *mut T) -> Self {
        PointerWrapper { ptr }
    }

    fn get(&self) -> *mut T {
        self.ptr
    }
}


#[export_name = "airx_version"]
pub extern "C" fn airx_version() -> i32 {
    20230129
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
    text_service_listen_port: u16,
) -> *mut AirXService<'static> {
    let addr = CStr::from_ptr(text_service_listen_addr);

    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port,
        discovery_service_client_port,
        text_service_listen_addr: addr.to_str().unwrap(),
        text_service_listen_port,
    };
    let airx = AirXService::new(&config);
    AIRX_SERVICE = match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    };
    AIRX_SERVICE
}

#[export_name = "airx_restore"]
pub unsafe extern "C" fn airx_restore_service() -> *mut AirXService<'static> {
    AIRX_SERVICE
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

// `&'static` mut is actually a pointer type.
#[export_name = "airx_lan_discovery_service_async"]
pub extern "C" fn airx_lan_discovery_service_async(airx_ptr: &'static mut AirXService) {
    let wrapper = PointerWrapper::new(airx_ptr);
    std::thread::spawn(move || {
        let airx_ptr = wrapper.get();
        let airx_ptr = unsafe { &mut *airx_ptr };
        airx_lan_discovery_service(airx_ptr);
    });
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

#[export_name = "airx_text_service_async"]
pub extern "C" fn airx_text_service_async(airx_ptr: &'static mut AirXService) {
    let wrapper = PointerWrapper::new(airx_ptr);
    std::thread::spawn(move || {
        let airx_ptr = wrapper.get();
        airx_text_service(airx_ptr);
    });
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
pub extern "C" fn airx_get_peers(airx_ptr: *mut AirXService, buffer: *mut c_char) -> u32 {
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
            return bytes.len() as u32;
        }
    }
    0
}

#[export_name = "airx_start_auto_broadcast"]
pub extern "C" fn airx_start_auto_broadcast(airx_ptr: &'static mut AirXService) {
    let airx = unsafe { &mut *airx_ptr };
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(2));
            airx_lan_broadcast(airx);
        }
    });
}
