use super::FIRMWARE_USECASES;
use crate::domain::ports::BootSectorSelector as _;

/// Handler for boot button click event
pub fn handle_boot_button_click() {
    FIRMWARE_USECASES.lock(|cell| {
        let mut cell = cell.borrow_mut();
        let firmware = cell.as_mut().unwrap();

        firmware.boot_factory().unwrap();
    });
}
