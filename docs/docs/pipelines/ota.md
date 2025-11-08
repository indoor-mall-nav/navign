# OTA (Over-The-Air) Update Pipeline

The OTA update pipeline enables remote firmware deployment to ESP32-C3 beacons without physical access, addressing the operational challenge of maintaining firmware across hundreds or thousands of deployed devices. The system implements a secure, fault-tolerant update mechanism that prevents bricking devices through failed updates while maintaining continuous operation of positioning and access control services.

## Pipeline Overview

The complete OTA process involves coordination across multiple components:

```
Firmware Upload → Orchestrator Distribution → WiFi Provisioning → Download → Flash Write → Validation → Activation
```

## Stage 1: Firmware Upload and Storage

Firmware updates begin with an administrator uploading a new binary to the server.

**Upload Endpoint:**

```
POST /api/firmwares/upload
Content-Type: multipart/form-data

{
  "device_type": "esp32c3",
  "version": "1.2.3",
  "binary": <file>,
  "description": "Fix nonce expiration bug"
}
```

**Server-Side Processing:**

The server validates and stores the firmware:

```rust
async fn upload_firmware(binary: Bytes, metadata: FirmwareMetadata) -> Result<FirmwareId> {
    // Validate binary format
    verify_esp32_binary_header(&binary)?;

    // Calculate checksum
    let checksum = sha256(&binary);

    // Store in database
    let firmware = Firmware {
        id: generate_id(),
        device_type: metadata.device_type,
        version: metadata.version,
        binary: binary.to_vec(),
        checksum,
        uploaded_at: current_timestamp(),
        description: metadata.description,
    };

    db.firmwares.insert(firmware).await?;
    Ok(firmware.id)
}
```

**Binary Validation:**

The server checks that the uploaded binary is a valid ESP32 firmware:
- Starts with ESP32 magic bytes (`0xE9`)
- Contains valid segment headers
- Total size < 2MB (fits in OTA partition)

Invalid binaries are rejected before storage to prevent distribution of corrupted firmware.

**Version Management:**

The system maintains multiple firmware versions simultaneously:
- Latest stable release
- Beta releases for testing
- Previous stable (rollback target)

Each firmware has a semantic version (major.minor.patch) and deployment status (testing, stable, deprecated).

## Stage 2: Firmware Distribution via Orchestrator

The orchestrator serves as the distribution point for firmware downloads, reducing load on the primary server.

**Latest Firmware Query:**

Beacons query for available updates:

```
GET /firmwares/latest/esp32c3
```

**Response:**

```json
{
  "firmware_id": "507f1f77bcf86cd799439011",
  "version": "1.2.3",
  "checksum": "a1b2c3d4...",
  "size": 1048576,
  "download_url": "/firmwares/507f1f77bcf86cd799439011/download",
  "required": false,
  "description": "Fix nonce expiration bug"
}
```

**Version Comparison:**

The beacon compares the available version with its current version:

```rust
fn should_update(current: &Version, available: &Version) -> bool {
    available > current
}
```

If a newer version is available, the beacon initiates the download process.

**Staged Rollout:**

To prevent mass bricking from bad firmware, updates roll out in stages:
1. 10% of beacons (canary deployment)
2. Wait 24 hours, monitor for failures
3. 50% of remaining beacons
4. Wait 12 hours
5. 100% of beacons

The orchestrator tracks which beacons have updated and enforces rollout percentages.

## Stage 3: WiFi Provisioning (BluFi)

ESP32-C3 beacons support both BLE and WiFi but don't have persistent WiFi credentials stored. WiFi provisioning occurs through BluFi, a protocol that transfers WiFi credentials over BLE.

**BluFi Protocol Flow:**

```
Mobile App → Beacon (BLE): WiFi SSID + Password (encrypted)
Beacon: Store credentials in flash
Beacon: Connect to WiFi
Beacon → Mobile App (BLE): Connection status
```

**Encryption:**

BluFi encrypts WiFi credentials using AES-128 with a session key derived from ECDH key exchange:

```rust
// Mobile and beacon perform ECDH
let mobile_privkey = generate_p256_key();
let beacon_pubkey = receive_beacon_pubkey();
let shared_secret = ecdh(mobile_privkey, beacon_pubkey);

// Derive AES key
let aes_key = hkdf(shared_secret, salt, info);

// Encrypt WiFi credentials
let encrypted = aes_encrypt(aes_key, wifi_ssid || wifi_password);
```

This prevents passive BLE eavesdroppers from capturing WiFi credentials.

**Credential Storage:**

The beacon stores WiFi credentials in NVS (Non-Volatile Storage):

```rust
let mut nvs = NvsDefault::new()?;
nvs.set_str("wifi_ssid", ssid)?;
nvs.set_str("wifi_pass", password)?;
```

On subsequent boots, the beacon automatically connects to WiFi using stored credentials.

**Connection Establishment:**

