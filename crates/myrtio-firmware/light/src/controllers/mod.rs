pub(crate) mod dependencies;
mod mqtt;
mod boot;

use myrtio_mqtt::runtime::MqttModule;

use dependencies::LIGHT_USECASES;
use mqtt::init_mqtt_module;
use boot::BootController;

use crate::domain::types::LightUsecasesPortRef;

pub(crate) fn init_controllers(usecases: LightUsecasesPortRef) -> (&'static mut dyn MqttModule, BootController) {
    LIGHT_USECASES.lock(|cell| {
        cell.borrow_mut().replace(usecases);
    });

    (init_mqtt_module(), BootController::new())
}
