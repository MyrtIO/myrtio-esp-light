use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};
use esp_println::println;

use crate::domain::entity::LightState;
use crate::domain::ports::{PersistenceHandler};
use crate::infrastructure::repositories::AppPersistentStorage;
use crate::infrastructure::services::persistence::LightStateReceiver;

const PERSISTENCE_DELAY: Duration = Duration::from_millis(5000);

#[embassy_executor::task]
pub async fn persistence_task(
    mut storage: AppPersistentStorage,
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
                            storage
                                .persist_light_state(state)
                                .expect("error persisting light state");
                            pending_state = None;
                        }
                    }
                }
            }
        }
    }
}
