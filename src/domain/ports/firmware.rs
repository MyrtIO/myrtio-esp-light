use crate::{core::net::http::AsyncChunkedReader, domain::entity::BootSlot};

/// Error type for the firmware operations
#[derive(Debug)]
pub enum FirmwareError {
    Erase,
    InvalidPartitionTable,
    Write,
    Read,
    Activate,
    Flash,
}

/// Trait for the HTTP firmware updater
pub trait HttpFirmwareUpdater {
    /// Update the firmware from HTTP
    fn update_firmware_from_http(
        &self,
        conn: &mut impl AsyncChunkedReader,
    ) -> impl Future<Output = Result<(), FirmwareError>>;
}

pub trait BootSectorWriter {
    /// Write the boot sector
    fn write_boot_sector(&mut self, slot: BootSlot) -> Result<(), FirmwareError>;
}

pub trait BootSectorReader {
    /// Read the boot sector
    fn read_boot_sector(&mut self) -> Result<BootSlot, FirmwareError>;
}

pub trait BootSectorSelector {
    /// Boot from system (ota0) slot
    fn boot_system(&mut self) -> impl Future<Output = Result<(), FirmwareError>>;

    /// Boot from factory slot
    fn boot_factory(&mut self) -> impl Future<Output = Result<(), FirmwareError>>;
}

pub trait FirmwareHandler: BootSectorSelector + HttpFirmwareUpdater + Sync + Send {}

pub trait FirmwareUsecasesPort: FirmwareHandler {}
