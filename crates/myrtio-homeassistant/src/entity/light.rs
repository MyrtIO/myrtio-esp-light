//! Light entity for Home Assistant MQTT integration
//!
//! Supports brightness, color temperature, RGB color, and effects.

use heapless::String;
use serde::{Deserialize, Serialize};

use crate::device::Device;

/// Maximum number of effects supported
pub const MAX_EFFECTS: usize = 8;
/// Maximum number of color modes supported
pub const MAX_COLOR_MODES: usize = 4;

/// RGB color representation
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Supported color modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    /// RGB color mode
    Rgb,
    /// Color temperature mode (in mireds)
    ColorTemp,
    /// Brightness only (no color)
    Brightness,
    /// On/Off only
    Onoff,
}

impl ColorMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ColorMode::Rgb => "rgb",
            ColorMode::ColorTemp => "color_temp",
            ColorMode::Brightness => "brightness",
            ColorMode::Onoff => "onoff",
        }
    }
}

/// Light entity configuration for Home Assistant discovery
#[derive(Clone)]
pub struct LightEntity<'a> {
    /// Human-readable name
    pub name: &'a str,
    /// Reference to parent device
    pub device: &'a Device<'a>,
    /// MDI icon (e.g., "mdi:lightbulb")
    pub icon: Option<&'a str>,
    /// Whether brightness is supported
    pub brightness: bool,
    /// Supported color modes
    pub color_modes: &'a [ColorMode],
    /// Available effects
    pub effects: Option<&'a [&'a str]>,
    /// Minimum color temperature in mireds
    pub min_mireds: Option<u16>,
    /// Maximum color temperature in mireds
    pub max_mireds: Option<u16>,
    /// Flag that defines if entity works in optimistic mode.
    pub optimistic: bool,
}

impl LightEntity<'_> {
    /// Get the unique ID for this entity
    pub fn unique_id<const N: usize>(&self) -> String<N> {
        let mut id = String::new();
        let _ = id.push_str(self.device.id);
        let _ = id.push('_');
        let _ = id.push_str("light");
        id
    }

    /// Get the state topic for this entity
    pub fn state_topic<const N: usize>(&self) -> String<N> {
        let mut topic = String::new();
        let _ = topic.push_str(self.device.id);
        let _ = topic.push('/');
        let _ = topic.push_str("light");
        topic
    }

    /// Get the command topic for this entity
    pub fn command_topic<const N: usize>(&self) -> String<N> {
        let mut topic: String<N> = self.state_topic();
        let _ = topic.push_str("/set");
        topic
    }

    /// Get the config topic for Home Assistant discovery
    pub fn config_topic<const N: usize>(&self) -> String<N> {
        let mut topic = String::new();
        let _ = topic.push_str("homeassistant/light/");
        let _ = topic.push_str(self.device.id);
        let _ = topic.push('_');
        let _ = topic.push_str("light");
        let _ = topic.push_str("/config");
        topic
    }
}

/// Registration for a light entity with callbacks
pub struct LightRegistration<'a> {
    pub entity: LightEntity<'a>,
    pub provide_state: fn() -> LightState<'static>,
    pub on_command: fn(LightCommand<'_>),
}

/// Builder for `LightEntity` with callbacks
pub struct LightBuilder<'a> {
    device: &'a Device<'a>,
    name: Option<&'a str>,
    icon: Option<&'a str>,
    brightness: bool,
    color_modes: &'a [ColorMode],
    effects: Option<&'a [&'a str]>,
    min_mireds: Option<u16>,
    max_mireds: Option<u16>,
    provide_state: Option<fn() -> LightState<'static>>,
    on_command: Option<fn(LightCommand<'_>)>,
}

impl<'a> LightBuilder<'a> {
    pub const fn new(device: &'a Device<'a>) -> Self {
        Self {
            device,
            name: None,
            icon: None,
            brightness: false,
            color_modes: &[],
            effects: None,
            min_mireds: None,
            max_mireds: None,
            provide_state: None,
            on_command: None,
        }
    }

    /// Set the entity name
    #[must_use]
    pub const fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the MDI icon
    #[must_use]
    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Enable brightness support
    #[must_use]
    pub const fn brightness(mut self, enabled: bool) -> Self {
        self.brightness = enabled;
        self
    }

    /// Set supported color modes
    #[must_use]
    pub const fn color_modes(mut self, modes: &'a [ColorMode]) -> Self {
        self.color_modes = modes;
        self
    }

    /// Set available effects
    #[must_use]
    pub const fn effects(mut self, effects: &'a [&'a str]) -> Self {
        self.effects = Some(effects);
        self
    }

    /// Set color temperature range in mireds
    #[must_use]
    pub const fn mireds_range(mut self, min: u16, max: u16) -> Self {
        self.min_mireds = Some(min);
        self.max_mireds = Some(max);
        self
    }

    /// Set the state provider callback
    #[must_use]
    pub const fn provide_state(mut self, f: fn() -> LightState<'static>) -> Self {
        self.provide_state = Some(f);
        self
    }

    /// Set the command handler callback
    #[must_use]
    pub const fn on_command(mut self, f: fn(LightCommand<'_>)) -> Self {
        self.on_command = Some(f);
        self
    }

    /// Build the `LightRegistration` (entity + callbacks)
    ///
    /// # Panics
    /// Panics if `provide_state` or `on_command` callbacks are not set.
    pub const fn build(self) -> LightRegistration<'a> {
        LightRegistration {
            entity: LightEntity {
                name: match self.name {
                    Some(n) => n,
                    None => "light",
                },
                device: self.device,
                icon: self.icon,
                brightness: self.brightness,
                color_modes: self.color_modes,
                effects: self.effects,
                min_mireds: self.min_mireds,
                max_mireds: self.max_mireds,
                optimistic: true,
            },
            provide_state: self.provide_state.expect("provide_state callback is required"),
            on_command: self.on_command.expect("on_command callback is required"),
        }
    }
}

/// Light state for publishing to Home Assistant
#[derive(Debug, Clone, Default, Serialize)]
pub struct LightState<'a> {
    /// Current on/off state
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
    pub color: Option<RgbColor>,
    /// Current effect name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<&'a str>,
}

impl<'a> LightState<'a> {
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
    pub const fn brightness(mut self, brightness: u8) -> Self {
        self.brightness = Some(brightness);
        self
    }

    /// Set color temperature
    #[must_use]
    pub const fn color_temp(mut self, mireds: u16) -> Self {
        self.color_temp = Some(mireds);
        self.color_mode = Some("color_temp");
        self
    }

    /// Set RGB color
    #[must_use]
    pub const fn rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some(RgbColor::new(r, g, b));
        self.color_mode = Some("rgb");
        self
    }

    /// Set current effect
    #[must_use]
    pub const fn effect(mut self, effect: &'a str) -> Self {
        self.effect = Some(effect);
        self
    }
}

/// Command received from Home Assistant
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LightCommand<'a> {
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
    pub color: Option<RgbColor>,
    /// Requested effect
    #[serde(default)]
    pub effect: Option<&'a str>,
}

impl<'a> LightCommand<'a> {
    /// Check if this is a turn on command
    pub fn is_on(&self) -> bool {
        self.state == Some("ON")
    }

    /// Check if this is a turn off command
    pub fn is_off(&self) -> bool {
        self.state == Some("OFF")
    }
}