```rust
let wifi = Wifi::new(peripherals.WIFI)?;
wifi.set_mode(WifiMode::Sta)?;
wifi.set_configuration(&Configuration::Client(ClientConfiguration {
    ssid: stored_ssid,
    password: stored_password,
    ..Default::default()
}))?;
wifi.start()?;
wifi.connect()?;
```

Connection typically completes within 2-5 seconds.

## Stage 4: Firmware Download

With WiFi connectivity established, the beacon downloads the new firmware over HTTP.

**Download Request:**

```
GET /firmwares/{firmware_id}/download
Host: orchestrator.local:8080
```

**Chunked Download:**

Firmwares are ~1MB, which exceeds available RAM. The beacon downloads in 4KB chunks, writing each directly to flash:

```rust
let mut ota = OtaManager::new(flash)?;
ota.begin_update(Some(firmware_size))?;

let mut http_client = HttpClient::new();
let mut stream = http_client.get(download_url).await?;

while let Some(chunk) = stream.next_chunk(4096).await? {
    ota.write_chunk(&chunk)?;
}

ota.finalize_update()?;
```

**Progress Indication:**

The beacon emits BLE advertisements with download progress:

```
Manufacturer Data: [0xFF, 0x59, 0x00, progress_percent]
```

Mobile apps can display "Beacon updating: 47%" to inform users of ongoing updates.

**Interruption Handling:**

If download is interrupted (WiFi disconnection, power loss):
- Partial firmware in OTA partition is discarded
- Next connection attempt restarts from beginning
- Beacon continues operating from current partition

The dual-bank partition system ensures the running firmware is never corrupted during download.

## Stage 5: Flash Write and Partitioning

The ESP32-C3 flash is organized into partitions defined at compile time:

```
0x000000-0x010000: Bootloader
0x010000-0x110000: Factory (initial firmware)
0x110000-0x210000: OTA_0
0x210000-0x310000: OTA_1
0x310000-0x320000: OTA Data
```

**Partition Selection:**

The OTA manager determines which partition to write:

```rust
let current_partition = ota.running_partition()?;
let target_partition = if current_partition == OTA_0 {
    OTA_1
} else {
    OTA_0
};
```

Updates always write to the inactive partition, leaving the running firmware untouched.

**Write Operation:**

Flash writes occur in 4KB sectors (ESP32-C3 flash page size):

```rust
fn write_chunk(&mut self, data: &[u8]) -> Result<()> {
    let offset = self.bytes_written;
    self.flash.write(target_partition_addr + offset, data)?;
    self.bytes_written += data.len();
    Ok(())
}
```

**Erase-Before-Write:**

Flash memory requires erasing before writing. The OTA manager automatically erases sectors:

```rust
fn begin_update(&mut self) -> Result<()> {
    let partition_size = 1MB;
    for sector in 0..(partition_size / 4KB) {
        self.flash.erase_sector(target_partition_addr + sector * 4KB)?;
    }
    Ok(())
}
```

This erase operation takes ~2 seconds for a 1MB partition.

## Stage 6: Checksum Verification

After downloading the complete firmware, the beacon verifies integrity:

```rust
fn verify_checksum(&self, expected_checksum: &[u8]) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];

    for offset in (0..self.bytes_written).step_by(4096) {
        let len = min(4096, self.bytes_written - offset);
        self.flash.read(target_partition_addr + offset, &mut buffer[..len])?;
        hasher.update(&buffer[..len]);
    }

    let computed = hasher.finalize();
    if computed.as_slice() == expected_checksum {
        Ok(())
    } else {
        Err(Error::ChecksumMismatch)
    }
}
```

**Mismatch Handling:**

If checksums don't match:
- Log error to flash
- Discard downloaded firmware
- Report failure to orchestrator
- Retry download (with exponential backoff)

Checksum verification prevents activation of corrupted firmware that would brick the device.

## Stage 7: Partition Activation

After successful verification, the beacon marks the new partition as active:

```rust
ota.set_boot_partition(target_partition)?;
```

This writes to the OTA Data partition:

```
struct OtaData {
    seq_label_0: u32,     // Partition 0 sequence number
    seq_label_1: u32,     // Partition 1 sequence number
    // Bootloader selects partition with higher sequence number
}
```

The sequence number increments:
```rust
ota_data.seq_label_1 += 1;  // Assuming OTA_1 is target
```

## Stage 8: Reboot and Bootloader Selection

The beacon reboots to activate the new firmware:

```rust
esp_hal::reset::software_reset();
```

**Bootloader Logic:**

On boot, the ESP-IDF bootloader reads the OTA Data partition and selects the partition with the highest sequence number:

```c
if (ota_data.seq_label_0 > ota_data.seq_label_1) {
    boot_partition = OTA_0;
} else {
    boot_partition = OTA_1;
}
```

The bootloader loads the firmware from the selected partition into RAM and transfers execution.

## Stage 9: Firmware Validation

The new firmware must mark itself as valid to prevent automatic rollback:

