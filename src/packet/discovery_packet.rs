use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::net::Ipv4Addr;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::hash::Hash;
use crate::packet::protocol::serialize::Serialize;

// Serialized as:
// 2 bytes:  magic number
// 4 bytes:  address
// 2 bytes:  server port
// 1 byte:   group identity
// 1 byte:   need response?
// 2 bytes:  hash of (addr,port,id)
// 12 bytes in total

pub const PACKET_SIZE: usize = 12;
const MAGIC_NUMBER: u16 = 0x8964;

pub struct DiscoveryPacket {
    magic_number: u16,
    sender_address: Ipv4Addr,
    server_port: u16,
    group_identity: u8,
    need_response: bool,
}

pub enum DiscoveryPacketError {
    InvalidMagicNumber,
    InvalidHash,
}

impl Debug for DiscoveryPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::write(
            f,
            format_args!(
                "DiscoveryPacketError: {}",
                match self {
                    DiscoveryPacketError::InvalidMagicNumber => "Invalid magic number",
                    DiscoveryPacketError::InvalidHash => "Invalid hash",
                }
            ),
        )
    }
}

impl Display for DiscoveryPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for DiscoveryPacketError {}

// 《企  业  级  哈  希  算  法》
// 《时  间  复  杂  度：O(1)》
// 《空  间  复  杂  度：O(1)》
// 没有50年功力的人，不要轻易使用这个哈希算法，极易遭到反噬。
fn packet_hash(address: Ipv4Addr, server_port: u16, group_identity: u8) -> u16 {
    (
        (address.octets()[0] as u16)
            .wrapping_add(address.octets()[1] as u16)
            .wrapping_add(address.octets()[2] as u16)
            .wrapping_add(address.octets()[3] as u16)
            .wrapping_add(server_port)
            .wrapping_add(group_identity as u16)
    ) / 3
}

impl Hash<u16> for DiscoveryPacket {
    fn is_hash_valid(&self, hash: &u16) -> bool {
        packet_hash(self.sender_address, self.server_port, self.group_identity)
            == *hash
    }
}

impl Serialize<[u8; PACKET_SIZE], DiscoveryPacketError> for DiscoveryPacket {
    fn serialize(&self) -> [u8; PACKET_SIZE] {
        let mut bytes = [0u8; PACKET_SIZE];
        bytes[0..=1].copy_from_slice(&self.magic_number.to_bytes());
        bytes[2..=5].copy_from_slice(&self.sender_address.octets());
        bytes[6..=7].copy_from_slice(&self.server_port.to_bytes());
        bytes[8] = self.group_identity;
        bytes[9] = if self.need_response { 0b0000_0001 } else { 0b0000_0000 };
        bytes[10..=11].copy_from_slice(
            &packet_hash(self.sender_address, self.server_port, self.group_identity).to_bytes());
        bytes
    }

    fn deserialize(data: [u8; PACKET_SIZE]) -> Result<Self, DiscoveryPacketError> {
        let magic_number = u16::from_bytes([data[0], data[1]]);

        if magic_number != MAGIC_NUMBER {
            return Err(DiscoveryPacketError::InvalidMagicNumber);
        }

        let address = Ipv4Addr::new(data[2], data[3], data[4], data[5]);
        let server_port = u16::from_bytes([data[6], data[7]]);
        let group_identity = data[8];
        let hash = u16::from_bytes([data[10], data[11]]);

        if hash != packet_hash(address, server_port, group_identity) {
            return Err(DiscoveryPacketError::InvalidHash);
        }

        let need_response = data[9] == 0b0000_0001;

        Ok(
            Self {
                magic_number,
                sender_address: address,
                server_port,
                group_identity,
                need_response,
            }
        )
    }
}

impl DiscoveryPacket {
    pub fn new(
        sender_address: Ipv4Addr,
        server_port: u16,
        group_identity: u8,
        need_response: bool,
    ) -> Self {
        Self {
            magic_number: MAGIC_NUMBER,
            sender_address,
            server_port,
            group_identity,
            need_response,
        }
    }

    pub fn sender_address(&self) -> Ipv4Addr {
        self.sender_address
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn group_identity(&self) -> u8 {
        self.group_identity
    }

    pub fn need_response(&self) -> bool {
        self.need_response
    }
}

