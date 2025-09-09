use esp_hal::rng::Rng;
use heapless::String;
use core::fmt::Write;

#[derive(Debug)]
pub struct Nonce(pub [u8; 16]);

impl Nonce {
    pub fn generate(rng: &mut Rng) -> Self {
        let mut nonce = [0u8; 16];
        for i in 0..16 {
            nonce[i] = rng.random() as u8;
        }
        Nonce(nonce)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn to_hex(&self) -> String<32> {
        let mut s: String<32> = String::new();
        for byte in &self.0 {
            write!(s, "{:02x}", byte).unwrap();
        }
        s
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            let byte_str = &s[i*2..i*2+2];
            if let Ok(byte) = u8::from_str_radix(byte_str, 16) {
                bytes[i] = byte;
            } else {
                return None;
            }
        }
        Some(Nonce(bytes))
    }
}

impl From<[u8; 16]> for Nonce {
    fn from(bytes: [u8; 16]) -> Self {
        Nonce(bytes)
    }
}

impl Into<[u8; 16]> for Nonce {
    fn into(self) -> [u8; 16] {
        self.0
    }
}