```rust
fn main() {
    // Initialize hardware
    let flash = FlashStorage::new(peripherals.FLASH);
    let mut ota = OtaManager::new(flash)?;

    // Check if this is first boot after update
    if let Ok(state) = ota.current_ota_state() {
        if matches!(state, OtaImageState::New | OtaImageState::PendingVerify) {
            // Firmware is new, mark as valid
            ota.set_current_ota_state(OtaImageState::Valid)?;
        }
    }

    // Continue normal operation
    run_beacon_firmware();
}
```

**Rollback Protection:**

If the new firmware is never marked valid (crashes before reaching `set_current_ota_state`), the bootloader's rollback mechanism triggers on next reboot:

```c
if (boot_count > 3 && ota_state != Valid) {
    // Firmware failed to validate itself after 3 boots
    // Rollback to previous partition
    boot_partition = previous_partition;
}
```

This automatic rollback requires ESP-IDF bootloader configuration with rollback support enabled.

## Stage 10: Update Reporting

After successful activation, the beacon reports completion to the orchestrator:

```
POST /api/beacons/{beacon_id}/ota_status
{
  "firmware_version": "1.2.3",
  "update_status": "success",
  "updated_at": 1735689085,
  "previous_version": "1.2.2"
}
```

The orchestrator tracks rollout progress and can halt deployment if failure rates exceed thresholds.

## Failure Recovery

The pipeline includes comprehensive failure handling:

**Download Failures:**
- Network errors: Retry with exponential backoff (2s, 4s, 8s, 16s, 32s)
- Timeout: Abort after 5 minutes, retry later
- Partial download: Discard and restart

**Flash Errors:**
- Write failure: Mark sector as bad, retry at different offset
- Erase failure: Mark entire partition as failed, use alternate partition

**Checksum Failures:**
- Discard firmware, log error
- Report to orchestrator (possible server-side corruption)
- Retry download (server may have fixed the issue)

**Boot Failures:**
- Bootloader automatic rollback after 3 failed boots
- Beacon resumes operation on previous firmware
- Reports failure to orchestrator

## Security Considerations

**Firmware Signing (Future):**

Currently, checksum verification ensures integrity but not authenticity. A malicious actor controlling the network could serve malicious firmware with correct checksums.

Future enhancement: RSA signature verification
```rust
let signature = firmware_metadata.signature;
let server_pubkey = embedded_server_pubkey();
rsa_verify(firmware_binary, signature, server_pubkey)?;
```

Only firmwares signed with the server's private key would be accepted.

**Encrypted Firmware (Future):**

Firmware downloaded over HTTP is visible to network observers. While this doesn't compromise running beacons (they're already deployed), it reveals implementation details.

Future enhancement: AES-encrypted firmware
```rust
let encrypted_firmware = download_firmware();
let decryption_key = derive_key_from_device_id();
let firmware = aes_decrypt(encrypted_firmware, decryption_key);
```

**Rollback Attacks:**

An attacker could force beacons to downgrade to older firmware with known vulnerabilities by serving old firmware as "latest."

Defense: Version monotonicity check
```rust
if new_version <= current_version {
    return Err(Error::DowngradeNotAllowed);
}
```

Beacons refuse to install older firmware versions.

## Performance and Timing

**Download Speed:**

Typical WiFi throughput: 500 KB/s - 2 MB/s
1MB firmware download time: 0.5 - 2 seconds

**Flash Write Speed:**

ESP32-C3 flash write: ~100 KB/s
1MB firmware write time: ~10 seconds

**Total Update Time:**

- WiFi connection: 2-5 seconds
- Firmware download: 0.5-2 seconds
- Checksum verification: 1-2 seconds
- Flash write: 10 seconds
- Reboot and validation: 2-3 seconds

**Total: 15-22 seconds** from update initiation to new firmware operational.

**Service Interruption:**

During download and flash write, the beacon continues BLE advertising and can perform access control. Only during reboot (2-3 seconds) is the beacon unavailable.

## Deployment Strategy

**Scheduled Updates:**

Updates occur during maintenance windows (e.g., 3 AM) to minimize user impact:

```rust
if current_hour() >= 3 && current_hour() < 5 {
    check_for_updates();
}
```

**Forced Emergency Updates:**

Critical security patches can be marked "required":

```json
{
  "required": true,
  "version": "1.2.4-security",
  "description": "Critical: Fixes CVE-2024-XXXX"
}
```

Beacons install required updates immediately regardless of schedule.

**Geographic Rollout:**

For large deployments, updates roll out by building/region:
- Building A: Week 1
- Building B: Week 2
- Remaining buildings: Week 3

This limits blast radius if issues are discovered post-release.

## Monitoring and Observability

The orchestrator tracks update metrics:

- Update success rate per firmware version
- Average update duration
- Failure reasons (checksum, network, flash)
- Rollback occurrences

Dashboard visualization enables administrators to monitor rollout health and halt deployment if problems emerge.

## Related Documentation

- [Beacon OTA Implementation](/components/beacon#ota-update-system)
- [Orchestrator Firmware Distribution](/components/admin/orchestrator#firmware-distribution)
- [WiFi Provisioning (BluFi)](/components/beacon/blufi)
