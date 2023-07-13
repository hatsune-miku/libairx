use std::net::{Ipv4Addr, SocketAddrV4};

#[test]
fn test_socket_addr_to_string() {
    let addr = SocketAddrV4::new(
        Ipv4Addr::new(10, 10, 10, 9), 1145);
    assert_eq!(addr.ip().to_string(), "10.10.10.9");
}
