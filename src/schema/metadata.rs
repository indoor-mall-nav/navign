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
        let next_page_url = next_offset.map(|offset| {
            format!(
                "{}?offset={}&limit={}",
                api_base_url, offset, current_limit
            )
        });
        let prev_page_url = prev_offset.map(|offset| {
            format!(
                "{}?offset={}&limit={}",
                api_base_url, offset, current_limit
            )
        });
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