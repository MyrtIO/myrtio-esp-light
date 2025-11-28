//! Home Assistant MQTT Client
//!
//! Wraps an MqttClient and provides high-level Home Assistant integration.

use heapless::{String, Vec};
use myrtio_mqtt::{
    client::{MqttClient, MqttEvent},
    transport::{MqttTransport, TransportError},
    QoS,
};
use serde::Serialize;

use crate::{
    entity::{
        light::{LightCommand, LightEntity, LightState},
        number::NumberEntity,
    },
    error::HaError,
};

/// Helper to check if bool is false for serde skip
fn is_false(v: &bool) -> bool {
    !*v
}

/// Configuration for discovery message serialization
#[derive(Serialize)]
struct LightDiscoveryConfig<'a, 'b> {
    name: &'a str,
    unique_id: &'b str,
    schema: &'a str,
    state_topic: &'b str,
    command_topic: &'b str,
    device: &'a crate::device::Device<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<&'a str>,
    #[serde(skip_serializing_if = "is_false")]
    brightness: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    effect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effect_list: Option<&'a [&'a str]>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    supported_color_modes: &'b [&'b str],
    #[serde(skip_serializing_if = "Option::is_none")]
    min_mireds: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_mireds: Option<u16>,
}

/// Configuration for number discovery message serialization
#[derive(Serialize)]
struct NumberDiscoveryConfig<'a, 'b> {
    name: &'a str,
    unique_id: &'b str,
    state_topic: &'b str,
    command_topic: &'b str,
    device: &'a crate::device::Device<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_of_measurement: Option<&'a str>,
    min: i32,
    max: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    step: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<&'a str>,
}

/// Registration for a light entity with callbacks
pub struct LightRegistration<'a> {
    pub entity: LightEntity<'a>,
    pub get_state: fn() -> LightState<'static>,
    pub handle_command: fn(LightCommand<'_>),
}

/// Registration for a number entity with callbacks
pub struct NumberRegistration<'a> {
    pub entity: NumberEntity<'a>,
    pub get_state: fn() -> i32,
    pub handle_command: fn(i32),
}

/// Home Assistant MQTT Client
///
/// Wraps an MqttClient and provides high-level operations for:
/// - Registering entities with callbacks
/// - Publishing discovery configs
/// - Publishing entity states
/// - Handling incoming commands
pub struct HomeAssistantClient<
    'a,
    T,
    const MAX_TOPICS: usize,
    const BUF_SIZE: usize,
    const MAX_LIGHTS: usize,
    const MAX_NUMBERS: usize,
> where
    T: MqttTransport,
{
    mqtt: MqttClient<'a, T, MAX_TOPICS, BUF_SIZE>,
    namespace: &'a str,
    lights: Vec<LightRegistration<'a>, MAX_LIGHTS>,
    numbers: Vec<NumberRegistration<'a>, MAX_NUMBERS>,
    buf: [u8; BUF_SIZE],
}

impl<
        'a,
        T,
        const MAX_TOPICS: usize,
        const BUF_SIZE: usize,
        const MAX_LIGHTS: usize,
        const MAX_NUMBERS: usize,
    > HomeAssistantClient<'a, T, MAX_TOPICS, BUF_SIZE, MAX_LIGHTS, MAX_NUMBERS>
