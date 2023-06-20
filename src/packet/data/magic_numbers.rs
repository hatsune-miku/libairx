pub enum MagicNumbers {
    FileComing, Text
}

impl MagicNumbers {
    pub fn value(&self) -> u16 {
        match self {
            MagicNumbers::FileComing => 0x3939,
            MagicNumbers::Text => 0x3940,
        }
    }
    
    pub fn from(value: u16) -> Option<Self> {
        match value {
            0x3939 => Some(MagicNumbers::FileComing),
            0x3940 => Some(MagicNumbers::Text),
            _ => None,
        }
    }
}
