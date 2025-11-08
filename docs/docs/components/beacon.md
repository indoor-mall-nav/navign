# Beacon Firmware

The beacon firmware represents a sophisticated embedded system running on ESP32-C3 microcontrollers, serving dual purposes: indoor positioning infrastructure and access control endpoints. Written in Rust for the bare-metal esp-hal environment, it operates without an operating system, managing BLE communication, cryptographic operations, and hardware control within severe resource constraints.

**Platform:** ESP32-C3 (RISC-V 32-bit @ 160 MHz)
**Language:** Rust (no_std)
**Memory:** 400 KB SRAM, 4 MB Flash
**Radio:** 2.4 GHz WiFi + BLE 5.0 (shared radio)

## Hardware Platform

The ESP32-C3 was chosen for several technical and practical reasons. Unlike the ESP32 (Xtensa architecture), the ESP32-C3 uses a RISC-V core, which has better Rust toolchain support and clearer licensing. The chip integrates both WiFi and BLE radios sharing a single 2.4 GHz transceiver, enabling future WiFi-based OTA updates while maintaining BLE functionality for positioning and access control.

The hardware constraints significantly shape the firmware design:

**Flash Limitations:**
With only 4 MB of flash, code size becomes critical. The release binary must fit alongside the bootloader, partition table, and OTA update partitions. This necessitates aggressive optimization—the firmware is compiled with `opt-level = "s"` (optimize for size) rather than the default `"z"` or `"3"`. Without size optimization, the binary exceeds flash capacity.

**RAM Constraints:**
400 KB of SRAM must accommodate the stack, heap, BLE protocol stack, and application data structures. The firmware uses `heapless` data structures (fixed-capacity collections allocated on the stack) to avoid heap fragmentation and allocation failures. All buffers have compile-time known sizes.

**No Operating System:**
The firmware runs directly on the hardware without FreeRTOS initially (though esp-rtos support was added later for WiFi operations). This means no threading, no dynamic task scheduling, and no blocking I/O. The programming model is cooperative multitasking—the main loop must poll all subsystems regularly to maintain responsiveness.

## GPIO Configuration

The beacon interfaces with several hardware peripherals through GPIO pins. The pin assignments reflect typical deployment scenarios:

- **GPIO 4**: DHT11 temperature/humidity sensor (bidirectional data line)
- **GPIO 3**: Button input (for manual unlock or pairing)
- **GPIO 7**: Relay output (controls electromagnetic lock or gate mechanism)
- **GPIO 8**: LED indicator (visual status feedback)
- **GPIO 1**: PIR motion sensor (human presence detection)

These GPIO configurations support three unlock mechanisms:

**Relay Control:**
The most straightforward method. GPIO 7 drives a transistor that switches a relay coil. When energized, the relay contacts close, completing a circuit that releases an electromagnetic lock or activates a gate motor. The firmware implements pulse timing—the relay energizes for 5 seconds, then de-energizes to prevent overheating and reduce power consumption.

**Servo Motor Control (PWM):**
For applications requiring precise angular control (e.g., turnstile gates), the firmware supports PWM-based servo control using ESP32-C3's LEDC peripheral. The servo rotates to an unlock position, holds briefly, then returns to the locked position. This method provides finer control than simple relay switching.

**Infrared Transmission (RMT):**
The ESP32-C3's RMT (Remote Control Transceiver) peripheral can generate precisely timed pulse sequences for IR transmission. This enables the beacon to send infrared commands to control access systems that use IR receivers (common in commercial building automation). The firmware configures RMT with custom pulse timing to match various IR protocols.

## BLE Protocol Architecture

The beacon's BLE implementation operates in two modes: advertising (for positioning) and connected (for access control). This dual-mode operation maximizes utility while managing power consumption.

### Advertising Mode

In advertising mode, the beacon broadcasts BLE advertisement packets at regular intervals (typically every 100-1000ms). These packets contain:

1. **Device Name**: "NAVIGN-BEACON" (appears in device lists)
2. **Service UUIDs**: Advertised services indicate capability
3. **Manufacturer Data**: Custom payload (future: encrypted position hints)

