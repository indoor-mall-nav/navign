use p256::ecdsa::signature::Signer;
use p256::ecdsa::SigningKey;
use p256::PublicKey;
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;
use anyhow::Result;
use mongodb::Database;

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
    pub fn password_jwt_sign(&self, payload: String, hash: String, user_id: &str, database: &Database) -> Result<String> {
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