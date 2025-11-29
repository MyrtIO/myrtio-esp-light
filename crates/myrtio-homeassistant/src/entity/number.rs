//! Number entity for Home Assistant MQTT integration
//!
//! Supports numeric values with min/max range, step, and unit of measurement.

use heapless::String;
use serde::Serialize;

use crate::device::Device;

/// Number entity configuration for Home Assistant discovery
#[derive(Clone)]
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

impl NumberEntity<'_> {
    /// Get the unique ID for this entity
    pub fn unique_id<const N: usize>(&self) -> String<N> {
        let mut id = String::new();
        let _ = id.push_str(self.device.id);
        let _ = id.push('_');
        let _ = id.push_str(self.id);
        id
    }

    /// Get the state topic for this entity
    pub fn state_topic<const N: usize>(&self) -> String<N> {
        let mut topic = String::new();
        let _ = topic.push_str(self.device.id);
        let _ = topic.push('/');
        let _ = topic.push_str(self.id);
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
        let _ = topic.push_str("homeassistant/number/");
        let _ = topic.push_str(self.device.id);
        let _ = topic.push('_');
        let _ = topic.push_str(self.id);
        let _ = topic.push_str("/config");
        topic
    }
}

/// Registration for a number entity with callbacks
pub struct NumberRegistration<'a> {
    pub entity: NumberEntity<'a>,
    pub provide_state: fn() -> i32,
    pub on_command: fn(i32),
}

/// Builder for `NumberEntity` with callbacks
pub struct NumberBuilder<'a> {
    id: &'a str,
    device: &'a Device<'a>,
    name: Option<&'a str>,
    icon: Option<&'a str>,
    device_class: Option<&'a str>,
    unit: Option<&'a str>,
    min: i32,
    max: i32,
    step: Option<f32>,
    mode: Option<&'a str>,
    provide_state: Option<fn() -> i32>,
    on_command: Option<fn(i32)>,
}

impl<'a> NumberBuilder<'a> {
    pub const fn new(id: &'a str, device: &'a Device<'a>) -> Self {
        Self {
            id,
            device,
            name: None,
            icon: None,
            device_class: None,
            unit: None,
            min: 0,
            max: 100,
            step: None,
            mode: None,
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

    /// Set the device class
    #[must_use]
    pub const fn device_class(mut self, device_class: &'a str) -> Self {
        self.device_class = Some(device_class);
        self
    }

    /// Set the unit of measurement
    #[must_use]
    pub const fn unit(mut self, unit: &'a str) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Set the minimum value
    #[must_use]
    pub const fn min(mut self, min: i32) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum value
    #[must_use]
    pub const fn max(mut self, max: i32) -> Self {
        self.max = max;
        self
    }

    /// Set the value range (min and max)
    #[must_use]
    pub const fn range(mut self, min: i32, max: i32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set the step increment
    #[must_use]
    pub const fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the display mode ("auto", "box", "slider")
    #[must_use]
    pub const fn mode(mut self, mode: &'a str) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set the state provider callback
    #[must_use]
    pub const fn provide_state(mut self, f: fn() -> i32) -> Self {
        self.provide_state = Some(f);
        self
    }

    /// Set the command handler callback
    #[must_use]
    pub const fn on_command(mut self, f: fn(i32)) -> Self {
        self.on_command = Some(f);
        self
    }

    /// Build the `NumberRegistration` (entity + callbacks)
    ///
    /// # Panics
    /// Panics if `provide_state` or `on_command` callbacks are not set.
    pub fn build(self) -> NumberRegistration<'a> {
        NumberRegistration {
            entity: NumberEntity {
                id: self.id,
                name: match self.name {
                    Some(n) => n,
                    None => self.id,
                },
                device: self.device,
                icon: self.icon,
                device_class: self.device_class,
                unit: self.unit,
                min: self.min,
                max: self.max,
                step: self.step,
                mode: self.mode,
            },
            provide_state: self
                .provide_state
                .expect("provide_state callback is required"),
            on_command: self.on_command.expect("on_command callback is required"),
        }
    }
}

/// Number state for serialization (just the value)
#[derive(Debug, Clone, Serialize)]
pub struct NumberState {
    pub value: i32,
}

impl NumberState {
    pub const fn new(value: i32) -> Self {
        Self { value }
    }
}
