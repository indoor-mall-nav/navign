# Unlock Pipeline

The unlock pipeline implements a cryptographically secure challenge-response protocol for controlling physical access points (doors, gates, turnstiles) via mobile devices. Unlike traditional access control systems relying on RFID cards or PIN codes, this implementation uses asymmetric cryptography to prevent replay attacks, relay attacks, and unauthorized access, while maintaining sub-second latency for user experience.

## Security Requirements

The protocol must satisfy several security properties:

1. **Authentication**: Only authorized users can unlock doors
2. **Non-Repudiation**: All unlock events are logged with cryptographic proof of who performed them
3. **Replay Prevention**: Captured unlock proofs cannot be reused
4. **Relay Resistance**: Attackers cannot relay communication between legitimate devices
5. **Beacon Authenticity**: Mobile can verify it's communicating with a genuine beacon
6. **User Privacy**: Unlock attempts don't leak user identity to passive observers

## Protocol Overview

The complete unlock sequence involves six phases across three participants (mobile, beacon, server):

```
Phase 1: Nonce Challenge (Mobile ↔ Beacon)
Phase 2: Server Verification (Mobile ↔ Server)
Phase 3: Proof Generation (Mobile)
Phase 4: Beacon Verification (Mobile ↔ Beacon)
Phase 5: Physical Unlock (Beacon)
Phase 6: Audit Logging (Mobile → Server)
```

## Phase 1: Nonce Challenge

The unlock process begins when the user taps "Unlock" in the mobile UI. The mobile app connects to the beacon via BLE GATT and requests a challenge nonce.

**Message Flow:**

```
Mobile → Beacon: NonceRequest (1 byte)
Beacon: Generate 32-byte random nonce
Beacon: Sign nonce with beacon private key
Beacon → Mobile: NonceResponse(nonce, signature_fragment)
```

**Nonce Generation:**

The beacon uses the ESP32-C3's hardware True Random Number Generator (TRNG):

```rust
let mut nonce = [0u8; 32];
rng.read(&mut nonce);
```

The TRNG derives entropy from radio noise and ADC fluctuations, providing cryptographic-quality randomness. This ensures nonces are unpredictable—an attacker cannot guess future nonces or determine past nonces from observation.

**Beacon Signature:**

The beacon signs the nonce with its P-256 ECDSA private key (stored in efuse):

```rust
let signature = signing_key.sign(nonce);
let signature_bytes = signature.to_bytes();  // 64 bytes
let signature_id = &signature_bytes[56..64]; // Last 8 bytes
```

Only the last 8 bytes are transmitted to the mobile. This truncated signature serves as a "fingerprint" proving the beacon possesses the private key without transmitting the full signature.

**Why Truncated Signatures?**

Full P-256 ECDSA signatures are 64 bytes. Transmitting this over BLE (20-byte MTU) requires fragmentation. The 8-byte fragment provides sufficient collision resistance (2^64 possibilities) while fitting in a single BLE packet alongside the 32-byte nonce.

The mobile doesn't verify this signature immediately—it trusts that a device capable of producing a plausible signature fragment is likely a genuine beacon. Full verification would require knowing the beacon's public key a priori, which the mobile doesn't have during initial connection.

**Nonce Storage:**

The beacon stores the nonce in its `NonceManager`:

```rust
struct NonceManager<const N: usize> {
    nonces: [(Nonce, u64); N],  // (nonce, timestamp)
    count: usize,
}
```

The manager tracks up to 32 active nonces with their generation timestamps. This enables replay detection and expiration enforcement.

## Phase 2: Server Verification

After receiving the nonce, the mobile contacts the server to create an unlock instance. This involves both authorization checking and TOTP generation.

**Request:**

```
POST /api/entities/{entity_id}/beacons/{beacon_id}/unlocker
Authorization: Bearer {jwt_token}
```

The JWT token identifies the user making the unlock request. The server validates:
1. Token is valid and not expired
2. User has permission to access this beacon
3. Beacon exists and is configured for access control

**Server-Side Logic:**

