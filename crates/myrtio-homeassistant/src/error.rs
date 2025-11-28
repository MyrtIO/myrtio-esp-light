//! Error types for the Home Assistant MQTT library

use core::fmt;
use myrtio_mqtt::error::MqttError;

/// Error type for Home Assistant operations
#[derive(Debug)]
pub enum HaError<E> {
    /// MQTT communication error
    Mqtt(MqttError<E>),
    /// JSON serialization error
    Serialization,
    /// JSON deserialization error
    Deserialization,
    /// Buffer too small
    BufferTooSmall,
    /// Maximum entities reached
    MaxEntitiesReached,
    /// Entity not found
    EntityNotFound,
}

impl<E: fmt::Debug> fmt::Display for HaError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HaError::Mqtt(e) => write!(f, "MQTT error: {:?}", e),
            HaError::Serialization => write!(f, "JSON serialization error"),
            HaError::Deserialization => write!(f, "JSON deserialization error"),
            HaError::BufferTooSmall => write!(f, "Buffer too small"),
            HaError::MaxEntitiesReached => write!(f, "Maximum entities reached"),
            HaError::EntityNotFound => write!(f, "Entity not found"),
        }
    }
}

impl<E> From<MqttError<E>> for HaError<E> {
    fn from(e: MqttError<E>) -> Self {
        HaError::Mqtt(e)
    }
}