The advertised UUIDs signal beacon capabilities to scanning devices:
- `0x183D`: Authorization Control Service (access control capability)
- `0x1819`: Location and Navigation Service
- `0x1821`: Indoor Positioning Service
- `0x181A`: Environmental Sensing Service (if DHT11 present)

Mobile devices performing passive scans receive these advertisements and extract RSSI values for trilateration without connecting. This minimizes beacon power consumption—advertising is significantly less expensive than maintaining connections.

### Connected Mode (GATT Protocol)

When a mobile device needs to identify a beacon or perform access control, it initiates a GATT connection. The beacon exposes a custom GATT service:

**Service UUID**: `134b1d88-cd91-8134-3e94-5c4052743845`
**Characteristic UUID**: `99d92823-9e38-72ff-6cf1-d2d593316af8`

This characteristic supports three operations:
- **Read**: Returns the last response message (retained for characteristic discovery)
- **Write**: Accepts request messages from the mobile
- **Notify**: Pushes response messages to subscribed clients

The write-notify pattern enables bidirectional communication. The mobile writes a request, the beacon processes it, and sends a response via notification. This approach is more efficient than polling reads.

### BLE Message Protocol

Messages use a compact binary protocol designed for BLE's 20-byte MTU limitations (though ESP32-C3 supports extended MTU up to 512 bytes). Each message starts with a single-byte identifier:

**Message Types:**
- `0x01`: DeviceRequest - Mobile queries beacon identity
- `0x02`: DeviceResponse - Beacon sends type, capabilities, database ID (27 bytes)
- `0x03`: NonceRequest - Mobile requests challenge nonce
- `0x04`: NonceResponse - Beacon sends 32-byte nonce + 8-byte signature fragment (41 bytes)
- `0x05`: UnlockRequest - Mobile sends proof signature
- `0x06`: UnlockResponse - Beacon sends success/failure + error code

The protocol handler maintains send and receive buffers sized at MAX_PACKET_SIZE (128 bytes). For messages exceeding BLE MTU, the handler implements fragmentation—splitting large messages across multiple BLE writes/notifications. The offset parameter tracks reassembly state.

**Buffer Management:**

The `BleProtocolHandler` uses `heapless::Vec` for fixed-capacity vectors. When serializing outbound messages, it constructs the binary representation in a stack-allocated buffer, then extracts chunks for BLE transmission. For inbound messages, it accumulates fragments until `expect_length()` confirms a complete message has arrived.

This design avoids dynamic allocation entirely. All buffers exist at compile time, preventing runtime allocation failures that would be catastrophic in an embedded system with no recovery mechanism.

## Cryptographic Operations

The beacon performs asymmetric cryptography operations despite the ESP32-C3's limited computational power. This is possible because P-256 ECDSA signature verification is optimized in the `p256` crate, leveraging constant-time implementations that prevent timing side-channel attacks.

### Key Storage

The beacon's private key resides in ESP32-C3 efuse `BLOCK_KEY0`. Efuses are one-time programmable memory with hardware read protection. Once programmed and locked, the key cannot be extracted through software or JTAG debugging. The bootloader and firmware can read the key, but external attackers cannot.

**Key Programming Procedure:**

The `maintenance-tool` utility programs keys during beacon initialization:
1. Generate 32-byte P-256 private key
2. Write key to efuse `BLOCK_KEY0`
3. Set read protection bits
4. Store corresponding public key in server database

Once programmed, the efuse is write-protected. If someone attempts to reprogram it, the efuse controller returns the existing value, making the operation idempotent but preventing key replacement.

### Nonce Generation and Management

When a mobile requests a nonce, the beacon generates a 32-byte random value using the ESP32-C3's hardware random number generator (TRNG). The TRNG derives entropy from radio noise and ADC fluctuations, providing cryptographically strong randomness.

The `NonceManager` tracks up to 32 active nonces with their generation timestamps. This enables several security checks:

**Replay Attack Prevention:**
When validating an unlock proof, the beacon checks if the nonce appears in the manager's history. If found, the proof is rejected—the nonce has already been used. This prevents attackers from capturing and replaying valid proofs.

**Expiration Enforcement:**
Nonces older than 5 seconds are considered expired. Even if a nonce hasn't been used, proofs referencing stale nonces are rejected. This bounds the attack window for relay attacks.

