//! OTA Controller
//!
//! Coordinates OTA firmware updates, managing the update session and
//! (optionally) driving LED progress indication.

use esp_println::println;

use crate::infrastructure::tasks::ota::OtaInvite;

/// OTA Controller
///
/// Provides a high-level API for firmware updates, coordinating between
/// the OTA service (flash operations) and LED feedback.
pub(crate) struct OtaController;

impl OtaController {
    /// Create a new OTA controller
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Start a firmware update from an OTA invite
    pub(crate) fn on_ota_start(&self, invite: &OtaInvite) {
        println!("ota: starting update, size={} bytes", invite.size);
    }

    /// Got a chunk of firmware data
    pub(crate) fn on_ota_chunk(&self, written: u32, total: u32) {
        let progress = written * 100 / total;
        println!("ota: progress {}% ({}/{} bytes)", progress, written, total);
    }

    /// Finish the firmware update and trigger reboot
    pub(crate) fn on_ota_complete(&self) {
        println!("ota: update successful, rebooting...");
        // ota_reboot();
    }

    /// Abort the current update
    pub(crate) fn on_ota_abort(&self) {
        println!("ota: aborting update");
    }
}
