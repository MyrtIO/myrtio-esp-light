use crate::{
    config::LightConfig,
    domain::{dto::LightChangeIntent, entity::LightState},
};

#[derive(Debug)]
pub enum LightError {
    /// Service processing to many requests at the same time
    Busy,
    /// Error persisting the light state
    PersistenceError,
    /// Too many LEDs
    TooManyLEDs,
}

/// Reader interface for the light state
pub trait LightStateReader {
    /// Get the current light state
    fn get_light_state(&self) -> LightState;
}

/// Applier interface for the light intent
pub trait LightStateChanger {
    /// Apply a light change intent
    fn apply_light_intent(
        &self,
        intent: LightChangeIntent,
    ) -> Result<(), LightError>;
}

pub trait LightConfigChanger {
    /// Apply a light config change intent
    fn set_config(&mut self, config: LightConfig) -> Result<(), LightError>;
}

/// Trait for the light usecases state handler
pub trait LightStateHandler:
    LightStateReader
    + LightConfigChanger
    + LightStateChanger
    + Sync
    + Send
{
}

/// Port interface for the light usecases
pub trait LightUsecasesPort:
    LightStateReader + LightConfigChanger + LightStateChanger + Sync + Send
{
    fn apply_intent_and_persist(
        &mut self,
        intent: LightChangeIntent,
    ) -> Result<(), LightError>;
}
