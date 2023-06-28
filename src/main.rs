use std::collections::HashSet;
use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use log::info;
use libairx::{airx_init};
use libairx::network::peer::Peer;
use libairx::packet::data::file_coming_packet::FileComingPacket;
use libairx::packet::data::file_part_packet::FilePartPacket;
use libairx::packet::data::file_receive_response_packet::FileReceiveResponsePacket;
use libairx::packet::data::local::file_sending_packet::FileSendingPacket;
use libairx::packet::data::magic_numbers::MagicNumbers;
use libairx::packet::data::text_packet::TextPacket;
use libairx::packet::protocol::serialize::Serialize;
use libairx::service::context::data_service_context::DataServiceContext;
use libairx::service::data_service::DataService;

static mut INTERRUPT: bool = false;

unsafe fn should_interrupt() -> bool {
    INTERRUPT
}

#[allow(unused)]
fn main() {
    airx_init();
    let should_interrupt_callback = || unsafe { should_interrupt() };

    /////////////////////////////////////////////////////////////////////////////////////////////////////

    // t1
    let mut peers1 = HashSet::<Peer>::new();
    peers1.insert(Peer::new(&String::from("127.0.0.1"), 8003, Some(&String::from("t2"))));

    let data1 = std::thread::spawn(move || {
        let text_callback = move |packet: &TextPacket, socket_addr: &SocketAddr| {

        };

        let file_coming_callback = move |packet: &FileComingPacket, socket_addr: &SocketAddr| {

        };

        let file_sending_callback = move |packet: &FileSendingPacket, socket_addr: &SocketAddr| {

        };

        let file_part_callback = move |packet: &FilePartPacket, socket_addr: &SocketAddr| {

        };

        let context = DataServiceContext::new(
            String::from("0.0.0.0"),
            8001,
            Arc::new(Box::from(text_callback)),
            Arc::new(Box::from(file_coming_callback)),
            Arc::new(Box::from(file_sending_callback)),
            Arc::new(Box::from(file_part_callback)),
        );
        let _ = DataService::run(context, Box::from(should_interrupt_callback));
    });

    /////////////////////////////////////////////////////////////////////////////////////////////////////

    let mut peers2 = HashSet::<Peer>::new();
    peers2.insert(Peer::new(&String::from("127.0.0.1"), 8001, Some(&String::from("t1"))));

    let data2 = std::thread::spawn(move || {
        let text_callback = move |packet: &TextPacket, socket_addr: &SocketAddr| {

        };

        let file_coming_callback = move |packet: &FileComingPacket, socket_addr: &SocketAddr| {
            let packet = FileReceiveResponsePacket::new(
                1, packet.file_size(), packet.file_name().clone(), true);
            let _ = DataService::send_once_with_retry(
                &peers2.iter().next().unwrap(),
                8001,
                MagicNumbers::FileReceiveResponse,
                &packet.serialize(),
                Duration::from_millis(500),
            );
        };

        let file_sending_callback = move |packet: &FileSendingPacket, socket_addr: &SocketAddr| {

        };

        let file_part_callback = move |packet: &FilePartPacket, socket_addr: &SocketAddr| {
            let packet = packet;
            info!(">>>>> Received file part packet from {}.", socket_addr);
        };

        let context = DataServiceContext::new(
            String::from("0.0.0.0"),
            8003,
            Arc::new(Box::from(text_callback)),
            Arc::new(Box::from(file_coming_callback)),
            Arc::new(Box::from(file_sending_callback)),
            Arc::new(Box::from(file_part_callback)),
        );
        let _ = DataService::run(context, Box::from(should_interrupt_callback));
    });

    /////////////////////////////////////////////////////////////////////////////////////////////////////

    let file_path = String::from("D:\\test.txt");
    let file = File::open(&file_path).unwrap();
    let file_size = file.metadata().unwrap().len();

    let packet = FileComingPacket::new(file_size, file_path.clone());
    let _ = DataService::send_once_with_retry(
        &peers1.iter().next().unwrap(),
        8003,
        MagicNumbers::FileComing,
        &packet.serialize(),
        Duration::from_millis(500),
    );

    /////////////////////////////////////////////////////////////////////////////////////////////////////

    sleep(Duration::from_millis(50000));
    unsafe { INTERRUPT = true; }
    println!("Interrupted.");

    data1.join().unwrap();
    data2.join().unwrap();
}
