use crate::config::DeviceConfig;
use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
// use crate::infrastructure::services::http_server::HttpConnection;
use crate::infrastructure::services::OtaError;

/// Reader interface for the light state
pub trait LightStateReader {
    /// Get the current light state
    fn get_light_state(&self) -> Option<LightState>;
}

/// Applier interface for the light intent
pub trait LightIntentApplier {
    /// Apply a light change intent
    fn apply_intent(&mut self, intent: LightChangeIntent) -> Result<(), ()>;
}

/// Port interface for the light usecases
pub trait LightUsecasesPort:
    LightStateReader + LightIntentApplier + Sync + Send
{
    fn apply_intent_and_persist(&mut self, intent: LightChangeIntent) -> Result<(), ()>;
}

/// Trait for the light usecases state handler
pub(crate) trait LightStateHandler:
    LightStateReader + LightIntentApplier + Sync + Send
{
}

/// Writer interface for the persisting light state to the power-loss-safe storage
pub(crate) trait PersistentLightStateUpdater {
    /// Set the current light state to the persistent storage
    fn update_persistent_light_state(&mut self, state: LightState) -> Result<(), ()>;
}

// /// Trait for the light usecases persistence handler
// pub(crate) trait PersistentLightStateHandler: Sync + Send {
//     /// Get the persistent light state
//     async fn get_persistent_light_state(&self) -> Option<LightState>;

//     /// Set the current light state to the persistent storage
//     async fn save_persistent_light_state(&mut self, state: LightState) -> Result<(), ()>;
// }

/// Trait for the boot controller
pub trait OnBootHandler: Sync + Send {
    /// On boot
    fn on_boot(&self, stored_state: Option<LightState>);
}

pub trait PersistenceHandler: Sync + Send {
    /// Get the persistent data
    fn get_persistent_data(&self) -> Option<(u8, LightState, DeviceConfig)>;

    /// Set the persistent data
    fn persist_light_state(&mut self, light_state: LightState) -> Option<()>;

    /// Persist the device config
    fn persist_device_config(&mut self, config: &DeviceConfig) -> Option<()>;

    /// Persist the boot count
    fn persist_boot_count(&mut self, boot_count: u8) -> Option<()>;
}

pub trait ConfigurationUsecasesPort: Sync + Send {
    /// Get the device config
    fn get_device_config(&self) -> Option<DeviceConfig>;

    /// Set the device config
    fn set_device_config(&mut self, config: &DeviceConfig) -> Option<()>;
}

// pub trait OtaUsecasesPort: Sync + Send {
//     /// Update the firmware from the HTTP server
//     async fn update_from_http(&mut self, conn: &mut HttpConnection<'_>) -> Result<(), OtaError>;
// }
