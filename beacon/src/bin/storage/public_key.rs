#![allow(unused)]
use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;
use p256::ecdsa::VerifyingKey;

pub enum PublicKeyError {
    StorageError,
    DecodeError,
}

pub fn read_public_key(storage: &mut FlashStorage) -> Result<VerifyingKey, PublicKeyError> {
    let mut buffer = [0u8; 65]; // P-256 public key in uncompressed form is 91 bytes
    storage
        .read(32, &mut buffer)
        .map_err(|_| PublicKeyError::StorageError)?;
    VerifyingKey::from_sec1_bytes(&buffer).map_err(|_| PublicKeyError::DecodeError)
}

pub fn write_public_key(
    storage: &mut FlashStorage,
    public_key: [u8; 65],
) -> Result<(), PublicKeyError> {
    storage.write(32, &public_key).map_err(|_| PublicKeyError::StorageError)
}
