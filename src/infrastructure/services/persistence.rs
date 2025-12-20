use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver};

use crate::domain::entity::LightState;
use crate::domain::ports::PersistentLightStateUpdater;
use crate::infrastructure::tasks::flash_actor::OTA_ACTIVE;
use core::sync::atomic::Ordering;

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
pub struct LightStatePersistenceService<'a> {
    _p: core::marker::PhantomData<&'a ()>,
}

impl LightStatePersistenceService<'_> {
    pub fn new() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}

impl PersistentLightStateUpdater for LightStatePersistenceService<'_> {
    /// Save the light state to the persistence channel
    fn update_persistent_light_state(&mut self, state: LightState) -> Result<(), ()> {
        if OTA_ACTIVE.load(Ordering::Relaxed) {
            return Err(());
        }
        PERSISTENCE_CHANNEL.try_send(state).map_err(|_| ())
    }
}

pub fn get_persistence_receiver() -> LightStateReceiver {
    PERSISTENCE_CHANNEL.receiver()
}
