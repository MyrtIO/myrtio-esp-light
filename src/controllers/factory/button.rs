use crate::{
    controllers::factory::FIRMWARE_USECASES,
    domain::ports::BootSectorSelector as _,
};

/// Handler for boot button click event
pub fn handle_boot_button_click() {
    let Ok(guard) = FIRMWARE_USECASES.try_lock() else {
        return;
    };
    let mut firmware_ref = guard.borrow_mut();
    let firmware = firmware_ref.as_mut().unwrap();
    firmware.boot_system().unwrap();
}
