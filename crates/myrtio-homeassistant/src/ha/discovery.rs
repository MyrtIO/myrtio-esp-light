//! Home Assistant discovery payload structures
//!
//! These types match the exact JSON schema expected by Home Assistant
//! for MQTT discovery messages.

use serde::Serialize;

/// Device information for Home Assistant discovery payload
#[derive(Clone, Serialize)]
pub struct HaDeviceInfo<'a> {
    /// Human-readable device name
    pub name: &'a str,
    /// Device identifiers array
    pub identifiers: &'a [&'a str],
    /// Manufacturer name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<&'a str>,
    /// Model name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<&'a str>,
    /// Software version (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sw_version: Option<&'a str>,
}

/// Light entity discovery payload for Home Assistant
#[derive(Serialize)]
pub struct HaLightDiscovery<'a> {
    /// Human-readable name
    pub name: &'a str,
    /// Unique identifier
    pub unique_id: &'a str,
    /// JSON schema type (always "json")
    pub schema: &'a str,
    /// Topic for state updates
    pub state_topic: &'a str,
    /// Topic for commands
    pub command_topic: &'a str,
    /// Device information
    pub device: HaDeviceInfo<'a>,
    /// MDI icon (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<&'a str>,
    /// Whether brightness is supported
    #[serde(skip_serializing_if = "is_false")]
    pub brightness: bool,
    /// Whether effects are supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<bool>,
    /// List of available effects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_list: Option<&'a [&'a str]>,
    /// Supported color modes
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub supported_color_modes: &'a [&'a str],
    /// Minimum color temperature in mireds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_mireds: Option<u16>,
    /// Maximum color temperature in mireds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_mireds: Option<u16>,
    /// Optimistic mode flag
    #[serde(skip_serializing_if = "is_false")]
    pub optimistic: bool,
}

/// Number entity discovery payload for Home Assistant
#[derive(Serialize)]
pub struct HaNumberDiscovery<'a> {
    /// Human-readable name
    pub name: &'a str,
    /// Unique identifier
    pub unique_id: &'a str,
    /// Topic for state updates
    pub state_topic: &'a str,
    /// Topic for commands
    pub command_topic: &'a str,
    /// Device information
    pub device: HaDeviceInfo<'a>,
    /// MDI icon (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<&'a str>,
    /// Device class (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<&'a str>,
    /// Unit of measurement (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_of_measurement: Option<&'a str>,
    /// Minimum value
    pub min: i32,
    /// Maximum value
    pub max: i32,
    /// Step increment (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,
    /// Display mode (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<&'a str>,
}

/// Helper to check if bool is false for serde skip
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(v: &bool) -> bool {
    !*v
}
