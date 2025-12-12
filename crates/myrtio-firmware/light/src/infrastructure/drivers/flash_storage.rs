//! Flash storage driver used by persistent light state storage.
//!
//! Flash is owned by the flash actor task; this driver uses a raw pointer
//! (single-owner assumption) to perform synchronous flash operations.

use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_storage::FlashStorage;
use myrtio_core::storage::{StorageDriver, StorageError};

pub(crate) const BLOCK_SIZE: u32 = 4096;

/// Base address of the `light_state` partition (defined in partitions.csv).
const LIGHT_STATE_PARTITION_OFFSET: u32 = 0x31_0000;

/// Persistent storage implementation using the flash owned by the flash actor.
///
/// The driver operates strictly within the `light_state` partition
/// at offset [`LIGHT_STATE_PARTITION_OFFSET`].
pub(crate) struct EspNorFlashStorageDriver<const SIZE: usize> {
    flash: *mut FlashStorage<'static>,
    addr: u32,
}

// Safety: This driver is only used by the flash actor task which is the sole flash owner.
// The raw pointer is never accessed concurrently from multiple tasks.
unsafe impl<const SIZE: usize> Send for EspNorFlashStorageDriver<SIZE> {}
unsafe impl<const SIZE: usize> Sync for EspNorFlashStorageDriver<SIZE> {}

impl<const SIZE: usize> EspNorFlashStorageDriver<SIZE> {
    pub(crate) fn new(flash: *mut FlashStorage<'static>) -> Self {
        Self {
            flash,
            addr: LIGHT_STATE_PARTITION_OFFSET,
        }
    }
}

impl<const SIZE: usize> StorageDriver<SIZE> for EspNorFlashStorageDriver<SIZE> {
    /// Read data from the storage
    async fn read(&self, buffer: &mut [u8]) -> Result<(), StorageError> {
        // Safety: flash is owned by the flash actor task; no concurrent access.
        unsafe { &mut *self.flash }
            .read(self.addr, buffer)
            .map_err(|_| StorageError::DriverError)
    }

    /// Write data to the storage
    async fn write(&self, buffer: &[u8]) -> Result<(), StorageError> {
        // Safety: flash is owned by the flash actor task; no concurrent access.
        let flash = unsafe { &mut *self.flash };
        flash
            .erase(self.addr, self.addr + BLOCK_SIZE)
            .map_err(|_| StorageError::DriverError)?;
        flash
            .write(self.addr, buffer)
            .map_err(|_| StorageError::DriverError)
    }
}
