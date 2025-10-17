use base64::Engine;
use digest::Digest;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChallenge {
    pub nonce: [u8; 16],
    pub instance_id: [u8; 24],
    pub timestamp: u64,
    pub user_id: [u8; 24],
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

    pub fn packetize(&self, signing_key: &SigningKey) -> (String, [u8; 8]) {
        let mut packet = Vec::with_capacity(8 + 64);
        packet.extend_from_slice(&self.timestamp.to_be_bytes());
        let signature = self.sign(signing_key);
        packet.extend(signature);
        let validator = signature[56..64].try_into().unwrap_or([0u8; 8]);
        (
            base64::engine::general_purpose::STANDARD.encode(packet),
            validator,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::rand_core::OsRng;
    use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey};

    #[test]
    fn test_server_challenge() {
        let signing_key = SigningKey::random(&mut OsRng);
        let pem = signing_key
            .to_pkcs8_pem(Default::default())
            .unwrap()
            .to_string();
        let device_key = SigningKey::from_pkcs8_pem(pem.as_str()).unwrap();

        let challenge = ServerChallenge::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            "abcdef123456abcdef123456",
            1625077765,
            "abcdef123456abcdef123456".to_string(),
        );

        let (packet_b64, validator) = challenge.packetize(&device_key);
        println!("Packet (base64): {}", packet_b64);
        println!("Validator: {:?}", validator);

        assert_eq!(packet_b64.len(), 96);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(packet_b64)
            .unwrap();
        assert_eq!(decoded.len(), 72);
        let validator_from_packet: [u8; 8] = decoded[64..72].try_into().unwrap();
        assert_eq!(validator, validator_from_packet);
        assert_eq!(validator.len(), 8);
    }
}
