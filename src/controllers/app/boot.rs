use embedded_hal::delay::DelayNs;
use esp_println::println;

use super::LIGHT_USECASES;
use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
use crate::domain::ports::{BootManagerPort, OnBootHandler, PersistenceHandler};

#[derive(Default)]
pub struct BootController<P: PersistenceHandler, B: BootManagerPort> {
    persistence: P,
    boot_manager: B,
}

impl<P: PersistenceHandler, B: BootManagerPort> BootController<P, B> {
    pub fn new(persistence: P, boot_manager: B) -> Self {
        Self {
            persistence,
            boot_manager,
        }
    }
}

impl<P: PersistenceHandler, B: BootManagerPort> OnBootHandler for BootController<P, B> {
    fn on_boot_start(&mut self) {
        println!("Boot start");
        let reboot_count = self.persistence.increment_reboot_count().unwrap();
        if reboot_count > 5 {
            println!("Boot start: rebooting to factory");
            self.persistence.reset_reboot_count();
            esp_hal::delay::Delay::new().delay_ms(200);
            self.boot_manager.boot_factory().unwrap();
        }
        println!("Boot start: reboot count: {}", reboot_count);
    }

    fn on_light_ready(&mut self) {
        let state = self
            .persistence
            .get_persistent_data()
            .map(|(_, state, _)| state)
            .unwrap_or_default();
        let intent: LightChangeIntent = state.into();

        LIGHT_USECASES.lock(|cell| {
            let mut cell_ref = cell.borrow_mut();
            cell_ref.as_mut().unwrap().apply_intent(intent).unwrap();
        });
    }

    fn on_boot_end(&mut self) {
        println!("Boot end");
    }

    fn on_magic_timeout(&mut self) {
        println!("Magic timeout");
        self.persistence.reset_reboot_count().unwrap();
    }
}
