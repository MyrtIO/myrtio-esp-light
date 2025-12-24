mod http;

use core::cell::RefCell;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
pub use http::FactoryHttpController;

use crate::{
    app::FirmwareUsecases,
    domain::types::ConfigurationUsecasesPortRef,
    infrastructure::services::OtaService,
};

pub(super) static CONFIGURATION_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<ConfigurationUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));

type FirmwareUsecasesImpl = FirmwareUsecases<OtaService>;

pub(super) static FIRMWARE_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<FirmwareUsecasesImpl>>,
> = Mutex::new(RefCell::new(None));

pub async fn init_factory_controllers(
    configuration: ConfigurationUsecasesPortRef,
    firmware: FirmwareUsecasesImpl,
) -> FactoryHttpController {
    let guard = CONFIGURATION_USECASES.lock().await;
    guard.borrow_mut().replace(configuration);

    let guard = FIRMWARE_USECASES.lock().await;
    guard.borrow_mut().replace(firmware);

    FactoryHttpController::default()
}
