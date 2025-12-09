//! Topic generation utilities for Home Assistant MQTT integration

use core::fmt::Write;
use heapless::String;

/// Generate a Home Assistant discovery config topic
///
/// Format: `homeassistant/{component}/{device_id}_{entity_id}/config`
pub fn config_topic<const N: usize>(
    component: &str,
    device_id: &str,
    entity_id: &str,
) -> String<N> {
    let mut topic = String::new();
    let _ = write!(
        topic,
        "homeassistant/{}/{}_{}/config",
        component, device_id, entity_id
    );
    topic
}

/// Generate a state topic for an entity
///
/// Format: `{device_id}/{entity_id}`
pub fn state_topic<const N: usize>(device_id: &str, entity_id: &str) -> String<N> {
    let mut topic = String::new();
    let _ = write!(topic, "{}/{}", device_id, entity_id);
    topic
}

/// Generate a command topic for an entity
///
/// Format: `{device_id}/{entity_id}/set`
pub fn command_topic<const N: usize>(device_id: &str, entity_id: &str) -> String<N> {
    let mut topic: String<N> = state_topic(device_id, entity_id);
    let _ = write!(topic, "/set");
    topic
}

/// Generate a unique ID for an entity
///
/// Format: `{device_id}_{entity_id}`
pub fn unique_id<const N: usize>(device_id: &str, entity_id: &str) -> String<N> {
    let mut id = String::new();
    let _ = write!(id, "{}_{}", device_id, entity_id);
    id
}
