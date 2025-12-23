use esp_bootloader_esp_idf::ota::Ota;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::{
    AppPartitionSubType, DataPartitionSubType, PartitionType, read_partition_table,
};
use esp_storage::FlashStorage;

use crate::domain::ports::BootManagerPort;

enum BootSlot {
    Unknown,
    Factory,
    Slot0,
}

impl BootSlot {
    pub fn partition_sub_type(self) -> AppPartitionSubType {
        match self {
            BootSlot::Factory => AppPartitionSubType::Factory,
            BootSlot::Slot0 => AppPartitionSubType::Ota0,
            BootSlot::Unknown => unreachable!(),
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

    pub fn get_boot_slot(&self) -> BootSlot {
        let ota = self.with_ota(|mut ota| ota.current_app_partition().unwrap());
        match ota {
            AppPartitionSubType::Factory => BootSlot::Factory,
            AppPartitionSubType::Ota0 => BootSlot::Slot0,
            _ => BootSlot::Unknown,
        }
    }

    pub fn set_boot_slot(&self, slot: BootSlot) {
        self.with_ota(|mut ota| {
            ota.set_current_app_partition(slot.partition_sub_type())
                .unwrap();
        });
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

impl BootManagerPort for BootManager {
    fn boot_system(&mut self) -> Option<()> {
        self.set_boot_slot(BootSlot::Slot0);
        esp_hal::system::software_reset();
    }

    fn boot_factory(&mut self) -> Option<()> {
        self.set_boot_slot(BootSlot::Factory);
        esp_hal::system::software_reset();
    }
}
