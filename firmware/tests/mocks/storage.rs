use heapless::Vec;

/// Mock flash storage backed by RAM
pub struct MockStorage {
    efuse_data: [u8; 32],
    nonces: Vec<([u8; 16], u64), 16>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            efuse_data: [0u8; 32],
            nonces: Vec::new(),
        }
    }

    pub fn set_private_key(&mut self, key: [u8; 32]) {
        self.efuse_data = key;
    }

    pub fn read_private_key(&self) -> [u8; 32] {
        self.efuse_data
    }

    pub fn store_nonce(&mut self, nonce: [u8; 16], timestamp: u64) -> Result<(), ()> {
        if self.nonces.len() >= 16 {
            self.nonces.remove(0);
        }
        self.nonces.push((nonce, timestamp)).map_err(|_| ())
    }

    pub fn check_nonce_used(&self, nonce: &[u8; 16]) -> bool {
        self.nonces.iter().any(|(n, _)| n == nonce)
    }

    pub fn cleanup_expired_nonces(&mut self, current_time: u64, ttl: u64) {
        self.nonces
            .retain(|(_, timestamp)| current_time - timestamp < ttl);
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}
