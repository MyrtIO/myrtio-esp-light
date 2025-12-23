//! Factory HTTP Server
//!
//! Provides a web interface for device provisioning and OTA updates.
//! Serves a configuration page and handles config saves and firmware uploads.

use embassy_time::Duration;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::GPIO2;

#[embassy_executor::task]
pub async fn blink_task(gpio: GPIO2<'static>) {
    let mut pin = Output::new(gpio, Level::High, OutputConfig::default());
    loop {
        pin.set_high();
        embassy_time::Timer::after(Duration::from_millis(300)).await;
        pin.set_low();
        embassy_time::Timer::after(Duration::from_millis(300)).await;
    }
}
