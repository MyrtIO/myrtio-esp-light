use crate::{
    config::DeviceConfig,
    domain::{dto::PersistentData, entity::LightState},
};

/// Error type for the persistence operations
#[derive(Debug)]
pub enum PersistenceError {
    Busy,
    DriverError,
}

/// Writer interface for the persisting data to the power-loss-safe storage
pub trait PersistentDataWriter {
    /// Update the persistent data
    fn write_persistent_data(
        &self,
        data: PersistentData,
    ) -> Result<(), PersistenceError>;
}

/// Reader interface for the persistent data
pub trait PersistentDataReader {
    /// Get the persistent data
    fn read_persistent_data(
        &self,
    ) -> Result<(LightState, DeviceConfig), PersistenceError>;
}

/// Trait for the persistence handler
pub trait PersistentDataHandler:
    PersistentDataWriter + PersistentDataReader + Sync + Send
{
}
