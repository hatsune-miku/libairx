use airx::packet::text_packet::TextPacket;
use airx::packet::protocol::serialize::Serialize;

#[test]
fn test_text_packet_serializable() {
    // Text generated with help of ChatGPT.
    let test_string =
        "😃 سلام عليكم 🇯🇵こんにちは؟ *&%^".to_string() +
            "另立💝天地💖宇宙👾分封😈乐园🍩伊甸☪︎ENGLISH~TEXT" +
            "🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐" +
            "🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐" +
            "🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐" +
            "🉐🉐🉐🉐🉐🉐🉐🉐🉐🉐" +
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

