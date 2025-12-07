#![allow(clippy::unreadable_literal)]

pub(crate) const DEVICE_MANUFACTURER: &str = "MyrtIO";

#[cfg(feature = "rs1")]
pub(crate) const DEVICE_NAME: &str = "MyrtIO RS1";
#[cfg(not(all(feature = "rs1")))]
pub(crate) const DEVICE_NAME: &str = "MyrtIO ESP32";

#[cfg(feature = "rs1")]
pub(crate) const DEVICE_MODEL: &str = "RS1";
#[cfg(not(all(feature = "rs1")))]
pub(crate) const DEVICE_MODEL: &str = "ESP32";

#[cfg(feature = "rs1")]
pub(crate) const DEVICE_ID: &str = "myrtio_rs1";
#[cfg(not(all(feature = "rs1")))]
pub(crate) const DEVICE_ID: &str = "myrtio_esp32";

pub(crate) const WIFI_SSID: &str = env!("WIFI_SSID");
pub(crate) const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

pub(crate) const MQTT_HOST: &str = env!("MQTT_HOST");
pub(crate) const MQTT_PORT: u16 = 1883;

#[cfg(feature = "rs1")]
pub(crate) const LIGHT_LED_COUNT: usize = 26;
#[cfg(not(all(feature = "rs1")))]
pub(crate) const LIGHT_LED_COUNT: usize = 6;

#[cfg(feature = "rs1")]
pub(crate) const LIGHT_COLOR_CORRECTION: u32 = 0xFFAA78;
#[cfg(not(all(feature = "rs1")))]
pub(crate) const LIGHT_COLOR_CORRECTION: u32 = 0xFFFFFF;

#[cfg(feature = "rs1")]
pub(crate) const LIGHT_MAX_BRIGHTNESS_SCALE: u8 = 100;
#[cfg(not(all(feature = "rs1")))]
pub(crate) const LIGHT_MAX_BRIGHTNESS_SCALE: u8 = 255;

pub(crate) const STORAGE_WRITE_DEBOUNCE_MS: u64 = 5000;

#[macro_export]
macro_rules! led_gpio {
    ($p:expr) => {
        $p.GPIO25
    };
}