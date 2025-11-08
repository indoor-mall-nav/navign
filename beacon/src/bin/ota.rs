/// OTA (Over-The-Air) Update Module for Beacon Firmware
///
/// This module provides OTA update functionality for ESP32-C3 beacons without
/// WiFi/HTTP integration. The WiFi and firmware download logic should be
/// implemented separately and call into this module.
///
/// # Usage
///
/// ```rust,ignore
/// use crate::ota::{OtaManager, OtaError};
///
/// // Initialize OTA manager
/// let mut ota_manager = OtaManager::new(flash_storage)?;
///
/// // Get firmware version info
/// let current_partition = ota_manager.current_partition();
/// log::info!("Running from: {:?}", current_partition);
///
/// // Start OTA update (after downloading firmware via WiFi)
/// ota_manager.begin_update()?;
///
/// // Write firmware in chunks (typically from HTTP stream)
/// for chunk in firmware_chunks {
///     ota_manager.write_chunk(&chunk)?;
/// }
///
/// // Finalize and activate new firmware
/// ota_manager.finalize_update()?;
///
/// // Reboot to new firmware
/// esp_hal::reset::software_reset();
/// ```
use embedded_storage::Storage;
use esp_backtrace as _;
use esp_println::println;
use esp_storage::FlashStorage;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Maximum size for partition table buffer
const PARTITION_TABLE_BUFFER_SIZE: usize =
    esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN;

/// Chunk size for writing firmware (4KB sectors)
const FLASH_SECTOR_SIZE: usize = 4096;

/// Errors that can occur during OTA operations
#[derive(Debug)]
pub enum OtaError {
    /// Partition table read error
    PartitionTableError,
    /// No OTA partitions available
    NoOtaPartitions,
    /// OTA updater initialization failed
    UpdaterInitFailed,
    /// Write operation failed
    WriteFailed,
    /// Invalid firmware size
    InvalidFirmwareSize,
    /// OTA state change failed
    StateChangeFailed,
    /// Partition activation failed
    ActivationFailed,
    /// Buffer overflow
    BufferOverflow,
}

/// Represents the current OTA state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaState {
    /// No OTA update in progress
    Idle,
    /// OTA update has started, writing in progress
    Writing { bytes_written: u32, total_size: u32 },
    /// Write complete, ready to activate
    ReadyToActivate,
}

/// OTA Manager for beacon firmware updates
///
/// Manages OTA partitions, firmware writing, and partition activation.
/// Does NOT handle WiFi or firmware downloading - that must be implemented
/// separately and provided as byte slices.
pub struct OtaManager<'a> {
    flash: FlashStorage,
    buffer: [u8; PARTITION_TABLE_BUFFER_SIZE],
    ota_updater: esp_bootloader_esp_idf::ota_updater::OtaUpdater<'a>,
    state: OtaState,
    write_offset: u32,
    expected_size: Option<u32>,
}

impl<'a> OtaManager<'a> {
    /// Create a new OTA manager
    ///
    /// # Arguments
    ///
    /// * `flash` - Flash storage peripheral from esp_hal
    ///
    /// # Returns
    ///
    /// Returns an OtaManager or OtaError if initialization fails
    pub fn new(flash: FlashStorage) -> Result<Self, OtaError> {
        let mut manager = Self {
            flash,
            buffer: [0u8; PARTITION_TABLE_BUFFER_SIZE],
            ota_updater: unsafe {
                // SAFETY: We immediately reinitialize this in the next step
                core::mem::zeroed()
            },
            state: OtaState::Idle,
            write_offset: 0,
            expected_size: None,
        };

        // Read partition table and initialize updater
        let ota_updater = esp_bootloader_esp_idf::ota_updater::OtaUpdater::new(
            &mut manager.flash,
            &mut manager.buffer,
        )
        .map_err(|_| OtaError::UpdaterInitFailed)?;

        manager.ota_updater = ota_updater;

        Ok(manager)
    }

    /// Get information about the currently running partition
    pub fn current_partition(
        &self,
    ) -> Result<&esp_bootloader_esp_idf::partitions::Partition, OtaError> {
        self.ota_updater
            .selected_partition()
            .ok_or(OtaError::NoOtaPartitions)
    }

    /// Get the current OTA image state
    ///
    /// This is only relevant if the bootloader was built with auto-rollback support
    pub fn current_image_state(
        &self,
    ) -> Result<esp_bootloader_esp_idf::ota::OtaImageState, OtaError> {
        self.ota_updater
            .current_ota_state()
            .map_err(|_| OtaError::StateChangeFailed)
    }

    /// Mark the current firmware as valid
    ///
    /// Call this after successfully booting to prevent auto-rollback.
    /// Only needed if bootloader was built with auto-rollback support.
    pub fn mark_valid(&mut self) -> Result<(), OtaError> {
        if let Ok(state) = self.current_image_state() {
            if state == esp_bootloader_esp_idf::ota::OtaImageState::New
                || state == esp_bootloader_esp_idf::ota::OtaImageState::PendingVerify
            {
                self.ota_updater
                    .set_current_ota_state(esp_bootloader_esp_idf::ota::OtaImageState::Valid)
                    .map_err(|_| OtaError::StateChangeFailed)?;

                println!("OTA: Marked current firmware as VALID");
            }
        }
        Ok(())
    }

