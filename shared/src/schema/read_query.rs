#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Query parameters for reading/listing resources
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReadQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub query: Option<String>,
    pub sort: Option<String>,
    pub asc: Option<bool>,
    pub case_sensitive: Option<bool>,
}

impl Default for ReadQuery {
    fn default() -> Self {
        Self {
            offset: None,
            limit: None,
            query: None,
            sort: None,
            asc: None,
            case_sensitive: None,
        }
    }
}

#[cfg(all(test, feature = "serde", feature = "alloc"))]
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
            sort: Some(alloc::string::String::from("name")),
            asc: Some(true),
            case_sensitive: None,
        };
        
        assert_eq!(query.sort, Some(alloc::string::String::from("name")));
        assert_eq!(query.asc, Some(true));
    }

    #[test]
    fn test_read_query_default() {
        let query = ReadQuery::default();
        
        assert!(query.offset.is_none());
        assert!(query.limit.is_none());
        assert!(query.query.is_none());
        assert!(query.sort.is_none());
        assert!(query.asc.is_none());
        assert!(query.case_sensitive.is_none());
    }
}
