use crate::config::DeviceConfig;

#[derive(Debug)]
pub enum ConfigurationError {
    StorageBusy,
    StackOverflow,
}

pub trait ConfigurationReader {
    /// Get the device config
    fn get_device_config(&self) -> Option<DeviceConfig>;
}

pub trait ConfigurationWriter {
    /// Set the device config
    fn set_device_config(
        &mut self,
        config: &DeviceConfig,
    ) -> Result<(), ConfigurationError>;
}

pub trait ConfigurationHandler:
    ConfigurationReader + ConfigurationWriter + Sync + Send
{
}

pub trait ConfigurationUsecasesPort: ConfigurationHandler {}
