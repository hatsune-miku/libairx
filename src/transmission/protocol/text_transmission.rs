pub trait SendText {
    fn send_text(&mut self, text: &str) -> Result<usize, String>;
}

pub trait ReadText {
    fn read_text(&mut self) -> Result<String, String>;
}
