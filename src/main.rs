mod network;
mod transmission;
mod util;

use crate::network::discovery_service::DiscoveryService;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // 一段儿临时代码，就不顾安全和稳定了
    // unwrap expect什么的全都用上

    let mut service = DiscoveryService::new(9818, 0)
        .expect("Failed to create discovery service");

    service.start()
        .expect("Failed to start service.");

    println!("Discovery service started.");

    loop {
        sleep(Duration::from_secs(1));
        println!(
            "Peer list: {}",
            service
                .get_peer_list()
                .unwrap()
                .iter()
                .map(|x| x.ip().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
}
