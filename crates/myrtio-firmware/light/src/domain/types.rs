use super::ports::LightUsecasesPort;

// Type alias for the light usecases port reference
pub(crate) type LightUsecasesPortRef = &'static mut dyn LightUsecasesPort;
