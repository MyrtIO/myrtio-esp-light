mod mqtt_homeassistant;

use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use mqtt_homeassistant::init_mqtt_homeassistant_module;
use myrtio_mqtt::runtime::MqttModule;

use crate::domain::types::LightUsecasesPortRef;

pub(super) static LIGHT_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<LightUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));

/// Initialize the app controllers with it's dependencies
pub fn init_app_controllers(
    light: LightUsecasesPortRef,
) -> &'static mut dyn MqttModule {
    LIGHT_USECASES.lock(|cell| {
        cell.borrow_mut().replace(light);
    });

    init_mqtt_homeassistant_module()
}
