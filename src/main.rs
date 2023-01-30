mod network;
mod service;
mod transmission;
mod util;

use std::borrow::Borrow;
use service::airx_service;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use crate::network::discovery_service::DiscoveryService;
use crate::service::text_service::TextService;
use crate::util::shared_mutable::SharedMutable;


fn test() {
    let config = airx_service::AirXServiceConfig {
        discovery_service_server_port: 9818,
        discovery_service_client_port: 0,
        text_service_listen_addr: "0.0.0.0",
        text_service_listen_port: 9819,
    };
    let airx = airx_service::AirXService::new(&config)
        .expect("Failed to create AirX service.");
    let service_disc = airx.discovery_service();
    let service_text = airx.text_service();

    let peers_ptr = service_disc.access().peers();
    let subscribers_ptr = service_text.access().subscribers();

    std::thread::spawn(move || {
        println!("Discovery service started.");
        let _ = DiscoveryService::run(
            config.discovery_service_server_port,
            peers_ptr,
        );
        println!("Discovery service stopped.")
    });

    std::thread::spawn(move || {
        println!("Text service started.");
        let _ = TextService::run(
            config.text_service_listen_addr,
            config.text_service_listen_port,
            subscribers_ptr,
        );
        println!("Text service stopped.")
    });

    loop {
        sleep(Duration::from_secs(1));

        println!("Peers:");
        service_disc.access().peers().access().iter().for_each(|peer| {
            println!("Peer: {}", peer);
        });

        service_disc.access().broadcast_discovery_request()
            .expect("Failed to broadcast discovery request.");
    }
}

fn main() {
    test();
}
