//! Home Assistant Device representation
//!
//! A Device represents the physical device that contains one or more entities.

use serde::Serialize;

/// Device information for Home Assistant discovery
#[derive(Clone, Serialize)]
pub struct Device<'a> {
    /// Device identifier (used in unique_id generation)
    #[serde(skip)]
    pub id: &'a str,
    /// Human-readable device name
    pub name: &'a str,
    /// Device identifiers array (contains id)
    pub identifiers: [&'a str; 1],
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

impl<'a> Device<'a> {
    /// Create a new device with the given ID and name
    pub const fn new(id: &'a str, name: &'a str) -> Self {
        Self {
            id,
            name,
            identifiers: [id],
            manufacturer: None,
            model: None,
            sw_version: None,
        }
    }

    /// Create a builder for more complex device configuration
    pub const fn builder(id: &'a str) -> DeviceBuilder<'a> {
        DeviceBuilder::new(id)
    }
}

/// Builder for Device configuration
pub struct DeviceBuilder<'a> {
    id: &'a str,
    name: Option<&'a str>,
    manufacturer: Option<&'a str>,
    model: Option<&'a str>,
    sw_version: Option<&'a str>,
}

impl<'a> DeviceBuilder<'a> {
    /// Create a new builder with the device ID
    pub const fn new(id: &'a str) -> Self {
        Self {
            id,
            name: None,
            manufacturer: None,
            model: None,
            sw_version: None,
        }
    }

    /// Set the device name
    pub const fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the manufacturer
    pub const fn manufacturer(mut self, manufacturer: &'a str) -> Self {
        self.manufacturer = Some(manufacturer);
        self
    }

    /// Set the model
    pub const fn model(mut self, model: &'a str) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the software version
    pub const fn sw_version(mut self, sw_version: &'a str) -> Self {
        self.sw_version = Some(sw_version);
        self
    }

    /// Build the Device
    pub const fn build(self) -> Device<'a> {
        Device {
            id: self.id,
            name: match self.name {
                Some(n) => n,
                None => self.id,
            },
            identifiers: [self.id],
            manufacturer: self.manufacturer,
            model: self.model,
            sw_version: self.sw_version,
        }
    }
}