**Rate Limiting:**
The `BeaconState` tracks failed unlock attempts. After 5 failures within a 5-minute window, further unlock attempts are rate-limited. This prevents brute-force attacks where an attacker tries many proofs rapidly.

### Proof Validation

The proof validation sequence demonstrates the cryptographic protocol's security properties:

1. **Rate Limit Check**: Reject if too many recent failures
2. **Replay Check**: Verify nonce hasn't been used
3. **Expiration Check**: Verify nonce age < 5 seconds
4. **Signature Verification**: Validate ECDSA signature using mobile's public key
5. **Server Signature Check**: Verify TOTP was signed by server (future enhancement)

Signature verification uses the `p256` crate's `VerifyingKey::verify()` method, which performs point multiplication on the P-256 elliptic curve. This computation takes approximately 10-20ms on the ESP32-C3, during which the beacon cannot process other BLE events. The firmware design accounts for this by deferring signature verification until after sending acknowledgments.

## State Machine and Execution Model

The beacon operates as a cooperative multitasking system with a central state machine managing unlock sequences and hardware control.

### BeaconState Structure

The `BeaconState` encapsulates all mutable state:
- GPIO handles (button, sensor, relay, LED)
- Unlock attempt counter
- Nonce manager
- Proof verification manager
- BLE protocol handler
- Timing state (last_open, last_relay_on)

This structure is instantiated once in `main()` and wrapped in `Rc<RefCell<>>` to enable shared mutable access across the BLE event handler and main loop.

### Main Loop Execution

The main loop follows this pattern:

```rust
loop {
    executor.borrow_mut().check_executors(now());
    // BLE connection handling
    // GATT server processing
    // Delay to prevent busy-waiting
}
```

The `check_executors()` method implements the state machine logic for managing physical unlock:

**Open State Machine:**
1. **Initial State**: Relay off, door locked
2. **Unlock Triggered**: Set `open` flag high, energize relay, record timestamp
3. **Motion Detection**: If PIR sensor detects motion, extend timeout
4. **Timeout**: After 10 seconds (or 5 seconds after last motion), de-energize relay
5. **Return to Initial State**

This state machine ensures the door remains open only as long as necessary, preventing security vulnerabilities (door left open indefinitely) and hardware wear (relay coil overheating).

### BLE Event Handling

When a GATT write event occurs, the firmware deserializes the message and dispatches to the appropriate handler:

**DeviceRequest Handler:**
```rust
BleMessage::DeviceRequest => {
    let response = BleMessage::DeviceResponse(
        device_type,
        capabilities,
        object_id
    );
    Some(response)
}
```

The response includes the beacon's 24-character hex database ID, allowing the mobile to query the server for full beacon metadata.

**NonceRequest Handler:**
```rust
BleMessage::NonceRequest => {
    let nonce = executor.generate_nonce(&mut rng);
    let signature = sign_nonce(&private_key, &nonce);
    let signature_id = &signature[signature.len()-8..];
    Some(BleMessage::NonceResponse(nonce.as_bytes(), signature_id))
}
```

The signature fragment serves as proof of beacon authenticity without requiring the mobile to know the beacon's public key in advance.

**UnlockRequest Handler:**
```rust
BleMessage::UnlockRequest(proof) => {
    match executor.validate_proof(&proof, now()) {
        Ok(_) => {
            executor.set_open(true, now());
            Some(BleMessage::UnlockResponse(true, None))
        },
        Err(error) => {
            Some(BleMessage::UnlockResponse(false, Some(error)))
        }
    }
}
```

On successful validation, the state machine transitions to the open state, triggering relay activation.

## OTA Update System

The beacon supports over-the-air firmware updates using ESP-IDF's bootloader partition system. This enables remote firmware deployment without physical access to beacons.

### Partition Layout

The ESP32-C3 flash is partitioned into:
- `0x000000-0x010000`: Bootloader
- `0x010000-0x110000`: Factory partition (initial firmware)
- `0x110000-0x210000`: OTA_0 (first update slot)
- `0x210000-0x310000`: OTA_1 (second update slot)
- `0x310000-0x320000`: OTA Data (active partition marker)

### Dual-Bank Update Strategy

