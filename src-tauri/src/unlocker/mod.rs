use crate::api::unlocker::{fetch_beacon_information, request_unlock_permission};
use anyhow::Result;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub nonce: [u8; 16],
    pub timestamp: u64,
    #[serde(with = "BigArray")]
    pub server_signature: [u8; 64],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceProof {
    pub challenge_hash: [u8; 32],
    #[serde(with = "BigArray")]
    pub device_signature: [u8; 64],
    pub timestamp: u64,
    pub counter: u64,
}

pub struct Unlocker {
    device_private_key: SigningKey,
    server_public_key: VerifyingKey,
    user_id: String,
    counter: u64,
    /// The JWT token for the user session
    user_token: String,
}

impl Unlocker {
    pub fn new(
        device_private_key: SigningKey,
        server_public_key: VerifyingKey,
        user_id: String,
        user_token: String,
    ) -> Self {
        Self {
            device_private_key,
            server_public_key,
            user_id,
            counter: 0,
            user_token,
        }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_user_token(&self) -> &str {
        &self.user_token
    }

    pub async fn request_unlock(&self, nonce: [u8; 16], beacon: String) -> Result<Challenge> {
        let device_timestamp = chrono::Utc::now().timestamp() as u64;
        let beacon_information =
            fetch_beacon_information(beacon.as_str(), &self.user_token).await?;
        // beacon timestamp regards the epoch time as 0 in its clock, so we need to add the epoch time to it.
        let timestamp = beacon_information
            .epoch_time
            .saturating_add(beacon_information.major as u64);
        request_unlock_permission(nonce, beacon, timestamp, &self.user_token).await
    }

    fn verify_server_challenge(&self, challenge: &Challenge) -> Result<()> {
        let mut signed_data = Vec::with_capacity(16 + 8);
        signed_data.extend_from_slice(&challenge.nonce);
        signed_data.extend_from_slice(&challenge.timestamp.to_be_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&signed_data);
        let digest = hasher.finalize();

        let signature = Signature::from_bytes(&challenge.server_signature.into())?;
        self.server_public_key.verify(&digest, &signature)?;

        Ok(())
    }

    fn hash_challenge(challenge: &Challenge) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&challenge.nonce);
        hasher.update(&challenge.timestamp.to_be_bytes());
        hasher.update(&challenge.server_signature);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    pub fn generate_device_proof(&mut self, challenge: &Challenge) -> Result<DeviceProof> {
        self.counter += 1;

        let challenge_hash = Self::hash_challenge(challenge);

        let mut data_to_sign = Vec::with_capacity(32 + 8 + 8);
        data_to_sign.extend_from_slice(&challenge_hash);
        data_to_sign.extend_from_slice(&challenge.timestamp.to_be_bytes());
        data_to_sign.extend_from_slice(&self.counter.to_be_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&data_to_sign);
        let digest = hasher.finalize();

        let signature: Signature = self.device_private_key.sign(&digest);

        Ok(DeviceProof {
            challenge_hash,
            device_signature: signature.to_bytes().into(),
            timestamp: challenge.timestamp,
            counter: self.counter,
        })
    }
}
