mod network;
mod transmission;
mod util;

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;
    use crate::network;
    use crate::network::discovery_service::DiscoveryService;
    use crate::network::tcp_server;
    use crate::network::socket;
    use crate::transmission::text::TextTransmission;
    use crate::transmission::protocol::text_transmission::SendText;
    use crate::transmission::protocol::text_transmission::ReadText;

    fn server_thread() {
        let mut socket_server = tcp_server::TcpServer::new();
        let pool = threadpool::Builder::new()
            .num_threads(64)
            .thread_stack_size(1024 * 1024 * 1024)
            .thread_name(String::from("张三"))
            .build();

        socket_server.listen("0.0.0.0", 6464).unwrap();

        let stream = match socket_server.accept() {
            Ok(stream) => stream,
            Err(_) => return
        };

        pool.execute(move || {
            let mut socket = socket::Socket::from(stream);
            let mut tt = TextTransmission::from(&mut socket);

            tt.send_text("Hi there! 👏 \\^O^/").unwrap();
        });
    }

    fn client_thread() {
        let mut socket = socket::Socket::new();
        socket.connect("127.0.0.1", 6464).unwrap();
        let mut tt = TextTransmission::from(&mut socket);
        println!("Server said: {}", tt.read_text().unwrap());
    }

    #[test]
    fn it_works() {
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
}
