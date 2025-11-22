#![allow(dead_code)]
mod github;
mod google;
mod handlers;
mod token;
mod wechat;

// pub use handlers::{login_handler, register_handler};

use anyhow::Result;
use gravatar::Gravatar;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use navign_shared::Account;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Row;
use std::env;
use std::fmt::{Debug, Display};
use std::str::FromStr;

pub trait Authenticator<'de, T: Sized + Clone + Debug + Serialize + Deserialize<'de>> {
    async fn authenticate(&self, credential: T, db: &PgPool) -> Result<String>;

    async fn username(&self, db: &PgPool) -> Result<String> {
        let result = sqlx::query("SELECT username FROM accounts WHERE id = $1")
            .bind(self.userid())
            .fetch_one(db)
            .await?;
        let username: String = result.try_get("username")?;
        Ok(username)
    }

    async fn email(&self, db: &PgPool) -> Result<String> {
        let result = sqlx::query("SELECT email FROM accounts WHERE id = $1")
            .bind(self.userid())
            .fetch_one(db)
            .await?;
        let email: String = result.try_get("email")?;
        Ok(email)
    }

    async fn avatar_url(&self, db: &PgPool) -> Result<String> {
        let email = self.email(db).await?;

        let gravatar = Gravatar::new(&email);
        Ok(gravatar.image_url().into_string())
    }

    fn userid(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    iss: String,
    sub: String,
    name: String,
    device: String,
    iat: i64,
    exp: i64,
    jti: String,
    scope: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(encoding_key) = env::var("JWT_SIGN_KEY") {
            let key = EncodingKey::from_secret(encoding_key.as_bytes());
            write!(
                f,
                "{}",
                jsonwebtoken::encode(&Header::default(), &self, &key)
                    .unwrap_or("<Invalid JWT Token>".to_string())
            )
        } else {
            write!(f, "<Invalid JWT Token>")
        }
    }
}

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let mut token = token;
        if token.starts_with("Bearer") {
            token = token.trim_start_matches("Bearer ").trim();
        }
        let decoding_key = env::var("JWT_SIGN_KEY")?;
        let key = DecodingKey::from_secret(decoding_key.as_bytes());
        let validation = Validation::default();
        jsonwebtoken::decode::<Self>(token, &key, &validation)
            .map(|token| token.claims)
            .map_err(Into::into)
    }
}

impl From<(&Account, String)> for Token {
    fn from((user, device): (&Account, String)) -> Self {
        let now = chrono::Utc::now().timestamp();
        Token {
            iss: "Navign".to_string(),
            sub: user.id.to_string(),
            name: user.username.clone(),
            device,
            iat: now,
            exp: now + 5 * 3600 * 24, // 5 days
            jti: uuid::Uuid::new_v4().to_string(),
            scope: "user".to_string(),
        }
    }
}
