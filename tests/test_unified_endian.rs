use libairx::compatibility::unified_endian::UnifiedEndian;

#[cfg(target_endian = "little")]
#[test]
fn test_unified_endian() {
    // Test u16
    let n: u16 = 0x1234;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0x34, 0x12]);
    let m = u16::from_bytes(bytes);
    assert_eq!(n, m);

    // Test u32
    let n: u32 = 0x12345678;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);
    let m = u32::from_bytes(bytes);
    assert_eq!(n, m);

    // Test i16
    let n: i16 = -0x1234;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0xcc, 0xed]);
    let m = i16::from_bytes(bytes);
    assert_eq!(n, m);

    // Test i32
    let n: i32 = -0x12345678;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0x88, 0xa9, 0xcb, 0xed]);
    let m = i32::from_bytes(bytes);
    assert_eq!(n, m);
}

#[cfg(target_endian = "big")]
#[test]
fn test_unified_endian() {
    // Test u16
    let n: u16 = 0x1234;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0x12, 0x34]);
    let m = u16::from_bytes(bytes);
    assert_eq!(n, m);

    // Test u32
    let n: u32 = 0x12345678;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0x12, 0x34, 0x56, 0x78]);
    let m = u32::from_bytes(bytes);
    assert_eq!(n, m);

    // Test i16
    let n: i16 = -0x1234;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0xed, 0xcc]);
    let m = i16::from_bytes(bytes);
    assert_eq!(n, m);

    // Test i32
    let n: i32 = -0x12345678;
    let bytes = n.to_bytes();
    assert_eq!(bytes, [0xed, 0xcb, 0xa9, 0x88]);
    let m = i32::from_bytes(bytes);
    assert_eq!(n, m);
}
