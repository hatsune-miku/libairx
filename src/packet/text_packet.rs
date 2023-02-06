use core::fmt;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::mem::size_of;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::hash::Hash;
use crate::packet::protocol::serialize::Serialize;

pub struct TextPacket {
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
type StringLenType = u16;

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
        let mut ret = Vec::with_capacity(
            self.text.len() + size_of::<StringLenType>());
        ret.extend_from_slice(self.text.as_bytes());
        ret.extend_from_slice(&text_hash(&self.text).to_bytes());
        ret
    }

    fn deserialize(data: Vec<u8>) -> Result<Self, TextPacketError> {
        let text_len = data.len() - size_of::<HashType>();

        if text_len <= 0 {
            return Err(TextPacketError::InvalidData);
        }

        let text = match String::from_utf8(data[0..text_len].to_vec()) {
            Ok(x) => x,
            Err(_) => return Err(TextPacketError::InvalidHash),
        };

        let hash = HashType::from_bytes([data[text_len], data[text_len + 1]]);

        if text_hash(&text) == hash {
            Ok(TextPacket { text })
        } else {
            Err(TextPacketError::InvalidHash)
        }
    }
}

impl TextPacket {
    pub fn new(text: String) -> Result<Self, TextPacketError> {
        if text.len() >= StringLenType::MAX as usize {
            return Err(TextPacketError::StringTooLong);
        }
        Ok(Self { text })
    }
}
