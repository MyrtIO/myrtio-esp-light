use esp_bootloader_esp_idf::{
    ota::Ota,
    partitions::{
        AppPartitionSubType,
        DataPartitionSubType,
        PartitionType,
        read_partition_table,
    },
};
use esp_storage::FlashStorage;

use crate::domain::{entity::BootSlot, ports};

impl From<BootSlot> for AppPartitionSubType {
    fn from(slot: BootSlot) -> Self {
        match slot {
            BootSlot::Factory => AppPartitionSubType::Factory,
            BootSlot::System => AppPartitionSubType::Ota0,
        }
    }
}

impl From<AppPartitionSubType> for BootSlot {
    fn from(sub_type: AppPartitionSubType) -> Self {
        match sub_type {
            AppPartitionSubType::Factory => BootSlot::Factory,
            AppPartitionSubType::Ota0 => BootSlot::System,
            _ => BootSlot::Factory,
        }
    }
}

pub struct BootManager {
    flash: *mut FlashStorage<'static>,
}

impl BootManager {
    pub fn new(flash: *mut FlashStorage<'static>) -> Self {
        Self { flash }
    }

    fn with_ota<R>(&self, f: impl FnOnce(Ota<'_, FlashStorage<'static>>) -> R) -> R {
        let flash_ref = unsafe { &mut *self.flash };
        let mut part_buffer = [0u8; 3072];
        let pt = read_partition_table(flash_ref, &mut part_buffer).unwrap();
        let ota_part = pt
            .find_partition(PartitionType::Data(DataPartitionSubType::Ota))
            .unwrap()
            .unwrap();
        let mut ota_part = ota_part.as_embedded_storage(flash_ref);
        let ota = Ota::new(&mut ota_part, 2).unwrap();
        f(ota)
    }
}

impl ports::BootSectorWriter for BootManager {
    fn write_boot_sector(
        &mut self,
        slot: BootSlot,
    ) -> Result<(), ports::FirmwareError> {
        self.with_ota(|mut ota| {
            ota.set_current_app_partition(slot.into())
                .map_err(|_| ports::FirmwareError::InvalidPartitionTable)
        })
    }
}

impl ports::BootSectorReader for BootManager {
    fn read_boot_sector(&mut self) -> Result<BootSlot, ports::FirmwareError> {
        let sub_type = self.with_ota(|mut ota| {
            ota.current_app_partition()
                .map_err(|_| ports::FirmwareError::InvalidPartitionTable)
        });
        sub_type.map(core::convert::Into::into)
    }
}
