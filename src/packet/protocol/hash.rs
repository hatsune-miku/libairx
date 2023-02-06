pub trait Hash<T> {
    fn is_hash_valid(&self, hash: &T) -> bool;
}
