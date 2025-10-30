use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub query: Option<String>,
    pub sort: Option<String>,
    pub asc: Option<bool>,
    pub case_sensitive: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_query_default_none() {
        let query = ReadQuery {
            offset: None,
            limit: None,
            query: None,
            sort: None,
            asc: None,
            case_sensitive: None,
        };
        
        assert!(query.offset.is_none());
        assert!(query.limit.is_none());
        assert!(query.query.is_none());
        assert!(query.sort.is_none());
        assert!(query.asc.is_none());
        assert!(query.case_sensitive.is_none());
    }

    #[test]
    fn test_read_query_with_pagination() {
        let query = ReadQuery {
            offset: Some(10),
            limit: Some(20),
            query: None,
            sort: None,
            asc: None,
            case_sensitive: None,
        };
        
        assert_eq!(query.offset, Some(10));
        assert_eq!(query.limit, Some(20));
    }

    #[test]
    fn test_read_query_with_sorting() {
        let query = ReadQuery {
            offset: None,
            limit: None,
            query: None,
            sort: Some("name".to_string()),
            asc: Some(true),
            case_sensitive: None,
        };
        
        assert_eq!(query.sort, Some("name".to_string()));
        assert_eq!(query.asc, Some(true));
    }

    #[test]
    fn test_read_query_serialization() {
        let query = ReadQuery {
            offset: Some(5),
            limit: Some(15),
            query: Some("test".to_string()),
            sort: Some("date".to_string()),
            asc: Some(false),
            case_sensitive: Some(true),
        };
        
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: ReadQuery = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(query.offset, deserialized.offset);
        assert_eq!(query.limit, deserialized.limit);
        assert_eq!(query.query, deserialized.query);
        assert_eq!(query.sort, deserialized.sort);
        assert_eq!(query.asc, deserialized.asc);
        assert_eq!(query.case_sensitive, deserialized.case_sensitive);
    }

    #[test]
    fn test_read_query_partial_deserialization() {
        let json = r#"{"offset":10,"limit":20}"#;
        let query: ReadQuery = serde_json::from_str(json).unwrap();
        
        assert_eq!(query.offset, Some(10));
        assert_eq!(query.limit, Some(20));
        assert!(query.query.is_none());
        assert!(query.sort.is_none());
    }
}
