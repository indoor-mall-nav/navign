pub const BASE_URL: &str = "http://192.168.0.107:3000/";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url_format() {
        assert!(BASE_URL.starts_with("http://") || BASE_URL.starts_with("https://"));
        assert!(BASE_URL.ends_with("/"));
    }

    #[test]
    fn test_base_url_not_empty() {
        assert!(!BASE_URL.is_empty());
        assert!(BASE_URL.len() > 10);
    }

    #[test]
    fn test_base_url_contains_port() {
        assert!(BASE_URL.contains(":3000"));
    }
}
