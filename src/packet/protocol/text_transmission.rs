use std::io;

pub trait SendText {
    fn send_text(&mut self, text: String) -> Result<usize, io::Error>;
}

pub trait ReadText {
    fn read_text(&mut self) -> Result<String, io::Error>;
}