//! Mapping between domain types and Home Assistant wire types
//!
//! This module provides conversions between domain DTOs and the HA-specific
//! wire formats used for MQTT communication.

use heapless::{String, Vec};

use crate::device::Device;
use crate::entity::light::{LightCommand, LightEntity, LightState, RgbColor};
use crate::entity::number::NumberEntity;
use crate::ha::command::HaLightCommand;
use crate::ha::discovery::{HaDeviceInfo, HaLightDiscovery, HaNumberDiscovery};
use crate::ha::state::{HaLightState, HaRgbColor as HaStateRgbColor};
use crate::ha::topic;

/// Convert a domain Device to [`HaDeviceInfo`]
pub fn device_to_ha<'a>(device: &'a Device<'a>, identifiers: &'a [&'a str]) -> HaDeviceInfo<'a> {
    HaDeviceInfo {
        name: device.name,
        identifiers,
        manufacturer: device.manufacturer,
        model: device.model,
        sw_version: device.sw_version,
    }
}

/// Context for building light discovery payload
pub struct LightDiscoveryContext<'a, const N: usize> {
    pub unique_id: String<N>,
    pub state_topic: String<N>,
    pub command_topic: String<N>,
    pub config_topic: String<N>,
    pub color_modes: Vec<&'a str, 4>,
    pub identifier: String<N>,
}

impl<'a, const N: usize> LightDiscoveryContext<'a, N> {
    /// Build context for a light entity
    pub fn new(entity: &LightEntity<'a>) -> Self {
        let unique_id = topic::unique_id(entity.device.id, entity.id);
        let state_topic = topic::state_topic(entity.device.id, entity.id);
        let command_topic = topic::command_topic(entity.device.id, entity.id);
        let config_topic = topic::config_topic("light", entity.device.id, entity.id);

        let mut color_modes: Vec<&str, 4> = Vec::new();
        for mode in entity.color_modes {
            let _ = color_modes.push(mode.as_str());
        }

        let mut identifier: String<N> = String::new();
        let _ = identifier.push_str(entity.device.id);

        Self {
            unique_id,
            state_topic,
            command_topic,
            config_topic,
            color_modes,
            identifier,
        }
    }
}

/// Build [`HaLightDiscovery`] from a [`LightEntity`] and pre-built context
pub fn light_to_discovery<'a, 'b, const N: usize>(
    entity: &'a LightEntity<'a>,
    ctx: &'b LightDiscoveryContext<'a, N>,
    identifier_slice: &'b [&'b str],
) -> HaLightDiscovery<'b>
where
    'a: 'b,
{
    HaLightDiscovery {
        name: entity.name,
        unique_id: ctx.unique_id.as_str(),
        schema: "json",
        state_topic: ctx.state_topic.as_str(),
        command_topic: ctx.command_topic.as_str(),
        device: HaDeviceInfo {
            name: entity.device.name,
            identifiers: identifier_slice,
            manufacturer: entity.device.manufacturer,
            model: entity.device.model,
            sw_version: entity.device.sw_version,
        },
        icon: entity.icon,
        brightness: entity.brightness,
        effect: entity.effects.map(|_| true),
        effect_list: entity.effects,
        supported_color_modes: ctx.color_modes.as_slice(),
        min_kelvin: entity.min_kelvin,
        max_kelvin: entity.max_kelvin,
        color_temp_kelvin: Some(true),
        optimistic: entity.optimistic,
    }
}

/// Context for building number discovery payload
pub struct NumberDiscoveryContext<const N: usize> {
    pub unique_id: String<N>,
    pub state_topic: String<N>,
    pub command_topic: String<N>,
    pub config_topic: String<N>,
    pub identifier: String<N>,
}

impl<const N: usize> NumberDiscoveryContext<N> {
    /// Build context for a number entity
    pub fn new(entity: &NumberEntity<'_>) -> Self {
        let unique_id = topic::unique_id(entity.device.id, entity.id);
        let state_topic = topic::state_topic(entity.device.id, entity.id);
        let command_topic = topic::command_topic(entity.device.id, entity.id);
        let config_topic = topic::config_topic("number", entity.device.id, entity.id);

        let mut identifier: String<N> = String::new();
        let _ = identifier.push_str(entity.device.id);

        Self {
            unique_id,
            state_topic,
            command_topic,
            config_topic,
            identifier,
        }
    }
}

/// Build [`HaNumberDiscovery`] from a [`NumberEntity`] and pre-built context
pub fn number_to_discovery<'a, 'b, const N: usize>(
    entity: &'a NumberEntity<'a>,
    ctx: &'b NumberDiscoveryContext<N>,
    identifier_slice: &'b [&'b str],
) -> HaNumberDiscovery<'b>
where
    'a: 'b,
{
    HaNumberDiscovery {
        name: entity.name,
        unique_id: ctx.unique_id.as_str(),
        state_topic: ctx.state_topic.as_str(),
        command_topic: ctx.command_topic.as_str(),
        device: HaDeviceInfo {
            name: entity.device.name,
            identifiers: identifier_slice,
            manufacturer: entity.device.manufacturer,
            model: entity.device.model,
            sw_version: entity.device.sw_version,
        },
        icon: entity.icon,
        device_class: entity.device_class,
        unit_of_measurement: entity.unit,
        min: entity.min,
        max: entity.max,
        step: entity.step,
        mode: entity.mode,
    }
}

/// Convert domain [`LightState`] to [`HaLightState`]
pub fn light_state_to_ha(state: &LightState) -> HaLightState<'static> {
    HaLightState {
        state: if state.is_on { "ON" } else { "OFF" },
        brightness: state.brightness,
        color_mode: state.color_mode.map(|m| m.as_str()),
        color_temp: state.color_temp,
        color: state.color.map(|c| HaStateRgbColor::new(c.r, c.g, c.b)),
        effect: state.effect,
    }
}

/// Convert [`HaLightCommand`] to domain [`LightCommand`]
pub fn ha_command_to_light<'a>(cmd: &HaLightCommand<'a>) -> LightCommand<'a> {
    LightCommand {
        state: cmd.state.map(|s| s == "ON"),
        brightness: cmd.brightness,
        color_temp: cmd.color_temp,
        color: cmd.color.map(|c| RgbColor::new(c.r, c.g, c.b)),
        effect: cmd.effect,
    }
}
