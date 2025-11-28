//! Home Assistant Entity types
//!
//! This module contains entity definitions for Home Assistant MQTT integration.

pub mod light;
pub mod number;

pub use light::{LightBuilder, LightCommand, LightEntity, LightState, ColorMode, RgbColor};
pub use number::{NumberBuilder, NumberEntity};

