pub trait Hash<T> {
    fn hash(&self) -> T;
    fn is_hash_valid(&self) -> bool;
}
