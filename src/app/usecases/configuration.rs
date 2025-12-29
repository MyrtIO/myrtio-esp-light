#[cfg(feature = "log")]
use esp_println::println;

use crate::{
    config::{DeviceConfig, LightConfig},
    domain::{
        dto::PersistentData,
        ports::{
            ConfigurationError,
            ConfigurationReader,
            ConfigurationServicePort,
            ConfigurationUsecasesPort,
            ConfigurationWriter,
            LightConfigurationSetter,
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
            LightError::TooManyLEDs => ConfigurationError::TooManyLEDs,
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
    fn save_device_config(
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

impl<P: PersistentDataHandler, S: LightStateHandler> ConfigurationServicePort
    for ConfigurationUsecases<P, S>
{
}

impl<P: PersistentDataHandler, S: LightStateHandler> LightConfigurationSetter
    for ConfigurationUsecases<P, S>
{
    fn set_light_config(
        &mut self,
        config: &LightConfig,
    ) -> Result<(), ConfigurationError> {
        self.state
            .set_config(*config)
            .map_err(ConfigurationError::LightError)
    }
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
