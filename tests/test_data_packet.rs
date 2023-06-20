use libairx::packet::data::text_packet::TextPacket;
use libairx::packet::data_packet::DataPacket;
use libairx::packet::protocol::serialize::Serialize;

#[test]
fn test_text_packet_serializable() {
    // Text including emojis, non-ASCII characters, RTL characters, and code.
    let test_string =
        "😃 سلام عليكم 🇯🇵こんにちは؟ *&%^".to_string() +
            "🉐🉐🉐" +
            "public static void main(String[] args) {" +
            "    System.out.println(\"Hello, world!\");" +
            "}" +
            "console.log(() => \"Hello, world!\"))XXXXXXX;" +
            "SYNC.SYNC:XXXXXXXXXXXXXXXXYXXXXXXXXXXXXXXXXX" +
            "3000";
    let packet = TextPacket::new(test_string.clone()).unwrap();
    let data_packet = DataPacket::new(1145u16, &packet.serialize());
    let bytes = data_packet.serialize();
    let data_packet2 = DataPacket::deserialize(&bytes).unwrap();
    let packet2 = TextPacket::deserialize(&data_packet2.data()).unwrap();

    assert_eq!(packet2.text, test_string);
}
