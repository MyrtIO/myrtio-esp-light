use crate::{
    core::net::http::AsyncChunkedReader,
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
    async fn boot_system(&mut self) -> Result<(), FirmwareError> {
        self.firmware.boot_system().await
    }

    async fn boot_factory(&mut self) -> Result<(), FirmwareError> {
        self.firmware.boot_factory().await
    }
}

impl<P: FirmwareHandler> FirmwareHandler for FirmwareUsecases<P> {}

impl<P: FirmwareHandler> HttpFirmwareUpdater for FirmwareUsecases<P> {
    async fn update_firmware_from_http(
        &self,
        conn: &mut impl AsyncChunkedReader,
    ) -> Result<(), FirmwareError> {
        self.firmware.update_firmware_from_http(conn).await
    }
}

impl<P: FirmwareHandler> FirmwareUsecasesPort for FirmwareUsecases<P> {}

unsafe impl<P: FirmwareHandler> Send for FirmwareUsecases<P> {}
unsafe impl<P: FirmwareHandler> Sync for FirmwareUsecases<P> {}
