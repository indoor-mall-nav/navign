"""
Navign Maintenance Tool - ESP32-C3 eFuse key management and beacon registration CLI.

This tool provides commands for:
- Generating P-256 ECDSA key pairs
- Programming keys to ESP32-C3 eFuse
- Registering beacons with the orchestrator
- Flashing firmware to beacons
"""

import sys
from pathlib import Path
from typing import Optional

import click

from . import crypto, esp_tools, grpc_client, models


@click.group()
@click.version_option(version="0.1.0")
def main():
    """ESP32-C3 eFuse key management and beacon registration tool."""
    pass


@main.command("fuse-priv-key")
@click.option(
    "--output-dir",
    "-o",
    type=click.Path(path_type=Path),
    default=Path("./keys"),
    help="Output directory for key files",
)
@click.option(
    "--key-name", "-k", default="esp32c3_key", help="Key name prefix"
)
@click.option("--force", "-f", is_flag=True, help="Skip confirmation prompts")
@click.option(
    "--port", "-p", help="ESP32-C3 serial port (e.g., /dev/ttyUSB0 or COM3)"
)
@click.option(
    "--dry-run",
    is_flag=True,
    help="Generate and store key but don't fuse",
)
@click.option(
    "--register",
    is_flag=True,
    help="Register beacon with orchestrator after key generation",
)
@click.option(
    "--orchestrator-addr",
    default="localhost:50051",
    help="Orchestrator gRPC address",
)
@click.option("--entity-id", help="Entity ID for beacon registration")
@click.option(
    "--device-id", help="Device ID (24-char hex, auto-generated if not provided)"
)
@click.option(
    "--device-type",
    default="Pathway",
    type=click.Choice(["Merchant", "Pathway", "Connection", "Turnstile"]),
    help="Device type",
)
@click.option(
    "--firmware-version", default="0.1.0", help="Firmware version"
)
@click.option(
    "--hardware-revision", default="v1.0", help="Hardware revision"
)
@click.option("--area-id", help="Area ID where beacon is located")
def fuse_priv_key(
    output_dir: Path,
    key_name: str,
    force: bool,
    port: Optional[str],
    dry_run: bool,
    register: bool,
    orchestrator_addr: str,
    entity_id: Optional[str],
    device_id: Optional[str],
    device_type: str,
    firmware_version: str,
    hardware_revision: str,
    area_id: Optional[str],
):
    """Generate, store, and fuse private key to BLOCK_KEY0."""
    click.echo("🔑 ESP32-C3 eFuse Private Key Management")
    click.echo("=" * 42)

    # Step 1: Check if espefuse.py exists
    if not dry_run:
        click.echo("🔍 Checking for espefuse.py command...")
        if not esp_tools.check_espefuse_available():
            click.secho(
                "❌ espefuse.py not found!\n"
                "Please install ESP-IDF tools:\n"
                "  - Install ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/get-started/\n"
                "  - Or install esptool: pip install esptool",
                fg="red",
                err=True,
            )
            sys.exit(1)
        click.secho("✅ Found espefuse.py", fg="green")

    # Step 2: Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Step 3: Generate ECDSA private key
    click.echo("\n📋 Step 1: Generating ECDSA P-256 private key...")
    private_key_bytes, public_key_bytes, public_key_hex = (
        crypto.generate_p256_key_pair()
    )

    click.secho("✅ Private key generated successfully", fg="green")
    click.echo(f"   Public key: {public_key_hex}")

    # Step 4: Store key files
    click.echo("\n💾 Step 2: Storing key files...")
    private_key_path = output_dir / f"{key_name}_private.bin"
    metadata_path = output_dir / f"{key_name}_metadata.json"

    # Check if files already exist
    if not force and (private_key_path.exists() or metadata_path.exists()):
        click.secho(
            "❌ Key files already exist. Use --force to overwrite.",
            fg="red",
            err=True,
        )
        sys.exit(1)

    # Write private key
    crypto.save_private_key(private_key_bytes, private_key_path)

    # Create and save metadata
    metadata = models.KeyMetadata(
        key_name=key_name,
        private_key_file=private_key_path.name,
        public_key_hex=public_key_hex,
        generated_at=crypto.get_timestamp_rfc3339(),
        fused=False,
        chip_info=None,
    )
    metadata.save(metadata_path)

    click.secho("✅ Key files stored:", fg="green")
    click.echo(f"   Private key: {private_key_path}")
    click.echo(f"   Metadata: {metadata_path}")

    if dry_run:
        click.echo("\n🏃 Dry run mode - skipping eFuse programming")
        return

    # Step 5: Get chip info (optional)
    click.echo("\n🔍 Step 3: Getting chip information...")
    try:
        chip_info = esp_tools.get_chip_info(port)
        click.secho(f"✅ Chip info: {chip_info}", fg="green")
    except Exception as e:
        click.secho(f"⚠️  Could not get chip info: {e}", fg="yellow")
        chip_info = None

    # Step 6: Confirm before fusing
    if not force:
        click.echo("\n⚠️  WARNING: eFuse programming is IRREVERSIBLE!")
        click.echo("   This will permanently write the private key to BLOCK_KEY0")
        if not click.confirm("   Continue?", default=False):
            click.secho("❌ Operation cancelled", fg="red")
            sys.exit(1)

    # Step 7: Fuse the key
    click.echo("\n🔥 Step 4: Fusing private key to BLOCK_KEY0...")
    try:
        esp_tools.fuse_key_to_efuse(private_key_path, port, force=force)
    except Exception as e:
        click.secho(f"❌ Failed to fuse key: {e}", fg="red", err=True)
        sys.exit(1)

    # Step 8: Update metadata
    metadata.fused = True
    metadata.chip_info = chip_info
    metadata.save(metadata_path)

    click.secho("✅ Private key successfully fused to BLOCK_KEY0!", fg="green")
    click.echo("\n📄 Summary:")
    click.echo(f"   Key name: {key_name}")
    click.echo(f"   Private key file: {private_key_path}")
    click.echo(f"   Metadata file: {metadata_path}")
    click.echo("   eFuse status: PROGRAMMED")

    # Register beacon if requested
    if register:
        if not entity_id:
            click.secho(
                "❌ --entity-id is required when --register is specified",
                fg="red",
                err=True,
            )
            sys.exit(1)

        click.echo("\n📡 Step 5: Registering beacon with orchestrator...")

        # Generate device_id if not provided
        if not device_id:
            device_id = crypto.generate_device_id()

        # Convert public key to PEM
        public_key_pem = crypto.public_key_to_pem(public_key_bytes)

        # Register with orchestrator
        try:
            client = grpc_client.BeaconRegistrationClient(orchestrator_addr)
            client.connect()

            client.register_beacon(
                entity_id=entity_id,
                device_id=device_id,
                device_type=device_type,
                public_key_pem=public_key_pem,
                firmware_version=firmware_version,
                hardware_revision=hardware_revision,
                capabilities=["UnlockGate"],
                area_id=area_id,
            )

            client.close()

            click.secho(
                "✅ Beacon successfully registered with orchestrator!", fg="green"
            )
            click.echo(f"   Device ID: {device_id}")

        except Exception as e:
            click.secho(
                f"❌ Failed to register beacon: {e}", fg="red", err=True
            )
            sys.exit(1)


