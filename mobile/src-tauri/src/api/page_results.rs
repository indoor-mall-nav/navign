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
