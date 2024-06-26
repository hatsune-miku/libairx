use airx::packet::data::file_part_packet::FilePartPacket;
use airx::packet::protocol::serialize::Serialize;

#[test]
fn test_test_file_part_packet() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let packet = FilePartPacket::new(11, 45, data.len() as u64, data);
    let bytes = packet.serialize();
    let packet2 = FilePartPacket::deserialize(&bytes).unwrap();
    assert!(packet.eq(&packet2));
}
