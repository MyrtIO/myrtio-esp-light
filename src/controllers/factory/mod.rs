mod http;

use esp_storage::FlashStorage;
pub use http::FactoryHttpController;
use myrtio_light_composer::IntentSender;

use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::domain::types::{ConfigurationUsecasesPortRef};

pub(super) static CONFIGURATION_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<ConfigurationUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));

pub fn init_factory_controllers(
    configuration: ConfigurationUsecasesPortRef,
    flash: *mut FlashStorage<'static>,
    intents: IntentSender,
) -> FactoryHttpController {
    CONFIGURATION_USECASES.lock(|cell| {
        cell.borrow_mut().replace(configuration);
    });

    FactoryHttpController::new(flash, intents)
}
