use core::cell::RefCell;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_storage::FlashStorage;
use esp_hal::peripherals::FLASH;
use myrtio_core::storage::{StorageDriver, StorageError};

pub(crate) const BLOCK_SIZE: u32 = 4096;

/// Base address of the `light_state` partition (defined in partitions.csv).
const LIGHT_STATE_PARTITION_OFFSET: u32 = 0x3F_0000;

/// Persistent storage implementation using [`FlashStorage`] wrapped in a mutex.
///
/// The driver operates strictly within the `light_state` partition
/// at offset [`LIGHT_STATE_PARTITION_OFFSET`].
pub(crate) struct EspNorFlashStorageDriver<const SIZE: usize> {
    storage: Mutex<CriticalSectionRawMutex, RefCell<FlashStorage<'static>>>,
    addr: u32,
}

impl<const SIZE: usize> EspNorFlashStorageDriver<SIZE> {
    pub(crate) fn new(flash: FLASH<'static>) -> Self {
        let storage = FlashStorage::new(flash);
        Self {
            storage: Mutex::new(RefCell::new(storage)),
            addr: LIGHT_STATE_PARTITION_OFFSET,
        }
    }
}

impl<const SIZE: usize> StorageDriver<SIZE> for EspNorFlashStorageDriver<SIZE> {
    /// Read data from the storage
    fn read(&self, buffer: &mut [u8]) -> Result<(), StorageError> {
        self.storage
            .lock(|cell| cell.borrow_mut().read(self.addr, buffer))
            .map_err(|_| StorageError::DriverError)
    }

    /// Write data to the storage
    fn write(&self, buffer: &[u8]) -> Result<(), StorageError> {
        self.storage.lock(|cell| {
            let mut cell_ref = cell.borrow_mut();
            cell_ref
                .erase(self.addr, self.addr + BLOCK_SIZE)
                .map_err(|_| StorageError::DriverError)?;
            cell_ref
                .write(self.addr, buffer)
                .map_err(|_| StorageError::DriverError)
        })
    }
}
