//! Home Assistant MQTT Module
//!
//! Implements the object-safe `MqttModule` trait from `myrtio-mqtt` for Home Assistant integration.
//!
//! # Design
//!
//! This module provides `HaModule`, which implements the `MqttModule` trait without any
//! transport-specific type parameters. All async I/O is handled by the runtime; the module
//! simply queues publish requests via the `PublishOutbox`.
//!
//! # Usage
//!
//! ```ignore
//! use myrtio_homeassistant::{HaModule, LightEntity, LightRegistration, LightState};
//!
//! // Create and configure module
//! let mut ha_module: HaModule<4, 4, 512> = HaModule::new(Duration::from_secs(30));
//! ha_module.add_light(LightRegistration { ... })?;
//!
//! // Use with MqttRuntime - module implements MqttModule
//! let runtime = MqttRuntime::new(client, ha_module, rx);
//! ```

use core::fmt::Write;
use embassy_time::Duration;
use heapless::{String, Vec};
use myrtio_mqtt::{
    QoS,
    runtime::{MqttModule, Publish, PublishOutbox, TopicCollector},
};

use crate::{
    Error,
    entity::light::{LightCommand, LightEntity, LightRegistration, LightState},
    entity::number::{NumberEntity, NumberRegistration},
    ha::{self, HaLightCommand, LightDiscoveryContext, NumberDiscoveryContext},
};

/// Maximum length for a topic string
const MAX_TOPIC_LEN: usize = 128;

/// Internal storage for light registration with pre-computed command topic
struct LightEntry<'a> {
    entity: LightEntity<'a>,
    command_topic: String<MAX_TOPIC_LEN>,
    provide_state: fn() -> LightState,
    on_command: fn(&LightCommand),
}

/// Internal storage for number registration with pre-computed command topic
struct NumberEntry<'a> {
    entity: NumberEntity<'a>,
    command_topic: String<MAX_TOPIC_LEN>,
    provide_state: fn() -> i32,
    on_command: fn(i32),
}

/// Home Assistant MQTT Module
///
/// Implements the object-safe `MqttModule` trait for integrating with Home Assistant.
/// Handles discovery, state publishing, and command processing for light and number entities.
///
/// # Object Safety
///
/// This type implements `MqttModule` which is dyn-compatible. You can use it as:
/// - A concrete type for maximum performance
/// - `&mut dyn MqttModule` for trait objects
///
/// # Type Parameters
///
/// - `MAX_LIGHTS`: Maximum number of light entities
/// - `MAX_NUMBERS`: Maximum number of number entities
/// - `BUF_SIZE`: Size of internal serialization buffer
///
/// # Example
///
/// ```ignore
/// let mut ha_module: HaModule<4, 4, 512> = HaModule::new(Duration::from_secs(30));
///
/// ha_module.add_light(LightRegistration {
///     entity: LIGHT_ENTITY.clone(),
///     provide_state: get_light_state,
///     on_command: handle_light_command,
/// })?;
/// ```
pub struct HaModule<'a, const MAX_LIGHTS: usize, const MAX_NUMBERS: usize, const BUF_SIZE: usize> {
    lights: Vec<LightEntry<'a>, MAX_LIGHTS>,
    numbers: Vec<NumberEntry<'a>, MAX_NUMBERS>,
    buf: [u8; BUF_SIZE],
    tick_interval: Duration,
    needs_publish: bool,
}