where
    T: MqttTransport,
    T::Error: TransportError,
{
    /// Create a new Home Assistant client wrapping the given MQTT client
    ///
    /// # Arguments
    /// * `mqtt` - The underlying MQTT client
    /// * `namespace` - Base namespace for topics (e.g., "myrtlight")
    pub fn new(mqtt: MqttClient<'a, T, MAX_TOPICS, BUF_SIZE>, namespace: &'a str) -> Self {
        Self {
            mqtt,
            namespace,
            lights: Vec::new(),
            numbers: Vec::new(),
            buf: [0u8; BUF_SIZE],
        }
    }

    /// Register a light entity with state and command callbacks
    ///
    /// # Arguments
    /// * `entity` - The light entity configuration
    /// * `get_state` - Function to get current state for publishing
    /// * `handle_command` - Function to handle incoming commands
    pub fn register_light(
        &mut self,
        entity: LightEntity<'a>,
        get_state: fn() -> LightState<'static>,
        handle_command: fn(LightCommand<'_>),
    ) -> Result<(), HaError<T::Error>> {
        self.lights
            .push(LightRegistration {
                entity,
                get_state,
                handle_command,
            })
            .map_err(|_| HaError::MaxEntitiesReached)
    }

    /// Register a number entity with state and command callbacks
    ///
    /// # Arguments
    /// * `entity` - The number entity configuration
    /// * `get_state` - Function to get current value for publishing
    /// * `handle_command` - Function to handle incoming value changes
    pub fn register_number(
        &mut self,
        entity: NumberEntity<'a>,
        get_state: fn() -> i32,
        handle_command: fn(i32),
    ) -> Result<(), HaError<T::Error>> {
        self.numbers
            .push(NumberRegistration {
                entity,
                get_state,
                handle_command,
            })
            .map_err(|_| HaError::MaxEntitiesReached)
    }

    /// Announce all registered entities to Home Assistant
    ///
    /// Publishes discovery configs and subscribes to command topics
    pub async fn announce_all(&mut self) -> Result<(), HaError<T::Error>> {
        // Announce lights - collect indices first to avoid borrow issues
        let light_count = self.lights.len();
        for i in 0..light_count {
            let entity = self.lights[i].entity.clone();
            self.announce_light(&entity).await?;
        }

        // Announce numbers
        let number_count = self.numbers.len();
        for i in 0..number_count {
            let entity = self.numbers[i].entity.clone();
            self.announce_number(&entity).await?;
        }

        Ok(())
    }

    /// Announce a single light entity
    async fn announce_light(&mut self, entity: &LightEntity<'a>) -> Result<(), HaError<T::Error>> {
        // Build strings for serialization
        let unique_id: String<64> = entity.unique_id();
        let state_topic: String<128> = entity.state_topic(self.namespace);
        let command_topic: String<128> = entity.command_topic(self.namespace);
        let config_topic: String<128> = entity.config_topic();

        // Build color modes list
        let mut color_modes_strs: Vec<&str, 4> = Vec::new();
        for mode in entity.color_modes {
            let _ = color_modes_strs.push(mode.as_str());
        }

        let config = LightDiscoveryConfig {
            name: entity.name,
            unique_id: unique_id.as_str(),
            schema: "json",
            state_topic: state_topic.as_str(),
            command_topic: command_topic.as_str(),
            device: entity.device,
            icon: entity.icon,
            brightness: entity.brightness,
            effect: entity.effects.map(|_| true),
            effect_list: entity.effects,
            supported_color_modes: color_modes_strs.as_slice(),
            min_mireds: entity.min_mireds,
            max_mireds: entity.max_mireds,
        };

        // Serialize to buffer
        let json = serde_json_core::to_slice(&config, &mut self.buf)
            .map_err(|_| HaError::Serialization)?;

        // Publish discovery config
        self.mqtt
            .publish(config_topic.as_str(), &self.buf[..json], QoS::AtLeastOnce)
            .await?;

        // Subscribe to command topic
        self.mqtt
            .subscribe(command_topic.as_str(), QoS::AtLeastOnce)
            .await?;

        Ok(())
    }

    /// Announce a single number entity
    async fn announce_number(
        &mut self,
        entity: &NumberEntity<'a>,
    ) -> Result<(), HaError<T::Error>> {
        // Build strings for serialization
        let unique_id: String<64> = entity.unique_id();
        let state_topic: String<128> = entity.state_topic(self.namespace);
        let command_topic: String<128> = entity.command_topic(self.namespace);
        let config_topic: String<128> = entity.config_topic();

        let config = NumberDiscoveryConfig {
            name: entity.name,
            unique_id: unique_id.as_str(),
            state_topic: state_topic.as_str(),
            command_topic: command_topic.as_str(),
            device: entity.device,
            icon: entity.icon,
            device_class: entity.device_class,
            unit_of_measurement: entity.unit,
            min: entity.min,
            max: entity.max,
            step: entity.step,
            mode: entity.mode,
        };

        // Serialize to buffer
        let json = serde_json_core::to_slice(&config, &mut self.buf)
            .map_err(|_| HaError::Serialization)?;

        // Publish discovery config
        self.mqtt
            .publish(config_topic.as_str(), &self.buf[..json], QoS::AtLeastOnce)
            .await?;

        // Subscribe to command topic
        self.mqtt
            .subscribe(command_topic.as_str(), QoS::AtLeastOnce)
            .await?;

        Ok(())
    }

    /// Publish current states for all registered entities
    ///
    /// Calls each entity's get_state callback and publishes the result
    pub async fn publish_states(&mut self) -> Result<(), HaError<T::Error>> {
        // Publish light states
        let light_count = self.lights.len();
        for i in 0..light_count {
            let reg = &self.lights[i];
            let state = (reg.get_state)();
            let topic: String<128> = reg.entity.state_topic(self.namespace);

            let json = serde_json_core::to_slice(&state, &mut self.buf)
                .map_err(|_| HaError::Serialization)?;

            self.mqtt
                .publish(topic.as_str(), &self.buf[..json], QoS::AtMostOnce)
                .await?;
        }

        // Publish number states
        let number_count = self.numbers.len();
        for i in 0..number_count {
            let reg = &self.numbers[i];
            let value = (reg.get_state)();
            let topic: String<128> = reg.entity.state_topic(self.namespace);

            // Format number to buffer
            let len = format_i32(value, &mut self.buf);

            self.mqtt
                .publish(topic.as_str(), &self.buf[..len], QoS::AtMostOnce)
                .await?;
        }

        Ok(())
    }

    /// Poll for incoming messages and dispatch to handlers
    ///
    /// Returns Ok(true) if a command was processed, Ok(false) if no message
    pub async fn poll(&mut self) -> Result<bool, HaError<T::Error>> {
        let event = self.mqtt.poll().await?;

        if let Some(MqttEvent::Publish(msg)) = event {
            // Try to match against light command topics
            for reg in &self.lights {
                let cmd_topic: String<128> = reg.entity.command_topic(self.namespace);
                if msg.topic == cmd_topic.as_str() {
                    // Parse the command JSON
                    if let Ok((cmd, _)) =
                        serde_json_core::from_slice::<LightCommand<'_>>(msg.payload)
                    {
                        (reg.handle_command)(cmd);
                        return Ok(true);
                    }
                }
            }

            // Try to match against number command topics
            for reg in &self.numbers {
                let cmd_topic: String<128> = reg.entity.command_topic(self.namespace);
                if msg.topic == cmd_topic.as_str() {
                    // Parse the number value
                    if let Ok(value_str) = core::str::from_utf8(msg.payload) {
                        if let Ok(value) = value_str.trim().parse::<i32>() {
                            (reg.handle_command)(value);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get a reference to the underlying MQTT client
    pub fn mqtt(&self) -> &MqttClient<'a, T, MAX_TOPICS, BUF_SIZE> {
        &self.mqtt
    }

    /// Get a mutable reference to the underlying MQTT client
    pub fn mqtt_mut(&mut self) -> &mut MqttClient<'a, T, MAX_TOPICS, BUF_SIZE> {
        &mut self.mqtt
    }
}

/// Format an i32 to a byte buffer, returning the number of bytes written
fn format_i32(value: i32, buf: &mut [u8]) -> usize {
    use core::fmt::Write;

    struct SliceWriter<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }

    impl<'a> Write for SliceWriter<'a> {
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
