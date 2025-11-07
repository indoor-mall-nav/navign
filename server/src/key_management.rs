use anyhow::{Context, Result};
use log::info;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_KEY_FILE: &str = "private_key.pem";

/// Get the path to the private key file.
/// Checks the environment variable `PRIVATE_KEY_FILE` first,
/// otherwise uses the default path `./private_key.pem`.
fn get_key_file_path() -> PathBuf {
    std::env::var("PRIVATE_KEY_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_KEY_FILE))
}

/// Load the private key from a file, or generate a new one if it doesn't exist.
pub fn load_or_generate_key() -> Result<SigningKey> {
    let key_path = get_key_file_path();

    if key_path.exists() {
        info!("Loading existing private key from: {:?}", key_path);
        load_key_from_file(&key_path)
    } else {
        info!(
            "No existing private key found. Generating new key and saving to: {:?}",
            key_path
        );
        let key = SigningKey::random(&mut OsRng);
        save_key_to_file(&key, &key_path)?;
        Ok(key)
    }
}

/// Load a private key from a PEM file.
fn load_key_from_file(path: &Path) -> Result<SigningKey> {
    let pem_contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read private key file: {:?}", path))?;

    let key = SigningKey::from_pkcs8_pem(&pem_contents)
        .with_context(|| "Failed to parse private key from PEM format")?;

    Ok(key)
}

/// Save a private key to a PEM file with restricted permissions.
fn save_key_to_file(key: &SigningKey, path: &Path) -> Result<()> {
    let pem = key
        .to_pkcs8_pem(LineEnding::LF)
        .context("Failed to encode private key to PEM format")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Write the PEM file
    fs::write(path, pem.as_bytes())
        .with_context(|| format!("Failed to write private key to file: {:?}", path))?;

    // Set file permissions to 600 (read/write for owner only) on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to set permissions on key file: {:?}", path))?;
    }

    info!("Private key saved successfully to: {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_key() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test_key.pem");

        // Generate and save a key
        let original_key = SigningKey::random(&mut OsRng);
        save_key_to_file(&original_key, &key_path).unwrap();

        // Load the key back
        let loaded_key = load_key_from_file(&key_path).unwrap();

        // Verify the keys match by comparing their public keys
        assert_eq!(
            original_key.verifying_key().to_encoded_point(false),
            loaded_key.verifying_key().to_encoded_point(false)
        );
    }

    #[test]
    fn test_load_or_generate_creates_new_key() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("new_key.pem");

        // Set the environment variable to use our test path
        env::set_var("PRIVATE_KEY_FILE", key_path.to_str().unwrap());

        // This should generate a new key
        let key = load_or_generate_key().unwrap();

        // Verify the file was created
        assert!(key_path.exists());

        // Load the key again - should load the same key
        let key2 = load_or_generate_key().unwrap();

        // Verify they're the same
        assert_eq!(
            key.verifying_key().to_encoded_point(false),
            key2.verifying_key().to_encoded_point(false)
        );

        // Clean up
        env::remove_var("PRIVATE_KEY_FILE");
    }
}
