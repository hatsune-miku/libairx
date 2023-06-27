pub struct OSUtil;

impl OSUtil {
    pub fn hostname() -> String {
        match hostname::get() {
            Ok(n) => match n.into_string() {
                Ok(s) => s,
                Err(_) => String::from("<empty>"),
            },
            Err(_) => String::from("<empty>"),
        }
    }
}
