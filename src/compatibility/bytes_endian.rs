use std::mem;

pub const fn size_of<T>() -> usize {
    mem::size_of::<T>()
}

pub trait UniversalEndian<T> {
    fn to_bytes(&self) -> [T; size_of::<Self>()];
    fn from_bytes(bytes: [T; size_of::<Self>()]) -> Self;
}

macro_rules! impl_universal_endian {
    ($($t:ty),*) => {
        $(
            impl UniversalEndian<u8> for $t {
                fn to_bytes(&self) -> [u8; size_of::<Self>()] {
                    self.to_le_bytes()
                }

                fn from_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                    $t::from_le_bytes(bytes)
                }
            }
        )*
    };
}

impl_universal_endian!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
