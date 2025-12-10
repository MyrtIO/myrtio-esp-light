//! OTA Controller
//!
//! Coordinates OTA firmware updates, managing the update session and
//! (optionally) driving LED progress indication.

use esp_println::println;
use myrtio_light_composer::CommandSender;

use crate::infrastructure::services::{OtaInvite, OtaService, OtaSession, ota_reboot};

/// Progress step for logging (every 10%)
const PROGRESS_LOG_STEP: u8 = 10;

/// OTA Controller
///
/// Provides a high-level API for firmware updates, coordinating between
/// the OTA service (flash operations) and LED feedback.
pub(crate) struct OtaController {
    service: &'static OtaService,
    #[allow(dead_code)]
    cmd_sender: CommandSender,
    session: Option<OtaSession>,
    last_logged_progress: u8,
}

impl OtaController {
    /// Create a new OTA controller
    pub(crate) fn new(service: &'static OtaService, cmd_sender: CommandSender) -> Self {
        Self {
            service,
            cmd_sender,
            session: None,
            last_logged_progress: 0,
        }
    }

    /// Start a firmware update from an OTA invite
    pub(crate) fn start_ota(&mut self, invite: &OtaInvite) -> Result<(), ()> {
        println!("ota: starting update, size={} bytes", invite.size);

        let session = self.service.begin_update(invite.size).map_err(|e| {
            println!("ota: failed to start update: {:?}", e);
        })?;

        self.session = Some(session);
        self.last_logged_progress = 0;
        Ok(())
    }

    /// Write a chunk of firmware data
    pub(crate) fn write_firmware_chunk(&mut self, data: &[u8]) -> Result<(), ()> {
        let session = self.session.as_mut().ok_or(())?;
        
        session.write_chunk(data).map_err(|e| {
            println!("ota: write error: {:?}", e);
        })?;

        // Log progress and update LED feedback
        self.update_progress();

        Ok(())
    }

    /// Update progress logging and LED feedback
    fn update_progress(&mut self) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let progress = session.progress_percent();
        
        // Log progress every PROGRESS_LOG_STEP percent
        if progress >= self.last_logged_progress + PROGRESS_LOG_STEP {
            let written = session.bytes_written();
            let total = session.total_size();
            println!("ota: progress {}% ({}/{} bytes)", progress, written, total);
            self.last_logged_progress = progress;
        }

        // TODO: Send progress to LED strip via cmd_sender
        // Example: map progress to brightness or show progress bar effect
        // self.cmd_sender.try_send(Command::SetBrightness(progress * 255 / 100)).ok();
    }

    /// Finish the firmware update and trigger reboot
    pub(crate) fn finish_ota(&mut self) -> Result<(), ()> {
        println!("ota: download complete, finalizing...");

        let session = self.session.take().ok_or(())?;
        
        session.finalize().map_err(|e| {
            println!("ota: failed to finalize update: {:?}", e);
        })?;

        println!("ota: update successful, rebooting...");
        ota_reboot();
    }

    /// Abort the current update
    pub(crate) fn abort(&mut self) {
        println!("ota: aborting update");
        self.session = None;
        self.last_logged_progress = 0;
        // TODO: Restore LED state via cmd_sender
    }
}
