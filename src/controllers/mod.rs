mod boot;
pub mod dependencies;
mod mqtt;
pub mod ota;

use myrtio_mqtt::runtime::MqttModule;

use boot::BootController;
use dependencies::LIGHT_USECASES;
use mqtt::init_mqtt_module;

use crate::domain::types::LightUsecasesPortRef;

pub use ota::OtaController;

pub fn init_controllers(
    usecases: LightUsecasesPortRef,
) -> (&'static mut dyn MqttModule, BootController) {
    LIGHT_USECASES.lock(|cell| {
        cell.borrow_mut().replace(usecases);
    });

    (init_mqtt_module(), BootController::new())
}
