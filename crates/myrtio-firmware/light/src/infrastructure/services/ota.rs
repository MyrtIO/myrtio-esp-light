//! OTA Update Service
//!
//! This module provides functionality for performing over-the-air firmware updates
//! using the ESP-IDF bootloader OTA mechanism. It uses a shared flash mutex to
//! safely coordinate access with other flash operations (like persistent storage).

use embedded_storage::Storage;
use esp_bootloader_esp_idf::ota::{Ota, OtaImageState, Slot};
use esp_bootloader_esp_idf::partitions::{
    AppPartitionSubType, DataPartitionSubType, PARTITION_TABLE_MAX_LEN, PartitionType,
    read_partition_table,
};
use esp_println::println;
use heapless::String;

use crate::infrastructure::drivers::FlashStorageMutex;

/// Maximum host string length for OTA invite
const MAX_HOST_LEN: usize = 64;
/// Maximum path string length for OTA invite
const MAX_PATH_LEN: usize = 128;

/// OTA-specific errors
#[derive(Debug)]
pub(crate) enum OtaError {
    /// Failed to read or parse partition table
    PartitionTable,
    /// No OTA data partition found
    NoOtaDataPartition,
    /// No next OTA app partition available
    NoNextPartition,
    /// Invalid OTA state
    InvalidState,
    /// Failed to write firmware data
    WriteError,
    /// Failed to activate the new slot
    ActivationError,
    /// Failed to set OTA state
    StateError,
    /// No active session
    #[allow(dead_code)]
    NoSession,
}

/// Parsed OTA invite containing update parameters
///
/// Expected format (key=value lines):
/// ```text
/// HOST=192.168.1.100
/// PORT=8000
/// PATH=/firmware.bin
/// SIZE=552672
/// MD5=abc123...
/// ```
#[derive(Debug, Clone)]
pub(crate) struct OtaInvite {
    /// HTTP server host (IP or domain)
    pub host: String<MAX_HOST_LEN>,
    /// HTTP server port
    pub port: u16,
    /// Path to firmware file
    pub path: String<MAX_PATH_LEN>,
    /// Expected firmware size in bytes
    pub size: u32,
}

impl OtaInvite {
    /// Parse an invite from a text buffer
    ///
    /// Returns `None` if required fields (HOST, PORT, PATH, SIZE) are missing
    /// or malformed. MD5 is currently parsed but not stored.
    pub(crate) fn parse(data: &[u8]) -> Option<Self> {
        let text = core::str::from_utf8(data).ok()?;

        let mut host: Option<String<MAX_HOST_LEN>> = None;
        let mut port: Option<u16> = None;
        let mut path: Option<String<MAX_PATH_LEN>> = None;
        let mut size: Option<u32> = None;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "HOST" => {
                        let mut s = String::new();
                        if s.push_str(value).is_ok() {
                            host = Some(s);
                        }
                    }
                    "PORT" => {
                        port = value.parse().ok();
                    }
                    "PATH" => {
                        let mut s = String::new();
                        if s.push_str(value).is_ok() {
                            path = Some(s);
                        }
                    }
                    "SIZE" => {
                        size = value.parse().ok();
                    }
                    // MD5 is parsed but not stored for now
                    _ => {}
                }
            }
        }

        Some(Self {
            host: host?,
            port: port?,
            path: path?,
            size: size?,
        })
    }
}

/// OTA Service
///
/// Provides firmware update functionality using a shared flash mutex.
/// This service handles boot state verification and creates update sessions.
pub(crate) struct OtaService {
    flash: &'static FlashStorageMutex,
}

impl OtaService {
    /// Create a new OTA service with the shared flash mutex
    pub(crate) fn new(flash: &'static FlashStorageMutex) -> Self {
        Self { flash }
    }

