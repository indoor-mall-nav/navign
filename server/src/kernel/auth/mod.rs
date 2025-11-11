#![allow(dead_code)]
mod github;
mod google;
mod handlers;
mod password;
mod token;
mod wechat;

pub use handlers::{login_handler, register_handler};
pub use token::UserData;

use anyhow::{Result, anyhow};
use std::env;
use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::schema::User;
use gravatar::Gravatar;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait Authenticator<'de, T: Sized + Clone + Debug + Serialize + Deserialize<'de>> {
    async fn authenticate(&self, credential: T, db: &PgPool) -> Result<String>;

    async fn username(&self, db: &PgPool) -> Result<String> {
        let id = Uuid::from_str(self.userid().as_str())?;
        let document: User = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|_| anyhow!("User not found"))?;

        Ok(document.username)
    }

    async fn avatar_url(&self, db: &PgPool) -> Result<String> {
        let id = Uuid::from_str(self.userid().as_str())?;
        let document: User = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|_| anyhow!("User not found"))?;

        let gravatar = Gravatar::new(document.email.as_str()).image_url();
        Ok(gravatar.to_string())
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

impl From<(&User, String)> for Token {
    fn from((user, device): (&User, String)) -> Self {
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
