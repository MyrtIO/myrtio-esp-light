//! Light entity domain types
//!
//! These are domain DTOs for light entities, independent of the Home Assistant
//! wire format. They are converted to `ha::*` types for MQTT communication.

use crate::device::Device;
use myrtio_macros::ConstBuilder;

/// RGB color representation (domain type)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

/// Supported color modes (domain type)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// RGB color mode
    Rgb,
    /// Color temperature mode (in mireds)
    ColorTemp,
    /// Brightness only (no color)
    Brightness,
    /// On/Off only
    OnOff,
}

impl ColorMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ColorMode::Rgb => "rgb",
            ColorMode::ColorTemp => "color_temp",
            ColorMode::Brightness => "brightness",
            ColorMode::OnOff => "onoff",
        }
    }
}

/// Light entity configuration (domain type)
#[derive(Debug, Clone, ConstBuilder)]
pub struct LightEntity<'a> {
    /// Entity identifier suffix (combined with device id for `unique_id`)
    pub id: &'a str,
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
    /// Whether the entity works in optimistic mode
    pub optimistic: bool,
}

impl<'a> LightEntity<'a> {
    /// Create a new light entity with required fields
    pub const fn new(id: &'a str, name: &'a str, device: &'a Device<'a>) -> Self {
        Self {
            id,
            name,
            device,
            icon: None,
            brightness: true,
            color_modes: &[],
            effects: None,
            min_mireds: None,
            max_mireds: None,
            optimistic: false,
        }
    }

    /// Set icon
    #[must_use]
    pub const fn with_icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set brightness support
    #[must_use]
    pub const fn with_brightness(mut self, brightness: bool) -> Self {
        self.brightness = brightness;
        self
    }

    /// Set color modes
    #[must_use]
    pub const fn with_color_modes(mut self, modes: &'a [ColorMode]) -> Self {
        self.color_modes = modes;
        self
    }

    /// Set effects list
    #[must_use]
    pub const fn with_effects(mut self, effects: &'a [&'a str]) -> Self {
        self.effects = Some(effects);
        self
    }

    /// Set color temperature range
    #[must_use]
    pub const fn with_mireds_range(mut self, min: u16, max: u16) -> Self {
        self.min_mireds = Some(min);
        self.max_mireds = Some(max);
        self
    }

    /// Set optimistic mode
    #[must_use]
    pub const fn with_optimistic(mut self, optimistic: bool) -> Self {
        self.optimistic = optimistic;
        self
    }
}

/// Light state (domain type)
#[derive(Debug, Clone, Default)]
pub struct LightState {
    /// Current on/off state
    pub is_on: bool,
    /// Current brightness (0-255)
    pub brightness: Option<u8>,
    /// Current color mode
    pub color_mode: Option<ColorMode>,
    /// Current color temperature in mireds
    pub color_temp: Option<u16>,
    /// Current RGB color
    pub color: Option<RgbColor>,
    /// Current effect name
    pub effect: Option<&'static str>,
}

impl LightState {
    /// Create an "ON" state
    pub const fn on() -> Self {
        Self {
            is_on: true,
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
            is_on: false,
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
        self.color_mode = Some(ColorMode::ColorTemp);
        self
    }

    /// Set RGB color
    #[must_use]
    pub const fn with_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some(RgbColor::new(r, g, b));
        self.color_mode = Some(ColorMode::Rgb);
        self
    }

    /// Set current effect
    #[must_use]
    pub const fn with_effect(mut self, effect: &'static str) -> Self {
        self.effect = Some(effect);
        self
    }
}

/// Command received for a light (domain type)
#[derive(Debug, Clone, Default)]
pub struct LightCommand<'a> {
    /// Requested state
    pub state: Option<bool>,
    /// Requested brightness (0-255)
    pub brightness: Option<u8>,
    /// Requested color temperature in mireds
    pub color_temp: Option<u16>,
    /// Requested RGB color
    pub color: Option<RgbColor>,
    /// Requested effect
    pub effect: Option<&'a str>,
}

impl LightCommand<'_> {
    /// Check if this is a turn on command
    pub fn is_on(&self) -> bool {
        self.state == Some(true)
    }

    /// Check if this is a turn off command
    pub fn is_off(&self) -> bool {
        self.state == Some(false)
    }
}

/// Registration for a light entity with callbacks
pub struct LightRegistration<'a> {
    pub entity: LightEntity<'a>,
    pub provide_state: fn() -> LightState,
    pub on_command: fn(&LightCommand),
}
