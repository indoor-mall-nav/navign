use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use p256::SecretKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::FusePrivKey {
            output_dir,
            key_name,
            force,
            port,
            dry_run,
        } => {
            fuse_private_key(output_dir, key_name, *force, port.as_deref(), *dry_run)?;
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
) -> Result<()> {
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
        return Ok(());
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
            return Ok(());
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

    Ok(())
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
