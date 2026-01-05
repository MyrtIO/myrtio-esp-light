use super::light::LightError;
use crate::config::{DeviceConfig, LightConfig};

#[derive(Debug)]
pub enum ConfigurationError {
    LightError(LightError),
    StorageBusy,
    StackOverflow,
    TooManyLEDs,
}

pub trait ConfigurationReader {
    /// Get the device config
    fn get_device_config(&self) -> Option<DeviceConfig>;
}

pub trait ConfigurationWriter {
    /// Set the device config
    fn save_device_config(
        &mut self,
        config: &DeviceConfig,
    ) -> Result<(), ConfigurationError>;
}

pub trait LightConfigurationSetter {
    /// Set the device config
    fn set_light_config(
        &mut self,
        config: &LightConfig,
    ) -> Result<(), ConfigurationError>;
}

pub trait ConfigurationServicePort:
    ConfigurationReader + ConfigurationWriter + Sync + Send
{
}

pub trait ConfigurationUsecasesPort:
    ConfigurationReader + LightConfigurationSetter + ConfigurationWriter + Sync + Send
{
}
