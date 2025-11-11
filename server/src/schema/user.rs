#![allow(dead_code)]
use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::schema::Service;
use async_trait::async_trait;
use bcrypt::hash;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub github_id: Option<String>,
    pub google_id: Option<String>,
    pub wechat_id: Option<String>,
    pub avatar_url: Option<String>,
    pub public_key: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        github_id: Option<String>,
        google_id: Option<String>,
        wechat_id: Option<String>,
        password: String,
    ) -> Self {
        let hashed_password = hash(password, 12).expect("Failed to hash password");
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4(),
            username,
            email: Some(email),
            password_hash: Some(hashed_password),
            github_id,
            google_id,
            wechat_id,
            avatar_url: None,
            public_key: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        if let Some(ref hash) = self.password_hash {
            bcrypt::verify(password, hash).unwrap_or(false)
        } else {
            false
        }
    }
}

#[async_trait]
impl Service for User {
    type Id = Uuid;

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_name(&self) -> String {
        self.username.clone()
    }

    fn set_name(&mut self, name: String) {
        self.username = name;
    }

    fn get_description(&self) -> Option<String> {
        self.email.clone()
    }

    fn set_description(&mut self, description: Option<String>) {
        self.email = description;
    }

    fn get_table_name() -> &'static str {
        "users"
    }

    fn require_unique_name() -> bool {
        true
    }

    async fn get_one_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    async fn get_one_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    async fn get_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
            .fetch_all(pool)
            .await
    }

    async fn get_with_pagination(
        pool: &PgPool,
        page: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * limit;
        let sort_column = sort.unwrap_or("username");
        let order = if asc { "ASC" } else { "DESC" };

        let query = format!(
            "SELECT * FROM users ORDER BY {} {} LIMIT $1 OFFSET $2",
            sort_column, order
        );

        sqlx::query_as::<_, User>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    async fn create(&self, pool: &PgPool) -> Result<Uuid, sqlx::Error> {
        let result = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, username, email, password_hash, github_id, google_id, wechat_id, avatar_url, public_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING *"
        )
        .bind(self.id)
        .bind(&self.username)
        .bind(&self.email)
        .bind(&self.password_hash)
        .bind(&self.github_id)
        .bind(&self.google_id)
        .bind(&self.wechat_id)
        .bind(&self.avatar_url)
        .bind(&self.public_key)
        .bind(self.created_at)
        .bind(self.updated_at)
        .fetch_one(pool)
        .await?;

        Ok(result.id)
    }

    async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "UPDATE users
             SET username = $2, email = $3, password_hash = $4, github_id = $5, google_id = $6,
                 wechat_id = $7, avatar_url = $8, public_key = $9, updated_at = $10
             WHERE id = $1"
        )
        .bind(self.id)
        .bind(&self.username)
        .bind(&self.email)
        .bind(&self.password_hash)
        .bind(&self.github_id)
        .bind(&self.google_id)
        .bind(&self.wechat_id)
        .bind(&self.avatar_url)
        .bind(&self.public_key)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn delete_by_name(pool: &PgPool, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(name)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn search_and_page_by_name_pattern(
        pool: &PgPool,
        params: crate::schema::service::SearchQueryParams<'_>,
    ) -> Result<PaginationResponse<Self>, sqlx::Error> {
        let pattern = if params.case_insensitive {
            format!("%{}%", params.pattern.to_lowercase())
        } else {
            format!("%{}%", params.pattern)
        };

        let sort_column = params.sort.unwrap_or("username");
        let order = if params.asc { "ASC" } else { "DESC" };

        let count_query = if params.case_insensitive {
            "SELECT COUNT(*) as count FROM users WHERE LOWER(username) LIKE $1"
        } else {
            "SELECT COUNT(*) as count FROM users WHERE username LIKE $1"
        };

        let total: (i64,) = sqlx::query_as(count_query)
            .bind(&pattern)
            .fetch_one(pool)
            .await?;

        let query = if params.case_insensitive {
            format!(
                "SELECT * FROM users WHERE LOWER(username) LIKE $1 ORDER BY {} {} LIMIT $2 OFFSET $3",
                sort_column, order
            )
        } else {
            format!(
                "SELECT * FROM users WHERE username LIKE $1 ORDER BY {} {} LIMIT $2 OFFSET $3",
                sort_column, order
            )
        };

        let data = sqlx::query_as::<_, User>(&query)
            .bind(&pattern)
            .bind(params.limit)
            .bind(params.offset)
            .fetch_all(pool)
            .await?;

        Ok(PaginationResponse {
            data,
            metadata: PaginationResponseMetadata {
                total: total.0,
                offset: params.offset,
                limit: params.limit,
            },
        })
    }

    async fn search_by_description_pattern(
        pool: &PgPool,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let pattern = if case_insensitive {
            format!("%{}%", pattern.to_lowercase())
        } else {
            format!("%{}%", pattern)
        };

        let query = if case_insensitive {
            "SELECT * FROM users WHERE email IS NOT NULL AND LOWER(email) LIKE $1 ORDER BY username"
        } else {
            "SELECT * FROM users WHERE email IS NOT NULL AND email LIKE $1 ORDER BY username"
        };

        sqlx::query_as::<_, User>(query)
            .bind(&pattern)
            .fetch_all(pool)
            .await
    }

    async fn bulk_create(
        pool: &PgPool,
        users: Vec<Self>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut ids = Vec::new();

        for user in users {
            let id = user.create(pool).await?;
            ids.push(id);
        }

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            None,
            None,
            None,
            "password123".to_string(),
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(user.password_hash.is_some());
        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrongpassword"));
    }

    #[test]
    fn test_verify_password() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            None,
            None,
            None,
            "correct_password".to_string(),
        );

        assert!(user.verify_password("correct_password"));
        assert!(!user.verify_password("wrong_password"));
    }

    #[test]
    fn test_user_with_oauth() {
        let user = User::new(
            "githubuser".to_string(),
            "github@example.com".to_string(),
            Some("github123".to_string()),
            Some("google456".to_string()),
            Some("wechat789".to_string()),
            "password".to_string(),
        );

        assert_eq!(user.github_id, Some("github123".to_string()));
        assert_eq!(user.google_id, Some("google456".to_string()));
        assert_eq!(user.wechat_id, Some("wechat789".to_string()));
    }

    #[test]
    fn test_get_id_returns_uuid() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            None,
            None,
            None,
            "password".to_string(),
        );

        let id = user.get_id();
        assert_eq!(id, user.id);
    }

    #[test]
    fn test_user_no_password_verify_fails() {
        let mut user = User::new(
            "oauthuser".to_string(),
            "oauth@example.com".to_string(),
            Some("github123".to_string()),
            None,
            None,
            "temp".to_string(),
        );

        // Simulate OAuth user with no password
        user.password_hash = None;

        assert!(!user.verify_password("anypassword"));
    }
}
