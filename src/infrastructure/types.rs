use crate::{
    app::FirmwareUsecases,
    infrastructure::{drivers::EspLedDriver, services::FirmwareService},
};

pub(crate) type LightDriver = EspLedDriver<'static>;
pub type FirmwareUsecasesImpl = FirmwareUsecases<FirmwareService>;
