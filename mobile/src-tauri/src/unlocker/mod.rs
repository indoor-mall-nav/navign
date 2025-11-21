// Copyright (c) 2025 Ethan Wu
// SPDX-License-Identifier: MIT

//! Unlocker module for secure device authentication and communication with a server.
//! This module provides functionalities to handle cryptographic operations,
//! including key management, signing challenges, and generating proofs of device authenticity.
//! It uses ECDSA for signing and RSA for encrypting AES keys, ensuring secure communication.
//! The module is designed to work in a Tauri application environment, leveraging stronghold for secure key storage.
mod challenge;
mod pipeline;

// Re-export constants from navign-shared
pub mod constants {
    pub use navign_shared::constants::*;
}

pub use pipeline::unlock_handler;

use anyhow::Result;
use nanoid::nanoid;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use tauri::{AppHandle, Manager};
#[cfg(mobile)]
use tauri_plugin_biometric::AuthOptions;
#[cfg(mobile)]
use tauri_plugin_biometric::BiometricExt;
use tauri_plugin_stronghold::stronghold::Stronghold;

pub struct Unlocker {
    user_id: String,
    /// The JWT token for the user session
    user_token: String,
    device_id: String,
    signed_in: bool,
}

impl Unlocker {
    pub fn new(user_id: String, user_token: String) -> Self {
        Self {
            user_id,
            user_token,
            device_id: nanoid!(),
            signed_in: false,
        }
    }

    pub fn get_user_token(&self) -> Option<&str> {
        if self.signed_in {
            Some(&self.user_token)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn set_user_token(&mut self, token: String) {
        self.user_token = token;
    }

    #[allow(dead_code)]
    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }

    pub fn ensure_signing_key(&self, handle: &AppHandle) -> Result<SigningKey> {
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
        handle
            .biometric()
            .authenticate("Please authenticate to load data".to_string(), auth_options)
            .map_err(|_| anyhow::anyhow!("Biometric authentication failed"))?;

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

    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}
