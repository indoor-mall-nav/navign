use crate::schema::Service;
use bson::oid::ObjectId;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublicKeys {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub model: String,
    pub arch: String,
    pub identifier: String,
    pub public_key: String,
}

impl Service for UserPublicKeys {
    fn get_id(&self) -> String {
        self.id.to_hex()
    }
    fn get_name(&self) -> String {
        self.identifier.clone()
    }
    fn set_name(&mut self, name: String) {
        self.identifier = name;
    }
    fn get_description(&self) -> Option<String> {
        None
    }
    fn set_description(&mut self, _description: Option<String>) {}
    fn get_collection_name() -> &'static str {
        "user_public_keys"
    }
    fn require_unique_name() -> bool {
        true
    }
}

impl UserPublicKeys {
    pub fn public_key(&self) -> Option<VerifyingKey> {
        VerifyingKey::from_public_key_pem(self.public_key.as_str()).ok()
    }
}
