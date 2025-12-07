mod device;
mod homeassistant_module;

pub(crate) use homeassistant_module::init_home_assistant_module;
use myrtio_mqtt::runtime::MqttModule;

pub(crate) fn init_mqtt_module() -> &'static mut dyn MqttModule {
    init_home_assistant_module()
}