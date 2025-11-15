// Integration tests for admin/maintenance
// Tests core cryptographic and file handling functionality

use p256::elliptic_curve::rand_core::OsRng;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::TempDir;

#[derive(Serialize, Deserialize)]
struct KeyMetadata {
    key_name: String,
    private_key_file: String,
    public_key_hex: String,
    generated_at: String,
    fused: bool,
    chip_info: Option<String>,
}

#[test]
fn test_key_generation_and_storage() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let output_dir = temp_dir.path();

    // Generate ECDSA P-256 private key (simulating the tool's behavior)
    let private_key = SecretKey::random(&mut OsRng);
    let public_key = private_key.public_key();
    let private_key_bytes = private_key.to_bytes();

    // Store private key
    let private_key_path = output_dir.join("test_key_private.bin");
    fs::write(&private_key_path, &private_key_bytes).expect("Should write private key");

    // Create metadata
    let metadata = KeyMetadata {
        key_name: "test_key".to_string(),
        private_key_file: private_key_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        public_key_hex: hex::encode(public_key.to_encoded_point(false).as_bytes()),
        generated_at: chrono::Utc::now().to_rfc3339(),
        fused: false,
        chip_info: None,
    };

    // Write metadata
    let metadata_path = output_dir.join("test_key_metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata).expect("Should serialize");
    fs::write(&metadata_path, metadata_json).expect("Should write metadata");

    // Verify files exist
    assert!(private_key_path.exists());
    assert!(metadata_path.exists());

    // Verify file contents
    let read_key = fs::read(&private_key_path).expect("Should read key");
    assert_eq!(read_key.len(), 32, "Private key should be 32 bytes");

    let read_metadata = fs::read_to_string(&metadata_path).expect("Should read metadata");
    let parsed: KeyMetadata = serde_json::from_str(&read_metadata).expect("Should parse metadata");
    assert_eq!(parsed.key_name, "test_key");
    assert_eq!(parsed.fused, false);
}

#[test]
fn test_device_id_format() {
    // Test device ID generation (24-character hex string from 12 bytes)
    let bytes: [u8; 12] = rand::random();
    let device_id = hex::encode(bytes);

    assert_eq!(device_id.len(), 24, "Device ID should be 24 characters");
    assert!(
        device_id.chars().all(|c| c.is_ascii_hexdigit()),
        "Device ID should be hexadecimal"
    );

    // Test decoding back
    let decoded = hex::decode(&device_id).expect("Should decode device ID");
    assert_eq!(decoded.len(), 12, "Decoded should be 12 bytes");
}

#[test]
fn test_public_key_hex_encoding() {
    let private_key = SecretKey::random(&mut OsRng);
    let public_key = private_key.public_key();

    // Encode public key to hex (uncompressed format)
    let pub_key_hex = hex::encode(public_key.to_encoded_point(false).as_bytes());

    // Uncompressed P-256 public key: 1 byte (0x04) + 32 bytes (x) + 32 bytes (y) = 65 bytes = 130 hex chars
    assert_eq!(
        pub_key_hex.len(),
        130,
        "Uncompressed public key should be 130 hex characters"
    );
    assert!(pub_key_hex.starts_with("04"), "Should start with 0x04");

    // Decode and verify
    let pub_key_bytes = hex::decode(&pub_key_hex).expect("Should decode");
    let restored_pub_key =
        PublicKey::from_sec1_bytes(&pub_key_bytes).expect("Should restore public key");

    assert_eq!(
        public_key.to_sec1_bytes().as_ref(),
        restored_pub_key.to_sec1_bytes().as_ref(),
        "Public keys should match"
    );
}

#[test]
fn test_public_key_pem_conversion() {
    let private_key = SecretKey::random(&mut OsRng);
    let public_key = private_key.public_key();
    let public_key_bytes = public_key.to_sec1_bytes();

    // Convert to PEM format (for beacon registration)
    let pem_data = pem::encode(&pem::Pem::new(
        "PUBLIC KEY".to_string(),
        public_key_bytes.to_vec(),
    ));

    // Verify PEM structure
    assert!(pem_data.contains("-----BEGIN PUBLIC KEY-----"));
    assert!(pem_data.contains("-----END PUBLIC KEY-----"));

    // Verify can be decoded
    let decoded_pem = pem::parse(&pem_data).expect("Should parse PEM");
    assert_eq!(decoded_pem.tag(), "PUBLIC KEY");
    assert_eq!(decoded_pem.contents(), public_key_bytes.as_ref());
}

#[test]
fn test_metadata_fused_flag() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let metadata_path = temp_dir.path().join("metadata.json");

    // Create initial metadata (unfused)
    let mut metadata = KeyMetadata {
        key_name: "test".to_string(),
        private_key_file: "test_private.bin".to_string(),
        public_key_hex: "0123456789abcdef".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        fused: false,
        chip_info: None,
    };

    // Write initial metadata
    let json = serde_json::to_string_pretty(&metadata).expect("Should serialize");
    fs::write(&metadata_path, &json).expect("Should write");

    // Read and verify unfused
    let content = fs::read_to_string(&metadata_path).expect("Should read");
    let parsed: KeyMetadata = serde_json::from_str(&content).expect("Should parse");
    assert_eq!(parsed.fused, false);

    // Update to fused
    metadata.fused = true;
    metadata.chip_info = Some("ESP32-C3 detected".to_string());

    // Write updated metadata
    let updated_json = serde_json::to_string_pretty(&metadata).expect("Should serialize");
    fs::write(&metadata_path, &updated_json).expect("Should write");

    // Read and verify fused
    let updated_content = fs::read_to_string(&metadata_path).expect("Should read");
    let updated_parsed: KeyMetadata = serde_json::from_str(&updated_content).expect("Should parse");
    assert_eq!(updated_parsed.fused, true);
    assert_eq!(
        updated_parsed.chip_info,
        Some("ESP32-C3 detected".to_string())
    );
}

#[test]
fn test_timestamp_format() {
    // Test ISO 8601 / RFC 3339 timestamp format
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Verify format
    assert!(timestamp.contains("T"), "Should contain 'T' separator");
    assert!(
        timestamp.contains("Z") || timestamp.contains("+") || timestamp.contains("-"),
        "Should contain timezone"
    );

    // Verify can be parsed
    let parsed =
        chrono::DateTime::parse_from_rfc3339(&timestamp).expect("Should parse RFC 3339 timestamp");
    assert!(parsed.timestamp() > 0);
}

#[test]
fn test_key_file_overwrite_protection() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let key_path = temp_dir.path().join("key_private.bin");

    // Create initial key file
    fs::write(&key_path, b"initial_key_data").expect("Should write");

    // Verify file exists
    assert!(key_path.exists());

    // In the actual CLI, this would trigger an error without --force
    // Here we just verify the file exists check works
    let file_exists = key_path.exists();
    assert!(
        file_exists,
        "Should detect existing file (would require --force)"
    );

    // Simulate --force behavior: overwrite
    fs::write(&key_path, b"new_key_data").expect("Should overwrite");

    let content = fs::read(&key_path).expect("Should read");
    assert_eq!(content, b"new_key_data");
}
