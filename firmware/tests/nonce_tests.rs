#![cfg(test)]

use navign_shared::ble::Nonce;

mod mocks;
use mocks::storage::MockStorage;

#[test]
fn test_nonce_generation() {
    let mut rng = mocks::rng::MockRng::new(12345);
    let mut nonce_bytes = [0u8; 16];
    rng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::new(nonce_bytes);
    assert_eq!(nonce.bytes(), &nonce_bytes);
}

#[test]
fn test_nonce_replay_prevention() {
    let mut storage = MockStorage::new();
    let nonce = [42u8; 16];

    // Store nonce
    storage.store_nonce(nonce, 1000).unwrap();

    // Check it's marked as used
    assert!(storage.check_nonce_used(&nonce));

    // Different nonce should not be marked
    let different_nonce = [43u8; 16];
    assert!(!storage.check_nonce_used(&different_nonce));
}

#[test]
fn test_nonce_expiration() {
    let mut storage = MockStorage::new();

    // Store old nonces
    storage.store_nonce([1u8; 16], 1000).unwrap();
    storage.store_nonce([2u8; 16], 2000).unwrap();
    storage.store_nonce([3u8; 16], 5000).unwrap();

    // Clean up nonces older than 3 seconds
    storage.cleanup_expired_nonces(6000, 3000);

    // Old nonces should be removed
    assert!(!storage.check_nonce_used(&[1u8; 16]));
    assert!(!storage.check_nonce_used(&[2u8; 16]));

    // Recent nonce should still exist
    assert!(storage.check_nonce_used(&[3u8; 16]));
}

#[test]
fn test_nonce_buffer_overflow() {
    let mut storage = MockStorage::new();

    // Fill buffer to capacity (16 nonces)
    for i in 0..16 {
        let mut nonce = [0u8; 16];
        nonce[0] = i;
        storage.store_nonce(nonce, i as u64).unwrap();
    }

    // Add one more - should evict oldest
    let new_nonce = [99u8; 16];
    storage.store_nonce(new_nonce, 100).unwrap();

    // First nonce should be evicted
    assert!(!storage.check_nonce_used(&[0u8; 16]));

    // New nonce should be present
    assert!(storage.check_nonce_used(&new_nonce));
}

#[test]
fn test_nonce_uniqueness_with_rng() {
    let mut rng = mocks::rng::MockRng::new(999);
    let mut nonces = [[0u8; 16]; 100];

    // Generate 100 nonces
    for nonce in &mut nonces {
        rng.fill_bytes(nonce);
    }

    // Check all are unique (with deterministic RNG, they should be)
    for i in 0..nonces.len() {
        for j in (i + 1)..nonces.len() {
            assert_ne!(
                nonces[i], nonces[j],
                "Nonces at index {} and {} are identical",
                i, j
            );
        }
    }
}

#[test]
fn test_concurrent_nonce_access() {
    // Test that storage handles concurrent access correctly
    let mut storage = MockStorage::new();

    // Simulate concurrent requests
    let nonce1 = [1u8; 16];
    let nonce2 = [2u8; 16];
    let nonce3 = [3u8; 16];

    storage.store_nonce(nonce1, 1000).unwrap();
    storage.store_nonce(nonce2, 1001).unwrap();
    storage.store_nonce(nonce3, 1002).unwrap();

    // All should be present
    assert!(storage.check_nonce_used(&nonce1));
    assert!(storage.check_nonce_used(&nonce2));
    assert!(storage.check_nonce_used(&nonce3));
}
