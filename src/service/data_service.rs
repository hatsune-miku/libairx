use crate::network::peer::Peer;
use crate::network::tcp_server::TcpServer;
use crate::packet::data_transmission::DataTransmit;
use std::io;
use std::io::ErrorKind::{TimedOut, WouldBlock};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use log::{info, trace, warn};
use crate::packet::data::magic_numbers::MagicNumbers;
use crate::packet::data_packet::DataPacket;
use crate::packet::protocol::serialize::Serialize;
use crate::service::context::data_service_context::DataServiceContext;
use crate::service::handler::{file_coming_packet_handler, file_part_packet_handler, file_receive_response_packet_handler, text_packet_handler, file_part_response_packet_handler};
use crate::service::handler::context::{HandlerContext, ConnectionControl};
use crate::service::ShouldInterruptFunctionType;

pub type OnPacketReceivedFunctionType<T, R> = Arc<Box<dyn Fn(&T, Option<&Peer>) -> R + Send + Sync>>;


const TCP_ACCEPT_WAIT_MILLIS: u64 = 10;
const TCP_ACCEPT_TIMEOUT_COUNT: u64 = 100;

pub struct DataService {}

impl DataService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn send_once_with_retry(
        peer: &Peer,
        port: u16,
        magic_number: MagicNumbers,
        data: &Vec<u8>,
        connect_timeout: Duration,
    ) -> Result<(), io::Error> {
        let stream = connect(peer, port, connect_timeout)?;
        info!("Connection established with {}.", peer.to_string());

        let mut dt = DataTransmit::from(stream);

        // Wrap with data packet.
        let data_packet = DataPacket::new(magic_number.value(), data);
        let result = dt.send_data_progress_with_retry(&data_packet.serialize(), |_| ());
        let _ = dt.close();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn data_session<State, F>(
        peer: &Peer,
        port: u16,
        connect_timeout: Duration,
        session: &mut F,
        reconnect_try_count: u32,
        mut state: State,
    ) -> Result<(), io::Error> where F: FnMut(&mut DataTransmit, &mut State) -> Result<(), io::Error> {
        let mut tries = 0;
        while tries < reconnect_try_count {
            let stream = connect(peer, port, connect_timeout)?;
            info!("Data session established with {}.", peer.to_string());

            let mut dt = DataTransmit::from(stream);

            match session(&mut dt, &mut state) {
                Ok(_) => {
                    let _ = dt.close();
                    return Ok(());
                }
                Err(e) => {
                    warn!("Data session error: {:?}. Retrying...", e);
                    tries += 1;
                }
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "Failed to establish data session."))
    }

    fn dispatch_data_packet(
        tt: &mut DataTransmit,
        packet: &DataPacket,
        socket_addr: SocketAddr,
        data_service_context: &DataServiceContext,
    ) -> ConnectionControl {
        let context = HandlerContext::new(tt, packet, socket_addr, data_service_context);
        match MagicNumbers::from(packet.magic_number()) {
            Some(MagicNumbers::Text) => text_packet_handler::handle(context),
            Some(MagicNumbers::FileComing) => file_coming_packet_handler::handle(context),
            Some(MagicNumbers::FileReceiveResponse) => file_receive_response_packet_handler::handle(context),
            Some(MagicNumbers::FilePart) => file_part_packet_handler::handle(context),
            Some(MagicNumbers::FilePartResponse) => file_part_response_packet_handler::handle(context),
            _ => {
                warn!("Unknown magic number.");
                ConnectionControl::CloseConnection
            }
        }
    }

    fn handle_peer(stream: TcpStream, context: DataServiceContext) {
        let socket_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(_) => {
                warn!("Failed to get peer address.");
                return;
            }
        };
        let mut tt = DataTransmit::from(stream);

        loop {
            let raw_data = match tt.read_data_progress_with_retry(|portion| {
                trace!("Received data {:.2}% from {}.", portion * 100.0, socket_addr);
            }) {
                Ok(s) => s,
                Err(_) => break,
            };

            let data_packet = match DataPacket::deserialize(&raw_data) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to deserialize data ({:?}).", e);
                    break;
                }
            };

            trace!("Received data packet from {}, magic_nubmer={}.", socket_addr, data_packet.magic_number());
            match Self::dispatch_data_packet(&mut tt, &data_packet, socket_addr, &context) {
                ConnectionControl::CloseConnection => break,
                ConnectionControl::Default => (),
            }
        }

        info!("Session with {} is ended.", socket_addr);
    }

    pub fn run(context: DataServiceContext, should_interrupt: ShouldInterruptFunctionType) -> Result<(), io::Error> {
        let server_socket = TcpServer::create_and_listen(&context.host(), context.port())?;
        let mut timeout_counter = 0;

        info!("Data service online and ready for connections.");

        for stream in server_socket.incoming() {
            match stream {
                Ok(s) => {
                    let thread_context = context.clone();
                    std::thread::spawn(move || {
                        Self::handle_peer(s, thread_context);
                    });
                }
                Err(ref e) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                    // Check if interrupted.
                    sleep(Duration::from_millis(TCP_ACCEPT_WAIT_MILLIS));

                    // Check if timeout.
                    if timeout_counter > TCP_ACCEPT_TIMEOUT_COUNT {
                        timeout_counter = 0;
                        if should_interrupt() {
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

fn connect(peer: &Peer, port: u16, timeout: Duration) -> Result<TcpStream, io::Error> {
    let addr = format!("{}:{}", peer.host(), port);
    let socket_addr = match addr.parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };
    TcpStream::connect_timeout(&socket_addr, timeout)
}
