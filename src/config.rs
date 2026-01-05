use bytemuck::{Pod, Zeroable};
use embassy_net::Ipv4Address;
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

/// Debounce time for writing light state to the storage
pub const LIGHT_STATE_WRITE_DEBOUNCE: Duration = Duration::from_millis(5000);

/// Maximum supported LED count
pub const LED_COUNT_MAX: usize = 128;

/// Maximum supported connections
pub const MAX_NETWORK_CONNECTIONS: usize = 6;

/// Factory Access Point IP address
pub const FACTORY_AP_IP_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 4, 1);

/// Factory Access Point gateway
pub const FACTORY_AP_GATEWAY: Ipv4Address = Ipv4Address::new(192, 168, 4, 1);

/// Factory Access Point prefix length
pub const FACTORY_AP_PREFIX_LEN: u8 = 24;

/// Default transition timings
pub const DEFAULT_TRANSITION_TIMINGS: TransitionTimings = TransitionTimings {
    fade_out: Duration::from_millis(800),
    fade_in: Duration::from_millis(500),
    color_change: Duration::from_millis(200),
    brightness: Duration::from_millis(300),
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 1883,
            username: String::new(),
            password: String::new(),
        }
    }
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

impl Default for LightConfig {
    fn default() -> Self {
        Self {
            brightness_min: 0,
            brightness_max: 255,
            led_count: 20,
            skip_leds: 0,
            // Default: GRB order (0x01) in high byte, white color correction
            color_correction: pack_color_correction(ColorOrder::Grb, 0xFF_FFFF),
        }
    }
}

/// LED color channel order.
///
/// Different WS2812-compatible LED strips use different channel orderings.
/// This enum represents all 6 permutations of RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum ColorOrder {
    Rgb = 0,
    #[default]
    Grb = 1,
    Brg = 2,
    Rbg = 3,
    Gbr = 4,
    Bgr = 5,
}

impl ColorOrder {
    /// Convert from raw u8 value, returning default (GRB) for invalid values.
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Rgb,
            1 => Self::Grb,
            2 => Self::Brg,
            3 => Self::Rbg,
            4 => Self::Gbr,
            5 => Self::Bgr,
            _ => Self::Grb, // Default to GRB for invalid values
        }
    }

    /// Convert to raw u8 value.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Reorder RGB components according to this color order.
    ///
    /// Takes (r, g, b) and returns them reordered for the LED strip.
    #[inline]
    pub const fn reorder(self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        match self {
            Self::Rgb => (r, g, b),
            Self::Grb => (g, r, b),
            Self::Brg => (b, r, g),
            Self::Rbg => (r, b, g),
            Self::Gbr => (g, b, r),
            Self::Bgr => (b, g, r),
        }
    }
}

/// Pack color order and RGB24 color correction into a single `u32`.
///
/// Format: `(order_id << 24) | (rgb24 & 0x00FF_FFFF)`
///
/// This allows storing the color order in the high byte of the existing
/// `color_correction` field without changing the on-flash layout.
pub const fn pack_color_correction(order: ColorOrder, rgb24: u32) -> u32 {
    ((order.as_u8() as u32) << 24) | (rgb24 & 0x00FF_FFFF)
}

/// Unpack color order from packed `color_correction` value.
pub const fn unpack_color_order(packed: u32) -> ColorOrder {
    ColorOrder::from_u8((packed >> 24) as u8)
}

/// Unpack RGB24 color correction from packed `color_correction` value.
pub const fn unpack_color_correction_rgb24(packed: u32) -> u32 {
    packed & 0x00FF_FFFF
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

const HOSTNAME_PREFIX: &str = "myrtio-light";

/// Get the hostname
pub fn hostname() -> String<32> {
    use core::fmt::Write;
    let mut device_id = String::<32>::new();
    let id = hardware_id();
    let _ = write!(device_id, "{HOSTNAME_PREFIX}-{:04X}", id & 0xFFFF);
    device_id
}

const ACCESS_POINT_NAME_PREFIX: &str = "MyrtIO Светильник";

pub fn access_point_name() -> String<32> {
    use core::fmt::Write;
    let mut device_id = String::<32>::new();
    let id = hardware_id();
    let _ = write!(device_id, "{ACCESS_POINT_NAME_PREFIX} {:04X}", id & 0xFFFF);
    device_id
}

const DEVICE_ID_PREFIX: &str = "myrtio_light";

pub fn device_id() -> String<32> {
    use core::fmt::Write;
    let mut device_id = String::<32>::new();
    let id = hardware_id();
    let _ = write!(device_id, "{DEVICE_ID_PREFIX}_{:04X}", id & 0xFFFF);
    device_id
}

/// Get the LED GPIO pin from the peripherals
#[macro_export]
macro_rules! led_gpio {
    ($p:expr) => {
        $p.GPIO25
    };
}
