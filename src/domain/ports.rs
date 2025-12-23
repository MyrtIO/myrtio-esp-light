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
pub trait LightUsecasesPort: LightStateReader + LightIntentApplier + Sync + Send {
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
pub trait OnBootHandler {
    /// Called when the boot process starts
    fn on_boot_start(&mut self);

    /// Called when the light is ready
    fn on_light_ready(&mut self);

    /// Called when the boot process ends
    fn on_boot_end(&mut self);

    /// Called when the magic timeout occurs. This is a fallback mechanism to ensure the boot process completes if the firmware is corrupted.
    fn on_magic_timeout(&mut self);
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

    /// Get the reboot count
    fn get_reboot_count(&self) -> Option<u8>;

    /// Set the reboot count
    fn increment_reboot_count(&mut self) -> Option<u8>;

    /// Reset the reboot count
    fn reset_reboot_count(&mut self) -> Option<()>;
}

pub trait ConfigurationUsecasesPort: Sync + Send {
    /// Get the device config
    fn get_device_config(&self) -> Option<DeviceConfig>;

    /// Set the device config
    fn set_device_config(&mut self, config: &DeviceConfig) -> Option<()>;
}

pub trait BootManagerPort {
    /// Get the boot slot
    fn boot_system(&mut self) -> Option<()>;

    /// Reboot the system
    fn boot_factory(&mut self) -> Option<()>;
}
