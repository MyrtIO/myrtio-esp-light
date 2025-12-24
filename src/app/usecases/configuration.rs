#[cfg(feature = "log")]
use esp_println::println;

use crate::{
    config::DeviceConfig,
    domain::{
        dto::PersistentData,
        ports::{
            ConfigurationError,
            ConfigurationHandler,
            ConfigurationReader,
            ConfigurationUsecasesPort,
            ConfigurationWriter,
            LightError,
            LightStateHandler,
            PersistentDataHandler,
        },
    },
};

impl From<LightError> for ConfigurationError {
    fn from(error: LightError) -> Self {
        match error {
            LightError::Busy => ConfigurationError::StorageBusy,
            LightError::PersistenceError => ConfigurationError::StackOverflow,
        }
    }
}

pub struct ConfigurationUsecases<P: PersistentDataHandler, S: LightStateHandler> {
    configuration: P,
    state: S,
}

impl<P: PersistentDataHandler, S: LightStateHandler> ConfigurationUsecases<P, S> {
    pub fn new(configuration: P, state: S) -> Self
    where
        P: PersistentDataHandler,
        S: LightStateHandler,
    {
        Self {
            configuration,
            state,
        }
    }
}

impl<P: PersistentDataHandler, S: LightStateHandler> ConfigurationReader
    for ConfigurationUsecases<P, S>
{
    fn get_device_config(&self) -> Option<DeviceConfig> {
        self.configuration
            .read_persistent_data()
            .map(|(_, config)| config)
            .ok()
    }
}

impl<P: PersistentDataHandler, S: LightStateHandler> ConfigurationWriter
    for ConfigurationUsecases<P, S>
{
    fn set_device_config(
        &mut self,
        config: &DeviceConfig,
    ) -> Result<(), ConfigurationError> {
        self.configuration
            .write_persistent_data(PersistentData::DeviceConfig(config.clone()))
            .map_err(|_| ConfigurationError::StorageBusy)?;
        self.state.set_config(config.light)?;
        Ok(())
    }
}

impl<P: PersistentDataHandler, S: LightStateHandler> ConfigurationHandler
    for ConfigurationUsecases<P, S>
{
}

impl<P: PersistentDataHandler, S: LightStateHandler> ConfigurationUsecasesPort
    for ConfigurationUsecases<P, S>
{
}

unsafe impl<P: PersistentDataHandler, S: LightStateHandler> Send
    for ConfigurationUsecases<P, S>
{
}
unsafe impl<P: PersistentDataHandler, S: LightStateHandler> Sync
    for ConfigurationUsecases<P, S>
{
}
