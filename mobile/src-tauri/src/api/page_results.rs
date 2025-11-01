use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponseMetadata {
    pub total_items: u64,
    pub current_offset: u64,
    pub current_limit: u64,
    pub next_page_url: Option<String>,
    pub prev_page_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub metadata: PaginationResponseMetadata,
    pub data: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_metadata_basic_fields() {
        let metadata = PaginationResponseMetadata {
            total_items: 100,
            current_offset: 10,
            current_limit: 20,
            next_page_url: Some("/api/next".to_string()),
            prev_page_url: Some("/api/prev".to_string()),
        };

        assert_eq!(metadata.total_items, 100);
        assert_eq!(metadata.current_offset, 10);
        assert_eq!(metadata.current_limit, 20);
        assert_eq!(metadata.next_page_url, Some("/api/next".to_string()));
        assert_eq!(metadata.prev_page_url, Some("/api/prev".to_string()));
    }

    #[test]
    fn test_pagination_metadata_no_navigation_urls() {
        let metadata = PaginationResponseMetadata {
            total_items: 5,
            current_offset: 0,
            current_limit: 10,
            next_page_url: None,
            prev_page_url: None,
        };

        assert!(metadata.next_page_url.is_none());
        assert!(metadata.prev_page_url.is_none());
    }

    #[test]
    fn test_pagination_response_with_data() {
        let metadata = PaginationResponseMetadata {
            total_items: 3,
            current_offset: 0,
            current_limit: 10,
            next_page_url: None,
            prev_page_url: None,
        };

        let data = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
        let response = PaginationResponse { metadata, data: data.clone() };

        assert_eq!(response.data.len(), 3);
        assert_eq!(response.data, data);
    }

    #[test]
    fn test_pagination_response_empty_data() {
        let metadata = PaginationResponseMetadata {
            total_items: 0,
            current_offset: 0,
            current_limit: 10,
            next_page_url: None,
            prev_page_url: None,
        };

        let response: PaginationResponse<String> = PaginationResponse {
            metadata,
            data: vec![],
        };

        assert_eq!(response.data.len(), 0);
        assert_eq!(response.metadata.total_items, 0);
    }

    #[test]
    fn test_pagination_response_serialization() {
        let metadata = PaginationResponseMetadata {
            total_items: 50,
            current_offset: 20,
            current_limit: 10,
            next_page_url: Some("/api/items?offset=30".to_string()),
            prev_page_url: Some("/api/items?offset=10".to_string()),
        };

        let data = vec![1, 2, 3];
        let response = PaginationResponse { metadata, data };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PaginationResponse<i32> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.metadata.total_items, 50);
        assert_eq!(deserialized.metadata.current_offset, 20);
        assert_eq!(deserialized.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_pagination_metadata_clone() {
        let original = PaginationResponseMetadata {
            total_items: 100,
            current_offset: 0,
            current_limit: 25,
            next_page_url: Some("/next".to_string()),
            prev_page_url: None,
        };

        let cloned = original.clone();
        assert_eq!(original.total_items, cloned.total_items);
        assert_eq!(original.current_offset, cloned.current_offset);
        assert_eq!(original.next_page_url, cloned.next_page_url);
    }

    #[test]
    fn test_pagination_response_generic_type() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct TestItem {
            id: u32,
            name: String,
        }

        let items = vec![
            TestItem { id: 1, name: "Item 1".to_string() },
            TestItem { id: 2, name: "Item 2".to_string() },
        ];

        let metadata = PaginationResponseMetadata {
            total_items: 2,
            current_offset: 0,
            current_limit: 10,
            next_page_url: None,
            prev_page_url: None,
        };

        let response = PaginationResponse {
            metadata,
            data: items.clone(),
        };

        assert_eq!(response.data, items);
    }

    #[test]
    fn test_pagination_metadata_large_offset() {
        let metadata = PaginationResponseMetadata {
            total_items: 1000,
            current_offset: 980,
            current_limit: 20,
            next_page_url: Some("/api/items?offset=1000".to_string()),
            prev_page_url: Some("/api/items?offset=960".to_string()),
        };

        assert_eq!(metadata.current_offset, 980);
        assert!(metadata.next_page_url.is_some());
        assert!(metadata.prev_page_url.is_some());
    }
}
