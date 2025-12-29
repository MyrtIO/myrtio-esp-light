use crate::{
    app::{ConfigurationUsecases, FirmwareUsecases, LightUsecases},
    infrastructure::{
        drivers::EspLedDriver,
        services::{FirmwareService, LightStateService, PersistenceService},
    },
};

pub(crate) type LightDriver = EspLedDriver<'static>;

pub type LightUsecasesImpl = LightUsecases<LightStateService, PersistenceService>;
pub type FirmwareUsecasesImpl = FirmwareUsecases<FirmwareService>;
pub type ConfigurationUsecasesImpl =
    ConfigurationUsecases<PersistenceService, LightStateService>;