impl<'a, const MAX_LIGHTS: usize, const MAX_NUMBERS: usize, const BUF_SIZE: usize>
    HaModule<'a, MAX_LIGHTS, MAX_NUMBERS, BUF_SIZE>
{
    /// Create a new Home Assistant module.
    ///
    /// # Arguments
    ///
    /// - `tick_interval`: How often to publish state updates and re-announce discovery
    pub fn new(tick_interval: Duration) -> Self {
        Self {
            lights: Vec::new(),
            numbers: Vec::new(),
            buf: [0u8; BUF_SIZE],
            tick_interval,
            needs_publish: false,
        }
    }

    /// Add a light entity registration.
    ///
    /// # Arguments
    ///
    /// - `reg`: The light registration containing entity info and callbacks
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err(Error::MaxEntitiesReached)` if the light limit is reached.
    pub fn add_light(&mut self, reg: LightRegistration<'a>) -> Result<(), Error<()>> {
        let command_topic = ha::command_topic(reg.entity.device.id, reg.entity.id);
        self.lights
            .push(LightEntry {
                entity: reg.entity,
                command_topic,
                provide_state: reg.provide_state,
                on_command: reg.on_command,
            })
            .map_err(|_| Error::MaxEntitiesReached)
    }

    /// Add a number entity registration.
    ///
    /// # Arguments
    ///
    /// - `reg`: The number registration containing entity info and callbacks
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err(Error::MaxEntitiesReached)` if the number limit is reached.
    pub fn add_number(&mut self, reg: NumberRegistration<'a>) -> Result<(), Error<()>> {
        let command_topic = ha::command_topic(reg.entity.device.id, reg.entity.id);
        self.numbers
            .push(NumberEntry {
                entity: reg.entity,
                command_topic,
                provide_state: reg.provide_state,
                on_command: reg.on_command,
            })
            .map_err(|_| Error::MaxEntitiesReached)
    }

    /// Announce all registered entities to Home Assistant.
    ///
    /// Publishes discovery configs for all lights and numbers.
    fn announce_all(&mut self, outbox: &mut dyn PublishOutbox) {
        // Announce lights
        for i in 0..self.lights.len() {
            let entity = self.lights[i].entity.clone();
            self.announce_light(outbox, &entity);
        }

        // Announce numbers
        for i in 0..self.numbers.len() {
            let entity = self.numbers[i].entity.clone();
            self.announce_number(outbox, &entity);
        }
    }

    /// Announce a single light entity.
    fn announce_light(&mut self, outbox: &mut dyn PublishOutbox, entity: &LightEntity<'a>) {
        let discovery_ctx: LightDiscoveryContext<'_, MAX_TOPIC_LEN> =
            LightDiscoveryContext::new(entity);
        let identifier_str = discovery_ctx.identifier.as_str();
        let identifiers = [identifier_str];
        let config = ha::light_to_discovery(entity, &discovery_ctx, &identifiers);

        if let Ok(json_len) = serde_json_core::to_slice(&config, &mut self.buf) {
            outbox.publish(
                discovery_ctx.config_topic.as_str(),
                &self.buf[..json_len],
                QoS::AtLeastOnce,
            );
        }
    }

    /// Announce a single number entity.
    fn announce_number(&mut self, outbox: &mut dyn PublishOutbox, entity: &NumberEntity<'a>) {
        let discovery_ctx: NumberDiscoveryContext<MAX_TOPIC_LEN> =
            NumberDiscoveryContext::new(entity);
        let identifier_str = discovery_ctx.identifier.as_str();
        let identifiers = [identifier_str];
        let config = ha::number_to_discovery(entity, &discovery_ctx, &identifiers);

        if let Ok(json_len) = serde_json_core::to_slice(&config, &mut self.buf) {
            outbox.publish(
                discovery_ctx.config_topic.as_str(),
                &self.buf[..json_len],
                QoS::AtLeastOnce,
            );
        }
    }

    /// Publish current states for all registered entities.
    fn publish_states(&mut self, outbox: &mut dyn PublishOutbox) {
        // Publish light states
        for i in 0..self.lights.len() {
            let entry = &self.lights[i];
            let state = (entry.provide_state)();
            let ha_state = ha::light_state_to_ha(&state);
            let topic: String<MAX_TOPIC_LEN> =
                ha::state_topic(entry.entity.device.id, entry.entity.id);

            if let Ok(json_len) = serde_json_core::to_slice(&ha_state, &mut self.buf) {
                outbox.publish(topic.as_str(), &self.buf[..json_len], QoS::AtMostOnce);
            }
        }

        // Publish number states
        for i in 0..self.numbers.len() {
            let entry = &self.numbers[i];
            let value = (entry.provide_state)();
            let topic: String<MAX_TOPIC_LEN> =
                ha::state_topic(entry.entity.device.id, entry.entity.id);

            let len = format_i32(value, &mut self.buf);
            outbox.publish(topic.as_str(), &self.buf[..len], QoS::AtMostOnce);
        }
    }
}

impl<const MAX_LIGHTS: usize, const MAX_NUMBERS: usize, const BUF_SIZE: usize> MqttModule
    for HaModule<'_, MAX_LIGHTS, MAX_NUMBERS, BUF_SIZE>
{
    fn register(&self, collector: &mut dyn TopicCollector) {
        for entry in &self.lights {
            collector.add(entry.command_topic.as_str());
        }
        for entry in &self.numbers {
            collector.add(entry.command_topic.as_str());
        }
    }

    fn on_message(&mut self, msg: &Publish<'_>) {
        // Try to match against light command topics
        for entry in &self.lights {
            if msg.topic == entry.command_topic.as_str() {
                if let Ok((ha_cmd, _)) =
                    serde_json_core::from_slice::<HaLightCommand<'_>>(msg.payload)
                {
                    let cmd = ha::ha_command_to_light(&ha_cmd);
                    (entry.on_command)(&cmd);
                    self.needs_publish = true;
                    return;
                }
            }
        }

        // Try to match against number command topics
        for entry in &self.numbers {
            if msg.topic == entry.command_topic.as_str() {
                if let Ok(value_str) = core::str::from_utf8(msg.payload) {
                    if let Ok(value) = value_str.trim().parse::<i32>() {
                        (entry.on_command)(value);
                        self.needs_publish = true;
                        return;
                    }
                }
            }
        }
    }

    fn on_tick(&mut self, outbox: &mut dyn PublishOutbox) -> Duration {
        self.needs_publish = false;
        self.announce_all(outbox);
        self.publish_states(outbox);
        self.tick_interval
    }

    fn on_start(&mut self, outbox: &mut dyn PublishOutbox) {
        self.announce_all(outbox);
        self.publish_states(outbox);
    }

    fn needs_immediate_publish(&self) -> bool {
        self.needs_publish
    }
}

/// Format an i32 to a byte buffer, returning the number of bytes written
fn format_i32(value: i32, buf: &mut [u8]) -> usize {
    struct SliceWriter<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }

    impl Write for SliceWriter<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            if self.pos + bytes.len() > self.buf.len() {
                return Err(core::fmt::Error);
            }
            self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
            self.pos += bytes.len();
            Ok(())
        }
    }

    let mut writer = SliceWriter { buf, pos: 0 };
    let _ = write!(writer, "{}", value);
    writer.pos
}
