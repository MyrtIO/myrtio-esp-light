mod button;
mod http;

use core::cell::RefCell;

pub use button::handle_boot_button_click;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
pub use http::FactoryHttpController;

use crate::{
    domain::types::{ConfigurationUsecasesPortRef, FirmwareUsecasesPortRef},
    infrastructure::services::LightStateService,
};

pub(super) static CONFIGURATION_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<ConfigurationUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));

pub(super) static FIRMWARE_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<FirmwareUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));

/// Light state service for test color endpoint.
/// Uses `RefCell<Option<...>>` pattern for consistency, though `LightStateService`
/// is Copy.
pub(super) static LIGHT_STATE_SERVICE: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<LightStateService>>,
> = Mutex::new(RefCell::new(None));

pub async fn init_factory_controllers(
    configuration: ConfigurationUsecasesPortRef,
    firmware: FirmwareUsecasesPortRef,
    light: LightStateService,
) -> FactoryHttpController {
    let guard = CONFIGURATION_USECASES.lock().await;
    guard.borrow_mut().replace(configuration);

    let guard = FIRMWARE_USECASES.lock().await;
    guard.borrow_mut().replace(firmware);

    let guard = LIGHT_STATE_SERVICE.lock().await;
    guard.borrow_mut().replace(light);

    FactoryHttpController
}
