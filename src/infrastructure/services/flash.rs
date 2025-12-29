use core::ops::{Deref, DerefMut};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::{Mutex, MutexGuard, TryLockError},
};
use esp_hal::peripherals;
use esp_storage::FlashStorage;
use static_cell::StaticCell;

/// Global flash storage mutex.
///
/// This is initialized once via [`init_storage_services`].
pub(crate) static FLASH_STORAGE: FlashStorageMutex = FlashStorageMutex::new();

/// Internal flash storage cell.
static FLASH_STORAGE_CELL: StaticCell<FlashStorage<'static>> = StaticCell::new();

/// Thin wrapper around an Embassy mutex that stores the flash instance.
///
/// The key point: the guard derefs directly to `FlashStorage`, so call sites
/// don't need the usual `borrow_mut()/as_mut().unwrap()` boilerplate.
pub(crate) struct FlashStorageMutex {
    inner: Mutex<CriticalSectionRawMutex, Option<FlashStorage<'static>>>,
}

impl FlashStorageMutex {
    pub(crate) const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub(crate) async fn init_from_ptr(
        &'static self,
        flash: *mut FlashStorage<'static>,
    ) {
        let mut guard = self.inner.lock().await;
        *guard = Some(unsafe { core::ptr::read(flash) });
    }

    pub(crate) async fn lock(&'static self) -> FlashStorageLock<'static> {
        FlashStorageLock {
            guard: self.inner.lock().await,
        }
    }

    pub(crate) fn try_lock(
        &'static self,
    ) -> Result<FlashStorageLock<'static>, TryLockError> {
        self.inner
            .try_lock()
            .map(|guard| FlashStorageLock { guard })
    }
}

pub(crate) struct FlashStorageLock<'a> {
    guard: MutexGuard<'a, CriticalSectionRawMutex, Option<FlashStorage<'static>>>,
}

impl Deref for FlashStorageLock<'_> {
    type Target = FlashStorage<'static>;

    fn deref(&self) -> &Self::Target {
        self.guard
            .as_ref()
            .expect("FLASH_STORAGE is not initialized")
    }
}

impl DerefMut for FlashStorageLock<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .as_mut()
            .expect("FLASH_STORAGE is not initialized")
    }
}

/// Initialize the flash storage for services.
///
/// This function MUST be called before any other flash storage operations.
pub async fn init_flash_storage(raw_flash: peripherals::FLASH<'static>) {
    let flash = FlashStorage::new(raw_flash);
    let flash_ptr = FLASH_STORAGE_CELL.init(flash);

    FLASH_STORAGE.init_from_ptr(flash_ptr).await;
}
