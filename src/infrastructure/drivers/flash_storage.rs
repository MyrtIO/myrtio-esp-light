//! Flash storage driver used by persistent light state storage.
//!
//! Flash is owned by the flash actor task; this driver uses a raw pointer
//! (single-owner assumption) to perform synchronous flash operations.

use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_storage::FlashStorage;

pub(crate) const BLOCK_SIZE: u32 = 4096;
const MAGIC_HEADER: u16 = 0xBEEF;
pub const MAGIC_HEADER_SIZE: usize = MAGIC_HEADER.to_le_bytes().len();

#[derive(Debug)]
pub(crate) enum StorageError {
    DriverError,
    InvalidMagicHeader,
    InvalidData,
}

pub(crate) trait Encodable<const SIZE: usize>
where
    Self: Sized,
{
    fn encode(self) -> [u8; SIZE];
    fn decode(data: &[u8]) -> Option<Self>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait StorageDriver<const STORAGE_SIZE: usize> {
    async fn read(&self, buffer: &mut [u8]) -> Result<(), StorageError>;
    async fn write(&self, buffer: &[u8]) -> Result<(), StorageError>;
}

/// Persistent storage implementation using a storage driver.
pub(crate) struct PersistentStorage<DRIVER: StorageDriver<STORAGE_SIZE>, const STORAGE_SIZE: usize> {
    driver: DRIVER,
}

impl<DRIVER: StorageDriver<STORAGE_SIZE>, const STORAGE_SIZE: usize>
    PersistentStorage<DRIVER, STORAGE_SIZE>
{
    pub fn new(driver: DRIVER) -> Self {
        Self { driver }
    }

    /// Load persistent data from flash
    pub async fn load<const SIZE: usize, T: Encodable<SIZE>>(&self) -> Result<T, StorageError> {
        let mut buffer = [0u8; STORAGE_SIZE];

        match self.driver.read(&mut buffer).await {
            Ok(()) => {
                let magic = u16::from_le_bytes([buffer[0], buffer[1]]);
                if magic == MAGIC_HEADER {
                    return T::decode(&buffer[MAGIC_HEADER_SIZE..STORAGE_SIZE])
                        .ok_or(StorageError::InvalidData);
                }
            }
            Err(_) => {
                return Err(StorageError::DriverError);
            }
        }
        Err(StorageError::InvalidMagicHeader)
    }

    /// Save persistent data to flash
    pub async fn save<const SIZE: usize, T: Encodable<SIZE> + Clone>(
        &self,
        state: &T,
    ) -> Result<(), StorageError> {
        let mut data = [0u8; STORAGE_SIZE];

        data[0..MAGIC_HEADER_SIZE].copy_from_slice(&MAGIC_HEADER.to_le_bytes());
        let encoded = state.clone().encode();
        data[MAGIC_HEADER_SIZE..STORAGE_SIZE].copy_from_slice(&encoded);

        self.driver
            .write(&data)
            .await
            .map_err(|_| StorageError::DriverError)
    }
}

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
