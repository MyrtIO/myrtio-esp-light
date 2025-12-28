mod firmware;
mod persistence;
mod state;

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_storage::FlashStorage;
pub use firmware::FirmwareService;
pub use persistence::PersistenceService;
use persistence::init_persistence_service;
use firmware::init_boot_to_sector_task;

pub(super) static FLASH_STORAGE: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<FlashStorage<'static>>>,
> = Mutex::new(RefCell::new(None));

pub async fn init_storage_services(
    spawner: Spawner,
    flash: *mut FlashStorage<'static>,
) -> (FirmwareService, PersistenceService) {
    let guard = FLASH_STORAGE.lock().await;
    guard
        .borrow_mut()
        .replace(unsafe { core::ptr::read(flash) });

    init_boot_to_sector_task(spawner);
    let persistence_service = init_persistence_service(spawner);

    (FirmwareService, persistence_service)
}
