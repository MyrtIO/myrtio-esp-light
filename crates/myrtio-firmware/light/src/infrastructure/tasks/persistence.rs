use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};

use crate::domain::entity::LightState;
use crate::domain::ports::PersistentLightStateWriter;
use crate::infrastructure::config;
use crate::infrastructure::services::persistence::LightStateReceiver;
use crate::infrastructure::types::LightStorageMutex;

const PERSISTENCE_DELAY: Duration = Duration::from_millis(config::STORAGE_WRITE_DEBOUNCE_MS);

#[embassy_executor::task]
pub(crate) async fn storage_persistence_task(
    storage: &'static LightStorageMutex,
    receiver: LightStateReceiver,
) {
    let mut pending_state: Option<LightState> = None;

    loop {
        match pending_state {
            None => {
                let state = receiver.receive().await;
                pending_state = Some(state);
            }
            Some(_) => {
                let receive_fut = receiver.receive();
                let timer_fut = Timer::after(PERSISTENCE_DELAY);

                match select(receive_fut, timer_fut).await {
                    Either::First(state) => {
                        pending_state = Some(state);
                    }
                    Either::Second(()) => {
                        if let Some(state) = pending_state {
                            storage.lock(|cell| {
                                let mut storage = cell.borrow_mut();
                                storage
                                    .save_state(state)
                                    .expect("Failed to save light state to storage");
                            });
                            pending_state = None;
                        }
                    }
                }
            }
        }
    }
}
