use myrtio_core::storage::{Encodable, MAGIC_HEADER_SIZE, PersistentStorage, StorageDriver};

use crate::domain::entity::{ColorMode, LightState};
use crate::domain::ports::PersistentLightStateHandler;
use crate::infrastructure::drivers::EspNorFlashStorageDriver;

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

impl<DRIVER: Send + Sync> PersistentLightStateHandler for LightStorage<DRIVER>
where
    DRIVER: StorageDriver<{ STORAGE_SIZE }>,
{
    async fn get_persistent_light_state(&self) -> Option<LightState> {
        self.load::<{ LIGHT_STATE_SIZE }, LightState>()
            .await
            .map_err(|_| ())
            .ok()
    }

    async fn save_persistent_light_state(&mut self, state: LightState) -> Result<(), ()> {
        self.save::<{ LIGHT_STATE_SIZE }, LightState>(&state)
            .await
            .map_err(|_| ())
    }
}
