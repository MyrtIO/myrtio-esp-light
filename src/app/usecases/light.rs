use crate::{
    config::LightConfig,
    domain::{
        dto::{LightChangeIntent, PersistentData},
        entity::LightState,
        ports::{
            LightConfigChanger,
            light::{
                LightError,
                LightStateChanger,
                LightStateHandler,
                LightStateReader,
                LightUsecasesPort,
            },
            persistence::PersistentDataHandler,
        },
    },
};

pub struct LightUsecases<S: LightStateHandler, P: PersistentDataHandler> {
    state: S,
    persistence: P,
}

impl<S: LightStateHandler, P: PersistentDataHandler> LightUsecases<S, P> {
    pub fn new(state: S, persistence: P) -> Self {
        Self { state, persistence }
    }
}

impl<S: LightStateHandler, P: PersistentDataHandler> LightStateChanger
    for LightUsecases<S, P>
{
    fn apply_light_intent(
        &self,
        intent: LightChangeIntent,
    ) -> Result<(), LightError> {
        self.state.apply_light_intent(intent)?;
        Ok(())
    }
}

impl<S: LightStateHandler, P: PersistentDataHandler> LightStateReader
    for LightUsecases<S, P>
{
    fn get_light_state(&self) -> LightState {
        self.state.get_light_state()
    }
}

impl<S: LightStateHandler, P: PersistentDataHandler> LightUsecasesPort
    for LightUsecases<S, P>
{
    fn apply_intent_and_persist(
        &mut self,
        intent: LightChangeIntent,
    ) -> Result<(), LightError> {
        self.state.apply_light_intent(intent)?;
        self.persistence
            .write_persistent_data(PersistentData::LightState(
                self.state.get_light_state(),
            ))
            .map_err(|_e| {
                #[cfg(feature = "log")]
                esp_println::println!(
                    "light: error persisting light state: {:?}",
                    _e
                );

                LightError::PersistenceError
            })?;
        Ok(())
    }
}

impl<S: LightStateHandler, P: PersistentDataHandler> LightConfigChanger
    for LightUsecases<S, P>
{
    fn set_config(&mut self, config: LightConfig) -> Result<(), LightError> {
        self.state.set_config(config)
    }
}

unsafe impl<S: LightStateHandler, P: PersistentDataHandler> Send
    for LightUsecases<S, P>
{
}
unsafe impl<S: LightStateHandler, P: PersistentDataHandler> Sync
    for LightUsecases<S, P>
{
}

impl<S: LightStateHandler, P: PersistentDataHandler> LightStateHandler
    for LightUsecases<S, P>
{
}
