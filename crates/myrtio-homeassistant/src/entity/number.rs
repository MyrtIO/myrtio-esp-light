//! Number entity domain types
//!
//! These are domain DTOs for number entities, independent of the Home Assistant
//! wire format. They are converted to `ha::*` types for MQTT communication.

use crate::device::Device;
use myrtio_macros::ConstBuilder;

/// Number entity configuration (domain type)
#[derive(Clone, ConstBuilder)]
pub struct NumberEntity<'a> {
    /// Entity identifier suffix (combined with device id for `unique_id`)
    pub id: &'a str,
    /// Human-readable name
    pub name: &'a str,
    /// Reference to parent device
    pub device: &'a Device<'a>,
    /// MDI icon (e.g., "mdi:speedometer")
    pub icon: Option<&'a str>,
    /// Device class (e.g., "temperature", "humidity")
    pub device_class: Option<&'a str>,
    /// Unit of measurement (e.g., "°C", "%")
    pub unit: Option<&'a str>,
    /// Minimum value
    pub min: i32,
    /// Maximum value
    pub max: i32,
    /// Step increment
    pub step: Option<f32>,
    /// Display mode ("auto", "box", "slider")
    pub mode: Option<&'a str>,
}

impl<'a> NumberEntity<'a> {
    /// Create a new number entity with required fields
    pub const fn new(id: &'a str, name: &'a str, device: &'a Device<'a>) -> Self {
        Self {
            id,
            name,
            device,
            icon: None,
            device_class: None,
            unit: None,
            min: 0,
            max: 100,
            step: None,
            mode: None,
        }
    }

    /// Set icon
    #[must_use]
    pub const fn with_icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set device class
    #[must_use]
    pub const fn with_device_class(mut self, device_class: &'a str) -> Self {
        self.device_class = Some(device_class);
        self
    }

    /// Set unit of measurement
    #[must_use]
    pub const fn with_unit(mut self, unit: &'a str) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Set value range
    #[must_use]
    pub const fn with_range(mut self, min: i32, max: i32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set step increment
    #[must_use]
    pub const fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set display mode ("auto", "box", "slider")
    #[must_use]
    pub const fn with_mode(mut self, mode: &'a str) -> Self {
        self.mode = Some(mode);
        self
    }
}

/// Registration for a number entity with callbacks
pub struct NumberRegistration<'a> {
    pub entity: NumberEntity<'a>,
    pub provide_state: fn() -> i32,
    pub on_command: fn(i32),
}














