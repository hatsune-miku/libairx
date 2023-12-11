use std::net::Ipv4Addr;

pub trait ConvertIpU32 {
    fn to_u32(&self) -> u32;
    fn from_u32(value: u32) -> Self;
}

impl ConvertIpU32 for Ipv4Addr {
    fn to_u32(&self) -> u32 {
        let octets = self.octets();
        let mut result: u32 = 0;
        for i in 0..4 {
            result += (octets[i] as u32) << (8 * (3 - i));
        }
        result
    }
    
    fn from_u32(value: u32) -> Self {
        let mut octets = [0u8; 4];
        for i in 0..4 {
            octets[i] = ((value >> (8 * (3 - i))) & 0xFF) as u8;
        }
        Ipv4Addr::from(octets)
    }
}
