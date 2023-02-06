use airx::service::airx_service::{AirXService, AirXServiceConfig};
use airx::service::discovery_service::DiscoveryService;
use airx::service::text_service::TextService;

fn test() {
    let config = AirXServiceConfig {
        discovery_service_server_port: 9818,
        discovery_service_client_port: 0,
        text_service_listen_addr: String::from("0.0.0.0"),
        text_service_listen_port: 9819,
        group_identity: 0,
    };
    let airx = AirXService::new(&config).expect("Failed to create AirX service.");
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
}

fn main() {
    test();
}

