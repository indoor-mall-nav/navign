# Beacon OTA Integration Guide

This document explains how to integrate Over-The-Air (OTA) firmware updates into the beacon firmware.

## Overview

The OTA module (`src/bin/ota.rs`) provides partition management and firmware flashing functionality for ESP32-C3 beacons. It does **NOT** include WiFi or HTTP download code - that must be implemented separately.

## Architecture

```
┌─────────────┐    HTTP/WiFi    ┌──────────────┐
│ Server/Orch │◄───────────────►│ Beacon WiFi  │
│             │  (to implement) │              │
└─────────────┘                 └───────┬──────┘
                                        │
                                        ▼
                                ┌──────────────┐
                                │ OTA Manager  │
                                │              │
                                │ - Partition  │
                                │ - Flash      │
                                │ - Activate   │
                                └──────────────┘
```

## Usage Example

### 1. Initialize OTA Manager on Boot

```rust
use crate::ota::{OtaManager, OtaError};
use esp_storage::FlashStorage;

#[main]
fn main() -> ! {
    // ... existing initialization ...

    let flash = FlashStorage::new(peripherals.FLASH);
    let mut ota_manager = OtaManager::new(flash).expect("Failed to initialize OTA");

    // Mark current firmware as valid (prevents auto-rollback)
    ota_manager.mark_valid().ok();

    // Log partition info
    if let Ok(current) = ota_manager.current_partition() {
        log::info!("Running from partition: {:?}", current);
    }

    // For debugging, list all partitions
    ota_manager.list_partitions().ok();

    // ... rest of beacon code ...
}
```

### 2. Handle OTA Update Request (BLE)

Add OTA update capability to your BLE characteristics:

```rust
// In your BLE message handler
match message {
    Some(BleMessage::OtaUpdateRequest { version, size }) => {
        log::info!("OTA update requested: v{}, {} bytes", version, size);

        // Start OTA update
        ota_manager.begin_update(Some(size))?;

        // Respond with confirmation
        Some(BleMessage::OtaUpdateResponse { ready: true })
    },

    Some(BleMessage::OtaChunk { sequence, data }) => {
        // Write firmware chunk
        match ota_manager.write_chunk(&data) {
            Ok(written) => {
                log::debug!("OTA chunk {}: {} bytes written", sequence, written);
                Some(BleMessage::OtaChunkAck { sequence })
            },
            Err(e) => {
                log::error!("OTA write failed: {:?}", e);
                ota_manager.abort_update().ok();
                Some(BleMessage::OtaUpdateResponse { ready: false })
            }
        }
    },

    Some(BleMessage::OtaFinalizeRequest) => {
        // Finalize update and reboot
        match ota_manager.finalize_update() {
            Ok(_) => {
                log::info!("OTA update finalized, rebooting...");
                // Reboot after a short delay
                esp_hal::reset::software_reset();
            },
            Err(e) => {
                log::error!("OTA finalization failed: {:?}", e);
                ota_manager.abort_update().ok();
                Some(BleMessage::OtaUpdateResponse { ready: false })
            }
        }
    },

    // ... existing message handlers ...
}
```

### 3. WiFi-Based OTA (Future Implementation)

When WiFi support is added, you can implement HTTP-based OTA:

```rust
use esp_wifi::wifi::{WifiController, WifiDevice, WifiEvent, WifiState};
use embedded_svc::http::client::Client;

async fn check_and_download_firmware(
    ota_manager: &mut OtaManager,
    server_url: &str,
) -> Result<(), OtaError> {
    // Connect to WiFi (code to be implemented)
    let mut wifi = connect_wifi().await?;

    // Query orchestrator for latest firmware
    let firmware_url = format!("{}/firmwares/latest/esp32c3", server_url);
    let metadata = http_get_json(&firmware_url).await?;

    let firmware_version = metadata["version"].as_str().unwrap();
    let firmware_size = metadata["file_size"].as_u64().unwrap() as u32;
    let firmware_id = metadata["id"].as_str().unwrap();

    log::info!("Latest firmware: v{}, {} bytes", firmware_version, firmware_size);

    // Check if update needed (compare versions)
    let current_version = env!("CARGO_PKG_VERSION");
    if firmware_version == current_version {
        log::info!("Already running latest firmware");
        return Ok(());
    }

    // Download firmware
    let download_url = format!("{}/firmwares/{}/download", server_url, firmware_id);

    log::info!("Downloading firmware from: {}", download_url);
    ota_manager.begin_update(Some(firmware_size))?;

    // Stream download in chunks
    let mut http_client = HttpClient::new();
    let mut response = http_client.get(&download_url).await?;

    while let Some(chunk) = response.next_chunk().await? {
        ota_manager.write_chunk(&chunk)?;
    }

    // Verify checksum (important!)
    let expected_checksum = metadata["checksum"].as_str().unwrap();
    // TODO: Calculate actual checksum and verify

    // Finalize and reboot
    ota_manager.finalize_update()?;
    log::info!("Firmware download complete, rebooting...");

    esp_hal::reset::software_reset();

    Ok(())
}
```

## OTA State Machine

The OTA module maintains a state machine:

```
Idle
  │
  ├─ begin_update()
  │
  ▼
Writing
  │
  ├─ write_chunk() (repeat)
  │
  ├─ finalize_update()
  │
  ▼
ReadyToActivate
  │
  ├─ esp_hal::reset::software_reset()
  │
  ▼
(Reboot to new firmware)
```

You can abort at any time with `abort_update()`.

## Partition Layout

The ESP32-C3 uses this partition layout for OTA:

