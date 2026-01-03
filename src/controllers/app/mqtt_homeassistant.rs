//! Home Assistant MQTT Module Initialization
//!
//! This module provides the Home Assistant integration configuration for the light.
//! It creates and configures the `HaModule` with the appropriate entities and
//! callbacks.

use embassy_time::Duration;
use heapless::String;
use myrtio_light_composer::EffectId;
use myrtio_mqtt::runtime::MqttModule;
use myrtio_mqtt_homeassistant::{
    ColorMode,
    Device,
    HaModule,
    LightCommand,
    LightEntity,
    LightRegistration,
    LightState,
};
use static_cell::StaticCell;

use super::LIGHT_USECASES;
use crate::{
    config::{
        self,
        BUILD_VERSION,
        DEVICE_MANUFACTURER,
        DEVICE_MODEL,
        TEMPERATURE_MAX_KELVIN,
        TEMPERATURE_MIN_KELVIN,
    },
    domain::dto::LightChangeIntent,
    mk_static,
};

const MAX_LIGHTS: usize = 4;
const MAX_NUMBERS: usize = 0;
const BUF_SIZE: usize = 1024;

/// Static cell to store the HA module
static HOME_ASSISTANT_MODULE: StaticCell<HomeAssistantModule> = StaticCell::new();

/// Type alias for the HA module used in this firmware
pub(crate) type HomeAssistantModule =
    HaModule<'static, MAX_LIGHTS, MAX_NUMBERS, BUF_SIZE>;

/// Get current light state from shared usecases
fn get_light_state() -> LightState {
    let state = LIGHT_USECASES.lock(|cell| {
        let cell_ref = cell.borrow();
        let usecases = cell_ref.as_ref().unwrap();
        usecases.get_light_state()
    });

    if state.power {
        let effect_id = EffectId::from_raw(state.mode_id).unwrap_or(EffectId::Static);
        return LightState::on()
            .with_brightness(state.brightness)
            .with_rgb(state.color.0, state.color.1, state.color.2)
            .with_effect(effect_id.as_str());
    }

    LightState::off()
}

/// Handle light commands from Home Assistant
fn handle_light_command(cmd: &LightCommand) {
    let mut intent = LightChangeIntent::new();

    if cmd.is_off() {
        intent = intent.with_power(false);
    } else if cmd.is_on() {
        intent = intent.with_power(true);
    }

    if let Some(brightness) = cmd.brightness {
        intent = intent.with_brightness(brightness);
    }

    if let Some(color) = cmd.color {
        intent = intent.with_color(color.r, color.g, color.b);
    } else if let Some(color_temp) = cmd.color_temp {
        intent = intent.with_color_temp(color_temp);
    }

    if let Some(effect_str) = cmd.effect {
        if let Some(id) = EffectId::parse_from_str(effect_str) {
            intent = intent.with_effect_id(id as u8);
        }
    }

    LIGHT_USECASES.lock(|cell| {
        let mut cell_ref = cell.borrow_mut();
        let usecases = cell_ref.as_mut().unwrap();
        usecases
            .apply_intent_and_persist(intent)
            .expect("Failed to apply intent");
    });
}

/// Initialize and return the Home Assistant MQTT module as a trait object.
pub(super) fn init_mqtt_homeassistant_module() -> &'static mut dyn MqttModule {
    let device_id = mk_static!(String<32>, config::device_id());
    let device_name = mk_static!(String<32>, config::access_point_name());

    esp_println::println!(
        "ha: device_id='{}' (len={}), device_name='{}' (len={})",
        device_id,
        device_id.len(),
        device_name,
        device_name.len()
    );
    esp_println::println!(
        "ha: BUILD_VERSION='{}' (len={})",
        BUILD_VERSION,
        BUILD_VERSION.len()
    );

    let device = mk_static!(
        Device<'static>,
        Device::builder()
            .id(device_id.as_str())
            .name(device_name.as_str())
            .manufacturer(Some(DEVICE_MANUFACTURER))
            .model(Some(DEVICE_MODEL))
            .sw_version(Some(BUILD_VERSION))
            .build()
    );
    let supported_effects = mk_static!(
        [&str; 8],
        [
            EffectId::Static.as_str(),
            EffectId::RainbowMirrored.as_str(),
            EffectId::RainbowLong.as_str(),
            EffectId::RainbowShort.as_str(),
            EffectId::RainbowLongInverse.as_str(),
            EffectId::RainbowShortInverse.as_str(),
            EffectId::Aurora.as_str(),
            EffectId::LavaLamp.as_str(),
        ]
    );

    let light_entity = LightEntity::builder()
        .id("light")
        .name("Свет")
        .device(device)
        .icon(Some("mdi:led-strip"))
        .brightness(true)
        .min_kelvin(Some(TEMPERATURE_MIN_KELVIN))
        .max_kelvin(Some(TEMPERATURE_MAX_KELVIN))
        .color_modes(&[ColorMode::Rgb, ColorMode::ColorTemp])
        .effects(Some(supported_effects.as_slice()))
        .optimistic(false)
        .build();

    let mut module = HomeAssistantModule::new(Duration::from_secs(30));

    let light_registration = LightRegistration {
        entity: light_entity,
        provide_state: get_light_state,
        on_command: handle_light_command,
    };

    module
        .add_light(light_registration)
        .expect("Failed to add light entity");

    let module: &'static mut HomeAssistantModule =
        HOME_ASSISTANT_MODULE.uninit().write(module);
    module as &'static mut dyn MqttModule
}
