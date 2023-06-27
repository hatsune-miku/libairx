pub enum MagicNumbers {
    FileComing, Text, FileReceiveResponse, FilePart, FilePartResponse,
}

impl MagicNumbers {
    pub fn value(&self) -> u16 {
        match self {
            MagicNumbers::FileComing => 0x3939,
            MagicNumbers::Text => 0x3940,
            MagicNumbers::FileReceiveResponse => 0x3941,
            MagicNumbers::FilePart => 0x3942,
            MagicNumbers::FilePartResponse => 0x3943,
        }
    }
    
    pub fn from(value: u16) -> Option<Self> {
        match value {
            0x3939 => Some(MagicNumbers::FileComing),
            0x3940 => Some(MagicNumbers::Text),
            0x3941 => Some(MagicNumbers::FileReceiveResponse),
            0x3942 => Some(MagicNumbers::FilePart),
            0x3943 => Some(MagicNumbers::FilePartResponse),
            _ => None,
        }
    }
}
