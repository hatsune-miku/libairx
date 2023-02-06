extern crate core;

mod network;
mod service;
mod packet;
mod util;
mod compatibility;

use crate::service::discovery_service::DiscoveryService;
use crate::service::text_service::TextService;
use service::airx_service;
use std::thread::sleep;
use std::time::Duration;

fn test() {
    let config = airx_service::AirXServiceConfig {
        discovery_service_server_port: 9818,
        discovery_service_client_port: 0,
        text_service_listen_addr: String::from("0.0.0.0"),
        text_service_listen_port: 9819,
        group_identity: 0,
    };
    let airx = airx_service::AirXService::new(&config).expect("Failed to create AirX service.");
    let service_disc = airx.discovery_service();
    let service_text = airx.text_service();

    let peers_ptr = service_disc.access().peers();
    let subscribers_ptr = service_text.access().subscribers();

    std::thread::spawn(move || {
        println!("Discovery service started.");
        let _ = DiscoveryService::run(
            config.discovery_service_client_port,
            config.discovery_service_server_port,
            peers_ptr,
            Box::new(|| true),
            config.group_identity,
        );
        println!("Discovery service stopped.")
    });

    std::thread::spawn(move || {
        println!("Text service started.");
        let _ = TextService::run(
            config.text_service_listen_addr.as_str(),
            config.text_service_listen_port,
            Box::new(|| true),
            subscribers_ptr,
        );
        println!("Text service stopped.")
    });

    loop {
        sleep(Duration::from_secs(1));

        println!("Peers:");
        service_disc
            .access()
            .peers()
            .access()
            .iter()
            .for_each(|peer| {
                println!("Peer: {}", peer);
            });
    }
}

fn main() {
    test();
}
