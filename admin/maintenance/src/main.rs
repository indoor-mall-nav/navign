use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use p256::elliptic_curve::rand_core::OsRng;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey, SecretKey};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// Include generated protobuf code
pub mod proto {
    pub mod navign {
        pub mod orchestrator {
            pub mod sync {
                tonic::include_proto!("navign.orchestrator.sync");
            }
        }
    }
}

use proto::navign::orchestrator::sync::{
    BeaconLocation, BeaconRegistrationRequest, Location,
    orchestrator_sync_client::OrchestratorSyncClient,
};

#[derive(Parser)]
#[command(name = "esp32c3-efuse")]
#[command(about = "ESP32-C3 eFuse key management tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate, store, and fuse private key to BLOCK_KEY0
    FusePrivKey {
        /// Output directory for key files
        #[arg(short, long, default_value = "./keys")]
        output_dir: PathBuf,

        /// Key name prefix
        #[arg(short, long, default_value = "esp32c3_key")]
        key_name: String,

        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,

        /// ESP32-C3 port (e.g., /dev/ttyUSB0 or COM3)
        #[arg(short, long)]
        port: Option<String>,

        /// Dry run - generate and store key but don't fuse
        #[arg(long)]
        dry_run: bool,

        /// Register beacon with orchestrator after key generation
        #[arg(long)]
        register: bool,

        /// Orchestrator address (e.g., http://localhost:50051)
        #[arg(long, default_value = "http://localhost:50051")]
        orchestrator_addr: String,

        /// Entity ID for beacon registration
        #[arg(long)]
        entity_id: Option<String>,

        /// Device ID (24-character hex string, auto-generated if not provided)
        #[arg(long)]
        device_id: Option<String>,

        /// Device type (Merchant, Pathway, Connection, Turnstile)
        #[arg(long, default_value = "Pathway")]
        device_type: String,

        /// Firmware version
        #[arg(long, default_value = "0.1.0")]
        firmware_version: String,

        /// Hardware revision
        #[arg(long, default_value = "v1.0")]
        hardware_revision: String,

        /// Area ID where beacon is located
        #[arg(long)]
        area_id: Option<String>,
    },

    /// Register an existing beacon with the orchestrator
    RegisterBeacon {
        /// Path to key metadata JSON file
        #[arg(short, long)]
        metadata: PathBuf,

        /// Orchestrator address (e.g., http://localhost:50051)
        #[arg(long, default_value = "http://localhost:50051")]
        orchestrator_addr: String,

        /// Entity ID for beacon registration
        #[arg(short, long)]
        entity_id: String,

        /// Device ID (24-character hex string, read from metadata if not provided)
        #[arg(long)]
        device_id: Option<String>,

        /// Device type (Merchant, Pathway, Connection, Turnstile)
        #[arg(long, default_value = "Pathway")]
        device_type: String,

        /// Firmware version
        #[arg(long, default_value = "0.1.0")]
        firmware_version: String,

        /// Hardware revision
        #[arg(long, default_value = "v1.0")]
        hardware_revision: String,

        /// Area ID where beacon is located
        #[arg(long)]
        area_id: Option<String>,
    },

    /// Flash firmware to ESP32-C3 beacon
    FlashFirmware {
        /// Path to firmware binary file
        #[arg(short, long)]
        firmware: PathBuf,

        /// ESP32-C3 port (e.g., /dev/ttyUSB0 or COM3)
        #[arg(short, long)]
        port: String,

        /// Baud rate for flashing (default: 921600)
        #[arg(short, long, default_value = "921600")]
        baud: u32,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,

        /// Erase flash before flashing
        #[arg(long)]
        erase: bool,

        /// Verify flash after writing
        #[arg(long, default_value = "true")]
        verify: bool,

        /// Monitor serial output after flashing
        #[arg(long)]
        monitor: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct KeyMetadata {
    key_name: String,
    private_key_file: String,
    public_key_hex: String,
    generated_at: String,
    fused: bool,
    chip_info: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::FusePrivKey {
            output_dir,
            key_name,
            force,
            port,
            dry_run,
            register,
            orchestrator_addr,
            entity_id,
            device_id,
            device_type,
            firmware_version,
            hardware_revision,
            area_id,
        } => {
            let (_metadata_path, public_key) =
                fuse_private_key(output_dir, key_name, *force, port.as_deref(), *dry_run)?;

            // Register beacon if requested
            if *register {
                if entity_id.is_none() {
                    bail!("--entity-id is required when --register is specified");
                }

                println!("\n📡 Step 5: Registering beacon with orchestrator...");

                // Generate device_id if not provided
                let device_id = device_id.clone().unwrap_or_else(|| {
                    let mut rng = rand::rng();
                    let bytes: [u8; 12] = rand::Rng::random(&mut rng);
                    hex::encode(bytes)
                });

                register_beacon_with_orchestrator(
                    orchestrator_addr,
                    entity_id.as_ref().unwrap(),
                    &device_id,
                    device_type,
                    &public_key,
                    firmware_version,
                    hardware_revision,
                    area_id.as_deref(),
                    vec!["UnlockGate".to_string()], // Default capability
                )
                .await?;

                println!("✅ Beacon successfully registered with orchestrator!");
                println!("   Device ID: {}", device_id);
            }
        }

        Commands::RegisterBeacon {
            metadata,
            orchestrator_addr,
            entity_id,
            device_id,
            device_type,
            firmware_version,
            hardware_revision,
            area_id,
        } => {
            println!("📡 Registering existing beacon with orchestrator...");

            // Read metadata file
            let metadata_content =
                std::fs::read_to_string(metadata).context("Failed to read metadata file")?;
            let key_metadata: KeyMetadata =
                serde_json::from_str(&metadata_content).context("Failed to parse metadata JSON")?;

            // Use device_id from args or generate new one
            let device_id = device_id.clone().unwrap_or_else(|| {
                let mut rng = rand::rng();
                let bytes: [u8; 12] = rand::Rng::random(&mut rng);
                hex::encode(bytes)
            });

            register_beacon_with_orchestrator(
                orchestrator_addr,
                entity_id,
                &device_id,
                device_type,
                &key_metadata.public_key_hex,
                firmware_version,
                hardware_revision,
                area_id.as_deref(),
                vec!["UnlockGate".to_string()], // Default capability
            )
            .await?;

            println!("✅ Beacon successfully registered!");
            println!("   Entity ID: {}", entity_id);
            println!("   Device ID: {}", device_id);
        }

        Commands::FlashFirmware {
            firmware,
            port,
            baud,
            force,
            erase,
            verify,
            monitor,
        } => {
            flash_firmware(firmware, port, *baud, *force, *erase, *verify, *monitor)?;
        }
    }

    Ok(())
}

