use esp_storage::FlashStorage;

use crate::config::DeviceConfig;
use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
use crate::domain::ports::{
    ConfigurationUsecasesPort, LightIntentApplier, LightStateHandler, LightStateReader,
    LightUsecasesPort, PersistenceHandler, PersistentLightStateUpdater,
};
// use crate::infrastructure::services::http_server::{HttpConnection, HttpResult, HttpServerError};
// use crate::infrastructure::services::{OtaError, update_from_http};

pub struct LightUsecases<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> {
    state: S,
    persistence: P,
}

impl<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> LightUsecases<S, P> {
    pub fn new(state: S, persistence: P) -> Self {
        Self { state, persistence }
    }
}

impl<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> LightIntentApplier
    for LightUsecases<S, P>
{
    fn apply_intent(&mut self, intent: LightChangeIntent) -> Result<(), ()> {
        self.state.apply_intent(intent)?;
        Ok(())
    }
}

impl<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> LightStateReader
    for LightUsecases<S, P>
{
    fn get_light_state(&self) -> Option<LightState> {
        self.state.get_light_state()
    }
}

impl<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> LightUsecasesPort
    for LightUsecases<S, P>
{
    fn apply_intent_and_persist(&mut self, intent: LightChangeIntent) -> Result<(), ()> {
        self.state.apply_intent(intent)?;
        let _ = self
            .persistence
            .update_persistent_light_state(self.state.get_light_state().ok_or(())?);
        Ok(())
    }
}

pub struct ConfigurationUsecases<P: PersistenceHandler + Send + Sync> {
    persistence: P,
}

impl<P: PersistenceHandler + Send + Sync> ConfigurationUsecases<P> {
    pub fn new(persistence: P) -> Self {
        Self { persistence }
    }
}

impl<P: PersistenceHandler + Send + Sync> ConfigurationUsecasesPort for ConfigurationUsecases<P> {
    fn get_device_config(&self) -> Option<DeviceConfig> {
        self.persistence
            .get_persistent_data()
            .map(|(_, _, config)| config)
    }

    fn set_device_config(&mut self, config: &DeviceConfig) -> Option<()> {
        self.persistence.persist_device_config(config)
    }
}
