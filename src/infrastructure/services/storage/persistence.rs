use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};

use super::FLASH_STORAGE;
use crate::{
    config::{DeviceConfig, LIGHT_STATE_WRITE_DEBOUNCE},
    domain::{
        dto::PersistentData,
        entity::LightState,
        ports::{
            PersistenceError,
            PersistentDataHandler,
            PersistentDataReader,
            PersistentDataWriter,
        },
    },
    infrastructure::{
        repositories::AppPersistentStorage,
        services::storage::state::StorageState,
    },
};

const PERSISTENT_DATA_CHANNEL_SIZE: usize = 4;

/// Type alias for the persistent data channel
type PersistentDataChannel =
    Channel<CriticalSectionRawMutex, PersistentData, PERSISTENT_DATA_CHANNEL_SIZE>;

/// Channel for persisting persistent data
pub(crate) static PERSISTENT_DATA_CHANNEL: PersistentDataChannel = Channel::new();

/// Service for persisting persistent data
#[derive(Debug, Default, Clone)]
pub struct PersistenceService;

impl PersistentDataWriter for PersistenceService {
    /// Save the light state to the persistence channel
    fn write_persistent_data(
        &self,
        data: PersistentData,
    ) -> Result<(), PersistenceError> {
        PERSISTENT_DATA_CHANNEL
            .try_send(data)
            .map_err(|_| PersistenceError::Busy)
    }
}

impl PersistentDataReader for PersistenceService {
    fn read_persistent_data(
        &self,
    ) -> Result<(LightState, DeviceConfig), PersistenceError> {
        FLASH_STORAGE
            .try_lock()
            .map_err(|_| PersistenceError::Busy)
            .and_then(|guard| {
                #[cfg(feature = "log")]
                println!("read_persistent_data: successfully locked");
                let mut cell = guard.borrow_mut();
                let flash_ref = cell.as_mut().unwrap();
                let storage = AppPersistentStorage::new(flash_ref, 0);

                storage.read_persistent_data()
            })
            .map_err(|_| PersistenceError::DriverError)
    }
}

impl PersistentDataHandler for PersistenceService {}

/// Task for persisting persistent data
#[embassy_executor::task]
async fn light_state_persistence_task(debounce: Duration) {
    #[cfg(feature = "log")]
    esp_println::println!("persistence: starting persistence task");

    // State to be persisted. Used for debouncing.
    let mut pending_state: Option<LightState> = None;
    let receiver = PERSISTENT_DATA_CHANNEL.receiver();

    loop {
        let receive_fut = receiver.receive();
        let timer_fut = Timer::after(debounce);

        match select(receive_fut, timer_fut).await {
            Either::First(data) => match data {
                PersistentData::LightState(state) => {
                    pending_state = Some(state);
                }
                PersistentData::DeviceConfig(config) => {
                    StorageState::wait_for_idle().await;

                    #[cfg(feature = "log")]
                    esp_println::println!(
                        "persistence: writing device config: {:?}",
                        config
                    );
                    write_persistent_data(PersistentData::DeviceConfig(config))
                        .await;
                }
            },
            Either::Second(()) => {
                let Some(state) = pending_state.take() else {
                    continue;
                };
                #[cfg(feature = "log")]
                esp_println::println!(
                    "persistence: writing persistent data: {:?}",
                    state
                );
                StorageState::wait_for_idle().await;
                write_persistent_data(PersistentData::LightState(state)).await;
            }
        }
    }
}

pub(super) fn init_persistence_service(spawner: Spawner) -> PersistenceService {
    spawner
        .spawn(light_state_persistence_task(LIGHT_STATE_WRITE_DEBOUNCE))
        .ok();

    PersistenceService
}

/// Write the persistent data to the storage
async fn write_persistent_data(data: PersistentData) {
    let guard = FLASH_STORAGE.lock().await;
    let mut cell = guard.borrow_mut();
    let flash_ref = cell.as_mut().unwrap();
    let storage = AppPersistentStorage::new(flash_ref, 0);

    StorageState::run_with(StorageState::UpdatingPersistentData, || {
        storage
            .write_persistent_data(data)
            .expect("error persisting persistent data");
    });
}
