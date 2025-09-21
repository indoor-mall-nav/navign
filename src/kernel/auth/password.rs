use crate::kernel::auth::{Authenticator, Token};
use crate::schema::User;
use anyhow::anyhow;
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;

pub struct PasswordAuthenticator {
    userid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPayload {
    pub userid: String,
    pub password: String,
    pub timestamp: u64,
    pub nonce: String,
    pub hash: String,
}

impl PasswordPayload {
    pub fn verify_hash(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.userid.as_bytes());
        hasher.update(self.password.as_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.nonce.as_bytes());
        let hash = hasher.finalize().to_vec();
        let hash_str = hex::encode(hash);
        self.hash == hash_str
    }
}

impl<'de> Authenticator<'de, PasswordPayload> for PasswordAuthenticator {
    async fn authenticate(
        &self,
        credential: PasswordPayload,
        db: &Database,
    ) -> anyhow::Result<String> {
        if !credential.verify_hash() {
            return Err(anyhow!("Hash does not match"));
        }
        let userid = ObjectId::from_str(&credential.userid)?;
        let user: User = match db
            .collection("user")
            .find_one(doc! {
                "_id": userid,
            })
            .await
            .ok()
            .flatten()
        {
            Some(user) => user,
            None => return Err(anyhow!("User does not exist")),
        };
        if user.verify_password(credential.password.as_str()) {
            let token = Token::from(&user);
            Ok(token.to_string())
        } else {
            Err(anyhow!("Wrong password"))
        }
    }

    fn userid(&self) -> String {
        self.userid.clone()
    }
}
