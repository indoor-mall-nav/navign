use super::nonce::Nonce;
use esp_hal::rng::Rng;
use esp_hal::sha::Digest;
use sha2::Sha256;

#[derive(Debug, Clone)]
pub struct Challenge {
    pub nonce: Nonce,
    pub timestamp: u64,
    pub server_signature: [u8; 64],
}

impl Challenge {
    pub fn new(nonce: Nonce, timestamp: u64, server_signature: [u8; 64]) -> Self {
        Self {
            nonce,
            timestamp,
            server_signature,
        }
    }

    pub(crate) fn get_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce.as_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.server_signature);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

pub struct ChallengeManager {
    rng: Rng,
}

impl ChallengeManager {
    pub fn new(rng: Rng) -> Self {
        Self { rng }
    }

    pub fn get_rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}
