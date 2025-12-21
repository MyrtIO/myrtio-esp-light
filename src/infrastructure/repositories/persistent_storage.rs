use core::str::FromStr;

use bytemuck::{Pod, Zeroable};
use heapless::String;

use crate::config::{DeviceConfig, LightConfig, MqttConfig, WifiConfig};
use crate::domain::entity::{ColorMode, LightState};
use crate::domain::ports::PersistenceHandler;
use crate::infrastructure::drivers::EspPersistentStorage;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct PersistentLightState {
    pub power: u8,
    pub brightness: u8,
    pub mode_id: u8,
    pub color_mode: u8,
    pub color: [u8; 3],
    _padding: u8,
    pub color_temp: u16,
}

impl From<LightState> for PersistentLightState {
    fn from(state: LightState) -> Self {
        Self {
            power: if state.power { 1 } else { 0 },
            brightness: state.brightness,
            mode_id: state.mode_id,
            color_mode: state.color_mode.as_u8(),
            color: [state.color.0, state.color.1, state.color.2],
            color_temp: state.color_temp,
            _padding: 0,
        }
    }
}

impl From<PersistentLightState> for LightState {
    fn from(state: PersistentLightState) -> Self {
        Self {
            power: state.power != 0,
            brightness: state.brightness,
            mode_id: state.mode_id,
            color_mode: ColorMode::from_u8(state.color_mode).unwrap_or(ColorMode::Rgb),
            color: (state.color[0], state.color[1], state.color[2]),
            color_temp: state.color_temp,
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct PersistentWifiConfig {
    pub ssid: [u8; 32],
    pub password: [u8; 64],
}

impl From<WifiConfig> for PersistentWifiConfig {
    fn from(config: WifiConfig) -> Self {
        Self {
            ssid: string_to_array(&config.ssid),
            password: string_to_array(&config.password),
        }
    }
}

impl<'a> From<&'a PersistentWifiConfig> for WifiConfig {
    fn from(config: &'a PersistentWifiConfig) -> Self {
        Self {
            ssid: parse_padded_string(&config.ssid),
            password: parse_padded_string(&config.password),
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct PersistentMqttConfig {
    pub host: [u8; 64],
    pub port: u16,
    pub username: [u8; 32],
    pub password: [u8; 64],
}

impl From<MqttConfig> for PersistentMqttConfig {
    fn from(config: MqttConfig) -> Self {
        Self {
            host: string_to_array(&config.host),
            port: config.port,
            username: string_to_array(&config.username),
            password: string_to_array(&config.password),
        }
    }
}

impl<'a> From<&'a PersistentMqttConfig> for MqttConfig {
    fn from(config: &'a PersistentMqttConfig) -> Self {
        Self {
            host: parse_padded_string(&config.host),
            port: config.port,
            username: parse_padded_string(&config.username),
            password: parse_padded_string(&config.password),
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct PersistentDeviceConfig {
    pub light: LightConfig,
    pub wifi: PersistentWifiConfig,
    pub mqtt: PersistentMqttConfig,
    _padding: [u8; 2],
}

impl From<DeviceConfig> for PersistentDeviceConfig {
    fn from(config: DeviceConfig) -> Self {
        Self {
            light: config.light.into(),
            wifi: config.wifi.into(),
            mqtt: config.mqtt.into(),
            _padding: [0; 2],
        }
    }
}

impl From<&PersistentDeviceConfig> for DeviceConfig {
    fn from(config: &PersistentDeviceConfig) -> Self {
        Self {
            light: config.light.into(),
            wifi: (&config.wifi).into(),
            mqtt: (&config.mqtt).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AppPersistentData {
    light_state: PersistentLightState,
    reboot_count: u8,
    _padding: u8,
    config: PersistentDeviceConfig,
}

/// Concrete storage driver used by the firmware.
pub type AppPersistentStorage = EspPersistentStorage<AppPersistentData>;

// /// Safety: This is a single-owner assumption.
// unsafe impl Send for AppPersistentStorage {}
// unsafe impl Sync for AppPersistentStorage {}

impl AppPersistentStorage {
    fn get_raw_data(&self) -> Option<AppPersistentData> {
        let Ok(data) = self.load().map_err(|_| ()) else {
            return None;
        };
        Some(data)
    }

    fn save_raw_data(&self, data: AppPersistentData) -> Option<()> {
        self.save(&data).map_err(|_| ()).ok()
    }
}

impl PersistenceHandler for AppPersistentStorage {
    fn get_persistent_data(&self) -> Option<(u8, LightState, DeviceConfig)> {
        let data = self.get_raw_data()?;
        Some((
            data.reboot_count,
            data.light_state.into(),
            (&data.config).into(),
        ))
    }

    fn persist_light_state(&mut self, light_state: LightState) -> Option<()> {
        let mut data = self.get_raw_data()?;
        data.light_state = light_state.into();
        self.save_raw_data(data)
    }

    fn persist_device_config(&mut self, config: DeviceConfig) -> Option<()> {
        let mut data = self.get_raw_data()?;
        data.config = config.into();
        self.save_raw_data(data)
    }

    fn persist_boot_count(&mut self, boot_count: u8) -> Option<()> {
        let mut data = self.get_raw_data()?;
        data.reboot_count = boot_count;
        self.save_raw_data(data)
    }
}

/// Get the length of a string from a byte array
fn parse_padded_string<const N: usize>(bytes: &[u8]) -> String<N> {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = unsafe { core::str::from_utf8_unchecked(&bytes[..len]) };

    String::from_str(s).unwrap()
}

/// Convert a heapless::String to a fixed-size byte array, padding with zeros
fn string_to_array<const N: usize>(s: &String<N>) -> [u8; N] {
    let mut arr = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N);
    arr[..len].copy_from_slice(&bytes[..len]);
    arr
}
