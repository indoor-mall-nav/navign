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
        // Check that BASE_URL contains a port specification
        // Remove the protocol part and check if there's a colon with digits after it
        let url_without_protocol = BASE_URL.split("://").nth(1).expect("Invalid URL format");
        assert!(url_without_protocol.contains(':'));
        
        // Extract port and verify it's numeric
        let port_part = url_without_protocol.split(':').nth(1).expect("No port found");
        let port_str = port_part.trim_end_matches('/');
        assert!(port_str.parse::<u16>().is_ok(), "Port should be a valid number");
    }
}
