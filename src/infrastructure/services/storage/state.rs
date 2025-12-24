use core::sync::atomic::{AtomicU8, Ordering};

use embassy_time::{Duration, Timer};

static STORAGE_STATE: AtomicU8 = AtomicU8::new(StorageState::Idle as u8);

#[derive(PartialEq, Eq)]
pub(super) enum StorageState {
    Idle,
    UpdatingFirmware,
    UpdatingPersistentData,
    UpdatingBootSector,
}

impl StorageState {
    pub(super) fn get() -> StorageState {
        match STORAGE_STATE.load(Ordering::Relaxed) {
            0 => StorageState::Idle,
            1 => StorageState::UpdatingFirmware,
            2 => StorageState::UpdatingPersistentData,
            3 => StorageState::UpdatingBootSector,
            _ => StorageState::Idle,
        }
    }

    pub(super) fn run_with<T>(state: StorageState, f: impl FnOnce() -> T) -> T {
        StorageState::set(state);
        let result = f();
        StorageState::set(StorageState::Idle);
        result
    }

    pub(super) fn set(state: StorageState) {
        STORAGE_STATE.store(state as u8, Ordering::Relaxed);
    }

    pub(super) async fn wait_for_idle() {
        while StorageState::get() != StorageState::Idle {
            Timer::after(Duration::from_millis(10)).await;
        }
    }
}
