#[cfg(feature = "heapless")]
pub trait Packetize<const N: usize> {
    fn packetize(&self) -> heapless::Vec<u8, N>;

    #[cfg(feature = "crypto")]
    fn get_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let packet = self.packetize();
        let mut hasher = Sha256::new();
        hasher.update(&packet);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    #[cfg(feature = "crypto")]
    fn sign(&self, signing_key: &p256::ecdsa::SigningKey) -> [u8; 64] {
        use p256::ecdsa::signature::Signer;
        let hash = self.get_hash();
        let signature: p256::ecdsa::Signature = signing_key.sign(&hash);
        let signature_bytes = signature.to_bytes();
        let mut signature_array = [0u8; 64];
        signature_array.copy_from_slice(&signature_bytes);
        signature_array
    }

    #[cfg(feature = "crypto")]
    fn verify(&self, verify_key: &p256::ecdsa::VerifyingKey, signature: &[u8; 64]) -> bool {
        use p256::ecdsa::signature::Verifier;
        let hash = self.get_hash();
        let signature = match p256::ecdsa::Signature::from_bytes(signature.into()) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        verify_key.verify(&hash, &signature).is_ok()
    }
}

#[cfg(feature = "alloc")]
pub trait Packetize {
    fn packetize(&self) -> alloc::vec::Vec<u8>;

    #[cfg(feature = "base64")]
    fn packetize_to_base64(&self) -> alloc::string::String {
        use base64::Engine;
        let packet = self.packetize();
        base64::engine::general_purpose::STANDARD.encode(&packet)
    }

    #[cfg(feature = "crypto")]
    fn get_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let packet = self.packetize();
        let mut hasher = Sha256::new();
        hasher.update(&packet);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    #[cfg(feature = "crypto")]
    fn sign(&self, signing_key: &p256::ecdsa::SigningKey) -> [u8; 64] {
        use p256::ecdsa::signature::Signer;
        let hash = self.get_hash();
        let signature: p256::ecdsa::Signature = signing_key.sign(&hash);
        let signature_bytes = signature.to_bytes();
        let mut signature_array = [0u8; 64];
        signature_array.copy_from_slice(&signature_bytes);
        signature_array
    }

    #[cfg(feature = "crypto")]
    fn verify(&self, verify_key: &p256::ecdsa::VerifyingKey, signature: &[u8; 64]) -> bool {
        use p256::ecdsa::signature::Verifier;
        let hash = self.get_hash();
        let signature = match p256::ecdsa::Signature::from_bytes(signature.into()) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        verify_key.verify(&hash, &signature).is_ok()
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> Packetize<N> for [u8] {
    fn packetize(&self) -> heapless::Vec<u8, N> {
        let mut vec = heapless::Vec::<u8, N>::new();
        vec.extend_from_slice(self).unwrap();
        vec
    }
}

#[cfg(feature = "alloc")]
impl Packetize for [u8] {
    fn packetize(&self) -> alloc::vec::Vec<u8> {
        self.to_vec()
    }
}
