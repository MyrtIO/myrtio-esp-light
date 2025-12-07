use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver};

use crate::domain::entity::LightState;
use crate::domain::ports::{
    LightStateReader, PersistentLightStateHandler, PersistentLightStateWriter,
};
use crate::infrastructure::types::LightStorageMutex;

/// Type alias for the light state receiver
pub(crate) type LightStateReceiver =
    Receiver<'static, CriticalSectionRawMutex, LightState, LIGHT_STATE_CHANNEL_SIZE>;

/// Type alias for the light state channel
pub(crate) type LightStateChannel =
    Channel<CriticalSectionRawMutex, LightState, LIGHT_STATE_CHANNEL_SIZE>;

const LIGHT_STATE_CHANNEL_SIZE: usize = 4;

/// Channel for persisting light state
pub(crate) static PERSISTENCE_CHANNEL: LightStateChannel = Channel::new();

/// Service for persisting light state
pub(crate) struct LightStatePersistenceService<'a> {
    storage: &'a LightStorageMutex,
}

impl LightStatePersistenceService<'_> {
    pub(crate) fn new(storage: &'static LightStorageMutex) -> Self {
        Self { storage }
    }
}

impl PersistentLightStateWriter for LightStatePersistenceService<'_> {
    /// Save the light state to the persistence channel
    fn save_state(&mut self, state: LightState) -> Result<(), ()> {
        PERSISTENCE_CHANNEL.try_send(state).map_err(|_| ())
    }
}

impl LightStateReader for LightStatePersistenceService<'_> {
    fn get_light_state(&self) -> Option<LightState> {
        self.storage.lock(|cell| {
            let storage = cell.borrow();
            storage.get_light_state()
        })
    }
}

impl PersistentLightStateHandler for LightStatePersistenceService<'_> {}

pub(crate) fn get_persistence_receiver() -> LightStateReceiver {
    PERSISTENCE_CHANNEL.receiver()
}
