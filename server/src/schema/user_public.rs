use bson::oid::ObjectId;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublicKeys {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user: String,
    pub model: String,
    pub arch: String,
    pub identifier: String,
    pub public_key: String,
}

impl UserPublicKeys {
    pub fn public_key(&self) -> Option<VerifyingKey> {
        VerifyingKey::from_public_key_pem(self.public_key.as_str()).ok()
    }
}
