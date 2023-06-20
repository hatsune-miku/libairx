pub trait Serialize<T, E> {
    fn serialize(&self) -> T;
    fn deserialize(data: &T) -> Result<Self, E> where Self: Sized;
}
