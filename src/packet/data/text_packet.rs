use core::fmt;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::hash::Hash;
use crate::packet::protocol::serialize::Serialize;

pub const STRING_LENGTH_MAX: usize = 0xffff;

// Serialized as:
// 4 bytes: text length (UTF-8)
// N bytes: text (UTF-8)
// 2 bytes: hash of (text_length)
// 6 + N bytes in total
const BASE_PACKET_SIZE: usize = 6;

pub struct TextPacket {
    pub text_length: u32,
    pub text: String,
}

pub enum TextPacketError {
    InvalidData,
    InvalidHash,
    StringTooLong,
}

impl Debug for TextPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for TextPacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::write(
            f,
            format_args!(
                "{}",
                match self {
                    TextPacketError::InvalidData => "Invalid data.",
                    TextPacketError::InvalidHash => "Invalid hash.",
                    TextPacketError::StringTooLong => "String too long.",
                }
            ),
        )
    }
}

impl Error for TextPacketError {}

type HashType = u16;

fn text_hash(text: &String) -> HashType {
    let mut ret: HashType = HashType::MAX ^ 0x12 ^ 0x13 ^ 0x8;
    for (i, c) in text.chars().enumerate() {
        ret = ret.wrapping_add((i * c as usize) as HashType);
    }
    ret
}

impl Hash<HashType> for TextPacket {
    fn is_hash_valid(&self, hash: &HashType) -> bool {
        self.text.len() as HashType == *hash
    }
}

impl Serialize<Vec<u8>, TextPacketError> for TextPacket {
    fn serialize(&self) -> Vec<u8> {
        let text_bytes = self.text.as_bytes();
        let mut ret = Vec::with_capacity(text_bytes.len() + BASE_PACKET_SIZE);
        ret.extend_from_slice(&self.text_length.to_bytes());
        ret.extend_from_slice(&text_bytes);
        ret.extend_from_slice(&text_hash(&self.text).to_bytes());
        ret
    }

    fn deserialize(data: &Vec<u8>) -> Result<Self, TextPacketError> {
        let data_len = data.len();

        if data_len < BASE_PACKET_SIZE {
            return Err(TextPacketError::InvalidData);
        }

        let text_len = u32::from_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let text = String::from_utf8(data[4..4 + text_len].to_vec())
            .map_err(|_| TextPacketError::InvalidData)?;
        let hash = u16::from_bytes([data[4 + text_len], data[4 + text_len + 1]]);

        if text_hash(&text) == hash {
            match TextPacket::new(text.clone()) {
                Ok(x) => Ok(x),
                Err(_) => Err(TextPacketError::InvalidData),
            }
        } else {
            Err(TextPacketError::InvalidHash)
        }
    }
}

impl TextPacket {
    pub fn new(text: String) -> Result<Self, TextPacketError> {
        if text.len() > STRING_LENGTH_MAX {
            return Err(TextPacketError::StringTooLong);
        }
        Ok(Self { text_length: text.len() as u32, text })
    }

    pub fn text(&self) -> &String {
        &self.text
    }
}
