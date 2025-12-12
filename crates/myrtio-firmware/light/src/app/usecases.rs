use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
use crate::domain::ports::{
    LightIntentApplier, LightStateHandler, LightStateReader, LightUsecasesPort,
    PersistentLightStateUpdater,
};

pub(crate) struct LightUsecases<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> {
    state: S,
    persistence: P,
}

impl<S: LightStateHandler, P: PersistentLightStateUpdater + Send + Sync> LightUsecases<S, P> {
    pub(crate) fn new(state: S, persistence: P) -> Self {
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
        let _ =
            self.persistence
                .update_persistent_light_state(self.state.get_light_state().ok_or(())?);
        Ok(())
    }
}
