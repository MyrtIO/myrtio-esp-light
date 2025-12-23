use heapless::String;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::domain::entity::LightState;

/// Represents a user intent to change the light state.
///
/// This is a domain-neutral representation of what the user wants to do,
/// independent of the source (MQTT, button, HTTP, etc.).
#[derive(Clone, Debug)]
pub struct LightChangeIntent {
    /// Turn on (Some(true)), turn off (Some(false)), or no change (None)
    pub power: Option<bool>,
    /// Set brightness to this value (0-255)
    pub brightness: Option<u8>,
    /// Set color to this RGB value
    pub color: Option<(u8, u8, u8)>,
    /// Set color temperature to this value (1000-40000)
    pub color_temp: Option<u16>,
    /// Set mode by ID
    pub mode_id: Option<u8>,
}

impl LightChangeIntent {
    /// Create a new empty intent (no changes)
    pub(crate) const fn new() -> Self {
        Self {
            power: None,
            brightness: None,
            color: None,
            color_temp: None,
            mode_id: None,
        }
    }

    /// Set power state
    #[must_use]
    pub(crate) const fn with_power(mut self, on: bool) -> Self {
        self.power = Some(on);
        self
    }

    /// Set brightness
    #[must_use]
    pub(crate) const fn with_brightness(mut self, brightness: u8) -> Self {
        self.brightness = Some(brightness);
        self
    }

    /// Set color
    #[must_use]
    pub(crate) const fn with_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some((r, g, b));
        self
    }

    /// Set color temperature
    #[must_use]
    pub(crate) const fn with_color_temp(mut self, color_temp: u16) -> Self {
        self.color_temp = Some(color_temp);
        self
    }

    /// Set effect
    #[must_use]
    pub(crate) const fn with_effect_id(mut self, effect_id: u8) -> Self {
        self.mode_id = Some(effect_id);
        self
    }
}

impl From<LightState> for LightChangeIntent {
    fn from(state: LightState) -> Self {
        LightChangeIntent {
            power: Some(state.power),
            brightness: Some(state.brightness),
            color: Some(state.color),
            color_temp: Some(state.color_temp),
            mode_id: Some(state.mode_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInformation {
    pub build_version: String<32>,
    pub mac_address: [u8; 6],
}
