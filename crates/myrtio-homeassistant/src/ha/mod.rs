//! Home Assistant wire-format types
//!
//! This module contains structures that exactly match the JSON schema
//! expected by Home Assistant MQTT integration for discovery, state, and commands.

pub mod command;
pub mod discovery;
pub mod mapping;
pub mod state;
pub mod topic;

pub use command::HaLightCommand;
pub use discovery::{HaDeviceInfo, HaLightDiscovery, HaNumberDiscovery};
pub use mapping::{
    LightDiscoveryContext, NumberDiscoveryContext, ha_command_to_light, light_state_to_ha,
    light_to_discovery, number_to_discovery,
};
pub use state::HaLightState;
pub use topic::{command_topic, config_topic, state_topic, unique_id};
