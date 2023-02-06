use std::mem::size_of;

pub trait UnifiedEndian<const SIZE: usize> {
    fn to_bytes(&self) -> [u8; SIZE];
    fn from_bytes(bytes: [u8; SIZE]) -> Self;
}

macro_rules! impl_unified_endian {
    ($($t:ty),*) => {
        $(
            impl UnifiedEndian<{ size_of::<$t>() }> for $t {
                fn to_bytes(&self) -> [u8; size_of::<$t>()] {
                    self.to_le_bytes()
                }
                fn from_bytes(bytes: [u8; size_of::<$t>()]) -> Self {
                    <$t>::from_le_bytes(bytes)
                }
            }
        )*
    };
}

impl_unified_endian!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

