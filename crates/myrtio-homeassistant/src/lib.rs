//! Home Assistant MQTT integration for embedded devices
//!
//! This crate provides types and utilities for integrating with Home Assistant
//! via MQTT discovery. It is structured in three layers:
//!
//! - **Domain layer** (`device`, `entity`): Platform-independent DTOs for devices and entities
//! - **HA wire layer** (`ha`): JSON-serializable types matching Home Assistant's MQTT schema
//! - **MQTT integration** (`mqtt_module`): `HaModule` implementing `MqttModule` for `myrtio-mqtt`
//!
//! # Example
//!
//! ```ignore
//! use myrtio_homeassistant::{Device, LightEntity, LightRegistration, LightState, HaModule};
//!
//! const DEVICE: Device = Device::new("my_light", "My Light");
//! const LIGHT: LightEntity = LightEntity::new("main", "Main Light", &DEVICE)
//!     .with_brightness(true);
//!
//! fn provide_state() -> LightState {
//!     LightState::on().with_brightness(255)
//! }
//!
//! fn on_command(cmd: LightCommand) {
//!     // Handle command
//! }
//! ```

#![no_std]

pub mod device;
pub mod entity;
pub mod ha;
pub mod mqtt_module;

// Re-export domain types
pub use device::{Device, DeviceBuilder};
pub use entity::{
    ColorMode, LightCommand, LightEntity, LightEntityBuilder, LightRegistration, LightState,
    NumberEntity, NumberEntityBuilder, NumberRegistration, RgbColor,
};

// Re-export MQTT integration
pub use mqtt_module::HaModule;

// Re-export HA types for advanced usage
pub use ha::{HaDeviceInfo, HaLightCommand, HaLightDiscovery, HaLightState, HaNumberDiscovery};

use myrtio_mqtt::error::MqttError;

/// Error type for Home Assistant operations
#[derive(Debug)]
pub enum Error<E> {
    /// MQTT communication error
    Mqtt(MqttError<E>),
    /// JSON serialization error
    Serialization,
    /// JSON deserialization error
    Deserialization,
    /// Maximum entities reached
    MaxEntitiesReached,
}

impl<E: core::fmt::Debug> core::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Mqtt(e) => write!(f, "MQTT error: {:?}", e),
            Error::Serialization => write!(f, "JSON serialization error"),
            Error::Deserialization => write!(f, "JSON deserialization error"),
            Error::MaxEntitiesReached => write!(f, "Maximum entities reached"),
        }
    }
}

impl<E> From<MqttError<E>> for Error<E> {
    fn from(e: MqttError<E>) -> Self {
        Error::Mqtt(e)
    }
}
