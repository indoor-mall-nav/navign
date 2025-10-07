#![allow(unused)]
//! TODO proof management
use crate::shared::constants::*;
use crate::shared::CryptoError;
use esp_hal::sha::Digest;
use esp_println::println;
use heapless::Vec;
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, VerifyingKey};
use sha2::Sha256;

#[derive(Debug, Clone)]
pub struct Proof {
    pub nonce: [u8; 16],
    pub device_bytes: [u8; 8],
    pub verify_bytes: [u8; 8],
    pub timestamp: u64,
    pub server_signature: [u8; 64],
}

impl Proof {
    pub fn new(
        nonce: [u8; 16],
        device_bytes: [u8; 8],
        verify_bytes: [u8; 8],
        timestamp: u64,
        server_signature: [u8; 64],
    ) -> Self {
        Self {
            nonce,
            device_bytes,
            verify_bytes,
            timestamp,
            server_signature,
        }
    }

    pub fn packetize(&self) -> Vec<u8, { 16 + 8 + 8 + 8 + 64 }> {
        let mut packet = Vec::<u8, { 16 + 8 + 8 + 8 + 64 }>::new();
        packet.extend_from_slice(&self.nonce);
        packet.extend_from_slice(&self.device_bytes);
        packet.extend_from_slice(&self.verify_bytes);
        packet.extend_from_slice(&self.timestamp.to_be_bytes());
        packet.extend_from_slice(&self.server_signature);
        packet
    }

    pub fn depacketize(data: &[u8]) -> Option<Self> {
        if data.len() != UNLOCK_REQUEST_LENGTH - 1 {
            return None;
        }
        let device_bytes_offset = NONCE_LENGTH;
        let verify_bytes_offset = device_bytes_offset + DEVICE_BYTES_LENGTH;
        let timestamp_offset = verify_bytes_offset + VERIFY_BYTES_LENGTH;
        let server_signature_offset = timestamp_offset + TIMESTAMP_LENGTH;
        let mut nonce = [0u8; NONCE_LENGTH];
        nonce.copy_from_slice(&data[0..device_bytes_offset]);
        let mut device_bytes = [0u8; 8];
        device_bytes.copy_from_slice(&data[device_bytes_offset..verify_bytes_offset]);
        let mut verify_bytes = [0u8; 8];
        verify_bytes.copy_from_slice(&data[verify_bytes_offset..timestamp_offset]);
        let timestamp = u64::from_be_bytes(
            data[timestamp_offset..server_signature_offset]
                .try_into()
                .ok()?,
        );
        let mut server_signature = [0u8; 64];
        server_signature.copy_from_slice(
            &data[server_signature_offset..server_signature_offset + SERVER_SIGNATURE_LENGTH],
        );
        Some(Self {
            nonce,
            device_bytes,
            verify_bytes,
            timestamp,
            server_signature,
        })
    }
}

#[derive(Debug)]
pub struct ProofManager {
    device_private_key: [u8; 32],
    server_public_key: Option<VerifyingKey>,
    counter: u64,
}

impl ProofManager {
    pub fn new(private_key: [u8; 32]) -> Self {
        Self {
            device_private_key: private_key,
            server_public_key: None,
            counter: 0,
        }
    }

    pub fn set_server_public_key(&mut self, public_key: [u8; 65]) -> Result<(), CryptoError> {
        let key =
            VerifyingKey::from_sec1_bytes(&public_key).map_err(|_| CryptoError::InvalidKey)?;
        self.server_public_key = Some(key);
        Ok(())
    }

    pub fn sign_data(&self, data: &[u8]) -> Result<[u8; 64], CryptoError> {
        let signing_key = p256::ecdsa::SigningKey::from_slice(&self.device_private_key)
            .map_err(|_| CryptoError::InvalidKey)?;
        let signature: Signature = signing_key.sign(data);
        let signature_bytes = signature.to_bytes();
        let mut signature_array = [0u8; 64];
        signature_array.copy_from_slice(&signature_bytes);
        Ok(signature_array)
    }

    pub fn validate_proof(&mut self, proof: &Proof) -> Result<(), CryptoError> {
        let mut challenge_data = Vec::<u8, { 16 + 8 + 8 + 8 }>::new();

        println!("Validating proof: {:?}", proof);

        challenge_data
            .extend_from_slice(&proof.nonce)
            .map_err(|_| CryptoError::BufferFull)?;
        challenge_data
            .extend_from_slice(&proof.timestamp.to_be_bytes())
            .map_err(|_| CryptoError::BufferFull)?;
        challenge_data
            .extend_from_slice(self.counter.to_be_bytes().as_slice())
            .map_err(|_| CryptoError::BufferFull)?;
        challenge_data
            .extend_from_slice(&proof.device_bytes)
            .map_err(|_| CryptoError::BufferFull)?;

        let mut hasher = Sha256::new();
        hasher.update(challenge_data);
        let hashed = hasher.finalize();

        let server_signature = Signature::from_bytes(&proof.server_signature.into())
            .map_err(|_| CryptoError::InvalidSignature)?;

        println!("Validating proof with signature: {:?}", server_signature);
        println!("Hashed data: {:?}", hashed);

        self.server_public_key
            .as_ref()
            .ok_or(CryptoError::ServerPublicKeyNotSet)?
            .verify(&hashed, &server_signature)
            .map_err(|_| CryptoError::VerificationFailed)?;

        let device_signature = self.sign_data(&hashed)?;
        if device_signature[56..64] != proof.verify_bytes {
            return Err(CryptoError::InvalidSignature);
        }

        self.counter += 1;

        Ok(())
    }
}
