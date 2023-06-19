use std::net::Ipv4Addr;
use airx::packet::file_coming_packet::FileComingPacket;
use airx::packet::protocol::serialize::Serialize;

#[test]
fn test_file_coming_packet_serializable() {
    let packet = FileComingPacket::new(
        Ipv4Addr::new(114, 51, 41, 91),
        9818,
        0,
        1024,
        String::from("test中文测试 =_= 😃 RTL test سلام عليكم 🇯🇵こんにちは؟ *&%^.txt"),
    );

    let bytes = packet.serialize();
    let packet2 = FileComingPacket::deserialize(bytes).unwrap();

    assert_eq!(packet, packet2);
}
