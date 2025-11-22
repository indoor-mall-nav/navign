#![allow(unused)]
#![allow(clippy::too_many_arguments)]

#[cfg(feature = "sql")]
use sqlx::Result;
use sqlx::{Database, Pool};
#[cfg(feature = "sql")]
use uuid::Uuid;

#[cfg(feature = "sql")]
#[async_trait::async_trait]
pub trait IntRepository<T: Database>: Sized {
    async fn create(pool: &Pool<T>, item: &Self, entity: Uuid) -> Result<()>;
    async fn get_by_id(pool: &Pool<T>, id: i32, entity: Uuid) -> Result<Option<Self>>;
    async fn update(pool: &Pool<T>, item: &Self, entity: Uuid) -> Result<()>;
    async fn delete(pool: &Pool<T>, id: i32, entity: Uuid) -> Result<()>;
    async fn list(pool: &Pool<T>, offset: i64, limit: i64, entity: Uuid) -> Result<Vec<Self>>;
    async fn search(
        pool: &Pool<T>,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        entity: Uuid,
    ) -> Result<Vec<Self>>;
}

#[cfg(feature = "sql")]
#[async_trait::async_trait]
pub trait IntRepositoryInArea<T: Database>: IntRepository<T> {
    async fn search_in_area(
        pool: &Pool<T>,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: Uuid,
    ) -> Result<Vec<Self>>;
}

#[cfg(feature = "sql")]
#[async_trait::async_trait]
pub trait UuidRepository<T: Database>: Sized {
    async fn create(pool: &Pool<T>, item: &Self) -> Result<()>;
    async fn get_by_uuid(pool: &Pool<T>, uuid: Uuid) -> Result<Option<Self>>;
    async fn update(pool: &Pool<T>, item: &Self) -> Result<()>;
    async fn delete(pool: &Pool<T>, uuid: Uuid) -> Result<()>;
    async fn list(pool: &Pool<T>, offset: i64, limit: i64) -> Result<Vec<Self>>;
    async fn search(
        pool: &Pool<T>,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>>;
}
