use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResult<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub query: Option<String>,
    pub sort: Option<String>,
    pub asc: Option<bool>,
    pub case_sensitive: Option<bool>,
}
