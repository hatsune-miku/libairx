use std::fmt;
use std::fmt::{Debug, Formatter};
use crate::packet::protocol::serialize::Serialize;

pub enum ResponseKind {
    StopSending = 0x1,
    StopReceiving = 0x2,
}

pub struct FilePartResponsePacket {
    file_id: u8,
    response_kind: u8,
}

// Serialized as:
// 1 byte: file id
// 1 byte: response kind
// 2 bytes in total
const BASE_PACKET_SIZE: usize = 2;

impl FilePartResponsePacket {
    pub fn new(
        file_id: u8,
        response_kind: ResponseKind,
    ) -> FilePartResponsePacket {
        FilePartResponsePacket {
            file_id,
            response_kind: response_kind as u8,
        }
    }

    pub fn file_id(&self) -> u8 {
        self.file_id
    }

    pub fn response_kind(&self) -> ResponseKind {
        match self.response_kind {
            0x1 => ResponseKind::StopSending,
            0x2 => ResponseKind::StopReceiving,
            _ => panic!("Invalid response kind"),
        }
    }
}

impl Debug for FilePartResponsePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilePartResponsePacket")
            .field("file_id", &self.file_id)
            .field("response_kind", &self.response_kind)
            .finish()
    }
}

impl PartialEq for FilePartResponsePacket {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id && self.response_kind == other.response_kind
    }
}

pub enum FilePartResponsePacketError {
    CorruptedData
}

impl Debug for FilePartResponsePacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilePartResponsePacketError")
            .finish()
    }
}

impl Serialize<Vec<u8>, FilePartResponsePacketError> for FilePartResponsePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(BASE_PACKET_SIZE);
        data.push(self.file_id);
        data.push(self.response_kind);
        data
    }

    fn deserialize(data: &Vec<u8>) -> Result<FilePartResponsePacket, FilePartResponsePacketError> {
        if data.len() != BASE_PACKET_SIZE {
            return Err(FilePartResponsePacketError::CorruptedData);
        }

        let file_id = data[0];
        let response_kind = data[1];

        Ok(FilePartResponsePacket {
            file_id,
            response_kind,
        })
    }
}