```rust
async fn create_unlock_instance(
    user: AuthenticatedUser,
    beacon_id: BeaconId,
) -> Result<UnlockInstance> {
    // Check user authorization
    if !user.can_unlock(beacon_id) {
        return Err(UnauthorizedError);
    }

    // Generate TOTP code
    let totp_secret = load_beacon_secret(beacon_id);
    let totp_code = generate_totp(totp_secret, current_timestamp);

    // Create unlock instance
    let instance = UnlockInstance {
        id: generate_id(),
        user_id: user.id,
        beacon_id,
        totp_code,
        created_at: current_timestamp,
        status: "pending",
    };

    db.insert(instance).await?;
    Ok(instance)
}
```

**TOTP Generation:**

TOTP (Time-based One-Time Password) provides time-limited authorization:

```rust
fn generate_totp(secret: &[u8], timestamp: u64) -> u32 {
    let time_step = timestamp / 30;  // 30-second windows
    let message = time_step.to_be_bytes();
    let hmac = HMAC-SHA1(secret, message);
    let offset = hmac[19] & 0x0F;
    let code = u32::from_be_bytes(&hmac[offset..offset+4]) & 0x7FFFFFFF;
    code % 1_000_000  // 6-digit code
}
```

The TOTP changes every 30 seconds. If the mobile delays too long between requesting the instance and submitting the unlock proof, the TOTP expires and the unlock fails.

**Response:**

```json
{
  "instance_id": "507f1f77bcf86cd799439011",
  "totp_code": "123456",
  "expires_at": 1735689090
}
```

## Phase 3: Proof Generation

With the nonce from the beacon and TOTP from the server, the mobile constructs a cryptographic proof of authorization.

**Proof Structure:**

```rust
struct Proof {
    nonce: [u8; 32],            // Challenge from beacon
    device_bytes: [u8; 32],     // Mobile device identifier
    verify_bytes: [u8; 32],     // HMAC(device_id || totp)
    timestamp: u64,             // Proof generation time
    server_signature: [u8; 64], // (Future) Server signature of TOTP
}
```

**Signature Calculation:**

The mobile signs the proof with its private P-256 ECDSA key:

```rust
let signing_key = load_user_private_key();  // From Stronghold
let proof_payload = serialize_proof(proof);
let signature = signing_key.sign(proof_payload);
```

This signature proves possession of the private key without revealing it. Anyone with the mobile's public key (which the beacon has) can verify the signature, but only the mobile can create it.

**Device Identifier:**

The device_bytes field contains a unique identifier for the mobile device:

```rust
let device_id = generate_device_id();  // 24-character hex string
```

This prevents a user from sharing their unlock capability with others—each mobile device has a distinct identifier registered with the server.

**Verify Bytes (TOTP Hash):**

The verify_bytes field contains:

```rust
let verify_payload = device_id || totp_code;
let verify_bytes = HMAC-SHA256(server_public_key, verify_payload);
```

This binds the TOTP to the specific device, preventing TOTP reuse across devices.

## Phase 4: Beacon Verification

The mobile transmits the proof to the beacon, which performs several cryptographic verifications:

**Message Flow:**

```
Mobile → Beacon: UnlockRequest(proof)
Beacon: Verify ECDSA signature
Beacon: Check nonce validity
Beacon: Check rate limits
Beacon → Mobile: UnlockResponse(success, error_code)
```

**Signature Verification:**

```rust
let verifying_key = load_user_public_key(proof.device_bytes);
match verifying_key.verify(proof_payload, &signature) {
    Ok(_) => {},
    Err(_) => return Err(CryptoError::InvalidSignature),
}
```

The beacon retrieves the user's public key (either from local storage or by querying the server) and verifies the ECDSA signature. This computation takes ~10-20ms on the ESP32-C3.

**Nonce Validation:**

```rust
if !nonce_manager.check_and_mark_nonce(proof.nonce, proof.timestamp) {
    return Err(CryptoError::ReplayDetected);
}
```

