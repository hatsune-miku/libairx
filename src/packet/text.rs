use crate::network::socket;
use crate::packet::protocol::text_transmission;
use std::{io, usize};
use std::mem::size_of;
use crate::compatibility::unified_endian::UnifiedEndian;
use crate::packet::protocol::serialize::Serialize;
use crate::packet::text_packet::TextPacket;

pub const LENGTH_PRESERVE_SIZE: usize = 16;
pub const MESSAGE_MAX_SIZE: usize = (2 << (LENGTH_PRESERVE_SIZE - 1)) - 1;

pub struct TextTransmission<'a> {
    socket: &'a mut socket::Socket,
}

impl<'a> TextTransmission<'a> {
    pub fn from(socket: &'a mut socket::Socket) -> Self {
        Self { socket }
    }
}

const USIZE_SIZE: usize = size_of::<usize>();

impl text_transmission::SendText for TextTransmission<'_> {
    fn send_text(&mut self, message: String) -> Result<usize, io::Error> {
        // Strings are already utf8 encoded.
        let packet = match TextPacket::new(message) {
            Ok(x) => x,
            Err(e) => return Err(
                io::Error::new(
                    io::ErrorKind::Other, e)),
        };
        let packet_bytes = packet.serialize();
        let bytes = packet_bytes.as_slice();
        let len: usize = bytes.len();

        if len >= MESSAGE_MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Message must no longer than {}.", MESSAGE_MAX_SIZE),
            ));
        }

        let mut buf = vec![0u8; USIZE_SIZE + len];
        buf[0..USIZE_SIZE].copy_from_slice(len.to_bytes().as_slice());
        buf[USIZE_SIZE..].copy_from_slice(bytes);

        self.socket.send(&buf)
    }
}

impl text_transmission::ReadText for TextTransmission<'_> {
    fn read_text(&mut self) -> Result<String, io::Error> {
        let mut size_buf: [u8; USIZE_SIZE] = [0u8; USIZE_SIZE];

        self.socket.read_exact(&mut size_buf)?;
        let size = usize::from_bytes(size_buf);

        let mut buf = vec![0u8; size];
        self.socket.read_exact(&mut buf)?;

        match TextPacket::deserialize(buf) {
            Ok(x) => Ok(x.text),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}
