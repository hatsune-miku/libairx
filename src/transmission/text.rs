use crate::network::socket;
use crate::transmission::protocol::text_transmission;
use std::{io, usize};

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

impl text_transmission::SendText for TextTransmission<'_> {
    fn send_text(&mut self, message: &str) -> Result<usize, io::Error> {
        // Strings are already utf8 encoded.
        let bytes = message.as_bytes();
        let len = bytes.len();

        if len >= MESSAGE_MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Message must no longer than {}.", MESSAGE_MAX_SIZE),
            ));
        }

        let mut buf = vec![0u8; 8 + len];

        // First 16 bits for packet size.
        buf[0..8].copy_from_slice(len.to_ne_bytes().as_slice());
        buf[8..].copy_from_slice(bytes);

        self.socket.send(&buf)
    }
}

impl text_transmission::ReadText for TextTransmission<'_> {
    fn read_text(&mut self) -> Result<String, io::Error> {
        let mut size_buf: [u8; 8] = [0u8; 8];

        self.socket.read_exact(&mut size_buf)?;
        let size = usize::from_ne_bytes(size_buf);

        let mut buf = vec![0u8; size];
        self.socket.read_exact(&mut buf)?;

        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Invalid UTF-8 sequence: {}", e),
            )),
        }
    }
}