fn fuse_private_key(
    output_dir: &Path,
    key_name: &str,
    force: bool,
    port: Option<&str>,
    dry_run: bool,
) -> Result<(PathBuf, String)> {
    println!("🔑 ESP32-C3 eFuse Private Key Management");
    println!("==========================================");

    // Step 1: Check if espefuse.py exists
    check_espefuse_command()?;

    // Step 2: Create output directory
    create_dir_all(output_dir).context("Failed to create output directory")?;

    // Step 3: Generate ECDSA private key
    println!("\n📋 Step 1: Generating ECDSA P-256 private key...");
    let private_key = SecretKey::random(&mut OsRng);
    let public_key = private_key.public_key();
    let private_key_bytes = private_key.to_bytes();

    println!("✅ Private key generated successfully");
    println!(
        "   Public key: {}",
        hex::encode(public_key.to_encoded_point(false).as_bytes())
    );

    // Step 4: Store key files
    println!("\n💾 Step 2: Storing key files...");
    let private_key_path = output_dir.join(format!("{}_private.bin", key_name));
    let metadata_path = output_dir.join(format!("{}_metadata.json", key_name));

    // Check if files already exist
    if !force && (private_key_path.exists() || metadata_path.exists()) {
        bail!("Key files already exist. Use --force to overwrite.");
    }

    // Write private key binary file
    let mut private_file =
        File::create(&private_key_path).context("Failed to create private key file")?;
    private_file
        .write_all(&private_key_bytes)
        .context("Failed to write private key")?;

    // Create metadata
    let metadata = KeyMetadata {
        key_name: key_name.to_string(),
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
    let metadata_json =
        serde_json::to_string_pretty(&metadata).context("Failed to serialize metadata")?;
    std::fs::write(&metadata_path, metadata_json).context("Failed to write metadata file")?;

    println!("✅ Key files stored:");
    println!("   Private key: {}", private_key_path.display());
    println!("   Metadata: {}", metadata_path.display());

    if dry_run {
        println!("\n🏃 Dry run mode - skipping eFuse programming");
        let public_key_hex = hex::encode(public_key.to_encoded_point(false).as_bytes());
        return Ok((metadata_path, public_key_hex));
    }

    // Step 5: Get chip info (optional)
    println!("\n🔍 Step 3: Getting chip information...");
    let chip_info = get_chip_info(port);
    match &chip_info {
        Ok(info) => println!("✅ Chip info: {}", info),
        Err(e) => println!("⚠️  Could not get chip info: {}", e),
    }

    // Step 6: Confirm before fusing
    if !force {
        println!("\n⚠️  WARNING: eFuse programming is IRREVERSIBLE!");
        println!("   This will permanently write the private key to BLOCK_KEY0");
        print!("   Continue? (y/N): ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Operation cancelled");
            bail!("eFuse programming cancelled by user");
        }
    }

    // Step 7: Fuse the key
    println!("\n🔥 Step 4: Fusing private key to BLOCK_KEY0...");
    fuse_key_to_efuse(&private_key_path, port).context("Failed to fuse key to eFuse")?;

    // Step 8: Update metadata
    let mut updated_metadata = metadata;
    updated_metadata.fused = true;
    updated_metadata.chip_info = chip_info.ok();

    let updated_metadata_json = serde_json::to_string_pretty(&updated_metadata)
        .context("Failed to serialize updated metadata")?;
    std::fs::write(&metadata_path, updated_metadata_json)
        .context("Failed to update metadata file")?;

    println!("✅ Private key successfully fused to BLOCK_KEY0!");
    println!("\n📄 Summary:");
    println!("   Key name: {}", key_name);
    println!("   Private key file: {}", private_key_path.display());
    println!("   Metadata file: {}", metadata_path.display());
    println!("   eFuse status: PROGRAMMED");

    // Return metadata path and public key hex for optional registration
    let public_key_hex = hex::encode(public_key.to_encoded_point(false).as_bytes());
    Ok((metadata_path, public_key_hex))
}

