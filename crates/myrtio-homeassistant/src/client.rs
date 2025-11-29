//! Home Assistant MQTT Client
//!
//! Wraps an MqttClient and provides high-level Home Assistant integration.

use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Ticker};
use heapless::{String, Vec};
use myrtio_mqtt::{
    client::{MqttClient, MqttEvent},
    transport::{MqttTransport, TransportError},
    QoS,
};
use serde::Serialize;

use crate::{
    device::Device,
    entity::{
        light::{LightCommand, LightEntity, LightRegistration, LightState},
        number::{NumberEntity, NumberRegistration},
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
    device: &'a Device<'a>,
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
    device: &'a Device<'a>,
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

/// Internal storage for light registration
struct LightEntry<'a> {
    entity: LightEntity<'a>,
    provide_state: fn() -> LightState<'static>,
    on_command: fn(LightCommand<'_>),
}

/// Internal storage for number registration
struct NumberEntry<'a> {
    entity: NumberEntity<'a>,
    provide_state: fn() -> i32,
    on_command: fn(i32),
}

/// Home Assistant MQTT Client
///
/// Wraps an MqttClient and provides high-level operations for:
/// - Adding entities with callbacks
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
    lights: Vec<LightEntry<'a>, MAX_LIGHTS>,
    numbers: Vec<NumberEntry<'a>, MAX_NUMBERS>,
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
    /// Create a new Home Assistant client from a Device and MQTT client
    ///
    /// # Arguments
    /// * `device` - The device (namespace is taken from device)
    /// * `mqtt` - The underlying MQTT client
    pub fn new(_device: &'a Device<'a>, mqtt: MqttClient<'a, T, MAX_TOPICS, BUF_SIZE>) -> Self {
        Self {
            mqtt,
            lights: Vec::new(),
            numbers: Vec::new(),
            buf: [0u8; BUF_SIZE],
        }
    }

    /// Add a light entity registration
    pub fn add_light(&mut self, reg: LightRegistration<'a>) -> Result<(), HaError<T::Error>> {
        self.lights
            .push(LightEntry {
                entity: reg.entity,
                provide_state: reg.provide_state,
                on_command: reg.on_command,
            })
            .map_err(|_| HaError::MaxEntitiesReached)
    }

    /// Add a number entity registration
    pub fn add_number(&mut self, reg: NumberRegistration<'a>) -> Result<(), HaError<T::Error>> {
        self.numbers
            .push(NumberEntry {
                entity: reg.entity,
                provide_state: reg.provide_state,
                on_command: reg.on_command,
            })
            .map_err(|_| HaError::MaxEntitiesReached)
    }

    /// Announce all registered entities to Home Assistant
    ///
    /// Publishes discovery configs and subscribes to command topics
    pub async fn announce_all(&mut self) -> Result<(), HaError<T::Error>> {
        // Announce lights
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
        let state_topic: String<128> = entity.state_topic();
        let command_topic: String<128> = entity.command_topic();
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
        let state_topic: String<128> = entity.state_topic();
        let command_topic: String<128> = entity.command_topic();
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
    /// Calls each entity's provide_state callback and publishes the result
    pub async fn publish_states(&mut self) -> Result<(), HaError<T::Error>> {
        // Publish light states
        let light_count = self.lights.len();
        for i in 0..light_count {
            let entry = &self.lights[i];
            let state = (entry.provide_state)();
            let topic: String<128> = entry.entity.state_topic();

            let json = serde_json_core::to_slice(&state, &mut self.buf)
                .map_err(|_| HaError::Serialization)?;

            self.mqtt
                .publish(topic.as_str(), &self.buf[..json], QoS::AtMostOnce)
                .await?;
        }

        // Publish number states
        let number_count = self.numbers.len();
        for i in 0..number_count {
            let entry = &self.numbers[i];
            let value = (entry.provide_state)();
            let topic: String<128> = entry.entity.state_topic();

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
    async fn poll_internal(&mut self) -> Result<bool, HaError<T::Error>> {
        let event = self.mqtt.poll().await?;

        if let Some(MqttEvent::Publish(msg)) = event {
            // Try to match against light command topics
            for entry in &self.lights {
                let cmd_topic: String<128> = entry.entity.command_topic();
                if msg.topic == cmd_topic.as_str() {
                    // Parse the command JSON
                    if let Ok((cmd, _)) =
                        serde_json_core::from_slice::<LightCommand<'_>>(msg.payload)
                    {
                        (entry.on_command)(cmd);
                        return Ok(true);
                    }
                }
            }

            // Try to match against number command topics
            for entry in &self.numbers {
                let cmd_topic: String<128> = entry.entity.command_topic();
                if msg.topic == cmd_topic.as_str() {
                    // Parse the number value
                    if let Ok(value_str) = core::str::from_utf8(msg.payload) {
                        if let Ok(value) = value_str.trim().parse::<i32>() {
                            (entry.on_command)(value);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Run the Home Assistant client
    ///
    /// This method:
    /// 1. Connects to the MQTT broker
    /// 2. Announces all entities to Home Assistant
    /// 3. Publishes initial states
    /// 4. Enters a loop that polls for commands and periodically re-announces/publishes state
    ///
    /// # Arguments
    /// * `interval` - How often to re-announce and publish state
    ///
    /// # Returns
    /// This method runs forever unless an error occurs
    pub async fn run(&mut self, interval: Duration) -> Result<(), HaError<T::Error>> {
        // Connect to MQTT broker
        self.mqtt.connect().await?;

        // Initial announce and state publish
        self.announce_all().await?;
        self.publish_states().await?;

        let mut ticker = Ticker::every(interval);

        // Main loop
        loop {
            match select(self.poll_internal(), ticker.next()).await {
                Either::First(result) => {
                    if result? {
                        // Command was handled, publish updated state
                        self.publish_states().await?;
                    }
                }
                Either::Second(_) => {
                    // Periodic: re-announce and publish state
                    self.announce_all().await?;
                    self.publish_states().await?;
                }
            }
        }
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
