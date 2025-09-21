use super::challenge::Challenge;
use crate::shared::CryptoError;
use esp_hal::sha::Digest;
use esp_println::println;
use heapless::Vec;
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, VerifyingKey};
use p256::FieldBytes;
use sha2::Sha256;

#[derive(Debug, Clone)]
pub struct Proof {
    pub challenge_hash: [u8; 32],
    pub device_signature: [u8; 64],
    pub timestamp: u64,
    pub counter: u64,
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

    pub fn generate_proof(
        &mut self,
        challenge_hash: [u8; 32],
        current_timestamp: u64,
    ) -> Result<Proof, CryptoError> {
        self.counter += 1;

        let mut data_to_sign = Vec::<u8, 96>::new();
        data_to_sign
            .extend_from_slice(&challenge_hash)
            .map_err(|_| CryptoError::BufferFull)?;
        data_to_sign
            .extend_from_slice(&current_timestamp.to_be_bytes())
            .map_err(|_| CryptoError::BufferFull)?;
        data_to_sign
            .extend_from_slice(&self.counter.to_be_bytes())
            .map_err(|_| CryptoError::BufferFull)?;

        let key = p256::ecdsa::SigningKey::from_bytes(&FieldBytes::from(self.device_private_key))
            .map_err(|_| CryptoError::InvalidKey)?;
        let signature: p256::ecdsa::Signature = key.sign(&data_to_sign);
        let device_signature: [u8; 64] = signature.to_bytes().into();
        Ok(Proof {
            challenge_hash,
            device_signature,
            timestamp: current_timestamp,
            counter: self.counter,
        })
    }

    pub fn validate_proof(&mut self, proof: &Proof) -> Result<(), CryptoError> {
        let mut challenge_data = Vec::<u8, 128>::new();

        println!("Validating proof: {:?}", proof);

        challenge_data
            .extend_from_slice(&proof.challenge_hash)
            .map_err(|_| CryptoError::BufferFull)?;
        challenge_data
            .extend_from_slice(&proof.timestamp.to_be_bytes())
            .map_err(|_| CryptoError::BufferFull)?;

        let mut hasher = Sha256::new();
        hasher.update(challenge_data);
        let hashed = hasher.finalize();

        let signature = Signature::from_bytes(&proof.device_signature.into())
            .map_err(|_| CryptoError::InvalidSignature)?;

        println!("Validating proof with signature: {:?}", signature);
        println!("Hashed data: {:?}", hashed);

        return Ok(());

        self.server_public_key
            .as_ref()
            .ok_or(CryptoError::ServerPublicKeyNotSet)?
            .verify(&hashed, &signature).map_err(|_| CryptoError::VerificationFailed)
    }

    pub fn verify_server_challenge(&self, challenge: &Challenge) -> Result<(), CryptoError> {
        let mut challenge_data = Vec::<u8, 64>::new();

        challenge_data
            .extend_from_slice(challenge.nonce.as_bytes())
            .map_err(|_| CryptoError::BufferFull)?;
        challenge_data
            .extend_from_slice(&challenge.timestamp.to_be_bytes())
            .map_err(|_| CryptoError::BufferFull)?;
        challenge_data
            .extend_from_slice(&challenge.server_signature)
            .map_err(|_| CryptoError::BufferFull)?;

        let mut hasher = Sha256::new();
        hasher.update(challenge_data);
        let hashed = hasher.finalize();

        let signature = p256::ecdsa::Signature::from_bytes(&challenge.server_signature.into())
            .map_err(|_| CryptoError::InvalidSignature)?;
        self.server_public_key
            .as_ref()
            .ok_or(CryptoError::InvalidSignature)?
            .verify(&hashed, &signature)
            .map_err(|_| CryptoError::VerificationFailed)
    }
}
