use std::fmt;
use std::fmt::{Debug, Formatter};
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::serialize::Serialize;

pub struct FileReceiveResponsePacket {
    file_id: u8,
    file_size: u64,
    file_name_length: u32,
    file_name: String,
    accepted: bool,
}

// Serialized as:
// 1 byte: file id
// 8 bytes: file size in bytes
// 4 bytes: file name length (UTF-8)
// N bytes: file name (UTF-8)
// 1 byte: accepted
// 14 + N bytes in total
pub const BASE_PACKET_SIZE: usize = 14;

impl FileReceiveResponsePacket {
    pub fn new(
        file_id: u8,
        file_size: u64,
        file_name: String,
        accepted: bool,
    ) -> FileReceiveResponsePacket {
        FileReceiveResponsePacket {
            file_id,
            file_size,
            file_name_length: file_name.len() as u32,
            file_name,
            accepted,
        }
    }

    pub fn file_id(&self) -> u8 {
        self.file_id
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn file_name(&self) -> &String {
        &self.file_name
    }

    pub fn accepted(&self) -> bool {
        self.accepted
    }
}

impl Debug for FileReceiveResponsePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileReceiveResponsePacket")
            .field("file_id", &self.file_id)
            .field("file_size", &self.file_size)
            .field("file_name", &self.file_name)
            .field("accepted", &self.accepted)
            .finish()
    }
}

impl PartialEq for FileReceiveResponsePacket {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
            && self.file_size == other.file_size
            && self.file_name == other.file_name
            && self.accepted == other.accepted
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub enum FileReceiveResponsePacketError {
    CorruptedData
}

impl Debug for FileReceiveResponsePacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::write(
            f,
            format_args!(
                "DiscoveryPacketError: {}",
                match self {
                    FileReceiveResponsePacketError::CorruptedData => "Corrupted packet",
                }
            ),
        )
    }
}

impl Serialize<Vec<u8>, FileReceiveResponsePacketError> for FileReceiveResponsePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(BASE_PACKET_SIZE + self.file_name_length as usize);
        data.push(self.file_id);
        data.extend_from_slice(&self.file_size.to_bytes());
        data.extend_from_slice(&self.file_name_length.to_bytes());
        data.extend_from_slice(self.file_name.as_bytes());
        data.push(self.accepted as u8);
        data
    }

    fn deserialize(data: &Vec<u8>) -> Result<Self, FileReceiveResponsePacketError> where Self: Sized {
        if data.len() < BASE_PACKET_SIZE {
            return Err(FileReceiveResponsePacketError::CorruptedData);
        }
        let file_id = data[0];
        let file_size = u64::from_bytes([data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8]]);
        let file_name_length = u32::from_bytes([data[9], data[10], data[11], data[12]]);
        if data.len() < BASE_PACKET_SIZE + file_name_length as usize {
            return Err(FileReceiveResponsePacketError::CorruptedData);
        }
        let file_name = String::from_utf8_lossy(&data[13..13 + file_name_length as usize]).to_string();
        let accepted = data[13 + file_name_length as usize] != 0;
        Ok(FileReceiveResponsePacket {
            file_id,
            file_size,
            file_name_length,
            file_name,
            accepted,
        })
    }
}
