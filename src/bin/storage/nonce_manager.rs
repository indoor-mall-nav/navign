use heapless::index_map::FnvIndexMap;
use crate::crypto::Nonce;

#[derive(Debug)]
pub struct NonceManager<const N: usize> {
    /// A set of used nonces to prevent replay attacks.
    /// The first element is the nonce, the second element is the timestamp when it was used.
    /// The set can hold up to `N` nonces. We assume that within the time window (3 minutes), there won't be more than 32 unique nonces.
    used_nonces: FnvIndexMap<Nonce, u64, N>,
    /// A set of used challenge hashes to prevent replay attacks.
    /// The first element is the challenge hash, the second element is the timestamp when it was used.
    /// The set can hold up to 32 challenge hashes. We assume that within the time
    /// window (3 minutes), there won't be more than `N` unique challenge hashes.
    used_challenge_hashes: FnvIndexMap<[u8; 32], u64, N>,
}

impl<const N: usize> NonceManager<N> {
    pub fn new() -> Self {
        Self {
            used_nonces: FnvIndexMap::new(),
            used_challenge_hashes: FnvIndexMap::new(),
        }
    }

    /// Check if the nonce is valid (not used before and within the time window).
    /// If valid, mark it as used.
    pub fn check_and_mark_nonce(&mut self, nonce: Nonce, timestamp: u64) -> bool {
        // Clean up old nonces
        self.clear_expired(timestamp);

        if self.used_nonces.contains_key(&nonce) {
            // Nonce has been used before
            false
        } else {
            // Mark nonce as used
            self.used_nonces.insert(nonce, timestamp).ok();
            true
        }
    }

    /// Check if the challenge hash is valid (not used before and within the time window).
    /// If valid, mark it as used.
    pub fn check_and_mark_challenge_hash(
        &mut self,
        challenge_hash: [u8; 32],
        timestamp: u64,
    ) -> bool {
        // Clean up old challenge hashes
        self.clear_expired(timestamp);

        if self.used_challenge_hashes.contains_key(&challenge_hash) {
            // Challenge hash has been used before
            false
        } else {
            // Mark challenge hash as used
            match self.used_challenge_hashes.insert(challenge_hash, timestamp) {
                Ok(_) => {}
                Err(_) => {
                    // If the map is full, remove the oldest entry
                    self.remove_oldest_challenge_hash();
                    self.used_challenge_hashes.insert(challenge_hash, timestamp).ok();
                }
            }
            true
        }
    }

    /// Clear expired nonces
    pub fn clear_expired(&mut self, current_timestamp: u64) {
        self.used_nonces
            .retain(|_, &mut ts| current_timestamp.saturating_sub(ts) <= 180_000);
        self.used_challenge_hashes
            .retain(|_, &mut ts| current_timestamp.saturating_sub(ts) <= 180_000);
    }

    pub fn remove_oldest_nonce(&mut self) {
        if let Some(oldest_key) = self.used_nonces.keys().next().cloned() {
            self.used_nonces.remove(&oldest_key);
        }
    }

    pub fn remove_oldest_challenge_hash(&mut self) {
        if let Some(oldest_key) = self.used_challenge_hashes.keys().next().cloned() {
            self.used_challenge_hashes.remove(&oldest_key);
        }
    }
    
    pub fn generate_nonce(&mut self, rng: &mut esp_hal::rng::Rng) -> Nonce {
        Nonce::generate(rng)
    }
}
