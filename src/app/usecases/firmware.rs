extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use crate::{
    core::net::http::HttpConnection,
    domain::ports::{
        BootSectorSelector,
        FirmwareError,
        FirmwareHandler,
        FirmwareUsecasesPort,
        HttpFirmwareUpdater,
    },
};

pub struct FirmwareUsecases<P: FirmwareHandler> {
    firmware: P,
}

impl<P: FirmwareHandler> FirmwareUsecases<P> {
    pub fn new(firmware: P) -> Self {
        Self { firmware }
    }
}

impl<P: FirmwareHandler> BootSectorSelector for FirmwareUsecases<P> {
    fn boot_system(&mut self) -> Result<(), FirmwareError> {
        self.firmware.boot_system()
    }

    fn boot_factory(&mut self) -> Result<(), FirmwareError> {
        self.firmware.boot_factory()
    }
}

impl<P: FirmwareHandler> FirmwareHandler for FirmwareUsecases<P> {}

impl<P: FirmwareHandler> HttpFirmwareUpdater for FirmwareUsecases<P> {
    fn update_firmware_from_http<'s, 'c>(
        &'s self,
        conn: &'c mut HttpConnection<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FirmwareError>> + 's>>
    where
        'c: 's,
    {
        self.firmware.update_firmware_from_http(conn)
    }
}

impl<P: FirmwareHandler> FirmwareUsecasesPort for FirmwareUsecases<P> {}

unsafe impl<P: FirmwareHandler> Send for FirmwareUsecases<P> {}
unsafe impl<P: FirmwareHandler> Sync for FirmwareUsecases<P> {}
