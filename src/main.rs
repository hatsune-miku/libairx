mod network;
mod transmission;
mod util;

use std::io;
use std::io::Error;
use std::sync::{Arc, Mutex};
use crate::network::discovery_service;
use std::thread::sleep;
use std::time::Duration;
use clipboard_master::{CallbackResult, ClipboardHandler};
use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::transmission::protocol::text_transmission::{ReadText, SendText};

type DiscoveryServiceType = Arc<Mutex<discovery_service::DiscoveryService>>;

struct Handler {
    clipboard: arboard::Clipboard,
    service_ref: DiscoveryServiceType,
    last_text: String,
}

impl Handler {
    fn new(service_ref: DiscoveryServiceType) -> Result<Self, arboard::Error> {
        Ok(
            Self {
                clipboard: arboard::Clipboard::new()?,
                last_text: String::new(),
                service_ref,
            }
        )
    }
}

fn send_sync_request(peer: &Peer, text: &String) -> Result<(), io::Error> {
    let mut socket = Socket::connect(peer.host(), 9819u16)?;
    let mut tt = transmission::text::TextTransmission::from(&mut socket);
    tt.send_text(&*String::from(format!("SYNC:{}", text)))?;
    Ok(())
}

fn server_routine() -> Result<(), io::Error> {
    let mut server_socket = TcpServer::create_and_listen(
        "0.0.0.0", 9819u16,
    )?;
    println!("Listening for incoming clipboard sync...");
    while let Ok((stream, socket_addr)) = server_socket.accept() {
        let mut socket = Socket::from(stream);
        let mut tt = transmission::text::TextTransmission::from(&mut socket);
        if let Ok(s) = tt.read_text() {
            if (!s.starts_with("SYNC:")) {
                continue;
            }
            let text = s.split(":").collect::<Vec<&str>>()[1..].join("");
            println!("Received clipboard from {}: {}", socket_addr.to_string(), text);
            if let Ok(mut clip) = arboard::Clipboard::new() {
                let _ = clip.set_text(text);
            }
        }
    };
    Ok(())
}

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let text = match self.clipboard.get_text() {
            Ok(s) => s,
            Err(_) => return CallbackResult::Next,
        };
        if text == self.last_text {
            return CallbackResult::Next;
        }
        self.last_text = text.clone();
        println!("Clipboard changed: {}", text);

        if let Ok(locked) = self.service_ref.lock() {
            if let Ok(list) = locked.get_peer_list() {
                list.iter().for_each(|peer| {
                    println!("Sending to {}", peer);
                    let _ = send_sync_request(peer, &text);
                });
            }
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: Error) -> CallbackResult {
        println!("Error: {}", error);
        CallbackResult::Next
    }
}

fn clipboard_setup(service_ref: DiscoveryServiceType) {
    std::thread::spawn(|| {
        let mut handler = Handler::new(service_ref).unwrap();
        let mut master = clipboard_master::Master::new(handler);
        master.run().expect("Failed to run clipboard master");
    });

    std::thread::spawn(|| {
        let _ = server_routine();
    });
}

fn main() {
    // 一段儿临时代码，就不顾安全和稳定了
    // unwrap expect什么的全都用上

    let mut service = discovery_service::DiscoveryService::new(9818, 0)
        .expect("Failed to create discovery service");
    let service_ref: DiscoveryServiceType = Arc::new(Mutex::new(service));

    service_ref.lock().unwrap().start()
        .expect("Failed to start service.");

    println!("Discovery service started.");

    clipboard_setup(service_ref.clone());
    println!("Clipboard service started.");

    loop {
        sleep(Duration::from_secs(1));
        println!(
            "Peer list: {}",
            service_ref.lock().unwrap()
                .get_peer_list()
                .unwrap()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
}