```
┌──────────────┐ 0x0000
│  Bootloader  │
├──────────────┤ 0x10000
│   Factory    │  (initial firmware)
├──────────────┤ 0x110000
│     OTA0     │  (first update slot)
├──────────────┤ 0x210000
│     OTA1     │  (second update slot)
├──────────────┤
│   OTA Data   │  (tracks active partition)
└──────────────┘
```

The OTA manager automatically selects the next available slot.

## BLE Message Protocol Extension

To support OTA via BLE, add these message types to `shared/src/ble/message.rs`:

```rust
pub enum BleMessage {
    // ... existing messages ...

    /// Request OTA update
    OtaUpdateRequest { version: String, size: u32 },

    /// Response to OTA update request
    OtaUpdateResponse { ready: bool },

    /// Firmware chunk data
    OtaChunk { sequence: u32, data: Vec<u8> },

    /// ACK for firmware chunk
    OtaChunkAck { sequence: u32 },

    /// Finalize OTA update
    OtaFinalizeRequest,
}
```

## Progress Tracking

Monitor OTA progress:

```rust
if let Some((written, total)) = ota_manager.get_progress() {
    let percent = (written as f32 / total as f32) * 100.0;
    log::info!("OTA progress: {:.1}% ({} / {} bytes)", percent, written, total);
}
```

## Error Handling

The OTA module uses a custom error type:

```rust
pub enum OtaError {
    PartitionTableError,    // Can't read partition table
    NoOtaPartitions,        // No OTA partitions available
    UpdaterInitFailed,      // OTA updater init failed
    WriteFailed,            // Flash write failed
    InvalidFirmwareSize,    // Size mismatch
    StateChangeFailed,      // Invalid state transition
    ActivationFailed,       // Can't activate partition
    BufferOverflow,         // Write buffer overflow
}
```

Always check errors and abort on failure:

```rust
if let Err(e) = ota_manager.write_chunk(&data) {
    log::error!("OTA write failed: {:?}", e);
    ota_manager.abort_update().ok();
    return Err(e);
}
```

## Rollback Protection

The bootloader supports automatic rollback if the new firmware fails to boot:

1. After `finalize_update()`, state is set to `New`
2. On first boot, bootloader sets state to `PendingVerify`
3. Your firmware must call `mark_valid()` on successful boot
4. If firmware crashes before calling `mark_valid()`, bootloader rolls back

**Important**: Only supported if bootloader compiled with rollback feature!

## Security Considerations

### 1. Firmware Signature Verification

**Currently NOT implemented** - you should add signature verification:

```rust
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

fn verify_firmware_signature(
    firmware: &[u8],
    signature: &[u8],
    public_key: &VerifyingKey,
) -> Result<(), CryptoError> {
    let sig = Signature::try_from(signature)?;
    public_key.verify(firmware, &sig)?;
    Ok(())
}
```

### 2. Encrypted Firmware

For enhanced security, encrypt firmware downloads:

```rust
use aes_gcm::{Aes256Gcm, Nonce, KeyInit};
use aes_gcm::aead::Aead;

fn decrypt_firmware_chunk(
    encrypted: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher.decrypt(nonce, encrypted.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed)
}
```

### 3. Checksum Verification

Always verify the firmware checksum before activating:

```rust
use sha2::{Sha256, Digest};

fn calculate_firmware_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
```

## Testing

### Test OTA Locally

```bash
# 1. Build current firmware
cd beacon
cargo build --release

# 2. Flash to device
espflash flash target/riscv32imc-esp-espidf/release/navign-beacon

# 3. Build "new" firmware (with version bump)
# Edit Cargo.toml: version = "0.1.1"
cargo build --release

# 4. Save firmware image
espflash save-image --chip=esp32c3 \
    target/riscv32imc-esp-espidf/release/navign-beacon \
    firmware.bin

# 5. Upload to server
curl -X POST http://localhost:3000/api/firmwares/upload \
  -F "version=0.1.1" \
  -F "device=esp32c3" \
  -F "file=@firmware.bin" \
  -F "mark_latest=true"

# 6. Trigger OTA via BLE or WiFi
# (implementation-dependent)
```

## Troubleshooting

### "No OTA partitions available"

Check partition table:
```bash
espflash partition-table target/riscv32imc-esp-espidf/release/navign-beacon
```

Ensure you have OTA0 and OTA1 partitions defined.

### "Write failed"

- Check flash permissions
- Ensure partition has enough space
- Verify firmware size doesn't exceed partition size
- Check for flash wear (OTA partitions have limited write cycles)

### Firmware boots then immediately rolls back

- Call `mark_valid()` early in your firmware startup
- Check bootloader rollback timeout settings
- Review firmware crash logs

## Dependencies

The OTA module requires these dependencies in `Cargo.toml`:

```toml
[dependencies]
esp-bootloader-esp-idf = "0.1"
esp-storage = "0.8"
embedded-storage = "0.3"
esp-println = "0.13"
```

## Next Steps

1. ✅ OTA partition management implemented
2. ⏳ Implement WiFi connectivity
3. ⏳ Implement HTTP firmware download
4. ⏳ Add BLE OTA message types to shared library
5. ⏳ Implement signature verification
6. ⏳ Add checksum verification
7. ⏳ Implement encrypted firmware downloads
8. ⏳ Add OTA progress reporting via BLE
9. ⏳ Test with real beacon hardware

## References

- [ESP-IDF OTA Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/ota.html)
- [esp-bootloader-esp-idf crate](https://docs.rs/esp-bootloader-esp-idf/)
- [ESP32 Partition Tables](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html)
