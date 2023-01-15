mod network;
mod transmission;
mod util;

use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use crate::network::discovery_service::DiscoveryService;

fn handler(s: &SocketAddr) {
    println!("Received: {}", s.ip().to_string());
}

fn main() {
    let mut service = DiscoveryService::new(9818, 12355)
        .expect("Failed to create discovery service");

    service.start(handler)
        .expect("Failed to start service.");

    sleep(Duration::from_secs(64));
}
