use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::hash::Hash;
use crate::packet::protocol::serialize::Serialize;

// Serialized as:
// 8 bytes: file size in bytes
// 4 bytes: file name length (UTF-8)
// N bytes: file name (UTF-8)
// 2 bytes: hash of (file_size,file_name_length)
// 14 + N bytes in total
const BASE_PACKET_SIZE: usize = 12;

pub struct FileComingPacket {
    file_size: u64,
    file_name_length: u32,
    file_name: String,
}

impl FileComingPacket {
    pub fn new(
        file_size: u64,
        file_name: String,
    ) -> FileComingPacket {
        FileComingPacket {
            file_size,
            file_name_length: file_name.len() as u32,
            file_name,
        }
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn file_name(&self) -> &String {
        &self.file_name
    }
}

impl Debug for FileComingPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileComingPacket")
            .field("file_size", &self.file_size)
            .field("file_name", &self.file_name)
            .finish()
    }
}

impl PartialEq for FileComingPacket {
    fn eq(&self, other: &Self) -> bool {
        self.file_size == other.file_size
            && self.file_name == other.file_name
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub enum FileComingPacketError {
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
    (packet.file_size as u32 + packet.file_name_length) as u16
}

impl Hash<u16> for FileComingPacket {
    fn is_hash_valid(&self, hash: &u16) -> bool {
        packet_hash(self) == *hash
    }
}

impl Serialize<Vec<u8>, FileComingPacketError> for FileComingPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.extend_from_slice(&self.file_size.to_bytes());
        bytes.extend_from_slice(&self.file_name_length.to_bytes());
        bytes.extend_from_slice(self.file_name.as_bytes());
        bytes.extend_from_slice(&packet_hash(self).to_bytes());
        bytes
    }

    fn deserialize(data: &Vec<u8>) -> Result<Self, FileComingPacketError> {
        if data.len() < BASE_PACKET_SIZE {
            return Err(FileComingPacketError::CorruptedPacket);
        }

        let file_size = u64::from_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        let file_name_length = u32::from_bytes([data[8], data[9], data[10], data[11]]);
        let file_name_length = file_name_length as usize;
        let file_name = String::from_utf8(
            data[12..(12 + file_name_length)].to_vec()
        ).map_err(|_| FileComingPacketError::FileNameTooLong)?;

        let hash = u16::from_bytes([
            data[12 + file_name_length],
            data[12 + file_name_length + 1],
        ]);

        let ret = FileComingPacket::new(
            file_size,
            file_name,
        );

        if hash != packet_hash(&ret) {
            return Err(FileComingPacketError::InvalidHash);
        }

        Ok(ret)
    }
}
