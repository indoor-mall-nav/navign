use bson::oid::ObjectId;
use p256::pkcs8::DecodePrivateKey;
use serde::{Deserialize, Serialize};
use crate::schema::Service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconSecrets {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub mac: String,
    pub last_epoch: u64,
    pub counter: u64,
    /// The PEM format of the ECDSA private key
    pub edcsa_key: String,
}

impl Service for BeaconSecrets {
    fn get_id(&self) -> String {
        self.id.to_hex()
    }
    fn get_name(&self) -> String {
        self.mac.clone()
    }
    fn set_name(&mut self, name: String) {
        self.mac = name;
    }
    fn get_description(&self) -> Option<String> {
        None
    }
    fn set_description(&mut self, _description: Option<String>) {}
    fn get_collection_name() -> &'static str {
        "beacon_secrets"
    }
    fn require_unique_name() -> bool {
        true
    }
}

impl BeaconSecrets {
    pub fn edcsa_key(&self) -> Option<p256::ecdsa::SigningKey> {
        p256::ecdsa::SigningKey::from_pkcs8_pem(self.edcsa_key.as_str()).ok()
    }

    pub async fn increment_counter(&mut self, db: &mongodb::Database) -> anyhow::Result<()> {
        self.counter.checked_add(1).ok_or_else(|| anyhow::anyhow!("Counter overflow"))?;
        let collection = db.collection::<BeaconSecrets>(Self::get_collection_name());
        collection.update_one(
            bson::doc! { "_id": &self.id },
            bson::doc! { "$set": { "counter": self.counter as i64 } },
        ).await?;
        Ok(())
    }
}