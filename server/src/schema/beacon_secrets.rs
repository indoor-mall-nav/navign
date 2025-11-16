use bson::oid::ObjectId;
use p256::pkcs8::DecodePrivateKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconSecrets {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub mac: String,
    pub last_epoch: u64,
    pub counter: u64,
    /// The PEM format of the ECDSA private key
    pub ecdsa_key: String,
}

impl BeaconSecrets {
    pub fn new(mac: String, ecdsa_key: String) -> Self {
        Self {
            id: ObjectId::new(),
            mac,
            last_epoch: 0,
            counter: 0,
            ecdsa_key,
        }
    }

    pub fn epoch(&mut self, epoch: u64) {
        self.last_epoch = epoch;
    }

    pub fn ecdsa_key(&self) -> Option<p256::ecdsa::SigningKey> {
        p256::ecdsa::SigningKey::from_pkcs8_pem(self.ecdsa_key.as_str()).ok()
    }

    pub async fn increment_counter(&mut self, db: &mongodb::Database) -> anyhow::Result<()> {
        let new_counter = self
            .counter
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Counter overflow"))?;

        if new_counter > i64::MAX as u64 {
            return Err(anyhow::anyhow!(
                "Counter exceeds max value allowed in database"
            ));
        }

        // TODO
        // let collection = db.collection::<BeaconSecrets>(Self::get_collection_name());
        // collection
        //     .update_one(
        //         bson::doc! { "_id": &self.id },
        //         bson::doc! { "$set": { "counter": new_counter as i64 } },
        //     )
        //     .await?;
        Ok(())
    }
}
