use airx::packet::data::file_coming_packet::FileComingPacket;
use airx::packet::protocol::serialize::Serialize;

#[test]
fn test_file_coming_packet_serializable() {
    let packet = FileComingPacket::new(
        1024,
        String::from("test中文测试 \\^O^/ 😃 RTL test سلام عليكم 🇯🇵こんにちは؟ *&%^.txt"),
    );

    let bytes = packet.serialize();
    let packet2 = FileComingPacket::deserialize(&bytes).unwrap();

    assert_eq!(packet, packet2);
}
