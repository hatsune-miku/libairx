use std::net::Ipv4Addr;
use libairx::packet::discovery_packet::DiscoveryPacket;
use libairx::packet::protocol::serialize::Serialize;

#[test]
fn test_discovery_packet_serializable() {
    let packet = DiscoveryPacket::new(
        Ipv4Addr::new(114, 51, 41, 91),
        9818,
        0,
        true,
    );
    let bytes = packet.serialize();
    let packet2 = DiscoveryPacket::deserialize(bytes).unwrap();

    assert_eq!(packet2.server_port(), 9818);
    assert_eq!(packet2.group_identity(), 0);
    assert_eq!(packet2.sender_address(), Ipv4Addr::new(114, 51, 41, 91));
    assert_eq!(packet2.need_response(), true);
}
