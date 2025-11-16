#![allow(async_fn_in_trait)]
#![allow(clippy::too_many_arguments)]

#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "sql")]
use sqlx::Result;
#[cfg(not(feature = "postgres"))]
use sqlx::SqlitePool;
#[cfg(feature = "sql")]
use uuid::Uuid;

#[cfg(feature = "sql")]
#[async_trait::async_trait]
pub trait IntRepository: Sized {
    #[cfg(feature = "postgres")]
    async fn create(pool: &PgPool, item: &Self, entity: Uuid) -> Result<()>;
    #[cfg(not(feature = "postgres"))]
    async fn create(pool: &SqlitePool, item: &Self, entity: Uuid) -> Result<()>;
    #[cfg(feature = "postgres")]
    async fn get_by_id(pool: &PgPool, id: i32, entity: Uuid) -> Result<Option<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn get_by_id(pool: &SqlitePool, id: i32, entity: Uuid) -> Result<Option<Self>>;
    #[cfg(feature = "postgres")]
    async fn update(pool: &PgPool, item: &Self, entity: Uuid) -> Result<()>;
    #[cfg(not(feature = "postgres"))]
    async fn update(pool: &SqlitePool, item: &Self, entity: Uuid) -> Result<()>;
    #[cfg(feature = "postgres")]
    async fn delete(pool: &PgPool, id: i32, entity: Uuid) -> Result<()>;
    #[cfg(not(feature = "postgres"))]
    async fn delete(pool: &SqlitePool, id: i32, entity: Uuid) -> Result<()>;
    #[cfg(feature = "postgres")]
    async fn list(pool: &PgPool, offset: i64, limit: i64, entity: Uuid) -> Result<Vec<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn list(pool: &SqlitePool, offset: i64, limit: i64, entity: Uuid) -> Result<Vec<Self>>;
    #[cfg(feature = "postgres")]
    async fn search(
        pool: &PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        entity: Uuid,
    ) -> Result<Vec<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn search(
        pool: &SqlitePool,
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
pub trait IntRepositoryInArea: IntRepository {
    #[cfg(feature = "postgres")]
    async fn search_in_area(
        pool: &PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: Uuid,
    ) -> Result<Vec<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn search_in_area(
        pool: &SqlitePool,
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
pub trait UuidRepository: Sized {
    #[cfg(feature = "postgres")]
    async fn create(pool: &PgPool, item: &Self) -> Result<()>;
    #[cfg(not(feature = "postgres"))]
    async fn create(pool: &SqlitePool, item: &Self) -> Result<()>;
    #[cfg(feature = "postgres")]
    async fn get_by_uuid(pool: &PgPool, uuid: Uuid) -> Result<Option<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn get_by_uuid(pool: &SqlitePool, uuid: Uuid) -> Result<Option<Self>>;
    #[cfg(feature = "postgres")]
    async fn update(pool: &PgPool, item: &Self) -> Result<()>;
    #[cfg(not(feature = "postgres"))]
    async fn update(pool: &SqlitePool, item: &Self) -> Result<()>;
    #[cfg(feature = "postgres")]
    async fn delete(pool: &PgPool, uuid: Uuid) -> Result<()>;
    #[cfg(not(feature = "postgres"))]
    async fn delete(pool: &SqlitePool, uuid: Uuid) -> Result<()>;
    #[cfg(feature = "postgres")]
    async fn list(pool: &PgPool, offset: i64, limit: i64) -> Result<Vec<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn list(pool: &SqlitePool, offset: i64, limit: i64) -> Result<Vec<Self>>;
    #[cfg(feature = "postgres")]
    async fn search(
        pool: &PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>>;
    #[cfg(not(feature = "postgres"))]
    async fn search(
        pool: &SqlitePool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>>;
}
