extern crate jni;

use crate::network::peer::{Peer};
use crate::service::airx_service::{AirXService};
use crate::service::discovery_service::DiscoveryService;
use std::sync::{Arc};
use std::time::Duration;
use android_logger::{Config, FilterBuilder};
use jni::objects::{JObject, JValue};
use jni::sys::{jboolean, jint, jlong, jshort};
use log::{error, info, LevelFilter};
use crate::lib_util::{AIRX_COMPATIBLE_NUMBER, AIRX_VERSION, shared_airx_version_code, CONNECTION_TIMEOUT_MILLIS, shared_airx_init, shared_airx_broadcast_text, shared_airx_try_send_file, shared_airx_respond_to_file, shared_airx_data_service};
use crate::packet::data::file_coming_packet::FileComingPacket;
use crate::packet::data::file_part_packet::FilePartPacket;
use crate::packet::data::local::file_sending_packet::FileSendingPacket;
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data::text_packet::TextPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service;
use crate::service::context::data_service_context::DataServiceContext;
use crate::service::data_service::{DataService};

use self::jni::JNIEnv;
use self::jni::objects::{JClass, JString};
use self::jni::sys::jstring;

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXVersion(
    _: JNIEnv,
    _: JClass,
) -> i32 {
    AIRX_VERSION
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXCompatibilityNumber(
    _: JNIEnv,
    _: JClass,
) -> i32 {
    AIRX_COMPATIBLE_NUMBER
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXVersionString(
    env: JNIEnv,
    _: JClass,
) -> jstring {
    let version = shared_airx_version_code();
    env.new_string(version).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXInit(
    _: JNIEnv,
    _: JClass,
) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libairx4a")
            .with_filter(
                FilterBuilder::new()
                    .parse("debug,hello::crate=error")
                    .build(),
            ),
    );

    shared_airx_init()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_airx_AirXBridge_airXCreateService(
    mut env: JNIEnv,
    _: JClass,
    discovery_service_server_port: jshort,
    discovery_service_client_port: jshort,
    text_service_listen_addr: JString,
    text_service_listen_port: jshort,
    group_identifier: jint,
) -> u64 {
    let addr: String = env.get_string(text_service_listen_addr.as_ref()).unwrap().into();
    let config = service::airx_service::AirXServiceConfig {
        discovery_service_server_port: discovery_service_server_port as u16,
        discovery_service_client_port: discovery_service_client_port as u16,
        text_service_listen_addr: addr.clone(),
        data_service_listen_port: text_service_listen_port as u16,
        group_identifier: group_identifier as u32,
    };
    let airx = AirXService::new(&config);
    let airx = match airx {
        Ok(airx) => Box::into_raw(Box::new(airx)),
        Err(_) => std::ptr::null_mut(),
    };

    info!("lib: AirX config created (addr={}:{},gid={})",
          addr, text_service_listen_port, group_identifier);

    airx as u64
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXLanDiscoveryService(
    _: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
) {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
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
        Box::new(|| false),
        config.group_identifier,
    );

    info!("lib: Discovery service stopped.");
}

//noinspection Duplicates
#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXDataService(
    env: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
) {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let config = airx.config();

    let jvm = Arc::new(env.get_java_vm().unwrap());

    let call_text_callback_jvm = jvm.clone();
    let call_text_callback = move |text: String, socket_address: String| {
        let mut env = call_text_callback_jvm.attach_current_thread().unwrap();
        let text = env.new_string(text).unwrap();
        let socket_address = env.new_string(socket_address).unwrap();
        env.call_static_method(
            "com/airx/AirXBridge",
            "onTextPacketReceived",
            "(Ljava/lang/String;Ljava/lang/String)V",
            &[
                JValue::Object(JObject::from(text).as_ref()),
                JValue::Object(JObject::from(socket_address).as_ref()),
            ],
        ).expect("Unable to call method onTextPacketReceived");
    };

    let call_file_coming_callback_jvm = jvm.clone();
    let call_file_coming_callback = move |file_size: u64, remote_full_path: String, socket_address: String| {
        let mut env = call_file_coming_callback_jvm.attach_current_thread().unwrap();
        let remote_full_path = env.new_string(remote_full_path).unwrap();
        let socket_address = env.new_string(socket_address).unwrap();
        env.call_static_method(
            "com/airx/AirXBridge",
            "onFileComingPacketReceived",
            "(JLjava/lang/String;Ljava/lang/String)V",
            &[
                JValue::Long(file_size as jlong),
                JValue::Object(JObject::from(remote_full_path).as_ref()),
                JValue::Object(JObject::from(socket_address).as_ref()),
            ],
        ).expect("Unable to call method onFileComingPacketReceived");
    };

    let call_file_sending_callback_jvm = jvm.clone();
    let call_file_sending_callback = move |file_id: u8, progress: u64, total: u64, status: u8| {
        let mut env = call_file_sending_callback_jvm.attach_current_thread().unwrap();
        env.call_static_method(
            "com/airx/AirXBridge",
            "onFileSendingPacketReceived",
            "(SJJS)V",
            &[
                JValue::Short(file_id as jshort),
                JValue::Long(progress as jlong),
                JValue::Short(status as jshort),
                JValue::Long(total as jlong),
            ],
        ).expect("Unable to call method onFileSendingPacketReceived");
    };

    let call_file_part_callback_jvm = jvm.clone();
    let call_file_part_callback = move |file_id: u8, offset: u64, length: u64, data: Vec<u8>| {
        let mut env = call_file_part_callback_jvm.attach_current_thread().unwrap();
        let data = env.byte_array_from_slice(&data).unwrap();
        env.call_static_method(
            "com/airx/AirXBridge",
            "onFilePartPacketReceived",
            "(SJJ[B)V",
            &[
                JValue::Short(file_id as jshort),
                JValue::Long(offset as jlong),
                JValue::Long(length as jlong),
                JValue::Object(JObject::from(data).as_ref()),
            ],
        ).expect("Unable to call method onFilePartPacketReceived");
    };

    let text_callback = move |text_packet: &TextPacket, peer: Option<&Peer>| {
        let socket_addr_str = match peer {
            Some(p) => p.to_string(),
            None => Peer::default().to_string(),
        };
        call_text_callback(text_packet.text().to_string(), socket_addr_str);
    };

    let file_coming_callback = move |file_coming_packet: &FileComingPacket, peer: Option<&Peer>| {
        let socket_addr_str = match peer {
            Some(p) => p.to_string(),
            None => Peer::default().to_string(),
        };
        call_file_coming_callback(
            file_coming_packet.file_size(),
            file_coming_packet.file_name().to_string(),
            socket_addr_str,
        );
    };

    let file_sending_callback = move |file_sending_packet: &FileSendingPacket, _: Option<&Peer>| {
        call_file_sending_callback(
            file_sending_packet.file_id(),
            file_sending_packet.progress(),
            file_sending_packet.total(),
            file_sending_packet.status().to_u8(),
        );
    };

    let file_part_callback = move |file_part_packet: &FilePartPacket, _: Option<&Peer>| -> bool {
        let data = file_part_packet.data().clone();

        call_file_part_callback(
            file_part_packet.file_id(),
            file_part_packet.offset(),
            file_part_packet.length(),
            data,
        );
        false
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

    shared_airx_data_service(context, &config, Box::new(|| false));
}

#[deprecated]
#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXLanBroadcast(
    _: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
) -> jboolean {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let config = airx.config();
    match DiscoveryService::broadcast_discovery_request(
        config.discovery_service_client_port,
        config.discovery_service_server_port,
        config.group_identifier,
    ) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXGetPeers(
    env: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
) -> jstring {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let service_disc = airx.discovery_service();

    if let Ok(peers_ptr) = service_disc.peers().lock() {
        let joined = peers_ptr
            .iter()
            .map(|peer| peer.to_string())
            .collect::<Vec<String>>()
            .join(",");

        info!("lib: Get peers (peers={})", joined);
        return env.new_string(joined).unwrap().into_raw();
    }
    error!("lib: Failed to get peers");
    env.new_string("").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXSendText(
    mut env: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
    host: JString,
    text: JString,
) {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let config = airx.config();

    let host = env.get_string(host.as_ref()).expect("Couldn't get java string").into();
    let text = env.get_string(text.as_ref()).expect("Couldn't get java string").into();

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
        Duration::from_millis(CONNECTION_TIMEOUT_MILLIS),
    );
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXBroadcastText(
    mut env: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
    text: JString,
) {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let config = airx.config();
    let service_disc = airx.discovery_service();
    let text = env.get_string(text.as_ref()).expect("Couldn't get java string").into();

    shared_airx_broadcast_text(text, service_disc.clone(), &config);
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXTrySendFile(
    mut env: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
    host: JString,
    file_path: JString,
) {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let config = airx.config();
    let host = env.get_string(host.as_ref()).expect("Couldn't get java string").into();
    let file_path = env.get_string(file_path.as_ref()).expect("Couldn't get java string").into();

    shared_airx_try_send_file(host, file_path, &config);
}

#[no_mangle]
pub extern "C" fn Java_com_airx_AirXBridge_airXRespondToFile(
    mut env: JNIEnv,
    _: JClass,
    airx_ptr: jlong,
    host: JString,
    file_id: jshort,
    file_size: jlong,
    file_path: JString,
    accept: jboolean,
) {
    let airx = unsafe { &mut *(airx_ptr as *mut AirXService) };
    let config = airx.config();
    let host = env.get_string(host.as_ref()).expect("Couldn't get java string").into();
    let file_path = env.get_string(file_path.as_ref()).expect("Couldn't get java string").into();
    let accept = match accept {
        0 => false,
        _ => true,
    };

    shared_airx_respond_to_file(host, file_id as u8, file_size as u64, file_path, accept, &config);
}
