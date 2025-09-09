use esp_hal::rng::Rng;
use esp_hal::sha::Digest;
use sha2::Sha256;
use crate::crypto::nonce::Nonce;

pub struct ChallengeManager {
    rng: Rng
}

impl ChallengeManager {
    pub fn new(rng: Rng) -> Self {
        Self { rng }
    }

    pub fn get_rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    pub(crate) fn create_challenge_hash(&self, nonce: Nonce, timestamp: u64, server_signature: [u8; 86]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(nonce.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&server_signature);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
}