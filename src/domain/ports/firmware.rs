extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use crate::{core::net::http::HttpConnection, domain::entity::BootSlot};

/// Error type for the firmware operations
#[derive(Debug)]
pub enum FirmwareError {
    /// The device is already booting to a sector
    AlreadyBooting,
    Busy,
    Erase,
    InvalidPartitionTable,
    Write,
    Read,
    Activate,
    Flash,
}

/// Trait for the HTTP firmware updater (object-safe)
pub trait HttpFirmwareUpdater {
    /// Update the firmware from HTTP
    ///
    /// Note: Uses separate lifetimes to allow the mutex guard (`&self`) to have
    /// a shorter lifetime than `conn`. The future lives as long as `&self`.
    fn update_firmware_from_http<'s, 'c>(
        &'s self,
        conn: &'c mut HttpConnection<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FirmwareError>> + 's>>
    where
        'c: 's;
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
    fn boot_system(&mut self) -> Result<(), FirmwareError>;

    /// Boot from factory slot
    fn boot_factory(&mut self) -> Result<(), FirmwareError>;
}

pub trait FirmwareHandler: BootSectorSelector + HttpFirmwareUpdater + Sync + Send {}

pub trait FirmwareUsecasesPort: FirmwareHandler {}
