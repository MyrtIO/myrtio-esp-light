//! Home Assistant command payload structures
//!
//! These types match the exact JSON schema sent by Home Assistant
//! for MQTT command messages.

use serde::Deserialize;

/// RGB color representation for HA commands
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct HaRgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Light command payload from Home Assistant
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HaLightCommand<'a> {
    /// Requested state ("ON" or "OFF")
    #[serde(default)]
    pub state: Option<&'a str>,
    /// Requested brightness (0-255)
    #[serde(default)]
    pub brightness: Option<u8>,
    /// Requested color temperature in mireds
    #[serde(default)]
    pub color_temp: Option<u16>,
    /// Requested RGB color
    #[serde(default)]
    pub color: Option<HaRgbColor>,
    /// Requested effect
    #[serde(default)]
    pub effect: Option<&'a str>,
}

impl HaLightCommand<'_> {
    /// Check if this is a turn on command
    pub fn is_on(&self) -> bool {
        self.state == Some("ON")
    }

    /// Check if this is a turn off command
    pub fn is_off(&self) -> bool {
        self.state == Some("OFF")
    }
}


