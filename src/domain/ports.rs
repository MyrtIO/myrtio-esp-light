use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;

/// Reader interface for the light state
pub(crate) trait LightStateReader {
    /// Get the current light state
    fn get_light_state(&self) -> Option<LightState>;
}

/// Applier interface for the light intent
pub(crate) trait LightIntentApplier {
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

/// Trait for the light usecases persistence handler
pub(crate) trait PersistentLightStateHandler: Sync + Send {
    /// Get the persistent light state
    async fn get_persistent_light_state(&self) -> Option<LightState>;

    /// Set the current light state to the persistent storage
    async fn save_persistent_light_state(&mut self, state: LightState) -> Result<(), ()>;
}

/// Trait for the boot controller
pub trait OnBootHandler: Sync + Send {
    /// On boot
    fn on_boot(&self, stored_state: Option<LightState>);
}
