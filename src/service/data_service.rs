use crate::network::peer::Peer;
use crate::network::socket::Socket;
use crate::network::tcp_server::TcpServer;
use crate::packet::data_transmission::DataTransmission;
use std::io;
use std::io::ErrorKind::{TimedOut, WouldBlock};
use std::net::{SocketAddr, TcpStream};
use std::thread::sleep;
use std::time::Duration;
use log::{info, warn};
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::data::{ReadDataWithRetry, SendDataWithRetry};
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;
use crate::service::handler::{file_coming_packet_handler, file_part_packet_handler, file_receive_response_packet_handler, text_packet_handler};

pub type OnPacketReceivedFunctionType<T> = Box<dyn Fn(&T, &SocketAddr) + Send + Sync>;


const TCP_ACCEPT_WAIT_MILLIS: u64 = 10;
const TCP_ACCEPT_TIMEOUT_COUNT: u64 = 100;

#[allow(dead_code)]
pub struct DataService {

}

#[allow(dead_code)]
impl DataService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn send_data_with_retry(
        peer: &Peer,
        port: u16,
        magic_number: MagicNumbers,
        data: &Vec<u8>,
        connect_timeout: Duration,
    ) -> Result<(), io::Error> {
        let mut socket = Socket::connect(peer.host(), port, connect_timeout)?;
        let mut dt = DataTransmission::from(&mut socket);

        // Wrap with data packet.
        let data_packet = DataPacket::new(magic_number.value(), data);
        let result = dt.send_data_with_retry(&data_packet.serialize());
        let _ = socket.close();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn data_session<F>(
        peer: &Peer,
        port: u16,
        connect_timeout: Duration,
        session: &mut F,
    ) -> Result<(), io::Error> where F: FnMut(&mut DataTransmission) {
        let mut socket = Socket::connect(peer.host(), port, connect_timeout)?;
        let mut dt = DataTransmission::from(&mut socket);

        session(&mut dt);

        let _ = socket.close();
        Ok(())
    }

    fn dispatch_data_packet(
        packet: &DataPacket,
        socket_addr: SocketAddr,
        context: &DataServiceContext
    ) {
        match MagicNumbers::from(packet.magic_number()) {
            Some(MagicNumbers::Text) => text_packet_handler::handle(packet, &socket_addr, context),
            Some(MagicNumbers::FileComing) => file_coming_packet_handler::handle(packet, &socket_addr, context),
            Some(MagicNumbers::FileReceiveResponse) => file_receive_response_packet_handler::handle(packet, &socket_addr, context),
            Some(MagicNumbers::FilePart) => file_part_packet_handler::handle(packet, &socket_addr, context),
            _ => warn!("Unknown magic number.")
        }
    }

    fn handle_peer(stream: TcpStream, context: &DataServiceContext) {
        let socket_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(_) => {
                warn!("Failed to get peer address.");
                return;
            }
        };
        let mut socket = Socket::from(stream);
        let mut tt = DataTransmission::from(&mut socket);

        let raw_data = match tt.read_data_with_retry() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to read data ({}).", e);
                return;
            }
        };

        let data_packet = match DataPacket::deserialize(&raw_data) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to deserialize data ({:?}).", e);
                return;
            }
        };

        Self::dispatch_data_packet(&data_packet, socket_addr, context);
    }

    #[allow(unused_assignments)]
    pub fn run(context: DataServiceContext) -> Result<(), io::Error> {
        let server_socket = TcpServer::create_and_listen(&context.host(), context.port())?;
        let mut timeout_counter = 0;

        info!("Data service online and ready for connections.");

        for stream in server_socket.incoming() {
            match stream {
                Ok(s) => {
                    Self::handle_peer(s, &context);
                }
                Err(ref e) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                    // Check if interrupted.
                    sleep(Duration::from_millis(TCP_ACCEPT_WAIT_MILLIS));

                    // Check if timeout.
                    if timeout_counter > TCP_ACCEPT_TIMEOUT_COUNT {
                        timeout_counter = 0;
                        if (context.should_interrupt())() {
                            info!("Data service is interrupted by caller.");
                            break;
                        }
                    }

                    timeout_counter += 1;
                    continue;
                }
                Err(_) => {
                    break;
                }
            }
        }

        Ok(())
    }
}
