mod google;
mod github;
mod wechat;
mod password;

use std::env;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use anyhow::{anyhow, Result};

use bson::{doc};
use bson::oid::ObjectId;
use gravatar::Gravatar;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use crate::schema::User;

pub trait Authenticator<'de, T: Sized + Clone + Debug + Serialize + Deserialize<'de>> {
    async fn authenticate(&self, credential: T, db: &Database) -> Result<String>;

    async fn username(&self, db: &Database) -> Result<String> {
        let id = ObjectId::from_str(self.userid().as_str())?;
        let document: User = match db.collection("users").find_one(doc!{
            "_id": id,
        }).await? {
            Some(doc) => doc,
            None => return Err(anyhow!("User not found")),
        };
        Ok(document.username)
    }

    async fn avatar_url(&self, db: &Database) -> Result<String> {
        let id = ObjectId::from_str(self.userid().as_str())?;
        let document: User = match db.collection("users").find_one(doc!{
            "_id": id,
        }).await? {
            Some(doc) => doc,
            None => return Err(anyhow!("User not found")),
        };
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
    iat: i64,
    exp: i64,
    jti: String,
    scope: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(encoding_key) = env::var("JWT_SIGN_KEY") {
            let key = EncodingKey::from_secret(encoding_key.as_bytes());
            write!(f, "{}", jsonwebtoken::encode(&Header::default(), &self, &key).unwrap_or("<Invalid JWT Token>".to_string()))
        } else {
            write!(f, "<Invalid JWT Token>")
        }
    }
}

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let decoding_key = env::var("JWT_SIGN_KEY")?;
        let key = DecodingKey::from_secret(decoding_key.as_bytes());
        let validation = Validation::default();
        jsonwebtoken::decode::<Self>(&token, &key, &validation).map(|token| token.claims).map_err(Into::into)
    }
}

impl From<&User> for Token {
    fn from(user: &User) -> Self {
        let current = chrono::Utc::now();
        let iat = current.timestamp();
        let exp = current.timestamp() + chrono::Duration::minutes(30).num_seconds();
        let token_id = uuid::Uuid::new_v4().to_string();
        Self {
            iss: String::from("Navign"),
            sub: user.id.to_string(),
            name: user.username.to_string(),
            iat,
            exp,
            jti: token_id,
            scope: String::from("admin")
        }
    }
}