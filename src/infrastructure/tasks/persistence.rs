use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};
use esp_println::println;

use crate::domain::entity::LightState;
use crate::domain::ports::PersistentLightStateHandler;
use crate::infrastructure::config;
use crate::infrastructure::services::persistence::LightStateReceiver;
use crate::infrastructure::types::LightStorageMutex;

const PERSISTENCE_DELAY: Duration = Duration::from_millis(config::STORAGE.write_debounce_ms);

#[embassy_executor::task]
pub(crate) async fn storage_persistence_task(
    storage: &'static LightStorageMutex,
    receiver: LightStateReceiver,
) {
    println!("persistence: starting persistence task");
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
                            let storage_guard = storage.lock().await;
                            let mut storage = storage_guard.borrow_mut();
                            storage.save_persistent_light_state(state).await.expect("Failed to save light state to storage");
                            pending_state = None;
                        }
                    }
                }
            }
        }
    }
}
