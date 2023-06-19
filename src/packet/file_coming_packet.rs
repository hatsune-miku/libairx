use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::net::Ipv4Addr;
use log::{trace};
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::hash::Hash;
use crate::packet::protocol::serialize::Serialize;

// Serialized as:
// 2 bytes: magic number
// 4 bytes: address
// 2 bytes: server port
// 1 byte:  group identity
// 4 bytes: file size in bytes
// 4 bytes: file name length (UTF-8)
// N bytes: file name (UTF-8)
// 2 bytes: hash of (addr,port,id,file_size)
// 19 + N bytes in total
pub const BASE_PACKET_SIZE: usize = 19;

const MAGIC_NUMBER: u16 = 0x3939;

pub struct FileComingPacket {
    magic_number: u16,
    address: Ipv4Addr,
    server_port: u16,
    group_identity: u8,
    file_size: u32,
    file_name_length: u32,
    file_name: String,
}

impl FileComingPacket {
    pub fn new(
        address: Ipv4Addr,
        server_port: u16,
        group_identity: u8,
        file_size: u32,
        file_name: String,
    ) -> FileComingPacket {
        FileComingPacket {
            magic_number: MAGIC_NUMBER,
            address,
            server_port,
            group_identity,
            file_size,
            file_name_length: file_name.len() as u32,
            file_name,
        }
    }

    pub fn sender_address(&self) -> &Ipv4Addr {
        &self.address
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn group_identity(&self) -> u8 {
        self.group_identity
    }

    pub fn file_size(&self) -> u32 {
        self.file_size
    }

    pub fn file_name(&self) -> &String {
        &self.file_name
    }
}

impl Debug for FileComingPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileComingPacket")
            .field("magic_number", &self.magic_number)
            .field("address", &self.address)
            .field("server_port", &self.server_port)
            .field("group_identity", &self.group_identity)
            .field("file_size", &self.file_size)
            .field("file_name", &self.file_name)
            .finish()
    }
}

impl PartialEq for FileComingPacket {
    fn eq(&self, other: &Self) -> bool {
        self.magic_number == other.magic_number
            && self.address == other.address
            && self.server_port == other.server_port
            && self.group_identity == other.group_identity
            && self.file_size == other.file_size
            && self.file_name == other.file_name
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub enum FileComingPacketError {
    InvalidMagicNumber,
    InvalidHash,
    CorruptedPacket,
    FileNameTooLong,
    FileTooLarge,
}

impl Debug for FileComingPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::write(
            f,
            format_args!(
                "DiscoveryPacketError: {}",
                match self {
                    FileComingPacketError::InvalidMagicNumber => "Invalid magic number",
                    FileComingPacketError::InvalidHash => "Invalid hash",
                    FileComingPacketError::FileNameTooLong => "File name too long",
                    FileComingPacketError::FileTooLarge => "File too large",
                    FileComingPacketError::CorruptedPacket => "Corrupted packet",
                }
            ),
        )
    }
}

impl Display for FileComingPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for FileComingPacketError {}

// 没有150年功力的人，不要轻易使用这个哈希算法，极易遭到反噬！
// 怎么IDEA连注释都算重复啊，IDEA他不懂编程
fn packet_hash(packet: &FileComingPacket) -> u16 {
    (
        (packet.address.octets()[0] as u16)
            .wrapping_add(packet.address.octets()[1] as u16)
            .wrapping_add(packet.address.octets()[2] as u16)
            .wrapping_add(packet.address.octets()[3] as u16)
            .wrapping_add(packet.server_port)
            .wrapping_add(packet.group_identity as u16)
            .wrapping_add(packet.file_size as u16)
    ) / 7
}

impl Hash<u16> for FileComingPacket {
    fn is_hash_valid(&self, hash: &u16) -> bool {
        packet_hash(self) == *hash
    }
}

// Serialized as:
// 2 bytes: magic number
// 4 bytes: address
// 2 bytes: server port
// 1 byte:  group identity
// 4 bytes: file size in bytes
// 4 bytes: file name length (UTF-8)
// N bytes: file name (UTF-8)
// 2 bytes: hash of (addr,port,id,file_size)
// 19 + N bytes in total

impl Serialize<Box<[u8]>, FileComingPacketError> for FileComingPacket {
    fn serialize(&self) -> Box<[u8]> {
        let mut bytes = Vec::<u8>::new();
        bytes.extend_from_slice(&self.magic_number.to_bytes());
        bytes.extend_from_slice(&self.address.octets());
        bytes.extend_from_slice(&self.server_port.to_bytes());
        bytes.push(self.group_identity);
        bytes.extend_from_slice(&self.file_size.to_bytes());
        bytes.extend_from_slice(&self.file_name_length.to_bytes());
        bytes.extend_from_slice(self.file_name.as_bytes());
        bytes.extend_from_slice(&packet_hash(self).to_bytes());
        Box::from(bytes)
    }

    fn deserialize(data: Box<[u8]>) -> Result<Self, FileComingPacketError> {
        // Preliminary size check
        trace!(
            "FileComingPacket::deserialize, preliminary size check, BASE_PACKET_SIZE: {}",
            BASE_PACKET_SIZE
        );
        if data.len() < BASE_PACKET_SIZE {
            return Err(FileComingPacketError::CorruptedPacket);
        }

        let magic_number = u16::from_bytes([data[0], data[1]]);
        if magic_number != MAGIC_NUMBER {
            return Err(FileComingPacketError::InvalidMagicNumber);
        }

        let address = Ipv4Addr::new(data[2], data[3], data[4], data[5]);
        let server_port = u16::from_bytes([data[6], data[7]]);
        let group_identity = data[8];
        let file_size = u32::from_bytes([data[9], data[10], data[11], data[12]]);
        let file_name_length = u32::from_bytes([data[13], data[14], data[15], data[16]]);

        // Secondary size check
        let expected_size = BASE_PACKET_SIZE + file_name_length as usize;
        trace!(
            "FileComingPacket::deserialize, secondary size check, expected size: {}",
            expected_size
        );
        if data.len() != expected_size {
            return Err(FileComingPacketError::CorruptedPacket);
        }

        let file_name = String::from_utf8(
            data[17..=(17 + file_name_length as usize - 1)].to_vec()
        ).map_err(|_| FileComingPacketError::FileNameTooLong)?;
        let hash = u16::from_bytes(
            [
                data[17 + file_name_length as usize - 1 + 1],
                data[17 + file_name_length as usize - 1 + 2],
            ]
        );

        let ret = FileComingPacket {
            magic_number,
            address,
            server_port,
            group_identity,
            file_size,
            file_name_length,
            file_name: file_name.clone(),
        };

        if hash != packet_hash(&ret) {
            return Err(FileComingPacketError::InvalidHash);
        }

        Ok(ret)
    }
}
