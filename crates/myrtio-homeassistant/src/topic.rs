//! Topic generation utilities for Home Assistant MQTT integration

use heapless::String;

/// Default topic buffer size
pub const DEFAULT_TOPIC_SIZE: usize = 128;

/// Generate a Home Assistant discovery config topic
///
/// Format: `homeassistant/{component}/{device_id}_{entity_id}/config`
pub fn config_topic<const N: usize>(
    component: &str,
    device_id: &str,
    entity_id: &str,
) -> String<N> {
    let mut topic = String::new();
    let _ = topic.push_str("homeassistant/");
    let _ = topic.push_str(component);
    let _ = topic.push('/');
    let _ = topic.push_str(device_id);
    let _ = topic.push('_');
    let _ = topic.push_str(entity_id);
    let _ = topic.push_str("/config");
    topic
}

/// Generate a state topic for an entity
///
/// Format: `{namespace}/{entity_id}`
pub fn state_topic<const N: usize>(namespace: &str, entity_id: &str) -> String<N> {
    let mut topic = String::new();
    let _ = topic.push_str(namespace);
    let _ = topic.push('/');
    let _ = topic.push_str(entity_id);
    topic
}

/// Generate a command topic for an entity
///
/// Format: `{namespace}/{entity_id}/set`
pub fn command_topic<const N: usize>(namespace: &str, entity_id: &str) -> String<N> {
    let mut topic: String<N> = state_topic(namespace, entity_id);
    let _ = topic.push_str("/set");
    topic
}

/// Generate a unique ID for an entity
///
/// Format: `{device_id}_{entity_id}`
pub fn unique_id<const N: usize>(device_id: &str, entity_id: &str) -> String<N> {
    let mut id = String::new();
    let _ = id.push_str(device_id);
    let _ = id.push('_');
    let _ = id.push_str(entity_id);
    id
}

