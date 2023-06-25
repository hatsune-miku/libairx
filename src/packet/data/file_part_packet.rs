use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::serialize::Serialize;

pub struct FilePartPacket {
    file_id: u8,
    offset: u32,
    length: u32,
    // TODO: SEVERE PERFORMANCE ISSUE
    data: Arc<Box<[u8]>>,
}

// Serialized as:
// 1 byte: file id
// 4 bytes: offset
// 4 bytes: length
// N bytes: data
// 9 + N bytes in total
const BASE_PACKET_SIZE: usize = 9;

impl FilePartPacket {
    pub fn new(
        file_id: u8,
        offset: u32,
        length: u32,
        data: Box<[u8]>,
    ) -> FilePartPacket {
        FilePartPacket {
            file_id,
            offset,
            length,
            data: Arc::new(data),
        }
    }

    pub fn file_id(&self) -> u8 {
        self.file_id
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn data(&self) -> Arc<Box<[u8]>> {
        self.data.clone()
    }
}

impl Debug for FilePartPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilePartPacket")
            .field("file_id", &self.file_id)
            .field("offset", &self.offset)
            .field("length", &self.length)
            .field("data", &self.data)
            .finish()
    }
}

impl PartialEq for FilePartPacket {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
            && self.offset == other.offset
            && self.length == other.length
            && self.data == other.data
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub enum FilePartPacketError {
    CorruptedData,
}

impl Debug for FilePartPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::write(
            f,
            format_args!(
                "DiscoveryPacketError: {}",
                match self {
                    FilePartPacketError::CorruptedData => "Corrupted packet",
                }
            ),
        )
    }
}

impl Serialize<Vec<u8>, FilePartPacketError> for FilePartPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(BASE_PACKET_SIZE + self.data.len());
        serialized.push(self.file_id);
        serialized.extend_from_slice(&self.offset.to_bytes());
        serialized.extend_from_slice(&self.length.to_bytes());
        serialized.extend_from_slice(&self.data);
        serialized
    }

    fn deserialize(serialized: &Vec<u8>) -> Result<FilePartPacket, FilePartPacketError> {
        if serialized.len() < BASE_PACKET_SIZE {
            return Err(FilePartPacketError::CorruptedData);
        }

        let file_id = serialized[0];
        let offset = u32::from_bytes([
            serialized[1],
            serialized[2],
            serialized[3],
            serialized[4],
        ]);
        let length = u32::from_bytes([
            serialized[5],
            serialized[6],
            serialized[7],
            serialized[8],
        ]);

        if serialized.len() != BASE_PACKET_SIZE + length as usize {
            return Err(FilePartPacketError::CorruptedData);
        }

        let data = &serialized[BASE_PACKET_SIZE..(BASE_PACKET_SIZE + length as usize)];

        Ok(FilePartPacket::new(
            file_id,
            offset,
            length,
            Box::from(data),
        ))
    }
}