fn check_espefuse_command() -> Result<()> {
    println!("🔍 Checking for espefuse.py command...");

    match which::which("espefuse.py") {
        Ok(path) => {
            println!("✅ Found espefuse.py at: {}", path.display());
            Ok(())
        }
        Err(_) => {
            // Try alternative locations
            let alternatives = ["esptool.py", "python", "python3"];

            for alt in &alternatives {
                if which::which(alt).is_ok() {
                    // Check if espefuse.py can be run via python
                    let output = Command::new(alt)
                        .args(["-m", "espefuse", "--help"])
                        .output();

                    if output.is_ok() {
                        println!("✅ Found espefuse via: {} -m espefuse", alt);
                        return Ok(());
                    }
                }
            }

            bail!(
                "❌ espefuse.py not found!\n\
                Please install ESP-IDF tools:\n\
                - Install ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/get-started/\n\
                - Or install esptool: pip install esptool"
            );
        }
    }
}

fn get_chip_info(port: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("espefuse.py");
    cmd.args(["--chip", "esp32c3"]);

    if let Some(p) = port {
        cmd.args(["--port", p]);
    }

    cmd.arg("summary");

    let output = cmd.output().context("Failed to execute espefuse.py")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Extract relevant chip info from output
        for line in stdout.lines() {
            if line.contains("Chip is") || line.contains("Features:") {
                return Ok(line.trim().to_string());
            }
        }
        Ok("ESP32-C3 detected".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("espefuse.py failed: {}", stderr);
    }
}

fn fuse_key_to_efuse(key_file: &Path, port: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("espefuse.py");
    cmd.args(["--chip", "esp32c3"]);

    if let Some(p) = port {
        cmd.args(["--port", p]);
    }

    cmd.args(["burn_key", "BLOCK_KEY0", key_file.to_str().unwrap(), "USER"]);

    println!(
        "   Executing: espefuse.py --chip esp32c3 {} burn_key BLOCK_KEY0 {} USER",
        port.map(|p| format!("--port {}", p)).unwrap_or_default(),
        key_file.display()
    );

    let output = cmd
        .output()
        .context("Failed to execute espefuse.py burn_key command")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("   eFuse programming output:");
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                println!("   {}", line);
            }
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to burn key to eFuse: {}", stderr);
    }
}

