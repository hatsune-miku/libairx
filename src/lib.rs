mod network;
mod transmission;
mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(
            "好耶".as_bytes(),
            String::from("好耶").as_bytes()
        );
    }
}
