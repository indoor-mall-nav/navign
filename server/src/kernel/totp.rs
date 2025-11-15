#![allow(unused)]
use crate::schema::Service;
use anyhow::{Context, Result};
/// Beacon and Server handshake.
/// 1. The beacon sends its info to the server, and the server compare it with the database.
/// 2. If the beacon is registered, the server sends back the timestamp back so that the beacon could adjust its clock.
/// 3. The beacon generates a TOTP code using the shared secret and the timestamp, and send it to the server.
/// 4. The server verifies the TOTP code, and if it is valid, the server set the beacon ready for unlock.
///
/// The procedure of unlocking.
/// 1. The user (with phone) connect with beacon via BLE.
/// 2. The beacon generate a challenge including its beacon ID, timestamp (since epoch), and a random nonce, and send it to the phone.
/// 3. The user forward the challenge to the server via Internet for permission check.
/// 4. If the challenge passed, the server generate a new TOTP code using the shared secret and the current timestamp, and send it to the phone.
/// 5. The phone forward the TOTP code to the beacon via BLE.
/// 6. The beacon verifies the TOTP code, and if it is valid, the beacon unlock the door.
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use totp_lite::{Sha1, totp_custom};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BeaconSecret {
    /// MAC address of the beacon
    pub(crate) mac: String,
    /// UUID of the beacon
    pub(crate) uuid: String,
    /// Database ID of the beacon
    pub(crate) id: ObjectId,
    /// Client secret for the beacon used for TOTP generation
    pub(crate) secret: String,
    /// Time zone diff to UTC in hours
    pub(crate) tz_diff: i64,
    /// Last epoch timestamp (so that the beacon could adjust its clock)
    pub(crate) last_timestamp: u64,
    /// Is active
    pub(crate) is_active: bool,
}

impl BeaconSecret {
    pub(crate) fn new(
        mac: String,
        uuid: String,
        id: ObjectId,
        secret: String,
        tz_diff: i64,
        last_timestamp: u64,
        is_active: bool,
    ) -> Self {
        Self {
            mac,
            uuid,
            id,
            secret,
            tz_diff,
            last_timestamp,
            is_active,
        }
    }

    pub(crate) fn generate_totp(&self) -> Result<String> {
        if !self.is_active {
            anyhow::bail!("Beacon is not active");
        }

        let timestamp = (chrono::Utc::now().timestamp() + (self.tz_diff * 3600)) / 30;

        println!("Timestamp: {}", chrono::Utc::now().timestamp());

        let time_diff = (self.last_timestamp as i64 - self.tz_diff * 3600) / 30;

        if timestamp <= time_diff {
            anyhow::bail!("Timestamp is not valid");
        }

        let timestamp = timestamp - time_diff;

        let counter = timestamp as u64 / 30;
        self.generate_with_counter(counter)
    }

    fn generate_with_counter(&self, counter: u64) -> Result<String> {
        // Use audited totp-lite crate (RFC 6238 compliant)
        let secret_bytes = self.secret.as_bytes();

        // Generate TOTP using totp-lite with custom time step (30 seconds)
        let code = totp_custom::<Sha1>(30, 6, secret_bytes, counter);

        Ok(format!("{:06}", code))
    }
}

impl Service for BeaconSecret {
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
        Some(self.uuid.clone())
    }

    fn set_description(&mut self, description: Option<String>) {
        if let Some(desc) = description {
            self.uuid = desc;
        }
    }

    fn get_collection_name() -> &'static str {
        "beacon_secrets"
    }

    fn require_unique_name() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation() {
        let beacon = BeaconSecret::new(
            "00:11:22:33:44:55".to_string(),
            "123e4567-e89b-12d3-a456-426614174000".to_string(),
            ObjectId::new(),
            "JBSWY3DPEHPK3PXP".to_string(), // Base32 for "Hello!"
            0,
            1757142749,
            true,
        );

        let totp = beacon
            .generate_totp()
            .expect("TOTP generation should succeed for active beacon");
        println!("Generated TOTP: {}", totp);
        assert_eq!(totp.len(), 6);
    }
}