The nonce manager checks:
1. **Nonce Exists**: The nonce was issued by this beacon
2. **Not Used**: The nonce hasn't been marked as used previously
3. **Not Expired**: `current_time - nonce_timestamp < 5 seconds`

If any check fails, the proof is rejected.

**Rate Limiting:**

```rust
if unlock_attempts >= 5 && current_time - first_attempt_time < 300 {
    return Err(CryptoError::RateLimited);
}
```

The beacon tracks failed unlock attempts per device. After 5 failures within 5 minutes, further attempts are rejected. This prevents brute-force attacks where an attacker tries many proofs rapidly.

**Acceptance Criteria:**

All of the following must be true for unlock to succeed:
- ✅ Signature verifies correctly
- ✅ Nonce is valid and not expired
- ✅ Nonce hasn't been used before
- ✅ Rate limit not exceeded
- ✅ Device is authorized (public key exists in beacon's database)

## Phase 5: Physical Unlock

Upon successful verification, the beacon activates its unlock mechanism.

**Relay Control:**

For electromagnetic locks:

```rust
relay_output.set_high();  // Energize relay
start_timer(5_seconds);
```

The relay remains energized for 5 seconds, allowing the user to open the door. After the timeout, the relay de-energizes, re-locking the door.

**Motion-Based Extension:**

If a PIR motion sensor detects the user passing through:

```rust
if motion_sensor.is_high() {
    extend_timeout(3_seconds);
}
```

This prevents the door from locking while the user is mid-passage.

**State Machine:**

The beacon maintains an unlock state machine:

```
LOCKED → UNLOCKING → UNLOCKED → RELOCKING → LOCKED
```

- **LOCKED**: Relay off, door locked
- **UNLOCKING**: Relay energizing (transition state)
- **UNLOCKED**: Relay on, door unlocked, timer active
- **RELOCKING**: Relay de-energizing (transition state)

The state machine ensures the relay never stays energized indefinitely, which would drain power and overheat the coil.

## Phase 6: Audit Logging

After the beacon responds, the mobile reports the unlock outcome to the server for audit purposes.

**Success Report:**

```
PUT /api/entities/{entity_id}/beacons/{beacon_id}/unlocker/{instance_id}/outcome
{
  "success": true,
  "timestamp": 1735689085,
  "location": { "latitude": 37.7749, "longitude": -122.4194 }
}
```

**Failure Report:**

```
PUT /api/entities/{entity_id}/beacons/{beacon_id}/unlocker/{instance_id}/outcome
{
  "success": false,
  "error_code": "REPLAY_DETECTED",
  "timestamp": 1735689085
}
```

**Audit Trail:**

The server stores all unlock attempts (success and failure) in the database:

```rust
struct UnlockAuditLog {
    instance_id: String,
    user_id: String,
    beacon_id: String,
    timestamp: u64,
    success: bool,
    error_code: Option<String>,
    location: Option<GeoPoint>,
    device_id: String,
}
```

This audit trail enables:
- **Security monitoring**: Detect unusual patterns (many failures, unusual times)
- **Compliance**: Prove who accessed what and when
- **Debugging**: Investigate user reports of unlock failures

## Attack Resistance Analysis

**Replay Attacks:**

An attacker captures a valid unlock proof and tries to reuse it.

*Defense*: Nonces are single-use. The beacon marks each nonce as used after successful unlock. Replayed proofs fail the `check_and_mark_nonce()` check.

**Relay Attacks:**

An attacker positions themselves between the mobile and beacon, relaying communication to extend physical distance.

*Defense*: The 5-second nonce expiration bounds the attack window. For a relay attack to succeed, the attacker must:
1. Relay NonceRequest from mobile to beacon
2. Relay NonceResponse from beacon to mobile
3. Wait for mobile to contact server and generate proof
4. Relay UnlockRequest from mobile to beacon

All within 5 seconds. This is impractical for most scenarios, especially considering server round-trip latency.

**Beacon Impersonation:**

An attacker deploys a rogue device pretending to be a legitimate beacon.

*Defense*: The beacon signs its nonce with a private key stored in hardware efuse. Rogue devices cannot obtain this key. While the mobile doesn't fully verify the signature (lacks beacon's public key), the server does—unauthorized beacons aren't in the database and have no associated TOTP secrets.

