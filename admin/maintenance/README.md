# Navign Maintenance Tool (Python)

ESP32-C3 eFuse key management and beacon registration CLI tool.

## Features

- **Key Generation**: Generate P-256 ECDSA private/public key pairs
- **eFuse Programming**: Securely burn private keys to ESP32-C3 BLOCK_KEY0
- **Beacon Registration**: Register beacons with the orchestrator via gRPC
- **Firmware Flashing**: Flash firmware to ESP32-C3 beacons

## Prerequisites

- Python 3.12+
- ESP-IDF tools (for eFuse programming):
  - `espefuse.py` (included with ESP-IDF or esptool)
  - Install: `pip install esptool`
- Protocol Buffers compiler (for development):
  - Debian/Ubuntu: `apt-get install protobuf-compiler`
  - macOS: `brew install protobuf`

## Installation

```bash
cd admin/maintenance

# Sync dependencies with uv
uv sync

# Or install with pip (if uv is not available)
pip install -e .
```

## Generate Protobuf Code

Before using the tool, generate the gRPC code:

```bash
./generate_proto.sh
```

## Usage

### Flash Firmware to Beacon

Flash firmware to an ESP32-C3 beacon:

```bash
uv run navign-maintenance flash-firmware \
  --firmware ../firmware/target/riscv32imc-esp-espidf/release/navign-firmware \
  --port /dev/ttyUSB0
```

Options:
- `--firmware, -f`: Path to firmware binary file (required)
- `--port, -p`: ESP32-C3 serial port (required, e.g., `/dev/ttyUSB0`, `COM3`)
- `--baud, -b`: Baud rate for flashing (default: `921600`)
- `--force`: Skip confirmation prompt
- `--erase`: Erase flash before flashing (recommended for initial flash)
- `--verify/--no-verify`: Verify flash after writing (default: `--verify`)
- `--monitor`: Monitor serial output after flashing

Example with full options:
```bash
uv run navign-maintenance flash-firmware \
  --firmware path/to/firmware.bin \
  --port /dev/ttyUSB0 \
  --baud 921600 \
  --erase \
  --monitor
```

**Flash tools:**
The maintenance tool automatically detects and uses:
- `espflash` (Rust-based, recommended): `cargo install espflash`
- `esptool.py` (Python-based): `pip install esptool`

### Generate and Fuse Private Key

Generate a new P-256 private key and fuse it to ESP32-C3 eFuse:

```bash
uv run navign-maintenance fuse-priv-key \
  --output-dir ./keys \
  --key-name beacon_001 \
  --port /dev/ttyUSB0
```

Options:
- `--output-dir, -o`: Directory to store key files (default: `./keys`)
- `--key-name, -k`: Key name prefix (default: `esp32c3_key`)
- `--port, -p`: ESP32-C3 serial port (e.g., `/dev/ttyUSB0`, `COM3`)
- `--force, -f`: Skip confirmation prompts
- `--dry-run`: Generate and store key without fusing to eFuse

### Generate Key and Register Beacon

Generate a key and immediately register the beacon with the orchestrator:

```bash
uv run navign-maintenance fuse-priv-key \
  --output-dir ./keys \
  --key-name beacon_001 \
  --port /dev/ttyUSB0 \
  --register \
  --orchestrator-addr localhost:50051 \
  --entity-id mall-123 \
  --device-type Pathway \
  --area-id entrance-area
```

Additional registration options:
- `--register`: Enable beacon registration after key generation
- `--orchestrator-addr`: Orchestrator gRPC address (default: `localhost:50051`)
- `--entity-id`: Entity/mall identifier (required when `--register` is used)
- `--device-id`: Device ID (24-char hex, auto-generated if not provided)
- `--device-type`: Device type: `Merchant`, `Pathway`, `Connection`, `Turnstile` (default: `Pathway`)
- `--firmware-version`: Firmware version (default: `0.1.0`)
- `--hardware-revision`: Hardware revision (default: `v1.0`)
- `--area-id`: Area ID where beacon is located (optional)

### Register Existing Beacon

Register a previously generated beacon with the orchestrator:

```bash
uv run navign-maintenance register-beacon \
  --metadata ./keys/beacon_001_metadata.json \
  --orchestrator-addr localhost:50051 \
  --entity-id mall-123 \
  --device-type Pathway \
  --area-id entrance-area
```

Options:
- `--metadata, -m`: Path to key metadata JSON file (required)
- `--orchestrator-addr`: Orchestrator gRPC address
- `--entity-id, -e`: Entity/mall identifier (required)
- `--device-id`: Override device ID from metadata
- `--device-type`: Device type
- `--firmware-version`: Firmware version
- `--hardware-revision`: Hardware revision
- `--area-id`: Area ID where beacon is located

## Output Files

The tool generates two files in the output directory:

1. **Private Key Binary**: `{key_name}_private.bin`
   - Raw 32-byte P-256 private key
   - Use with `espefuse.py burn_key`

2. **Metadata JSON**: `{key_name}_metadata.json`
   - Contains:
     - Key name
     - Private key filename
     - Public key (hex-encoded)
     - Generation timestamp
     - Fusing status
     - Chip information (if available)

Example metadata:
```json
{
  "key_name": "beacon_001",
  "private_key_file": "beacon_001_private.bin",
  "public_key_hex": "04a1b2c3...",
  "generated_at": "2025-01-15T10:30:00Z",
  "fused": true,
  "chip_info": "ESP32-C3 (revision 3)"
}
```

## Security Considerations

- **Irreversible**: eFuse programming is permanent and cannot be undone
- **Read Protection**: BLOCK_KEY0 is read-protected after programming
- **Key Storage**: Private key files should be stored securely and backed up
- **Write Protection**: Attempting to reprogram returns existing value

## Integration with Orchestrator

When the `--register` flag is used or when calling `register-beacon`, the tool:

1. Converts the public key to PEM format
2. Connects to the orchestrator via gRPC
3. Sends a `BeaconRegistrationRequest` with:
   - Entity ID
   - Device ID
   - Device type and capabilities
   - Public key (PEM)
   - Firmware and hardware versions
   - Location information (if provided)
4. Receives registration response with:
   - Beacon ID (assigned by orchestrator)
   - Sync interval
   - Firmware update availability

The orchestrator then syncs the beacon information with the central server.

## Development

Install development dependencies:
```bash
uv sync --extra dev
```

Run tests:
```bash
uv run pytest
```

Run tests with coverage:
```bash
uv run pytest --cov=navign_maintenance --cov-report=html
```

Format and lint:
```bash
uvx ruff format
uvx ruff check
```

Generate protobuf code:
```bash
./generate_proto.sh
```

## License

MIT
