// Copyright (c) 2025 Ethan Wu
// SPDX-License-Identifier: MIT

//! Unlocker module for secure device authentication and communication with a server.
//! This module provides functionalities to handle cryptographic operations,
//! including key management, signing challenges, and generating proofs of device authenticity.
//! It uses ECDSA for signing and RSA for encrypting AES keys, ensuring secure communication.
//! The module is designed to work in a Tauri application environment, leveraging stronghold for secure key storage.
use crate::api::unlocker::{fetch_beacon_information, request_unlock_permission};
use aes_gcm::aead::Aead;
use aes_gcm::KeyInit;
use anyhow::Result;
use base64::Engine;
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use rsa::Pkcs1v15Encrypt;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
#[cfg(mobile)]
use tauri_plugin_biometric::AuthOptions;
#[cfg(mobile)]
use tauri_plugin_biometric::BiometricExt;
use tauri_plugin_stronghold::stronghold::Stronghold;

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
    server_public_key: VerifyingKey,
    user_id: String,
    counter: u64,
    /// The JWT token for the user session
    user_token: String,
}

impl Unlocker {
    pub fn new(server_public_key: VerifyingKey, user_id: String, user_token: String) -> Self {
        Self {
            server_public_key,
            user_id,
            counter: 0,
            user_token,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_user_token(&self) -> &str {
        &self.user_token
    }

    pub fn set_user_token(&mut self, token: String) {
        self.user_token = token;
    }

    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }

    pub fn ensure_signing_key(&self, handle: AppHandle) -> Result<SigningKey> {
        #[cfg(mobile)]
        let auth_options = AuthOptions {
            allow_device_credential: true,
            cancel_title: Some("Cancel".to_string()),
            fallback_title: Some("Use Passcode".to_string()),
            title: Some("Authenticate to unlock".to_string()),
            subtitle: Some("Please authenticate to load data".to_string()),
            confirmation_required: Some(true),
        };

        #[cfg(mobile)]
        app.biometric()
            .authenticate("Please authenticate to load data".to_string(), auth_options)
            .map_err(|_| ())?;

        #[cfg(all(desktop, not(debug_assertions)))]
        panic!("Desktop release build is not supported due to biometric limitations.");

        let user_device_private_key_path = handle.path().app_local_data_dir()?.join("holder.db");
        let client_path = handle.path().app_local_data_dir()?.join("client.db");
        let path = client_path.to_string_lossy();
        if !user_device_private_key_path.exists() {
            let signing_key = SigningKey::random(&mut OsRng);
            let der = signing_key.to_pkcs8_der()?;
            let holder = Stronghold::new(
                user_device_private_key_path.clone(),
                der.as_bytes().to_vec(),
            )?;
            holder.load_client(path.as_ref())?.store().insert(
                "private_key".as_bytes().to_vec(),
                der.as_bytes().to_vec(),
                None,
            )?;
            holder.save()?;
        }
        let holder = Stronghold::new(user_device_private_key_path.clone(), vec![])?;
        let key = holder
            .load_client(path.as_ref())?
            .store()
            .get("private_key".as_bytes())?
            .ok_or_else(|| anyhow::anyhow!("No private key found in stronghold"))
            .and_then(|data| {
                SigningKey::from_pkcs8_der(&data)
                    .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))
            })?;
        Ok(key)
    }

    pub fn sign_server_challenge(&self, buffer: [u8; 32], handle: AppHandle) -> Result<[u8; 64]> {
        let key = self.ensure_signing_key(handle)?;
        let signature: Signature = key.sign(&buffer);
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&signature.to_bytes());
        Ok(sig_bytes)
    }

    /// Submit server key:
    /// 1. Get server public certificate for asymmetric encryption for AES key exchange
    /// 2. Generate AES key (encrypted by server public key) and IV
    /// 3. Generate device signing key (ECDSA P-256) if not exists
    /// 4. Return verifying key (public key of device signing key) and encrypted AES key
    pub fn assemble_verifying_key_packet(
        &self,
        server_cert: &[u8],
        handle: AppHandle,
    ) -> Result<String> {
        let server_public_key = VerifyingKey::from_public_key_der(server_cert)?;

        let aes_key_unencrypted = rand::random::<[u8; 16]>();
        let aes_iv = rand::random::<[u8; 16]>();

        let rsa_public_key = rsa::RsaPublicKey::from_pkcs1_der(server_cert)?;
        let encrypted_aes_key =
            rsa_public_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, &aes_key_unencrypted)?;

        let device_key = self.ensure_signing_key(handle)?;
        let public_key_der = device_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)?;

        let encrypted_public_key = aes_gcm::Aes128Gcm::new_from_slice(&aes_key_unencrypted)?
            .encrypt(
                aes_gcm::Nonce::from_slice(&aes_iv),
                public_key_der.as_bytes(),
            )?;

        let aes_key = base64::engine::general_purpose::STANDARD.encode(encrypted_aes_key);
        let iv = base64::engine::general_purpose::STANDARD.encode(aes_iv);
        let public_key = base64::engine::general_purpose::STANDARD.encode(encrypted_public_key);

        let packet = serde_json::json!({
            "aes_key": aes_key,
            "aes_iv": iv,
            "public_key": public_key,
        });

        Ok(packet.to_string())
    }

    pub async fn request_unlock(
        &self,
        nonce: [u8; 16],
        entity: String,
        beacon: String,
    ) -> Result<Challenge> {
        println!("Requesting Unlock... Entity: {entity}; beacon: {beacon}");
        let device_timestamp = chrono::Utc::now().timestamp() as u64;
        let beacon_information =
            fetch_beacon_information(beacon.as_str(), entity.as_str(), &self.user_token).await?;
        // beacon timestamp regards the epoch time as 0 in its clock, so we need to add the epoch time to it.
        let timestamp = device_timestamp
            .checked_sub(beacon_information.last_boot)
            .ok_or_else(|| anyhow::anyhow!("Timestamp overflow"))?;
        request_unlock_permission(nonce, entity, beacon, timestamp, &self.user_token).await
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
        hasher.update(challenge.nonce);
        hasher.update(challenge.timestamp.to_be_bytes());
        hasher.update(challenge.server_signature);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    pub fn generate_device_proof(&mut self, challenge: &Challenge) -> Result<DeviceProof> {
        self.counter += 1;

        let challenge_hash = Self::hash_challenge(challenge);

        Ok(DeviceProof {
            challenge_hash,
            device_signature: challenge.server_signature,
            timestamp: challenge.timestamp,
            counter: self.counter,
        })
    }
}
