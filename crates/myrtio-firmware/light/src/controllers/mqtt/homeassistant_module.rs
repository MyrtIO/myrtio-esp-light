//! Home Assistant MQTT Module Initialization
//!
//! This module provides the Home Assistant integration configuration for the light.
//! It creates and configures the `HaModule` with the appropriate entities and callbacks.

use embassy_time::Duration;
use esp_println::println;
use myrtio_homeassistant::{HaModule, LightCommand, LightRegistration, LightState};
use myrtio_light_composer::ModeId;
use myrtio_mqtt::runtime::MqttModule;
use static_cell::StaticCell;

use crate::controllers::dependencies::LIGHT_USECASES;
use crate::controllers::mqtt::device::LIGHT_ENTITY;
use crate::domain::dto::LightChangeIntent;

const MAX_LIGHTS: usize = 4;
const MAX_NUMBERS: usize = 0;
const BUF_SIZE: usize = 512;

/// Static cell to store the HA module
static HOME_ASSISTANT_MODULE: StaticCell<HomeAssistantModule> = StaticCell::new();

/// Type alias for the HA module used in this firmware
pub(crate) type HomeAssistantModule = HaModule<'static, MAX_LIGHTS, MAX_NUMBERS, BUF_SIZE>;

/// Get current light state from shared usecases
fn get_light_state() -> LightState {
    let state = LIGHT_USECASES.lock(|cell| {
        let cell_ref = cell.borrow();
        let usecases = cell_ref.as_ref().unwrap();
        usecases.get_light_state().unwrap()
    });

    if state.power {
        let mode_id = ModeId::from_raw(state.mode_id).expect("Invalid mode ID");
        return LightState::on()
            .with_brightness(state.brightness)
            .with_rgb(state.color.0, state.color.1, state.color.2)
            .with_effect(mode_id.as_str());
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
    }

    if let Some(effect_str) = cmd.effect {
        if let Some(id) = ModeId::parse_from_str(effect_str) {
            intent = intent.with_effect_id(id as u8);
        }
    }

    println!("intent: {:?}", intent);

    LIGHT_USECASES.lock(|cell| {
        let mut cell_ref = cell.borrow_mut();
        let usecases = cell_ref.as_mut().unwrap();
        usecases
            .apply_intent_and_persist(intent)
            .expect("Failed to apply intent");
    });
}

/// Initialize and return the Home Assistant MQTT module as a trait object.
pub(crate) fn init_home_assistant_module() -> &'static mut dyn MqttModule {
    let mut module = HomeAssistantModule::new(Duration::from_secs(30));

    let light_registration = LightRegistration {
        entity: LIGHT_ENTITY.clone(),
        provide_state: get_light_state,
        on_command: handle_light_command,
    };

    module
        .add_light(light_registration)
        .expect("Failed to add light entity");

    let module: &'static mut HomeAssistantModule = HOME_ASSISTANT_MODULE.uninit().write(module);
    module as &'static mut dyn MqttModule
}
