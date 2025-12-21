use bytemuck::{Pod, Zeroable};
use heapless::String;

pub const LIGHT_STATE_PARTITION_OFFSET: u32 = 0x31_0000;
pub(crate) const BUILD_VERSION: &str = env!("BUILD_VERSION");

#[derive(Debug, Clone)]
pub(crate) struct WifiConfig {
    pub ssid: String<32>,
    pub password: String<64>,
}

#[derive(Debug, Clone)]
pub(crate) struct MqttConfig {
    pub host: String<64>,
    pub port: u16,
    pub username: String<32>,
    pub password: String<64>,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub(crate) struct LightConfig {
    pub brightness_min: u8,
    pub brightness_max: u8,
    pub led_count: u8,
    pub skip_leds: u8,
    pub color_correction: u32,
}

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub wifi: WifiConfig,
    pub mqtt: MqttConfig,
    pub light: LightConfig,
}
