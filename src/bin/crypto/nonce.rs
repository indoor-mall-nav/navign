use crate::shared::constants::NONCE_LENGTH;
use core::fmt::Write;
use esp_hal::rng::Rng;
use heapless::String;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Nonce([u8; NONCE_LENGTH]);

impl Nonce {
    pub fn generate(rng: &mut Rng) -> Self {
        let mut nonce = [0u8; NONCE_LENGTH];
        for item in nonce.iter_mut() {
            *item = rng.random() as u8;
        }
        Nonce(nonce)
    }

    pub fn as_bytes(&self) -> &[u8; NONCE_LENGTH] {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8; NONCE_LENGTH]) -> Self {
        Nonce(*bytes)
    }

    pub fn to_hex(&self) -> String<{ NONCE_LENGTH * 2 }> {
        let mut s = String::<{ NONCE_LENGTH * 2 }>::new();
        for byte in &self.0 {
            write!(s, "{:02x}", byte).unwrap();
        }
        s
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != NONCE_LENGTH * 2 {
            return None;
        }
        let mut bytes = [0u8; NONCE_LENGTH];
        for i in 0..NONCE_LENGTH {
            let byte_str = &s[i * 2..i * 2 + 2];
            if let Ok(byte) = u8::from_str_radix(byte_str, 16) {
                bytes[i] = byte;
            } else {
                return None;
            }
        }
        Some(Nonce(bytes))
    }
}

impl From<[u8; NONCE_LENGTH]> for Nonce {
    fn from(bytes: [u8; NONCE_LENGTH]) -> Self {
        Nonce(bytes)
    }
}

impl From<Nonce> for [u8; NONCE_LENGTH] {
    fn from(nonce: Nonce) -> Self {
        nonce.0
    }
}
