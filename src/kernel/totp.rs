use anyhow::Result;
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
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;

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
        Ok(self.generate_with_counter(counter))
    }

    fn generate_with_counter(&self, counter: u64) -> String {
        // Convert counter to 8-byte big-endian array
        let counter_bytes = counter.to_be_bytes();

        // Create HMAC-SHA1 hash
        let mut mac = Hmac::<Sha1>::new_from_slice(self.secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(&counter_bytes);
        let result = mac.finalize().into_bytes();

        // Dynamic truncation
        let offset = (result[19] & 0xf) as usize;
        let binary = ((result[offset] & 0x7f) as u32) << 24
            | (result[offset + 1] as u32) << 16
            | (result[offset + 2] as u32) << 8
            | result[offset + 3] as u32;

        // Generate the final code
        let code = binary % (10_u32.pow(6));
        format!("{:0width$}", code, width = 6usize)
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

        let totp = beacon.generate_totp().unwrap();
        println!("Generated TOTP: {}", totp);
        assert_eq!(totp.len(), 6);
    }
}
