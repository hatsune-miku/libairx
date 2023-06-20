use std::fmt;
use std::fmt::{Debug, Formatter};
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::serialize::Serialize;

/**
 * Serialized as:
    * 2 bytes: magic number
    * 4 bytes: data length in bytes
    * N bytes: data
    * 2 bytes: hash of (data_length)
    * 8 + N bytes in total
*/
pub const BASE_PACKET_SIZE: usize = 8;

pub struct DataPacket {
    magic_number: u16,
    data: Vec<u8>,
}

pub enum DataPacketError {
    InvalidMagicNumber,
    InvalidHash,
    CorruptedData,
}

impl Debug for DataPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::write(
            f,
            format_args!(
                "DataPacketError::{}",
                match self {
                    DataPacketError::InvalidMagicNumber => "Invalid magic number",
                    DataPacketError::InvalidHash => "Invalid hash",
                    DataPacketError::CorruptedData => "Corrupted data",
                },
            )
        )
    }
}

impl DataPacket {
    pub fn new(
        magic_number: u16,
        data: &Vec<u8>,
    ) -> DataPacket {
        DataPacket {
            magic_number,
            data: data.clone(),
        }
    }

    pub fn magic_number(&self) -> u16 {
        self.magic_number
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

fn packet_hash(packet: &DataPacket) -> u16 {
    (packet.data().len() / 2) as u16
}

impl Serialize<Vec<u8>, DataPacketError> for DataPacket {
    fn serialize(&self) -> Vec<u8> {
        let data_len = self.data.len();
        let mut bytes = Vec::with_capacity(BASE_PACKET_SIZE + data_len);
        bytes.extend_from_slice(&self.magic_number.to_bytes());
        bytes.extend_from_slice(&(data_len as u32).to_bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&packet_hash(self).to_bytes());
        bytes
    }

    fn deserialize(data: &Vec<u8>) -> Result<Self, DataPacketError> where Self: Sized {
        let data_len = data.len();
        if data_len < BASE_PACKET_SIZE {
            return Err(DataPacketError::InvalidMagicNumber);
        }

        let magic_number = u16::from_bytes([data[0], data[1]]);
        let actual_data_len = u32::from_bytes([data[2], data[3], data[4], data[5]]) as usize;

        if data_len != BASE_PACKET_SIZE + actual_data_len {
            return Err(DataPacketError::CorruptedData);
        }

        let wrapping_data = data[6..6 + actual_data_len].to_vec();
        let hash = u16::from_bytes([data[6 + actual_data_len], data[6 + actual_data_len + 1]]);

        let ret = DataPacket::new(
            magic_number,
            &wrapping_data,
        );
        if hash != packet_hash(&ret) {
            return Err(DataPacketError::InvalidHash);
        }

        Ok(ret)
    }
}
