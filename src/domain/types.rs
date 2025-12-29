use super::ports::{ConfigurationUsecasesPort, FirmwareUsecasesPort, LightUsecasesPort};

// Type alias for the light usecases port reference
pub type LightUsecasesPortRef = &'static mut dyn LightUsecasesPort;

/// Type alias for the configuration usecases port reference
pub type ConfigurationUsecasesPortRef = &'static mut dyn ConfigurationUsecasesPort;

/// Type alias for the firmware usecases port reference
pub type FirmwareUsecasesPortRef = &'static mut dyn FirmwareUsecasesPort;