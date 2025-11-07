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
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to cleanup environment variables after each test
    fn cleanup_env() {
        unsafe {
            env::remove_var("PRIVATE_KEY_FILE");
        }
    }

    #[test]
    fn test_load_or_generate_creates_new_key() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("new_key.pem");

        // Set the environment variable to use our test path
        unsafe {
            env::set_var("PRIVATE_KEY_FILE", key_path.to_str().unwrap());
        }

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
        cleanup_env();
    }

    #[test]
    fn test_load_or_generate_loads_existing_key() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("existing_key.pem");

        // Create a key and save it first
        let original_key = SigningKey::random(&mut OsRng);
        save_key_to_file(&original_key, &key_path).unwrap();

        // Set environment to use this key
        unsafe { env::set_var("PRIVATE_KEY_FILE", key_path.to_str().unwrap()) };

        // load_or_generate should load the existing key
        let loaded_key = load_or_generate_key().unwrap();

        // Verify it's the same key
        assert_eq!(
            original_key.verifying_key().to_encoded_point(false),
            loaded_key.verifying_key().to_encoded_point(false)
        );

        cleanup_env();
    }

    #[test]
    fn test_save_key_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("subdir1/subdir2/test_key.pem");

        // Parent directories don't exist yet
        assert!(!key_path.parent().unwrap().exists());

        // Save should create parent directories
        let key = SigningKey::random(&mut OsRng);
        save_key_to_file(&key, &key_path).unwrap();

        // Verify parent directories were created
        assert!(key_path.parent().unwrap().exists());
        assert!(key_path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_saved_key_has_correct_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("secure_key.pem");

        // Save a key
        let key = SigningKey::random(&mut OsRng);
        save_key_to_file(&key, &key_path).unwrap();

        // Check permissions are 0o600 (read/write for owner only)
        let metadata = fs::metadata(&key_path).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    #[test]
    fn test_load_key_with_invalid_pem() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("invalid.pem");

        // Write invalid PEM content
        let mut file = fs::File::create(&key_path).unwrap();
        file.write_all(b"This is not a valid PEM file").unwrap();

        // Loading should fail
        let result = load_key_from_file(&key_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse private key")
        );
    }

    #[test]
    fn test_load_key_with_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("nonexistent.pem");

        // Loading non-existent file should fail
        let result = load_key_from_file(&key_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read private key file")
        );
    }

    #[test]
    fn test_get_key_file_path_default() {
        // Without environment variable, should use default
        cleanup_env();
        let path = get_key_file_path();
        assert_eq!(path, PathBuf::from(DEFAULT_KEY_FILE));
    }

    #[test]
    fn test_get_key_file_path_from_env() {
        let custom_path = "/custom/path/to/key.pem";
        unsafe {
            env::set_var("PRIVATE_KEY_FILE", custom_path);
        }

        let path = get_key_file_path();
        assert_eq!(path, PathBuf::from(custom_path));

        cleanup_env();
    }

    #[test]
    fn test_key_pem_format_is_valid() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("format_test.pem");

        // Save a key
        let key = SigningKey::random(&mut OsRng);
        save_key_to_file(&key, &key_path).unwrap();

        // Read the PEM file
        let pem_contents = fs::read_to_string(&key_path).unwrap();

        // Verify it starts with PEM header and ends with footer
        assert!(pem_contents.starts_with("-----BEGIN PRIVATE KEY-----"));
        assert!(
            pem_contents
                .trim_end()
                .ends_with("-----END PRIVATE KEY-----")
        );

        // Verify it contains base64 encoded data
        let lines: Vec<&str> = pem_contents.lines().collect();
        assert!(lines.len() > 2); // At least header + data + footer
    }

    #[test]
    fn test_multiple_save_and_load_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("cycle_test.pem");

        // Generate original key
        let original_key = SigningKey::random(&mut OsRng);
        let original_pub = original_key.verifying_key().to_encoded_point(false);

        // Save and load 5 times
        for i in 0..5 {
            if i == 0 {
                save_key_to_file(&original_key, &key_path).unwrap();
            }

            let loaded_key = load_key_from_file(&key_path).unwrap();
            assert_eq!(
                loaded_key.verifying_key().to_encoded_point(false),
                original_pub,
                "Key mismatch on cycle {}",
                i
            );

            // Overwrite with the same key
            save_key_to_file(&loaded_key, &key_path).unwrap();
        }
    }

    #[test]
    fn test_concurrent_key_operations() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = Arc::new(TempDir::new().unwrap());
        let mut handles = vec![];

        // Create multiple threads that try to save keys
        for i in 0..5 {
            let temp_dir = Arc::clone(&temp_dir);
            let handle = thread::spawn(move || {
                let key_path = temp_dir.path().join(format!("concurrent_key_{}.pem", i));
                let key = SigningKey::random(&mut OsRng);
                save_key_to_file(&key, &key_path).unwrap();
                load_key_from_file(&key_path).unwrap()
            });
            handles.push(handle);
        }

        // All operations should succeed
        for handle in handles {
            assert!(handle.join().is_ok());
        }
    }
}
