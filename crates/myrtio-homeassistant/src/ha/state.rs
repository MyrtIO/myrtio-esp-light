//! Home Assistant state payload structures
//!
//! These types match the exact JSON schema expected by Home Assistant
//! for MQTT state messages.

use serde::Serialize;

/// RGB color representation for HA state
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct HaRgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl HaRgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Light state payload for Home Assistant
#[derive(Debug, Clone, Default, Serialize)]
pub struct HaLightState<'a> {
    /// Current on/off state ("ON" or "OFF")
    pub state: &'a str,
    /// Current brightness (0-255)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u8>,
    /// Current color mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_mode: Option<&'a str>,
    /// Current color temperature in mireds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_temp: Option<u16>,
    /// Current RGB color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<HaRgbColor>,
    /// Current effect name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<&'a str>,
}

impl<'a> HaLightState<'a> {
    /// Create an "ON" state
    pub const fn on() -> Self {
        Self {
            state: "ON",
            brightness: None,
            color_mode: None,
            color_temp: None,
            color: None,
            effect: None,
        }
    }

    /// Create an "OFF" state
    pub const fn off() -> Self {
        Self {
            state: "OFF",
            brightness: None,
            color_mode: None,
            color_temp: None,
            color: None,
            effect: None,
        }
    }

    /// Set brightness
    #[must_use]
    pub const fn with_brightness(mut self, brightness: u8) -> Self {
        self.brightness = Some(brightness);
        self
    }

    /// Set color temperature
    #[must_use]
    pub const fn with_color_temp(mut self, mireds: u16) -> Self {
        self.color_temp = Some(mireds);
        self.color_mode = Some("color_temp");
        self
    }

    /// Set RGB color
    #[must_use]
    pub const fn with_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some(HaRgbColor::new(r, g, b));
        self.color_mode = Some("rgb");
        self
    }

    /// Set current effect
    #[must_use]
    pub const fn with_effect(mut self, effect: &'a str) -> Self {
        self.effect = Some(effect);
        self
    }
}