/// Register beacon with orchestrator via gRPC
#[allow(clippy::too_many_arguments)]
async fn register_beacon_with_orchestrator(
    orchestrator_addr: &str,
    entity_id: &str,
    device_id: &str,
    device_type: &str,
    public_key_hex: &str,
    firmware_version: &str,
    hardware_revision: &str,
    area_id: Option<&str>,
    capabilities: Vec<String>,
) -> Result<()> {
    // Convert hex public key to PEM format
    let public_key_bytes =
        hex::decode(public_key_hex).context("Failed to decode public key hex")?;

    let public_key =
        PublicKey::from_sec1_bytes(&public_key_bytes).context("Failed to parse public key")?;

    let public_key_pem = pem::encode(&pem::Pem::new(
        "PUBLIC KEY".to_string(),
        public_key.to_sec1_bytes().to_vec(),
    ));

    // Connect to orchestrator
    println!("   Connecting to orchestrator at {}...", orchestrator_addr);
    let mut client = OrchestratorSyncClient::connect(orchestrator_addr.to_string())
        .await
        .context("Failed to connect to orchestrator")?;

    println!("   Connected successfully!");

    // Prepare beacon registration request
    let request = tonic::Request::new(BeaconRegistrationRequest {
        entity_id: entity_id.to_string(),
        device_id: device_id.to_string(),
        device_type: device_type.to_string(),
        capabilities,
        public_key: public_key_pem,
        firmware_version: firmware_version.to_string(),
        hardware_revision: hardware_revision.to_string(),
        location: area_id.map(|aid| BeaconLocation {
            area_id: aid.to_string(),
            coordinates: Some(Location {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                floor: String::new(),
            }),
        }),
        registered_at: Some(Timestamp::from(std::time::SystemTime::now())),
    });

    println!("   Sending beacon registration request...");
    let response = client
        .register_beacon(request)
        .await
        .context("Failed to register beacon")?;

    let beacon_response = response.into_inner();

    if beacon_response.approved {
        println!("   ✅ Beacon approved by orchestrator");
        println!("      Beacon ID: {}", beacon_response.beacon_id);
        println!("      Entity ID: {}", beacon_response.entity_id);
        println!(
            "      Sync interval: {} seconds",
            beacon_response.sync_interval_seconds
        );

        if beacon_response.firmware_update_available {
            println!("      ⚠️  Firmware update available");
            if let Some(firmware) = beacon_response.latest_firmware {
                println!("         Latest version: {}", firmware.version);
            }
        }
    } else {
        bail!("Beacon registration was not approved by orchestrator");
    }

    Ok(())
}

/// Flash firmware to ESP32-C3 beacon
fn flash_firmware(
    firmware_path: &Path,
    port: &str,
    baud: u32,
    force: bool,
    erase: bool,
    verify: bool,
    monitor: bool,
) -> Result<()> {
    println!("🔥 ESP32-C3 Firmware Flashing");
    println!("==============================");

    // Verify firmware file exists
    if !firmware_path.exists() {
        bail!("Firmware file does not exist: {}", firmware_path.display());
    }

    let firmware_size = std::fs::metadata(firmware_path)
        .context("Failed to get firmware file metadata")?
        .len();

    println!("\n📋 Flash Configuration:");
    println!("   Firmware: {}", firmware_path.display());
    println!(
        "   Size: {} bytes ({:.2} KB)",
        firmware_size,
        firmware_size as f64 / 1024.0
    );
    println!("   Port: {}", port);
    println!("   Baud rate: {}", baud);
    println!("   Erase flash: {}", if erase { "Yes" } else { "No" });
    println!("   Verify: {}", if verify { "Yes" } else { "No" });

    // Check for espflash or esptool.py
    let flash_tool = detect_flash_tool()?;
    println!("   Flash tool: {}", flash_tool);

    // Confirmation prompt
    if !force {
        println!("\n⚠️  WARNING: This will flash firmware to the ESP32-C3 device.");
        if erase {
            println!("   All data on the flash will be erased!");
        }
        print!("   Continue? (y/N): ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Operation cancelled");
            return Ok(());
        }
    }

    // Flash the firmware
    println!("\n🔥 Flashing firmware...");

    match flash_tool.as_str() {
        "espflash" => flash_with_espflash(firmware_path, port, baud, erase, verify, monitor)?,
        "esptool.py" => flash_with_esptool(firmware_path, port, baud, erase, verify, monitor)?,
        _ => bail!("Unsupported flash tool: {}", flash_tool),
    }

    println!("\n✅ Firmware flashed successfully!");

    if monitor {
        println!("\n📡 Starting serial monitor (press Ctrl+C to exit)...");
        monitor_serial(port, 115200)?;
    }

    Ok(())
}

