//! Entity domain types
//!
//! This module contains domain DTOs for Home Assistant entities.

pub mod light;
pub mod number;

pub use light::{
    ColorMode, LightCommand, LightEntity, LightEntityBuilder, LightRegistration, LightState,
    RgbColor,
};
pub use number::{NumberEntity, NumberEntityBuilder, NumberRegistration};