The dual-bank design provides safe updates:
1. Currently running firmware executes from one partition (e.g., Factory)
2. New firmware downloads to an inactive partition (e.g., OTA_0)
3. After complete download, OTA Data partition updates to mark OTA_0 as active
4. Device reboots, bootloader loads firmware from OTA_0
5. New firmware validates itself, marks partition as confirmed

If the new firmware fails to boot (crashes, hangs, etc.), the bootloader can automatically roll back to the previous partition. This requires bootloader configuration with rollback support enabled.

### Update Validation

The firmware marks itself as valid on successful boot:

```rust
if let Ok(state) = ota.current_ota_state() {
    if matches!(state, OtaImageState::New | OtaImageState::PendingVerify) {
        ota.set_current_ota_state(OtaImageState::Valid).ok();
    }
}
```

This prevents bootloader rollback after successful initialization, confirming the new firmware is stable.

### WiFi Integration (Future)

The current implementation includes OTA partition management but lacks WiFi download functionality. Future integration will:

1. Beacon connects to WiFi using BluFi provisioning (BLE-based WiFi credential transfer)
2. Queries orchestrator for latest firmware: `GET /firmwares/latest/esp32c3`
3. Downloads firmware binary over HTTP
4. Writes to inactive OTA partition
5. Verifies checksum
6. Marks partition active and reboots

This process occurs during maintenance windows (e.g., 3 AM) to avoid disrupting positioning services.

## Environmental Sensing

Beacons with DHT11 sensors collect temperature and humidity data. The DHT11 uses a single-wire protocol that requires precise timing:

1. Host pulls data line low for 18ms (start signal)
2. Host releases line, pulls high
3. DHT11 responds with 40 bits of data (16 bits humidity, 16 bits temperature, 8 bits checksum)
4. Each bit represented by pulse width: 26-28μs = 0, 70μs = 1

The `embedded-dht-rs` crate handles this timing-critical protocol using the `Delay` abstraction. Readings occur periodically (every 60 seconds) and are cached for BLE characteristic reads when mobile devices query environmental conditions.

## Power Management

While the current implementation doesn't implement aggressive power saving, the architecture supports future enhancements:

**Deep Sleep Mode:**
ESP32-C3 can enter deep sleep, consuming <5μA. A ULP (Ultra-Low Power) coprocessor can wake the main CPU on timer or GPIO events. Beacons could sleep between advertisement intervals, waking only to broadcast.

**Advertising Interval Tuning:**
Longer advertising intervals (500ms vs. 100ms) reduce power consumption proportionally but decrease positioning accuracy. Adaptive advertising could adjust intervals based on detected activity (shorten interval when motion detected).

**Connection Timeout:**
GATT connections consume power. The beacon could implement aggressive connection timeouts, disconnecting if no message received within 10 seconds.

## Debugging and Diagnostics

The firmware includes extensive `esp_println!()` debug output. On development hardware with USB-JTAG, these messages appear on the serial console, providing visibility into:

- BLE connection events
- Message parsing (with hex dumps)
- Cryptographic operations
- State machine transitions
- Error conditions

For production deployments, compile-time feature flags can disable debug output, reducing binary size and eliminating information leakage.

## Security Considerations

Several design decisions enhance security:

**No Remote Key Updates:**
The private key in efuse cannot be updated remotely. This prevents attackers from replacing keys even if they gain code execution.

**Constant-Time Crypto:**
The `p256` crate uses constant-time implementations to prevent timing side-channel attacks. Signature verification time doesn't leak information about key bits.

**Nonce Randomness:**
Hardware TRNG provides cryptographic randomness. Software PRNGs would be vulnerable if an attacker could predict the seed.

**Rate Limiting:**
Prevents brute-force attacks where an attacker tries many unlock proofs rapidly.

**Minimal Attack Surface:**
No remote shell, no filesystem, no network stack (initially). The only attack vector is BLE message parsing, which uses type-safe Rust with bounds checking.

## Related Documentation

- [BLE Message Protocol Specification](/components/beacon/ble-protocol)
- [Unlock Pipeline (End-to-End)](/pipelines/unlock)
- [OTA Update Integration Guide](/components/beacon/ota)
- [Hardware Setup and Key Programming](/components/beacon/setup)
