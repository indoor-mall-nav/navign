use crate::schema::Service;
use uuid::Uuid;
use sqlx::{FromRow, PgPool};
use p256::pkcs8::DecodePrivateKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BeaconSecrets {
    pub id: Uuid,
    pub mac: String,
    pub last_epoch: i64,
    pub counter: i64,
    /// The PEM format of the ECDSA private key
    pub ecdsa_key: String,
}

impl Service for BeaconSecrets {
    type Id = Uuid;

    fn get_id(&self) -> Uuid {
        self.id
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
    fn get_table_name() -> &'static str {
        "beacon_secrets"
    }
    fn require_unique_name() -> bool {
        true
    }
}

impl BeaconSecrets {
    pub fn new(mac: String, ecdsa_key: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            mac,
            last_epoch: 0,
            counter: 0,
            ecdsa_key,
        }
    }

    pub fn epoch(&mut self, epoch: u64) {
        self.last_epoch = epoch as i64;
    }

    pub fn ecdsa_key(&self) -> Option<p256::ecdsa::SigningKey> {
        p256::ecdsa::SigningKey::from_pkcs8_pem(self.ecdsa_key.as_str()).ok()
    }

    pub async fn increment_counter(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        let new_counter = self
            .counter
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Counter overflow"))?;

        if new_counter > i64::MAX {
            return Err(anyhow::anyhow!(
                "Counter exceeds max value allowed in database"
            ));
        }

        sqlx::query!(
            "UPDATE beacon_secrets SET counter = $1 WHERE id = $2",
            new_counter,
            self.id
        )
        .execute(pool)
        .await?;

        self.counter = new_counter;
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
    async fn test_increment_counter() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/navign_test".to_string());
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Create test table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS beacon_secrets (
                id UUID PRIMARY KEY,
                mac TEXT NOT NULL,
                last_epoch BIGINT NOT NULL,
                counter BIGINT NOT NULL,
                ecdsa_key TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut beacon = BeaconSecrets::new("AA:BB:CC:DD:EE:FF".to_string(), "".to_string());
        beacon.counter = 0;

        // Insert test beacon
        sqlx::query!(
            "INSERT INTO beacon_secrets (id, mac, last_epoch, counter, ecdsa_key) VALUES ($1, $2, $3, $4, $5)",
            beacon.id,
            beacon.mac,
            beacon.last_epoch,
            beacon.counter,
            beacon.ecdsa_key
        )
        .execute(&pool)
        .await
        .unwrap();

        // This should succeed
        assert!(beacon.increment_counter(&pool).await.is_ok());

        let updated_beacon = sqlx::query_as!(
            BeaconSecrets,
            "SELECT id, mac, last_epoch, counter, ecdsa_key FROM beacon_secrets WHERE id = $1",
            beacon.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(updated_beacon.counter, 1);

        // Cleanup
        sqlx::query!("DELETE FROM beacon_secrets WHERE id = $1", beacon.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_increment_counter_panic() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/navign_test".to_string());
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Create test table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS beacon_secrets (
                id UUID PRIMARY KEY,
                mac TEXT NOT NULL,
                last_epoch BIGINT NOT NULL,
                counter BIGINT NOT NULL,
                ecdsa_key TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut beacon = BeaconSecrets::new("AA:BB:CC:DD:EE:FF".to_string(), "".to_string());
        beacon.counter = i64::MAX;

        // Insert test beacon
        sqlx::query!(
            "INSERT INTO beacon_secrets (id, mac, last_epoch, counter, ecdsa_key) VALUES ($1, $2, $3, $4, $5)",
            beacon.id,
            beacon.mac,
            beacon.last_epoch,
            beacon.counter,
            beacon.ecdsa_key
        )
        .execute(&pool)
        .await
        .unwrap();

        // This should fail with counter overflow
        beacon.increment_counter(&pool).await.unwrap();

        // Cleanup
        sqlx::query!("DELETE FROM beacon_secrets WHERE id = $1", beacon.id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