    /// Handle OTA boot state on startup.
    ///
    /// This function should be called early during boot to mark the current
    /// firmware image as valid if it was just updated. This prevents the bootloader
    /// from rolling back to the previous image on the next reboot.
    pub(crate) fn handle_boot_state(&self) {
        self.flash.lock(|cell| {
            let mut flash = cell.borrow_mut();
            let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];

            // Read partition table
            let Ok(pt) = read_partition_table(&mut *flash, &mut buffer) else {
                println!("ota: failed to read partition table");
                return;
            };

            // Find OTA data partition
            let Ok(Some(ota_data_part)) =
                pt.find_partition(PartitionType::Data(DataPartitionSubType::Ota))
            else {
                println!("ota: no OTA data partition found (OTA not configured)");
                return;
            };

            // Create flash region for OTA data partition
            let mut ota_region = ota_data_part.as_embedded_storage(&mut *flash);

            let Ok(mut ota) = Ota::new(&mut ota_region) else {
                println!("ota: failed to initialize Ota for boot state check");
                return;
            };

            // Check current slot and state
            let Ok(current_slot) = ota.current_slot() else {
                println!("ota: failed to get current slot");
                return;
            };

            if current_slot == Slot::None {
                println!("ota: current image state: Undefined");
                return;
            }

            // Check current OTA state and mark as valid if needed
            if let Ok(state) = ota.current_ota_state() {
                match state {
                    OtaImageState::New | OtaImageState::PendingVerify => {
                        if ota.set_current_ota_state(OtaImageState::Valid).is_ok() {
                            println!("ota: marked current image as VALID");
                        } else {
                            println!("ota: failed to mark current image as VALID");
                        }
                    }
                    OtaImageState::Valid => {
                        println!("ota: current image already VALID");
                    }
                    _ => {
                        println!("ota: current image state: {:?}", state);
                    }
                }
            }
        });
    }

    /// Begin a new OTA update session
    ///
    /// This determines the target OTA slot and prepares for writing firmware data.
    pub(crate) fn begin_update(&self, total_size: u32) -> Result<OtaSession, OtaError> {
        // Determine target slot and partition info
        let (target_slot, target_partition_offset, target_partition_size) =
            self.flash.lock(|cell| {
                let mut flash = cell.borrow_mut();
                let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];

                // First, determine the current slot and calculate the target slot
                let pt = read_partition_table(&mut *flash, &mut buffer)
                    .map_err(|_| OtaError::PartitionTable)?;

                let ota_data_part = pt
                    .find_partition(PartitionType::Data(DataPartitionSubType::Ota))
                    .map_err(|_| OtaError::PartitionTable)?
                    .ok_or(OtaError::NoOtaDataPartition)?;

                let mut ota_region = ota_data_part.as_embedded_storage(&mut *flash);
                let mut ota = Ota::new(&mut ota_region).map_err(|_| OtaError::InvalidState)?;

                let current_slot = ota.current_slot().map_err(|_| OtaError::InvalidState)?;
                let target_slot = current_slot.next();

                // Now find the target OTA app partition
                // Re-read partition table since we dropped the previous references
                let pt = read_partition_table(&mut *flash, &mut buffer)
                    .map_err(|_| OtaError::PartitionTable)?;

                let target_partition_type = match target_slot {
                    Slot::None | Slot::Slot0 => PartitionType::App(AppPartitionSubType::Ota0),
                    Slot::Slot1 => PartitionType::App(AppPartitionSubType::Ota1),
                };

                let target_partition = pt
                    .find_partition(target_partition_type)
                    .map_err(|_| OtaError::PartitionTable)?
                    .ok_or(OtaError::NoNextPartition)?;

                Ok((
                    target_slot,
                    target_partition.offset(),
                    target_partition.len(),
                ))
            })?;

        println!(
            "ota: beginning update to slot {:?}, partition at offset 0x{:X}, size {} bytes, image size {} bytes",
            target_slot, target_partition_offset, target_partition_size, total_size
        );

        if total_size > target_partition_size {
            println!(
                "ota: warning - image size {} exceeds partition size {}",
                total_size, target_partition_size
            );
        }

        Ok(OtaSession {
            flash: self.flash,
            target_partition_offset,
            target_slot,
            bytes_written: 0,
            total_size,
        })
    }
}

/// OTA Update Session
///
/// Manages the state of an ongoing OTA update operation.
/// This handles writing firmware data to the OTA app partition.
pub(crate) struct OtaSession {
    flash: &'static FlashStorageMutex,
    /// Offset of the target OTA app partition
    target_partition_offset: u32,
    /// Which slot we're writing to
    target_slot: Slot,
    /// Number of bytes written so far
    bytes_written: u32,
    /// Total expected size
    total_size: u32,
}

impl OtaSession {
    /// Write a chunk of firmware data
    pub(crate) fn write_chunk(&mut self, data: &[u8]) -> Result<(), OtaError> {
        if data.is_empty() {
            return Ok(());
        }

        let write_offset = self.target_partition_offset + self.bytes_written;

        self.flash.lock(|cell| {
            cell.borrow_mut()
                .write(write_offset, data)
                .map_err(|_| OtaError::WriteError)
        })?;

        #[allow(clippy::cast_possible_truncation)]
        {
            self.bytes_written += data.len() as u32;
        }

        Ok(())
    }

    /// Get the number of bytes written so far
    pub(crate) fn bytes_written(&self) -> u32 {
        self.bytes_written
    }

    /// Get the total expected size
    pub(crate) fn total_size(&self) -> u32 {
        self.total_size
    }

    /// Get the current progress percentage (0-100)
    pub(crate) fn progress_percent(&self) -> u8 {
        if self.total_size == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            ((u64::from(self.bytes_written) * 100) / u64::from(self.total_size)) as u8
        }
    }

    /// Finalize the OTA update
    ///
    /// This activates the new slot and sets the OTA state.
    /// After calling this, a reboot is required to boot into the new firmware.
    pub(crate) fn finalize(self) -> Result<(), OtaError> {
        if self.bytes_written != self.total_size {
            println!(
                "ota: warning - wrote {} bytes but expected {}",
                self.bytes_written, self.total_size
            );
        }

        println!(
            "ota: finalizing update, {} bytes written to slot {:?}",
            self.bytes_written, self.target_slot
        );

        self.flash.lock(|cell| {
            let mut flash = cell.borrow_mut();
            let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];

            // Re-read partition table to get OTA data partition
            let pt = read_partition_table(&mut *flash, &mut buffer)
                .map_err(|_| OtaError::PartitionTable)?;

            let ota_data_part = pt
                .find_partition(PartitionType::Data(DataPartitionSubType::Ota))
                .map_err(|_| OtaError::PartitionTable)?
                .ok_or(OtaError::NoOtaDataPartition)?;

            let mut ota_region = ota_data_part.as_embedded_storage(&mut *flash);
            let mut ota = Ota::new(&mut ota_region).map_err(|_| OtaError::InvalidState)?;

            // Activate the new slot
            ota.set_current_slot(self.target_slot)
                .map_err(|_| OtaError::ActivationError)?;

            // Set state to New for verification on next boot
            ota.set_current_ota_state(OtaImageState::New)
                .map_err(|_| OtaError::StateError)?;

            println!(
                "ota: update complete, slot {:?} activated",
                self.target_slot
            );

            Ok(())
        })
    }
}

/// Perform a software reset to reboot into the new firmware
pub(crate) fn reboot() -> ! {
    println!("ota: rebooting...");
    esp_hal::system::software_reset();
}
