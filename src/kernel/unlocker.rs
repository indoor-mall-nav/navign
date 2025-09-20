use std::str::FromStr;
use axum::{Extension, Json};
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Database;
use crate::kernel::cryptography::UnlockChallenge;
use crate::schema::{Beacon, User};
use anyhow::{anyhow, Result};
use chrono::Duration;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Unlocker {
    nonce: String,
    beacon: String,
    timestamp: u64
}

pub async fn unlocker_instance(
    unlocker: Unlocker,
    private_key: &SigningKey,
    permission_checker: fn(String) -> bool,
    db: &Database
) -> Result<UnlockChallenge> {
    let beacon_id = ObjectId::from_str(&unlocker.beacon)?;
    let beacon_instance: Beacon = match db.collection("beacons").find_one(doc! {
        "beacon": beacon_id
    }).await.ok().flatten() {
        Some(beacon) => beacon,
        None => anyhow::bail!("Can not find beacon")
    };
    let timestamp_login = match beacon_instance.last_boot.checked_add(unlocker.timestamp) {
        Some(timestamp) => timestamp,
        None => anyhow::bail!("Can not add beacon")
    };
    let timestamp_current = chrono::Utc::now().timestamp() as u64;
    // Window: 5 minutes
    if timestamp_current > timestamp_login + Duration::minutes(5).num_seconds() as u64 {
        anyhow::bail!("Timestamp is too old")
    }
    if !permission_checker(unlocker.beacon.clone()) {
        anyhow::bail!("Unlocker permission checker failure")
    }

    let nonce = hex::decode(unlocker.nonce)?;
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    hasher.update(unlocker.timestamp.to_be_bytes());
    let hash = hasher.finalize();

    let signature: Signature = private_key.sign(&hash);

    let nonce: [u8; 16] = nonce.try_into().map_err(|_| anyhow!("Nonce does not satisfy the format"))?;

    Ok(UnlockChallenge {
        nonce,
        timestamp: unlocker.timestamp,
        server_signature: signature.to_bytes().into()
    })
}