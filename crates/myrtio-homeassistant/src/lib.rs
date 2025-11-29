//! # Home Assistant MQTT Integration Library
//!
//! `myrtio-homeassistant` provides a `no_std` compatible library for integrating
//! devices with Home Assistant via MQTT discovery protocol.
//!
//! ## Features
//!
//! - **`no_std` & `no_alloc`:** Designed for embedded systems using `heapless` collections
//! - **Builder Pattern:** Fluent API for configuring entities with callbacks
//! - **Device-centric:** Create entities from device, namespace included
//! - **Automatic Discovery:** Publishes Home Assistant MQTT discovery configs
//!
//! ## Example
//!
//! ```rust,ignore
//! use myrtio_homeassistant::{Device, HomeAssistantClient, LightState, LightCommand};
//!
//! static DEVICE: Device = Device::builder("myrt_light_01")
//!     .namespace("myrtlight")
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
//!     let mut ha = HomeAssistantClient::<_, 8, 512, 4, 4>::new(&DEVICE, mqtt);
//!
//!     ha.add_light(DEVICE.light("main")
//!         .name("Main Light")
//!         .brightness(true)
//!         .provide_state(get_light_state)
//!         .on_command(handle_light_command)
//!         .build()).unwrap();
//!
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
pub use entity::light::{ColorMode, LightBuilder, LightCommand, LightEntity, LightRegistration, LightState, RgbColor};
pub use entity::number::{NumberBuilder, NumberEntity, NumberRegistration, NumberState};
pub use error::HaError;
