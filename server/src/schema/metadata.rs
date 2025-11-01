use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponseMetadata {
    pub total_items: u64,
    pub current_offset: u64,
    pub current_limit: u64,
    pub next_page_url: Option<String>,
    pub prev_page_url: Option<String>,
}

impl PaginationResponseMetadata {
    pub fn new(
        total_items: u64,
        current_offset: u64,
        current_limit: u64,
        api_base_url: &str,
    ) -> Self {
        let next_offset = if current_offset + current_limit < total_items {
            Some(current_offset + current_limit)
        } else {
            None
        };
        let prev_offset = if current_offset >= current_limit {
            Some(current_offset - current_limit)
        } else {
            None
        };
        let next_page_url = next_offset
            .map(|offset| format!("{}?offset={}&limit={}", api_base_url, offset, current_limit));
        let prev_page_url = prev_offset
            .map(|offset| format!("{}?offset={}&limit={}", api_base_url, offset, current_limit));
        Self {
            total_items,
            current_offset,
            current_limit,
            next_page_url,
            prev_page_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub metadata: PaginationResponseMetadata,
    pub data: Vec<T>,
}

impl<T> PaginationResponse<T> {
    pub fn new(
        total_items: u64,
        current_offset: u64,
        current_limit: u64,
        api_base_url: &str,
        data: Vec<T>,
    ) -> Self {
        let metadata = PaginationResponseMetadata::new(
            total_items,
            current_offset,
            current_limit,
            api_base_url,
        );
        Self { metadata, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_metadata_first_page() {
        let metadata = PaginationResponseMetadata::new(100, 0, 10, "/api/items");
        
        assert_eq!(metadata.total_items, 100);
        assert_eq!(metadata.current_offset, 0);
        assert_eq!(metadata.current_limit, 10);
        assert_eq!(metadata.next_page_url, Some("/api/items?offset=10&limit=10".to_string()));
        assert_eq!(metadata.prev_page_url, None);
    }

    #[test]
    fn test_pagination_metadata_middle_page() {
        let metadata = PaginationResponseMetadata::new(100, 20, 10, "/api/items");
        
        assert_eq!(metadata.next_page_url, Some("/api/items?offset=30&limit=10".to_string()));
        assert_eq!(metadata.prev_page_url, Some("/api/items?offset=10&limit=10".to_string()));
    }

    #[test]
    fn test_pagination_metadata_last_page() {
        let metadata = PaginationResponseMetadata::new(100, 90, 10, "/api/items");
        
        assert_eq!(metadata.next_page_url, None);
        assert_eq!(metadata.prev_page_url, Some("/api/items?offset=80&limit=10".to_string()));
    }

    #[test]
    fn test_pagination_metadata_single_page() {
        let metadata = PaginationResponseMetadata::new(5, 0, 10, "/api/items");
        
        assert_eq!(metadata.next_page_url, None);
        assert_eq!(metadata.prev_page_url, None);
    }

    #[test]
    fn test_pagination_metadata_exact_page_boundary() {
        let metadata = PaginationResponseMetadata::new(100, 90, 10, "/api/items");
        
        // At exact boundary, no next page
        assert_eq!(metadata.next_page_url, None);
    }

    #[test]
    fn test_pagination_response_creation() {
        let data = vec!["item1".to_string(), "item2".to_string()];
        let response = PaginationResponse::new(50, 10, 5, "/api/items", data.clone());
        
        assert_eq!(response.metadata.total_items, 50);
        assert_eq!(response.metadata.current_offset, 10);
        assert_eq!(response.metadata.current_limit, 5);
        assert_eq!(response.data, data);
    }

    #[test]
    fn test_pagination_response_serialization() {
        let data = vec![1, 2, 3];
        let response = PaginationResponse::new(10, 0, 3, "/api/numbers", data);
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PaginationResponse<i32> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.metadata.total_items, 10);
        assert_eq!(deserialized.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_pagination_metadata_zero_offset() {
        let metadata = PaginationResponseMetadata::new(50, 0, 20, "/api/test");
        
        assert_eq!(metadata.prev_page_url, None);
        assert_eq!(metadata.next_page_url, Some("/api/test?offset=20&limit=20".to_string()));
    }

    #[test]
    fn test_pagination_metadata_custom_limit() {
        let metadata = PaginationResponseMetadata::new(100, 15, 15, "/api/custom");
        
        assert_eq!(metadata.next_page_url, Some("/api/custom?offset=30&limit=15".to_string()));
        assert_eq!(metadata.prev_page_url, Some("/api/custom?offset=0&limit=15".to_string()));
    }
}