/// Detect available flash tool
fn detect_flash_tool() -> Result<String> {
    // Try espflash first (Rust-based, faster)
    if which::which("espflash").is_ok() {
        return Ok("espflash".to_string());
    }

    // Try esptool.py (Python-based, more widely available)
    if which::which("esptool.py").is_ok() {
        return Ok("esptool.py".to_string());
    }

    // Try python -m esptool
    let alternatives = ["python3", "python"];
    for alt in &alternatives {
        if which::which(alt).is_ok() {
            let output = Command::new(alt)
                .args(["-m", "esptool", "version"])
                .output();

            if output.is_ok() && output.unwrap().status.success() {
                return Ok("esptool.py".to_string());
            }
        }
    }

    bail!(
        "❌ No flash tool found!\n\
        Please install one of the following:\n\
        - espflash: cargo install espflash (recommended)\n\
        - esptool: pip install esptool"
    );
}

/// Flash firmware using espflash
fn flash_with_espflash(
    firmware_path: &Path,
    port: &str,
    baud: u32,
    erase: bool,
    verify: bool,
    monitor: bool,
) -> Result<()> {
    let mut cmd = Command::new("espflash");
    cmd.arg("flash");
    cmd.arg("--port").arg(port);
    cmd.arg("--baud").arg(baud.to_string());

    if erase {
        cmd.arg("--erase-parts").arg("all");
    }

    if !verify {
        cmd.arg("--no-verify");
    }

    if monitor {
        cmd.arg("--monitor");
    }

    cmd.arg(firmware_path);

    println!(
        "   Executing: espflash flash --port {} --baud {} {}",
        port,
        baud,
        firmware_path.display()
    );

    let output = cmd.output().context("Failed to execute espflash")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                println!("   {}", line);
            }
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("espflash failed: {}", stderr);
    }
}

/// Flash firmware using esptool.py
fn flash_with_esptool(
    firmware_path: &Path,
    port: &str,
    baud: u32,
    erase: bool,
    _verify: bool,  // esptool.py verifies by default
    _monitor: bool, // Will handle monitoring separately
) -> Result<()> {
    // Erase flash if requested
    if erase {
        println!("   Erasing flash...");
        let mut erase_cmd = Command::new("esptool.py");
        erase_cmd.args(["--chip", "esp32c3", "--port", port, "erase_flash"]);

        let output = erase_cmd
            .output()
            .context("Failed to execute esptool.py erase_flash")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to erase flash: {}", stderr);
        }
        println!("   ✅ Flash erased");
    }

    // Flash firmware
    let mut cmd = Command::new("esptool.py");
    cmd.args(["--chip", "esp32c3"]);
    cmd.args(["--port", port]);
    cmd.args(["--baud", &baud.to_string()]);
    cmd.args(["write_flash", "0x0", firmware_path.to_str().unwrap()]);

    println!(
        "   Executing: esptool.py --chip esp32c3 --port {} --baud {} write_flash 0x0 {}",
        port,
        baud,
        firmware_path.display()
    );

    let output = cmd
        .output()
        .context("Failed to execute esptool.py write_flash")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                println!("   {}", line);
            }
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("esptool.py failed: {}", stderr);
    }
}

/// Monitor serial output
fn monitor_serial(port: &str, baud: u32) -> Result<()> {
    // Try to use screen or minicom for monitoring
    if which::which("screen").is_ok() {
        let mut cmd = Command::new("screen");
        cmd.arg(port).arg(baud.to_string());
        cmd.status().context("Failed to execute screen")?;
        Ok(())
    } else if which::which("minicom").is_ok() {
        let mut cmd = Command::new("minicom");
        cmd.args(["-D", port, "-b", &baud.to_string()]);
        cmd.status().context("Failed to execute minicom")?;
        Ok(())
    } else {
        println!("⚠️  No serial monitor tool found (screen or minicom)");
        println!("   Install with: sudo apt-get install screen");
        Ok(())
    }
}
