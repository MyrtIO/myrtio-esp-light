//! # Home Assistant MQTT Integration Library
//!
//! `myrtio-homeassistant` provides a `no_std` compatible library for integrating
//! devices with Home Assistant via MQTT discovery protocol.
//!
//! ## Features
//!
//! - **`no_std` & `no_alloc`:** Designed for embedded systems using `heapless` collections
//! - **Builder Pattern:** Fluent API for configuring entities
//! - **Callback-based:** Register entities with state providers and command handlers
//! - **Automatic Discovery:** Publishes Home Assistant MQTT discovery configs
//!
//! ## Example
//!
//! ```rust,ignore
//! use myrtio_homeassistant::{Device, HomeAssistantClient, LightEntity, LightState, LightCommand};
//!
//! static DEVICE: Device = Device::builder("myrt_light_01")
//!     .name("Myrt Light")
//!     .manufacturer("Myrtio")
//!     .build();
//!
//! fn get_light_state() -> LightState<'static> {
//!     LightState::on().brightness(128)
//! }
//!
//! fn handle_light_command(cmd: LightCommand<'_>) {
//!     if cmd.is_off() {
//!         // Turn off
//!     } else if let Some(brightness) = cmd.brightness {
//!         // Set brightness
//!     }
//! }
//!
//! async fn run(mqtt: MqttClient<...>) {
//!     let mut ha = HomeAssistantClient::<_, 8, 512, 4, 4>::new(mqtt, "myrtlight");
//!
//!     let light = LightEntity::builder("main", &DEVICE)
//!         .name("Main Light")
//!         .brightness(true)
//!         .build();
//!
//!     ha.register_light(light, get_light_state, handle_light_command).unwrap();
//!     ha.announce_all().await.unwrap();
//!
//!     loop {
//!         ha.poll().await.unwrap();
//!         ha.publish_states().await.unwrap();
//!     }
//! }
//! ```

#![no_std]

pub mod client;
pub mod device;
pub mod entity;
pub mod error;
pub mod topic;

// Re-export key types for convenient access
pub use client::HomeAssistantClient;
pub use device::{Device, DeviceBuilder};
pub use entity::light::{ColorMode, LightBuilder, LightCommand, LightEntity, LightState, RgbColor};
pub use entity::number::{NumberBuilder, NumberEntity, NumberState};
pub use error::HaError;
