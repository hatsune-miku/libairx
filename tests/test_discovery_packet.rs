use std::net::Ipv4Addr;
use protobuf::Message;
use airx::extension::ip_to_u32::ToU32;
use airx::proto::discovery_packet::DiscoveryPacket;

#[test]
fn test_discovery_packet_serializable() {
    let mut packet = DiscoveryPacket::new();
    packet.set_address(Ipv4Addr::new(114, 51, 41, 91).to_u32());
    packet.set_server_port(9818);
    packet.set_group_identifier(0);
    packet.set_need_response(true);
    packet.set_host_name(String::from("嘟嘟嘟"));

    let bytes = packet.write_to_bytes().unwrap();
    let packet2 = DiscoveryPacket::parse_from_bytes(bytes.as_slice()).unwrap();

    assert_eq!(packet2.server_port(), 9818);
    assert_eq!(packet2.group_identifier(), 0);
    assert_eq!(packet2.address(), Ipv4Addr::new(114, 51, 41, 91).to_u32());
    assert_eq!(packet2.need_response(), true);
}
