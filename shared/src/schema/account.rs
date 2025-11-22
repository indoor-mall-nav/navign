#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "postgres")]
use sqlx::Postgres;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    all(feature = "serde", not(feature = "postgres")),
    derive(Serialize, Deserialize)
)]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct Account {
    #[cfg(feature = "postgres")]
    pub id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    pub id: String,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub activated: bool,
    pub privileged: bool,
    #[cfg(feature = "postgres")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<String>, // Timestamp in milliseconds
    #[cfg(feature = "postgres")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<String>, // Timestamp in milliseconds
}

/// Request schema for user registration
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Request schema for user login
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response schema for authentication
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

/// Token claims for JWT authentication
#[cfg(all(feature = "alloc", feature = "serde"))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenClaims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
}

#[cfg(feature = "postgres")]
use crate::traits::repository::UuidRepository;

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl UuidRepository<Postgres> for Account {
    async fn create(pool: &sqlx::PgPool, item: &Self) -> sqlx::Result<()> {
        // Note: SQL schema has phone, google, wechat fields not in Account struct
        // Setting them to NULL for now
        sqlx::query(
            r#"INSERT INTO users (id, username, email, phone, google, wechat, hashed_password, activated, privileged)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#
        )
        .bind(item.id)
        .bind(&item.username)
        .bind(&item.email)
        .bind(None::<String>) // phone
        .bind(None::<String>) // google
        .bind(None::<String>) // wechat
        .bind(&item.hashed_password)
        .bind(item.activated)
        .bind(item.privileged)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_uuid(pool: &sqlx::PgPool, uuid: uuid::Uuid) -> sqlx::Result<Option<Self>> {
        let row = sqlx::query(
            r#"SELECT id, username, email, hashed_password, activated, privileged,
                      created_at, updated_at
               FROM users WHERE id = $1"#,
        )
        .bind(uuid)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            use sqlx::Row;
            Ok(Some(Account {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                email: row.try_get("email")?,
                hashed_password: row.try_get("hashed_password")?,
                activated: row.try_get("activated")?,
                privileged: row.try_get("privileged")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(pool: &sqlx::PgPool, id: uuid::Uuid, item: &Self) -> sqlx::Result<()> {
        sqlx::query(
            r#"UPDATE users
               SET username = $2, email = $3, hashed_password = $4, activated = $5, privileged = $6
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(&item.username)
        .bind(&item.email)
        .bind(&item.hashed_password)
        .bind(item.activated)
        .bind(item.privileged)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, uuid: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(pool: &sqlx::PgPool, offset: i64, limit: i64) -> sqlx::Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"SELECT id, username, email, hashed_password, activated, privileged,
                      created_at, updated_at
               FROM users
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let mut accounts = Vec::new();
        for row in rows {
            use sqlx::Row;
            accounts.push(Account {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                email: row.try_get("email")?,
                hashed_password: row.try_get("hashed_password")?,
                activated: row.try_get("activated")?,
                privileged: row.try_get("privileged")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(accounts)
    }

    async fn search(
        pool: &sqlx::PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, username, email, hashed_password, activated, privileged,
                          created_at, updated_at
                   FROM users
                   WHERE username ILIKE $1 OR email ILIKE $1
                   ORDER BY {} {}
                   LIMIT $2 OFFSET $3"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, username, email, hashed_password, activated, privileged,
                          created_at, updated_at
                   FROM users
                   WHERE username LIKE $1 OR email LIKE $1
                   ORDER BY {} {}
                   LIMIT $2 OFFSET $3"#,
                order_by, direction
            )
        };

        let rows = sqlx::query(&sql)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let mut accounts = Vec::new();
        for row in rows {
            use sqlx::Row;
            accounts.push(Account {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                email: row.try_get("email")?,
                hashed_password: row.try_get("hashed_password")?,
                activated: row.try_get("activated")?,
                privileged: row.try_get("privileged")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(accounts)
    }

    async fn count(pool: &sqlx::PgPool, query: &str, case_insensitive: bool) -> sqlx::Result<i64> {
        let like_pattern = format!("%{}%", query);

        let sql = if case_insensitive {
            r#"SELECT COUNT(*) as count
               FROM users
               WHERE username ILIKE $1 OR email ILIKE $1"#
        } else {
            r#"SELECT COUNT(*) as count
               FROM users
               WHERE username LIKE $1 OR email LIKE $1"#
        };

        let row: (i64,) = sqlx::query_as(sql)
            .bind(&like_pattern)
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }
}
