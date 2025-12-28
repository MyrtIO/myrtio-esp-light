mod button;
mod mqtt_homeassistant;

use core::cell::RefCell;

pub use button::handle_boot_button_click;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use mqtt_homeassistant::init_mqtt_homeassistant_module;
pub use myrtio_mqtt::runtime::MqttModule;

use crate::{
    domain::types::LightUsecasesPortRef,
    infrastructure::types::FirmwareUsecasesImpl,
};

pub(super) static LIGHT_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<LightUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));

pub(super) static FIRMWARE_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<FirmwareUsecasesImpl>>,
> = Mutex::new(RefCell::new(None));

/// Initialize the app controllers with it's dependencies
pub fn init_app_controllers(
    light: LightUsecasesPortRef,
    firmware: FirmwareUsecasesImpl,
) -> &'static mut dyn MqttModule {
    LIGHT_USECASES.lock(|cell| {
        cell.borrow_mut().replace(light);
    });

    FIRMWARE_USECASES.lock(|cell| {
        cell.borrow_mut().replace(firmware);
    });

    init_mqtt_homeassistant_module()
}
