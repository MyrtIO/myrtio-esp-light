//! Flash storage driver with shared mutex access
//!
//! Provides thread-safe access to the ESP32's internal flash memory via a
//! global mutex. Both persistent light state storage and OTA firmware writes
//! use this shared mutex to prevent concurrent flash access.

use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use myrtio_core::storage::{StorageDriver, StorageError};
use static_cell::StaticCell;

pub(crate) const BLOCK_SIZE: u32 = 4096;

/// Base address of the `light_state` partition (defined in partitions.csv).
const LIGHT_STATE_PARTITION_OFFSET: u32 = 0x31_0000;

/// Type alias for the shared flash storage mutex
pub(crate) type FlashStorageMutex = Mutex<CriticalSectionRawMutex, RefCell<FlashStorage<'static>>>;

/// Static cell for the flash storage mutex (initialized once)
static FLASH_STORAGE_CELL: StaticCell<FlashStorageMutex> = StaticCell::new();

/// Initialize the shared flash storage mutex from the FLASH peripheral.
///
/// This function should be called once during startup. It returns a static
/// reference to the mutex that can be shared between the persistent light
/// state storage and OTA service.
///
/// # Panics
/// Panics if called more than once.
pub(crate) fn init_flash_storage_mutex(flash: FLASH<'static>) -> &'static FlashStorageMutex {
    let flash_storage = FlashStorage::new(flash);
    FLASH_STORAGE_CELL.init(Mutex::new(RefCell::new(flash_storage)))
}

/// Persistent storage implementation using the shared [`FlashStorageMutex`].
///
/// The driver operates strictly within the `light_state` partition
/// at offset [`LIGHT_STATE_PARTITION_OFFSET`].
pub(crate) struct EspNorFlashStorageDriver<const SIZE: usize> {
    storage: &'static FlashStorageMutex,
    addr: u32,
}

impl<const SIZE: usize> EspNorFlashStorageDriver<SIZE> {
    pub(crate) fn new(storage: &'static FlashStorageMutex) -> Self {
        Self {
            storage,
            addr: LIGHT_STATE_PARTITION_OFFSET,
        }
    }
}

impl<const SIZE: usize> StorageDriver<SIZE> for EspNorFlashStorageDriver<SIZE> {
    /// Read data from the storage
    fn read(&self, buffer: &mut [u8]) -> Result<(), StorageError> {
        self.storage.lock(|cell| {
            cell.borrow_mut()
                .read(self.addr, buffer)
                .map_err(|_| StorageError::DriverError)
        })
    }

    /// Write data to the storage
    fn write(&self, buffer: &[u8]) -> Result<(), StorageError> {
        self.storage.lock(|cell| {
            let mut cell_ref = cell.borrow_mut();
            let erase_result = cell_ref.erase(self.addr, self.addr + BLOCK_SIZE);
            if erase_result.is_err() {
                esp_println::println!("Failed to erase flash storage: {:?}", erase_result);
                return Err(StorageError::DriverError);
            }
            let write_result = NorFlash::write(&mut *cell_ref, self.addr, buffer);
            if write_result.is_err() {
                esp_println::println!("Failed to write to flash storage: {:?}", write_result);
                return Err(StorageError::DriverError);
            }
            Ok(())
        })
    }
}
