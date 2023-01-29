mod network;
mod service;
mod transmission;
mod util;

use std::borrow::Borrow;
use service::airx_service;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use crate::util::shared_mutable::SharedMutable;

fn main() {
    let config = airx_service::AirXServiceConfig {
        discovery_service_server_port: 9818,
        discovery_service_client_port: 0,
        text_service_listen_addr: "0.0.0.0",
        text_service_listen_port: 9819,
    };
    let airx = airx_service::AirXService::new(&config)
        .expect("Failed to create AirX service.");
    let airx = SharedMutable::new(airx);
    let airx_ref_disc = airx.clone();
    let airx_ref_text = airx.clone();

    std::thread::spawn(move || {
        let _ = airx_ref_disc.access().run_discovery_service_sync();
    });

    std::thread::spawn(move || {
        let _ = airx_ref_text.access().run_text_service_sync();
    });

    let disc = airx.access().discovery_service();
    let disc = disc.access();

    disc.broadcast_discovery_request()
        .expect("Failed to broadcast discovery request.");

    sleep(Duration::from_secs(1));


    loop {
        sleep(Duration::from_secs(1));

        println!("Peers:");
        disc.get_peer_list().unwrap().iter().for_each(|peer| {
            println!("Peer: {}", peer);
        });
    }
}
