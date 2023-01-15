mod network;
mod transmission;
mod util;

use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use crate::network::discovery_service::DiscoveryService;

fn main() {
    let mut service1 = match DiscoveryService::new(9818, 12355) {
        Ok(service) => service,
        Err(_) => return
    };
    let mut service2 = match DiscoveryService::new(9819, 12356) {
        Ok(service) => service,
        Err(_) => return
    };

    let handler = |s: &SocketAddr| println!("Received: {}", s.ip().to_string());
    service1.start(handler).expect("Failed to start service #1.");
    service2.start(handler).expect("Failed to start service #2.");
    sleep(Duration::from_secs(64));
}
