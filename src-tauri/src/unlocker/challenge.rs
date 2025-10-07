use base64::Engine;
use digest::Digest;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChallenge {
    pub nonce: [u8; 16],
    pub instance_id: [u8; 24],
    pub timestamp: u64,
    pub user_id: [u8; 24]
}

impl ServerChallenge {
    pub fn new(nonce: [u8; 16], instance_id: &str, timestamp: u64, user_id: String) -> Self {
        if user_id.len() != 24 {
            panic!("User ID must be 24 bytes long");
        }
        let mut user_id_bytes = [0u8; 24];
        user_id_bytes.copy_from_slice(user_id.as_bytes());
        let mut instance_id_bytes = [0u8; 24];
        if instance_id.len() != 24 {
            panic!("Instance ID must be 24 bytes long");
        }
        instance_id_bytes.copy_from_slice(instance_id.as_bytes());
        Self {
            nonce,
            timestamp,
            instance_id: instance_id_bytes,
            user_id: user_id_bytes,
        }
    }

    pub fn get_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce);
        hasher.update(self.instance_id);
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.user_id);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    pub fn sign(&self, device_key: &SigningKey) -> [u8; 64] {
        let hash = self.get_hash();
        let signature: Signature = device_key.sign(&hash);
        let signature_bytes = signature.to_bytes();
        let mut signature_array = [0u8; 64];
        signature_array.copy_from_slice(&signature_bytes);
        signature_array
    }

    pub fn depacketize(packet: String, user: String) -> Option<Self> {
        // Server only sends the nonce and the instance ID
        let data = base64::engine::general_purpose::STANDARD.decode(packet).ok()?;
        if data.len() != 16 + 24 { return None };
        let mut nonce = [0u8; 16];
        nonce.copy_from_slice(&data[0..16]);
        let mut instance_id = [0u8; 24];
        instance_id.copy_from_slice(&data[16..40]);
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let instance_id = std::str::from_utf8(&instance_id).ok()?;
        Some(Self::new(nonce, instance_id, timestamp, user))
    }

    pub fn packetize(&self, signing_key: &SigningKey) -> (String, [u8; 8]) {
        let mut packet = Vec::with_capacity(8 + 64);
        packet.extend_from_slice(&self.timestamp.to_be_bytes());
        let signature = self.sign(signing_key);
        packet.extend(signature);
        let validator = signature[56..64].try_into().unwrap_or([0u8; 8]);
        (base64::engine::general_purpose::STANDARD.encode(packet), validator)
    }
}