@main.command("register-beacon")
@click.option(
    "--metadata",
    "-m",
    required=True,
    type=click.Path(exists=True, path_type=Path),
    help="Path to key metadata JSON file",
)
@click.option(
    "--orchestrator-addr",
    default="localhost:50051",
    help="Orchestrator gRPC address",
)
@click.option(
    "--entity-id", "-e", required=True, help="Entity ID for beacon registration"
)
@click.option(
    "--device-id", help="Device ID (24-char hex, read from metadata if not provided)"
)
@click.option(
    "--device-type",
    default="Pathway",
    type=click.Choice(["Merchant", "Pathway", "Connection", "Turnstile"]),
    help="Device type",
)
@click.option(
    "--firmware-version", default="0.1.0", help="Firmware version"
)
@click.option(
    "--hardware-revision", default="v1.0", help="Hardware revision"
)
@click.option("--area-id", help="Area ID where beacon is located")
def register_beacon(
    metadata: Path,
    orchestrator_addr: str,
    entity_id: str,
    device_id: Optional[str],
    device_type: str,
    firmware_version: str,
    hardware_revision: str,
    area_id: Optional[str],
):
    """Register an existing beacon with the orchestrator."""
    click.echo("📡 Registering existing beacon with orchestrator...")

    # Load metadata
    try:
        key_metadata = models.KeyMetadata.from_file(metadata)
    except Exception as e:
        click.secho(f"❌ Failed to read metadata: {e}", fg="red", err=True)
        sys.exit(1)

    # Generate device_id if not provided
    if not device_id:
        device_id = crypto.generate_device_id()

    # Convert public key hex to PEM
    public_key_bytes = bytes.fromhex(key_metadata.public_key_hex)
    public_key_pem = crypto.public_key_to_pem(public_key_bytes)

    # Register with orchestrator
    try:
        client = grpc_client.BeaconRegistrationClient(orchestrator_addr)
        client.connect()

        client.register_beacon(
            entity_id=entity_id,
            device_id=device_id,
            device_type=device_type,
            public_key_pem=public_key_pem,
            firmware_version=firmware_version,
            hardware_revision=hardware_revision,
            capabilities=["UnlockGate"],
            area_id=area_id,
        )

        client.close()

        click.secho("✅ Beacon successfully registered!", fg="green")
        click.echo(f"   Entity ID: {entity_id}")
        click.echo(f"   Device ID: {device_id}")

    except Exception as e:
        click.secho(f"❌ Failed to register beacon: {e}", fg="red", err=True)
        sys.exit(1)


