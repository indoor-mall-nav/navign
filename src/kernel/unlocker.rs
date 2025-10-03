use crate::kernel::cryptography::UnlockChallenge;
use crate::schema::{Beacon, Service};
use anyhow::{Result, anyhow};
use base64::Engine;
use bson::doc;
use chrono::Duration;
use log::info;
use mongodb::Database;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    let beacon_instance = match Beacon::get_one_by_id(db, unlocker.beacon.as_str()).await {
        Some(inst) => inst,
        None => anyhow::bail!("Beacon not found"),
    };
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