**User Impersonation:**

An attacker steals a user's device and attempts to unlock doors.

*Defense*: The mobile's private key is stored in Tauri Stronghold, which requires biometric authentication (Face ID, Touch ID, fingerprint) to access. Even if the device is unlocked, the attacker cannot retrieve the key without the user's biometric.

**Man-in-the-Middle:**

An attacker intercepts BLE communication between mobile and beacon.

*Defense*: While BLE communication isn't encrypted at the application layer (relies on BLE Security Manager pairing, which isn't currently enforced), the cryptographic protocol prevents MITM attacks:
- Nonce signing proves beacon authenticity
- Proof signing proves mobile authenticity
- TOTP binding prevents proof reuse across sessions

**Side-Channel Attacks:**

An attacker observes signature verification timing to extract key bits.

*Defense*: The `p256` crate uses constant-time implementations. Signature verification time is independent of secret key bits, preventing timing attacks.

## Performance Characteristics

**Latency Breakdown:**

- Phase 1 (Nonce): 50-100ms (BLE connection + message round-trip)
- Phase 2 (Server): 200-500ms (HTTPS request + database query + TOTP generation)
- Phase 3 (Proof): 10-20ms (Signature generation)
- Phase 4 (Verification): 50-100ms (BLE message + signature verification)
- Phase 5 (Physical): 100-500ms (Relay actuation)
- Phase 6 (Audit): 50-200ms (Background HTTPS request)

**Total**: 460-1420ms (typical: ~800ms)

From the user's perspective, tapping "Unlock" results in the door opening within 1 second—acceptable for most use cases.

**Battery Impact:**

- BLE connection: ~15mA for 200ms = 0.8mAh
- Signature generation: ~50mA for 20ms = 0.3mAh
- HTTPS requests (2x): ~100mA for 300ms = 8.3mAh
- Total: ~9.4mAh per unlock

For a 3000mAh smartphone battery, this represents 0.3% battery per unlock, or ~300 unlocks per full charge (assuming no other battery drain).

## Error Handling

The pipeline includes comprehensive error handling:

**Network Failures:**

If server is unreachable during Phase 2:
- Mobile displays "Offline - cannot verify authorization"
- Unlock fails gracefully
- Alternative: Pre-fetch TOTP codes during online periods (future enhancement)

**BLE Disconnection:**

If BLE connection drops during protocol:
- Mobile retries connection once
- If retry fails, display "Beacon unreachable"
- User can tap again to restart from Phase 1

**Cryptographic Failures:**

If signature verification fails:
- Beacon increments failure counter
- Returns specific error code (InvalidSignature, ReplayDetected, RateLimited)
- Mobile displays user-friendly error message

**Timeout Handling:**

If any phase exceeds timeout:
- Phase 1: 5 seconds → "Beacon not responding"
- Phase 2: 10 seconds → "Server timeout"
- Phase 4: 5 seconds → "Verification timeout"

## Future Enhancements

**Offline Unlock:**

Pre-fetch time-based unlock tokens during online periods, enabling unlock without server connectivity. Requires mobile clock synchronization and reduced security (no per-attempt TOTP).

**Biometric Integration:**

Require biometric authentication for every unlock (currently only required to access private key initially). Improves security for shared devices.

**NFC Fallback:**

Support NFC-based unlock for devices with NFC hardware. Faster than BLE (no connection establishment) but requires closer proximity.

**Multi-Factor Authentication:**

Require additional factor (PIN, voice recognition, location verification) for high-security doors.

**Delegation:**

Allow users to temporarily delegate unlock capability to others (visitors, delivery personnel) with time-bounded or usage-limited tokens.

## Related Documentation

- [Mobile Unlock Implementation](/components/mobile#access-control-system)
- [Beacon Cryptographic Protocol](/components/beacon#cryptographic-operations)
- [Server TOTP Generation](/components/server#access-control-unlocker)