@main.command("flash-firmware")
@click.option(
    "--firmware",
    "-f",
    required=True,
    type=click.Path(exists=True, path_type=Path),
    help="Path to firmware binary file",
)
@click.option(
    "--port",
    "-p",
    required=True,
    help="ESP32-C3 serial port (e.g., /dev/ttyUSB0 or COM3)",
)
@click.option(
    "--baud", "-b", default=921600, help="Baud rate for flashing"
)
@click.option("--force", is_flag=True, help="Skip confirmation prompt")
@click.option("--erase", is_flag=True, help="Erase flash before flashing")
@click.option(
    "--verify/--no-verify",
    default=True,
    help="Verify flash after writing",
)
@click.option("--monitor", is_flag=True, help="Monitor serial output after flashing")
def flash_firmware(
    firmware: Path,
    port: str,
    baud: int,
    force: bool,
    erase: bool,
    verify: bool,
    monitor: bool,
):
    """Flash firmware to ESP32-C3 beacon."""
    click.echo("🔥 ESP32-C3 Firmware Flashing")
    click.echo("=" * 30)

    # Get firmware size
    firmware_size = firmware.stat().st_size

    click.echo("\n📋 Flash Configuration:")
    click.echo(f"   Firmware: {firmware}")
    click.echo(f"   Size: {firmware_size} bytes ({firmware_size / 1024:.2f} KB)")
    click.echo(f"   Port: {port}")
    click.echo(f"   Baud rate: {baud}")
    click.echo(f"   Erase flash: {'Yes' if erase else 'No'}")
    click.echo(f"   Verify: {'Yes' if verify else 'No'}")

    # Detect flash tool
    try:
        flash_tool = esp_tools.detect_flash_tool()
        click.echo(f"   Flash tool: {flash_tool}")
    except esp_tools.ESPToolNotFoundError as e:
        click.secho(f"❌ {e}", fg="red", err=True)
        sys.exit(1)

    # Confirmation prompt
    if not force:
        click.echo("\n⚠️  WARNING: This will flash firmware to the ESP32-C3 device.")
        if erase:
            click.echo("   All data on the flash will be erased!")
        if not click.confirm("   Continue?", default=False):
            click.secho("❌ Operation cancelled", fg="red")
            return

    # Flash the firmware
    click.echo("\n🔥 Flashing firmware...")

    try:
        if flash_tool == "espflash":
            esp_tools.flash_with_espflash(
                firmware, port, baud, erase, verify, monitor
            )
        elif flash_tool == "esptool.py":
            esp_tools.flash_with_esptool(
                firmware, port, baud, erase, verify, monitor
            )
        else:
            click.secho(f"❌ Unsupported flash tool: {flash_tool}", fg="red", err=True)
            sys.exit(1)

        click.secho("\n✅ Firmware flashed successfully!", fg="green")

        if monitor and flash_tool != "espflash":
            click.echo("\n📡 Starting serial monitor (press Ctrl+C to exit)...")
            esp_tools.monitor_serial(port, 115200)

    except Exception as e:
        click.secho(f"❌ Flashing failed: {e}", fg="red", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
