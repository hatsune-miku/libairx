use airx::packet::text_packet::TextPacket;
use airx::packet::protocol::serialize::Serialize;

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
    let bytes = packet.serialize();
    let packet2 = TextPacket::deserialize(bytes).unwrap();

    assert_eq!(packet2.text, test_string);
}
