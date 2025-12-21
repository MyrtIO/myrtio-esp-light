//! OTA Controller
//!
//! Coordinates OTA firmware updates, managing the update session and
//! (optionally) driving LED progress indication.

use esp_println::println;

/// OTA Controller
///
/// Provides a high-level API for firmware updates, coordinating between
/// the OTA service (flash operations) and LED feedback.
pub struct OtaController;

impl OtaController {
    /// Create a new OTA controller
    pub fn new() -> Self {
        Self {}
    }

    /// Start a firmware update
    pub fn on_ota_start(&self, expected_size: u32) {
        println!("ota: starting update, size={} bytes", expected_size);
    }

    /// Got a chunk of firmware data
    pub fn on_ota_chunk(&self, written: u32, total: u32) {
        let progress = written * 100 / total;
        println!("ota: progress {}% ({}/{} bytes)", progress, written, total);
    }

    /// Finish the firmware update and trigger reboot
    pub fn on_ota_complete(&self) {
        println!("ota: update successful, rebooting...");
    }

    /// Abort the current update
    pub fn on_ota_abort(&self) {
        println!("ota: aborting update");
    }
}
