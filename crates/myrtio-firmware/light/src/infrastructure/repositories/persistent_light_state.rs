use core::cell::RefCell;

use embassy_sync::blocking_mutex::Mutex;
use esp_hal::peripherals::FLASH;
use myrtio_core::storage::{Encodable, MAGIC_HEADER_SIZE, PersistentStorage, StorageDriver};

use crate::domain::entity::{ColorMode, LightState};
use crate::domain::ports::{LightStateReader, PersistentLightStateWriter};
use crate::infrastructure::drivers::{EspNorFlashStorageDriver, FlashStorageMutex, init_flash_storage_mutex};
use crate::infrastructure::types::LightStorageMutex;

/// Concrete storage driver used by the firmware.
pub(crate) type LightStorageDriver = EspNorFlashStorageDriver<{ STORAGE_SIZE }>;

/// Convenience alias for persistent light storage with a generic driver.
pub(crate) type LightStorage<DRIVER> = PersistentStorage<DRIVER, { STORAGE_SIZE }>;

/// Concrete storage driver used by the firmware.
pub(crate) type LightNorFlashStorage = LightStorage<LightStorageDriver>;

const LIGHT_STATE_SIZE: usize = 10;

impl Encodable<LIGHT_STATE_SIZE> for LightState {
    fn encode(self) -> [u8; LIGHT_STATE_SIZE] {
        let color_temp_bytes = self.color_temp.to_le_bytes();
        [
            u8::from(self.power),
            self.brightness,
            self.mode_id,
            color_temp_bytes[0],
            color_temp_bytes[1],
            self.color_mode.as_u8(),
            self.color.0,
            self.color.1,
            self.color.2,
            0, // padding
        ]
    }

    fn decode(data: &[u8]) -> Option<LightState> {
        if data.len() != LIGHT_STATE_SIZE {
            return None;
        }
        Some(LightState {
            power: data[0] != 0,
            brightness: data[1],
            mode_id: data[2],
            color_temp: u16::from_le_bytes([data[3], data[4]]),
            color_mode: ColorMode::from_u8(data[5]).unwrap(),
            color: (data[6], data[7], data[8]),
        })
    }
}

/// Total size of the light state in storage
const STORAGE_SIZE: usize = LIGHT_STATE_SIZE + MAGIC_HEADER_SIZE;

impl<DRIVER> LightStateReader for LightStorage<DRIVER>
where
    DRIVER: StorageDriver<{ STORAGE_SIZE }>,
{
    fn get_light_state(&self) -> Option<LightState> {
        self.load::<{ LIGHT_STATE_SIZE }, LightState>()
            .map_err(|_| ())
            .ok()
    }
}

impl<DRIVER> PersistentLightStateWriter for LightStorage<DRIVER>
where
    DRIVER: StorageDriver<{ STORAGE_SIZE }>,
{
    fn save_state(&mut self, state: LightState) -> Result<(), ()> {
        self.save::<{ LIGHT_STATE_SIZE }, LightState>(&state)
            .map_err(|_| ())
    }
}

/// Initialize the flash storage subsystem.
///
/// This initializes the shared flash mutex and creates the light state storage.
/// Returns a tuple of:
/// - The light storage mutex (for persistence service)
/// - The flash storage mutex (for OTA service)
pub(crate) fn init_flash_storage(flash: FLASH<'static>) -> (LightStorageMutex, &'static FlashStorageMutex) {
    // Initialize the shared flash mutex
    let flash_mutex = init_flash_storage_mutex(flash);
    
    // Create the light storage driver using the shared flash mutex
    let driver = LightStorageDriver::new(flash_mutex);
    let storage = LightNorFlashStorage::new(driver);
    
    // Wrap in mutex for the persistence service
    let light_storage_mutex = Mutex::new(RefCell::new(storage));
    
    (light_storage_mutex, flash_mutex)
}
