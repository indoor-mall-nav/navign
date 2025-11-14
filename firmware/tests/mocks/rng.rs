/// Deterministic RNG for testing
pub struct MockRng {
    seed: u64,
}

impl MockRng {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn read(&mut self) -> u32 {
        // Simple LCG for deterministic randomness
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed / 65536) as u32
    }

    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        for i in 0..dest.len() {
            if i % 4 == 0 {
                let val = self.read();
                let bytes = val.to_le_bytes();
                let remaining = dest.len() - i;
                let copy_len = remaining.min(4);
                dest[i..i + copy_len].copy_from_slice(&bytes[..copy_len]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = MockRng::new(42);
        let mut rng2 = MockRng::new(42);

        assert_eq!(rng1.read(), rng2.read());
        assert_eq!(rng1.read(), rng2.read());
    }

    #[test]
    fn test_fill_bytes() {
        let mut rng = MockRng::new(12345);
        let mut buf1 = [0u8; 16];
        let mut buf2 = [0u8; 16];

        rng.fill_bytes(&mut buf1);

        let mut rng2 = MockRng::new(12345);
        rng2.fill_bytes(&mut buf2);

        assert_eq!(buf1, buf2);
    }
}
