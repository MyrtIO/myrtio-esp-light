//! Home Assistant Device representation
//!
//! A Device represents the physical device that contains one or more entities.

use myrtio_macros::ConstBuilder;

/// Device information for Home Assistant
///
/// This is the domain representation of a device. It is converted to
/// `ha::HaDeviceInfo` for serialization in discovery messages.
#[derive(Debug, Clone, ConstBuilder)]
pub struct Device<'a> {
    /// Device identifier (used in topic generation and `unique_id`)
    pub id: &'a str,
    /// Human-readable device name
    pub name: &'a str,
    /// Manufacturer name (optional)
    pub manufacturer: Option<&'a str>,
    /// Model name (optional)
    pub model: Option<&'a str>,
    /// Software version (optional)
    pub sw_version: Option<&'a str>,
}

impl<'a> Device<'a> {
    /// Create a new device with the given ID and name
    pub const fn new(id: &'a str, name: &'a str) -> Self {
        Self {
            id,
            name,
            manufacturer: None,
            model: None,
            sw_version: None,
        }
    }

    /// Set manufacturer
    #[must_use]
    pub const fn with_manufacturer(mut self, manufacturer: &'a str) -> Self {
        self.manufacturer = Some(manufacturer);
        self
    }

    /// Set model
    #[must_use]
    pub const fn with_model(mut self, model: &'a str) -> Self {
        self.model = Some(model);
        self
    }

    /// Set software version
    #[must_use]
    pub const fn with_sw_version(mut self, sw_version: &'a str) -> Self {
        self.sw_version = Some(sw_version);
        self
    }
}
