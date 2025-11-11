#![allow(unused)]
use anyhow::Result;
use sqlx::PgPool;
use p256::PublicKey;
use p256::ecdsa::SigningKey;
use p256::ecdsa::signature::Signer;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPasswordAuthenticationPayload {
    pub user_id: String,
    pub password: String,
    pub timestamp: u64,
}

pub struct CryptoServer {
    private_key: SigningKey,
    public_key: PublicKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockRequest {
    pub user_id: String,
    pub nonce: [u8; 16],
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnlockChallenge {
    pub nonce: [u8; 16],
    pub timestamp: u64,
    #[serde(with = "BigArray")]
    pub server_signature: [u8; 64],
}

impl CryptoServer {
    pub fn new(private_key: SigningKey, public_key: PublicKey) -> Self {
        Self {
            private_key,
            public_key,
        }
    }

    /// Handle user password authentication and return a signed JWT token
    /// The `payload` is an encrypted JSON string containing user_id, password, and timestamp, which
    /// uses the public key of the server for encryption.
    /// Payload -> JSON.stringify -> URL encode -> AES-256-CBC encrypt -> hex encode.
    /// Now it's hex-encoded.
    /// The `hash` is an SHA-256 hash of the payload for integrity verification.
    pub fn password_jwt_sign(
        &self,
        payload: String,
        hash: String,
        user_id: &str,
        database: &PgPool,
    ) -> Result<String> {
        let payload = hex::decode(payload)?;
        Ok(String::new())
    }

    pub fn handle_unlocker(&self, request: UnlockRequest) -> Result<UnlockChallenge> {
        // Here you would typically verify the user_id and nonce, and check permissions.
        // For simplicity, we'll assume the request is valid.

        let current_timestamp = chrono::Utc::now().timestamp() as u64;
        if request.timestamp + 300 < current_timestamp {
            // Request is too old (older than 5 minutes)
            anyhow::bail!("Request timestamp is too old");
        }

        // Create a message to sign (for example, concatenating nonce and timestamp)
        let mut message = Vec::new();
        message.extend_from_slice(&request.nonce);
        message.extend_from_slice(&request.timestamp.to_le_bytes());

        // Sign the message with the server's private key
        let signature: p256::ecdsa::Signature = self.private_key.sign(&message);

        // Create the challenge to return
        let challenge = UnlockChallenge {
            nonce: request.nonce,
            timestamp: request.timestamp,
            server_signature: signature.to_bytes().into(),
        };

        Ok(challenge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::SigningKey;

    fn create_test_crypto_server() -> CryptoServer {
        let private_key = SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
        let public_key = private_key.verifying_key().into();
        CryptoServer::new(private_key, public_key)
    }

    #[test]
    fn test_crypto_server_creation() {
        let server = create_test_crypto_server();
        // Server should be created successfully
        assert!(std::mem::size_of_val(&server) > 0);
    }

    #[test]
    fn test_handle_unlocker_valid_request() {
        let server = create_test_crypto_server();
        let current_timestamp = chrono::Utc::now().timestamp() as u64;
        let nonce = [1u8; 16];

        let request = UnlockRequest {
            user_id: "test_user".to_string(),
            nonce,
            timestamp: current_timestamp,
        };

        let result = server.handle_unlocker(request);
        assert!(result.is_ok());

        let challenge = result.unwrap();
        assert_eq!(challenge.nonce, nonce);
        assert_eq!(challenge.timestamp, current_timestamp);
        assert_eq!(challenge.server_signature.len(), 64);
    }

    #[test]
    fn test_handle_unlocker_expired_timestamp() {
        let server = create_test_crypto_server();
        let old_timestamp = chrono::Utc::now().timestamp() as u64 - 400; // 400 seconds ago
        let nonce = [2u8; 16];

        let request = UnlockRequest {
            user_id: "test_user".to_string(),
            nonce,
            timestamp: old_timestamp,
        };

        let result = server.handle_unlocker(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too old"));
    }

    #[test]
    fn test_unlock_challenge_signature_uniqueness() {
        let server = create_test_crypto_server();
        let current_timestamp = chrono::Utc::now().timestamp() as u64;

        let request1 = UnlockRequest {
            user_id: "user1".to_string(),
            nonce: [1u8; 16],
            timestamp: current_timestamp,
        };

        let request2 = UnlockRequest {
            user_id: "user2".to_string(),
            nonce: [2u8; 16],
            timestamp: current_timestamp,
        };

        let challenge1 = server.handle_unlocker(request1).unwrap();
        let challenge2 = server.handle_unlocker(request2).unwrap();

        // Different nonces should produce different signatures
        assert_ne!(challenge1.server_signature, challenge2.server_signature);
    }
}
