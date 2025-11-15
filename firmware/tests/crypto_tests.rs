#![cfg(test)]

use p256::ecdsa::{
    Signature, SigningKey, VerifyingKey,
    signature::{Signer, Verifier},
};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use sha2::{Digest, Sha256};

#[test]
fn test_signature_verification() {
    // Generate test keys
    let private_key = SigningKey::from_slice(&[42u8; 32]).unwrap();
    let public_key = private_key.verifying_key();

    // Sign a nonce
    let nonce = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    let hash = hasher.finalize();

    let signature: Signature = private_key.sign(&hash);

    // Verify signature
    assert!(public_key.verify(&hash, &signature).is_ok());
}

#[test]
fn test_invalid_signature_rejection() {
    let private_key1 = SigningKey::from_slice(&[42u8; 32]).unwrap();
    let private_key2 = SigningKey::from_slice(&[43u8; 32]).unwrap();
    let public_key1 = private_key1.verifying_key();

    let nonce = [1u8; 16];
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    let hash = hasher.finalize();

    // Sign with key2
    let signature: Signature = private_key2.sign(&hash);

    // Verify with key1 should fail
    assert!(public_key1.verify(&hash, &signature).is_err());
}

#[test]
fn test_unlock_proof_format() {
    // Test the proof includes: nonce || device_id
    let nonce = [42u8; 16];
    let device_id = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    ];

    let mut proof_data = [0u8; 40];
    proof_data[..16].copy_from_slice(&nonce);
    proof_data[16..].copy_from_slice(&device_id);

    // Hash and sign
    let private_key = SigningKey::from_slice(&[99u8; 32]).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&proof_data);
    let hash = hasher.finalize();
    let signature: Signature = private_key.sign(&hash);

    // Signature should be 64 bytes (r + s)
    assert_eq!(signature.to_bytes().len(), 64);
}

#[test]
fn test_nonce_signature_identifier() {
    // Test that signature identifier is last 8 bytes of signature
    let private_key = SigningKey::from_slice(&[123u8; 32]).unwrap();
    let nonce = [55u8; 16];

    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    let hash = hasher.finalize();
    let signature: Signature = private_key.sign(&hash);

    let sig_bytes = signature.to_bytes();
    let identifier = &sig_bytes[sig_bytes.len() - 8..];

    // Identifier should be 8 bytes
    assert_eq!(identifier.len(), 8);

    // Identifier should be deterministic for same nonce + key
    let mut hasher2 = Sha256::new();
    hasher2.update(&nonce);
    let hash2 = hasher2.finalize();
    let signature2: Signature = private_key.sign(&hash2);
    let sig_bytes2 = signature2.to_bytes();
    let identifier2 = &sig_bytes2[sig_bytes2.len() - 8..];

    assert_eq!(identifier, identifier2);
}

#[test]
fn test_different_nonces_different_signatures() {
    let private_key = SigningKey::from_bytes(&[200u8; 32].into()).unwrap();

    let nonce1 = [1u8; 16];
    let nonce2 = [2u8; 16];

    let mut hasher1 = Sha256::new();
    hasher1.update(&nonce1);
    let hash1 = hasher1.finalize();
    let sig1: Signature = private_key.sign(&hash1);

    let mut hasher2 = Sha256::new();
    hasher2.update(&nonce2);
    let hash2 = hasher2.finalize();
    let sig2: Signature = private_key.sign(&hash2);

    // Different nonces should produce different signatures
    assert_ne!(sig1.to_bytes(), sig2.to_bytes());
}

#[test]
fn test_public_key_encoding() {
    let private_key = SigningKey::from_slice(&[77u8; 32]).unwrap();
    let public_key = private_key.verifying_key();

    // Encode as uncompressed point (0x04 prefix + 64 bytes)
    let encoded = public_key.to_encoded_point(false);
    let bytes = encoded.as_bytes();

    // Should be 65 bytes: 0x04 || x || y
    assert_eq!(bytes.len(), 65);
    assert_eq!(bytes[0], 0x04);
}

#[test]
fn test_tampered_message_detection() {
    let private_key = SigningKey::from_slice(&[88u8; 32]).unwrap();
    let public_key = private_key.verifying_key();

    let original_message = b"unlock request";
    let mut hasher = Sha256::new();
    hasher.update(original_message);
    let hash = hasher.finalize();
    let signature: Signature = private_key.sign(&hash);

    // Tamper with message
    let tampered_message = b"unlock request!"; // Added one character
    let mut hasher2 = Sha256::new();
    hasher2.update(tampered_message);
    let hash2 = hasher2.finalize();

    // Verification with tampered message should fail
    assert!(public_key.verify(&hash2, &signature).is_err());
}
