use bytemuck::{Pod, Zeroable};
use embassy_time::Duration;
use heapless::String;
use myrtio_light_composer::TransitionTimings;
use serde::{Deserialize, Serialize};

/// Offset of the configuration partition in the flash
pub const CONFIGURATION_PARTITION_OFFSET: u32 = 0x31_0000;

/// Build version
pub(crate) const BUILD_VERSION: &str = env!("BUILD_VERSION");

/// Device manufacturer
pub const DEVICE_MANUFACTURER: &str = "MyrtIO";

/// Device model
pub const DEVICE_MODEL: &str = "Light RS1";

/// Maximum supported temperature in Kelvin
pub const TEMPERATURE_MAX_KELVIN: u16 = 6500;

/// Minimum supported temperature in Kelvin
pub const TEMPERATURE_MIN_KELVIN: u16 = 1500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String<32>,
    pub password: String<64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String<64>,
    pub port: u16,
    pub username: String<32>,
    pub password: String<64>,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod, Serialize, Deserialize)]
#[repr(C)]
pub struct LightConfig {
    pub brightness_min: u8,
    pub brightness_max: u8,
    pub led_count: u8,
    pub skip_leds: u8,
    pub color_correction: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub wifi: WifiConfig,
    pub mqtt: MqttConfig,
    pub light: LightConfig,
}

/// Get the hardware ID from the last 4 bytes of the MAC address
pub fn hardware_id() -> u32 {
    let mac = esp_hal::efuse::Efuse::mac_address();
    u32::from_be_bytes([mac[2], mac[3], mac[4], mac[5]])
}

/// Get the MAC address
pub fn mac_address() -> [u8; 6] {
    esp_hal::efuse::Efuse::mac_address()
}

pub const DEFAULT_TRANSITION_TIMINGS: TransitionTimings = TransitionTimings {
    fade_out: Duration::from_millis(800),
    fade_in: Duration::from_millis(500),
    color_change: Duration::from_millis(200),
    brightness: Duration::from_millis(300),
};

/// Get the LED GPIO pin from the peripherals
#[macro_export]
macro_rules! led_gpio {
    ($p:expr) => {
        $p.GPIO25
    };
}
