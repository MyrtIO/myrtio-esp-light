//! Flash storage driver used by persistent light state storage.
//!
//! Flash is owned by the flash actor task; this driver uses a raw pointer
//! (single-owner assumption) to perform synchronous flash operations.

use core::{marker::PhantomData, mem};

use bytemuck::Pod;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_storage::FlashStorage;

const MAGIC_HEADER: u16 = 0xBEEF;
const MAGIC_HEADER_SIZE: usize = MAGIC_HEADER.to_le_bytes().len();
const BLOCK_SIZE: u32 = 4096;

#[derive(Debug)]
pub(crate) enum StorageError {
    DriverError,
    InvalidMagicHeader,
}

/// Persistent storage implementation using a storage driver.
pub struct EspPersistentStorage<T: Pod> {
    flash: *mut FlashStorage<'static>,
    addr: u32,
    _phantom: PhantomData<T>,
}

impl<T: Pod> EspPersistentStorage<T> {
    pub fn new(flash: *mut FlashStorage<'static>, addr: u32) -> Self {
        Self {
            flash,
            addr,
            _phantom: PhantomData,
        }
    }

    /// Load persistent data from flash
    pub(crate) fn load(&self) -> Result<T, StorageError> {
        let mut buffer = [0u8; BLOCK_SIZE as usize];

        match unsafe { &mut *self.flash }.read(self.addr, &mut buffer) {
            Ok(()) => {
                let magic = u16::from_le_bytes([buffer[0], buffer[1]]);
                if magic == MAGIC_HEADER {
                    let data_end = MAGIC_HEADER_SIZE + mem::size_of::<T>();
                    // Use pod_read_unaligned because data starts after 2-byte magic header
                    // which may not be aligned to T's alignment requirements
                    let data: T =
                        bytemuck::pod_read_unaligned(&buffer[MAGIC_HEADER_SIZE..data_end]);

                    return Ok(data);
                }
            }
            Err(_) => {
                return Err(StorageError::DriverError);
            }
        }
        Err(StorageError::InvalidMagicHeader)
    }

    /// Save persistent data to flash
    ///
    /// NOR flash requires erase before write. This erases the entire block
    /// (4 KiB sector) before writing the data.
    pub(crate) fn save(&self, state: &T) -> Result<(), StorageError> {
        let flash = unsafe { &mut *self.flash };

        // Erase the block first (NOR flash can only flip 1→0, erase sets to 1)
        flash
            .erase(self.addr, self.addr + BLOCK_SIZE)
            .map_err(|_| StorageError::DriverError)?;

        let mut buffer: [u8; BLOCK_SIZE as usize] = [0xFFu8; BLOCK_SIZE as usize];

        buffer[0..MAGIC_HEADER_SIZE].copy_from_slice(&MAGIC_HEADER.to_le_bytes());
        let state_bytes = bytemuck::bytes_of(state);
        let data_end = MAGIC_HEADER_SIZE + state_bytes.len();
        buffer[MAGIC_HEADER_SIZE..data_end].copy_from_slice(state_bytes);

        flash
            .write(self.addr, &buffer)
            .map_err(|_| StorageError::DriverError)
    }
}

// // Safety: This driver is only used by the flash actor task which is the sole flash owner.
// // The raw pointer is never accessed concurrently from multiple tasks.
// unsafe impl<const SIZE: usize> Send for EspNorFlashStorageDriver<SIZE> {}
// unsafe impl<const SIZE: usize> Sync for EspNorFlashStorageDriver<SIZE> {}
