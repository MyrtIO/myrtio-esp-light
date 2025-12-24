// use crate::domain::ports::OtaUsecasesPort;

use super::ports::{ConfigurationUsecasesPort, FirmwareUsecasesPort, LightUsecasesPort};

// Type alias for the light usecases port reference
pub type LightUsecasesPortRef = &'static mut dyn LightUsecasesPort;

/// Type alias for the configuration usecases port reference
pub type ConfigurationUsecasesPortRef = &'static mut dyn ConfigurationUsecasesPort;

/// Type alias for the firmware usecases port reference
pub type FirmwareUsecasesImpl = &'static mut dyn FirmwareUsecasesPort;

// /// Type alias for the ota usecases port reference
// pub type OtaUsecasesPortRef = &'static mut dyn OtaUsecasesPort;
