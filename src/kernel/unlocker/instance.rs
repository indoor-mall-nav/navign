#![allow(unused)]
use crate::kernel::cryptography::UnlockChallenge;
use crate::schema::{BeaconSecrets, UserPublicKeys};
use crate::schema::{Beacon, Service};
use anyhow::{anyhow, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use base64::Engine;
use bson::doc;
use bson::oid::ObjectId;
use chrono::Duration;
use log::info;
use mongodb::Database;
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthenticationType {
    /// Bluetooth Low Energy, the pipeline implemented in this project
    Ble,
    /// Near-field communication
    Nfc,
    /// Traditional username/password
    Password,
    /// One-time password, usually from an authenticator app
    Otp,
    /// Direct biometrics on the unlocker device
    Biometrics
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum UnlockStage {
    Initiated,
    Verified,
    Completed,
    Failed,
}

impl std::fmt::Display for UnlockStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnlockStage::Initiated => write!(f, "initiated"),
            UnlockStage::Verified => write!(f, "verified"),
            UnlockStage::Completed => write!(f, "completed"),
            UnlockStage::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UnlockInstance {
    #[serde(rename = "_id")]
    id: ObjectId,
    pub beacon: String,
    pub timestamp: u64,
    pub beacon_nonce: String,
    pub challenge_nonce: String,
    pub user: String,
    pub device: String,
    pub stage: UnlockStage,
    pub outcome: String,
    pub r#type: AuthenticationType,
}

impl Service for UnlockInstance {
    fn get_id(&self) -> String {
        self.id.to_hex()
    }
    fn get_name(&self) -> String {
        self.id.to_hex()
    }
    fn set_name(&mut self, _name: String) {}
    fn get_description(&self) -> Option<String> {
        None
    }
    fn set_description(&mut self, _description: Option<String>) {}
    fn get_collection_name() -> &'static str {
        "unlock_instances"
    }
    fn require_unique_name() -> bool {
        true
    }
}

impl UnlockInstance {
    pub async fn create_instance(
        db: &Database,
        beacon: String,
        payload: String,
        device: String,
        user: String,
    ) -> Result<Self> {
        // Beacon message format: base64(nonce: 16; device signature tail: 4)
        let beacon = match BeaconSecrets::get_one_by_id(db, beacon.as_str()).await {
            Some(sec) => sec,
            None => anyhow::bail!("Beacon not found"),
        };
        let decoded = base64::engine::general_purpose::STANDARD.decode(payload)?;
        if decoded.len() != 20 {
            anyhow::bail!("Invalid beacon payload length");
        }
        let beacon_nonce: [u8; 16] = decoded[0..16]
            .try_into()
            .map_err(|_| anyhow!("Invalid beacon nonce length"))?;
        let beacon_signature_tail: [u8; 4] = decoded[16..20]
            .try_into()
            .map_err(|_| anyhow!("Invalid beacon signature tail length"))?;
        // Verify the beacon signature tail
        let mut hasher = Sha256::new();
        hasher.update(beacon_nonce);
        hasher.update(beacon.last_epoch.to_be_bytes());
        hasher.update(beacon.counter.to_be_bytes());
        let hash = hasher.finalize();
        let key = beacon
            .edcsa_key()
            .ok_or(anyhow!("Invalid beacon ECDSA key"))?;
        let signature: Signature = key.sign(&hash);
        if signature.to_bytes()[60..64] != beacon_signature_tail {
            anyhow::bail!("Invalid beacon signature tail");
        }
        info!("Beacon signature tail verified");
        let challenge_nonce = rand::random::<[u8; 16]>();
        // TODO verify if the user is allowed to unlock this beacon
        let instance = UnlockInstance {
            id: ObjectId::new(),
            beacon: beacon.get_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            beacon_nonce: base64::engine::general_purpose::STANDARD.encode(beacon_nonce),
            challenge_nonce: base64::engine::general_purpose::STANDARD.encode(challenge_nonce),
            device,
            user,
            stage: UnlockStage::Initiated,
            outcome: String::new(),
            r#type: AuthenticationType::Ble,
        };
        Ok(instance)
    }

    pub async fn update_instance(&self, db: &Database, signing_key: &SigningKey, signature: String, timestamp: u64) -> Result<String> {
        let signature = base64::engine::general_purpose::STANDARD.decode(signature)?;
        if signature.len() != 64 {
            anyhow::bail!("Invalid signature length");
        }
        let device = UserPublicKeys::get_one_by_id(db, self.device.as_str()).await;
        let device = device
            .and_then(|d| d.public_key())
            .ok_or(anyhow!("Device not found or invalid public key"))?;
        let signature = Signature::from_slice(&signature)
            .map_err(|_| anyhow!("Invalid signature format"))?;
        let mut hasher = Sha256::new();
        hasher.update(self.challenge_nonce.as_bytes());
        hasher.update(timestamp.to_be_bytes());
        let hash = hasher.finalize();
        if timestamp < self.timestamp {
            anyhow::bail!("Timestamp is older than the instance timestamp");
        }
        // Only allow biometrics in a 3-minute window
        if timestamp > self.timestamp + Duration::minutes(3).num_seconds() as u64 {
            anyhow::bail!("Timestamp is too old");
        }
        device.verify(&hash, &signature)
            .map_err(|_| anyhow!("Invalid signature"))?;
        info!("Device signature verified");
        self.update_stage(db, UnlockStage::Verified, None).await?;
        let proof = self.generate_proof(db, signing_key, signature.to_bytes()[56..64].try_into().unwrap()).await?;
        let proof_b64 = base64::engine::general_purpose::STANDARD.encode(proof);
        Ok(proof_b64)
    }

    async fn generate_proof(&self, db: &Database, signing_key: &SigningKey, device_signature_tail: [u8; 8]) -> Result<[u8; 64]> {
        let mut hasher = Sha256::new();
        hasher.update(self.beacon_nonce.as_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        let beacon = BeaconSecrets::get_one_by_id(db, self.beacon.as_str()).await.ok_or_else(|| anyhow!("Beacon not found"))?;
        hasher.update(beacon.counter.to_be_bytes());
        hasher.update(device_signature_tail);
        let hash = hasher.finalize();
        let signature: Signature = signing_key.sign(&hash);
        Ok(signature.to_bytes().into())
    }

    pub async fn record_results(&self, db: &Database, success: bool, outcome: String) -> Result<()> {
        if success {
            self.update_stage(db, UnlockStage::Completed, None).await?;
            let beacon = BeaconSecrets::get_one_by_id(db, self.beacon.as_str()).await;
            let mut beacon = beacon.ok_or(anyhow!("Beacon not found"))?;
            beacon.increment_counter(db).await?;
        } else {
            self.update_stage(db, UnlockStage::Failed, Some(outcome)).await?;
        }
        Ok(())
    }

    async fn update_stage(&self, db: &Database, stage: UnlockStage, outcome: Option<String>) -> Result<()> {
        let filter = doc! { "_id": &self.id };
        let update = doc! {
            "$set": {
                "stage": stage.to_string(),
                "outcome": outcome.unwrap_or_else(|| self.outcome.clone()),
            }
        };
        db.collection::<UnlockInstance>(UnlockInstance::get_collection_name()).update_one(filter, update).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Unlocker {
    nonce: String,
    beacon: String,
    timestamp: u64,
}

pub async fn unlocker_instance(
    unlocker: Unlocker,
    private_key: &SigningKey,
    permission_checker: fn(String) -> bool,
    db: &Database,
) -> Result<UnlockChallenge> {
    let beacon_instance = Beacon::get_one_by_id(db, unlocker.beacon.as_str()).await.ok_or_else(|| anyhow!("Beacon not found"))?;
    info!("Beacon instance: {:?}", beacon_instance);
    info!("Beacon last boot: {:?}", beacon_instance.last_boot);
    info!("Unlocker timestamp: {}", unlocker.timestamp);
    let timestamp_login = match unlocker
        .timestamp
        .checked_add(beacon_instance.last_boot.unwrap_or(0))
    {
        Some(timestamp) => timestamp,
        None => anyhow::bail!("Can not add beacon"),
    };
    let timestamp_current = chrono::Utc::now().timestamp() as u64;
    // Window: 5 minutes
    if timestamp_current > timestamp_login + Duration::minutes(5).num_seconds() as u64 {
        anyhow::bail!("Timestamp is too old")
    }
    info!(
        "Timestamp check passed: current {}, login {}",
        timestamp_current, timestamp_login
    );
    if !permission_checker(unlocker.beacon.clone()) {
        println!("Permission checker failed for beacon {}", unlocker.beacon);
        anyhow::bail!("Unlocker permission checker failure")
    }
    info!("Permission checker passed");

    let nonce = base64::engine::general_purpose::STANDARD.decode(&unlocker.nonce)?;
    info!("Nonce decoded: {:?}", nonce);
    println!("Hashed payload: {:?}", nonce);
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    hasher.update(unlocker.timestamp.to_be_bytes());
    let hash = hasher.finalize();

    info!("Hash generated: {:?}", hash);

    let signature: Signature = private_key.sign(&hash);

    info!("Signature: {:?}", signature);

    let nonce: [u8; 16] = nonce
        .try_into()
        .map_err(|_| anyhow!("Nonce does not satisfy the format"))?;

    info!("Unlock challenge generated successfully");

    Ok(UnlockChallenge {
        nonce,
        timestamp: unlocker.timestamp,
        server_signature: signature.to_bytes().into(),
    })
}

#[deprecated(note = "We will refactor this function later")]
pub async fn initiate_unlocker(
    State(state): State<AppState>,
    axum::Json(unlocker): axum::Json<Unlocker>,
) -> impl IntoResponse {
    info!("Handling unlock request for instance {:?}", unlocker);
    match unlocker_instance(unlocker, &state.private_key, |_| true, &state.db).await {
        Ok(challenge) => match serde_json::to_string(&challenge) {
            Ok(challenge) => (StatusCode::CREATED, challenge),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        },
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()),
    }
}
