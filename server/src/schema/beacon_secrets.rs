use crate::schema::Service;
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

        let collection = db.collection::<BeaconSecrets>(Self::get_collection_name());
        collection
            .update_one(
                bson::doc! { "_id": &self.id },
                bson::doc! { "$set": { "counter": new_counter as i64 } },
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::rand_core::OsRng;
    use p256::pkcs8::EncodePrivateKey;

    #[test]
    fn test_ecdsa_key() {
        let signing_key = SigningKey::random(&mut OsRng);
        let pem = signing_key
            .to_pkcs8_pem(Default::default())
            .unwrap()
            .to_string();
        let beacon = BeaconSecrets::new("AA:BB:CC:DD:EE:FF".to_string(), pem.clone());
        let recovered_key = beacon.ecdsa_key().unwrap();
        assert_eq!(signing_key.to_bytes(), recovered_key.to_bytes());
    }

    #[tokio::test]
    #[ignore = "requires MongoDB running on localhost:27017"]
    async fn test_increment_counter() {
        let mut beacon = BeaconSecrets::new("AA:BB:CC:DD:EE:FF".to_string(), "".to_string());
        let counter = 0;
        beacon.counter = counter;
        let db_test = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let db = db_test.database("test_db");
        let collection = db.collection::<BeaconSecrets>(BeaconSecrets::get_collection_name());
        collection.delete_many(bson::doc! {}).await.unwrap();
        collection.insert_one(&beacon).await.unwrap();

        // This should succeed
        assert!(beacon.increment_counter(&db).await.is_ok());
        let new_beacon = collection
            .find_one(bson::doc! { "_id": &beacon.id })
            .await
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(new_beacon.counter, 1);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_increment_counter_panic() {
        let mut beacon = BeaconSecrets::new("AA:BB:CC:DD:EE:FF".to_string(), "".to_string());
        let counter = i64::MAX as u64;
        beacon.counter = counter;
        let db_test = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let db = db_test.database("test_db");
        let collection = db.collection::<BeaconSecrets>(BeaconSecrets::get_collection_name());
        collection.delete_many(bson::doc! {}).await.unwrap();
        collection.insert_one(&beacon).await.unwrap();

        // This should succeed
        assert!(beacon.increment_counter(&db).await.is_ok());
        let new_beacon = collection
            .find_one(bson::doc! { "_id": &beacon.id })
            .await
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(new_beacon.counter, 1);
    }
}