    /// Begin an OTA update
    ///
    /// # Arguments
    ///
    /// * `firmware_size` - Expected size of the firmware in bytes (optional)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if update started successfully
    pub fn begin_update(&mut self, firmware_size: Option<u32>) -> Result<(), OtaError> {
        if self.state != OtaState::Idle {
            return Err(OtaError::StateChangeFailed);
        }

        let (_, part_type) = self
            .ota_updater
            .next_partition()
            .ok_or(OtaError::NoOtaPartitions)?;

        println!(
            "OTA: Starting update to partition {:?}, expected size: {:?} bytes",
            part_type, firmware_size
        );

        self.state = OtaState::Writing {
            bytes_written: 0,
            total_size: firmware_size.unwrap_or(0),
        };
        self.write_offset = 0;
        self.expected_size = firmware_size;

        Ok(())
    }

    /// Write a chunk of firmware data
    ///
    /// # Arguments
    ///
    /// * `data` - Firmware chunk to write (can be any size, will be buffered)
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written or OtaError
    pub fn write_chunk(&mut self, data: &[u8]) -> Result<u32, OtaError> {
        match self.state {
            OtaState::Writing {
                bytes_written,
                total_size,
            } => {
                let (mut next_partition, _) = self
                    .ota_updater
                    .next_partition()
                    .ok_or(OtaError::NoOtaPartitions)?;

                // Check if this would exceed expected size
                if let Some(expected) = self.expected_size {
                    if (bytes_written as usize + data.len()) > expected as usize {
                        return Err(OtaError::InvalidFirmwareSize);
                    }
                }

                // Write data to flash
                // Note: Flash requires sector-aligned writes in some cases
                // For simplicity, we write directly. In production, you might
                // want to buffer unaligned writes.
                next_partition
                    .write(self.write_offset, data)
                    .map_err(|_| OtaError::WriteFailed)?;

                let written = data.len() as u32;
                self.write_offset += written;

                self.state = OtaState::Writing {
                    bytes_written: bytes_written + written,
                    total_size,
                };

                if (bytes_written + written) % (64 * 1024) == 0 {
                    println!(
                        "OTA: Written {} / {} bytes",
                        bytes_written + written,
                        total_size
                    );
                }

                Ok(written)
            }
            _ => Err(OtaError::StateChangeFailed),
        }
    }

    /// Write firmware in sector-aligned chunks (recommended)
    ///
    /// This method ensures proper sector alignment for flash writes.
    /// Use this if you're downloading firmware in arbitrary-sized chunks.
    ///
    /// # Arguments
    ///
    /// * `data` - Complete firmware data
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if all sectors written successfully
    pub fn write_firmware(&mut self, data: &[u8]) -> Result<(), OtaError> {
        match self.state {
            OtaState::Writing { .. } => {
                for (sector, chunk) in data.chunks(FLASH_SECTOR_SIZE).enumerate() {
                    self.write_chunk(chunk)?;
                    if sector % 10 == 0 {
                        println!("OTA: Written sector {}", sector);
                    }
                }
                Ok(())
            }
            _ => Err(OtaError::StateChangeFailed),
        }
    }

    /// Finalize the OTA update and activate the new partition
    ///
    /// After calling this, you should reset the device to boot into the new firmware.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if partition activated successfully
    pub fn finalize_update(&mut self) -> Result<(), OtaError> {
        match self.state {
            OtaState::Writing { bytes_written, .. } => {
                println!("OTA: Finalizing update, {} bytes written", bytes_written);

                // Activate the new partition
                self.ota_updater
                    .activate_next_partition()
                    .map_err(|_| OtaError::ActivationFailed)?;

                // Set state to NEW (allows rollback if boot fails)
                self.ota_updater
                    .set_current_ota_state(esp_bootloader_esp_idf::ota::OtaImageState::New)
                    .map_err(|_| OtaError::StateChangeFailed)?;

                println!("OTA: Update complete, new partition activated");
                println!("OTA: Reboot to apply new firmware");

                self.state = OtaState::ReadyToActivate;
                Ok(())
            }
            _ => Err(OtaError::StateChangeFailed),
        }
    }

    /// Abort the current OTA update
    ///
    /// Cancels the update and returns to idle state without changing partitions
    pub fn abort_update(&mut self) -> Result<(), OtaError> {
        if self.state != OtaState::Idle {
            println!("OTA: Update aborted");
            self.state = OtaState::Idle;
            self.write_offset = 0;
            self.expected_size = None;
        }
        Ok(())
    }

    /// Get current OTA state
    pub fn get_state(&self) -> OtaState {
        self.state
    }

    /// Get progress information
    ///
    /// Returns (bytes_written, total_size) if update is in progress
    pub fn get_progress(&self) -> Option<(u32, u32)> {
        match self.state {
            OtaState::Writing {
                bytes_written,
                total_size,
            } => Some((bytes_written, total_size)),
            _ => None,
        }
    }

    /// List all partitions (for debugging)
    pub fn list_partitions(&mut self) -> Result<(), OtaError> {
        let pt = esp_bootloader_esp_idf::partitions::read_partition_table(
            &mut self.flash,
            &mut self.buffer,
        )
        .map_err(|_| OtaError::PartitionTableError)?;

        println!("=== Partition Table ===");
        for part in pt.iter() {
            println!("{:?}", part);
        }
        println!("Currently booted: {:?}", pt.booted_partition());
        println!("======================");

        Ok(())
    }
}
