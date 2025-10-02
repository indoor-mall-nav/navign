use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub query: Option<String>,
    pub sort: Option<String>,
    pub asc: Option<bool>,
    pub case_sensitive: Option<bool>,
}
