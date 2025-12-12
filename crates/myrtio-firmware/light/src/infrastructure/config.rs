#![allow(clippy::unreadable_literal)]

pub(crate) struct WifiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

pub(crate) struct MqttConfig {
    pub host: &'static str,
    pub port: u16,
}

pub(crate) struct DeviceConfig {
    pub manufacturer: &'static str,
    pub name: &'static str,
    pub model: &'static str,
    pub id: &'static str,
    pub hostname: &'static str,
}

pub(crate) struct LightConfig {
    pub led_count: usize,
    pub skip_leds: usize,
    pub color_correction: u32,
    pub brightness_min: u8,
    pub brightness_max: u8,
    pub temperature_max_kelvin: u16,
    pub temperature_min_kelvin: u16,
}

pub(crate) struct StorageConfig {
    pub write_debounce_ms: u64,
}

pub(crate) struct FirmwareConfig {
    pub version: &'static str,
}

pub(crate) const DEVICE_MANUFACTURER: &str = "MyrtIO";

pub(crate) const WIFI: WifiConfig = WifiConfig {
    ssid: env!("WIFI_SSID"),
    password: env!("WIFI_PASSWORD"),
};

pub(crate) const MQTT: MqttConfig = MqttConfig {
    host: env!("MQTT_HOST"),
    port: 1883,
};

#[cfg(feature = "rs1")]
pub(crate) const DEVICE: DeviceConfig = DeviceConfig {
    manufacturer: DEVICE_MANUFACTURER,
    name: "MyrtIO RS1",
    model: "RS1",
    id: "myrtio_rs1",
    hostname: "myrtio-rs1",
};
#[cfg(feature = "curtain")]
pub(crate) const DEVICE: DeviceConfig = DeviceConfig {
    manufacturer: DEVICE_MANUFACTURER,
    name: "MyrtIO Curtain",
    model: "Curtain",
    id: "myrtio_curtain",
    hostname: "myrtio-curtain",
};
#[cfg(feature = "bar")]
pub(crate) const DEVICE: DeviceConfig = DeviceConfig {
    manufacturer: DEVICE_MANUFACTURER,
    name: "MyrtIO Bar",
    model: "Bar",
    id: "myrtio_bar",
    hostname: "myrtio-bar",
};
#[cfg(feature = "ceiling")]
pub(crate) const DEVICE: DeviceConfig = DeviceConfig {
    manufacturer: DEVICE_MANUFACTURER,
    name: "MyrtIO Ceiling",
    model: "Ceiling",
    id: "myrtio_ceiling",
    hostname: "myrtio-ceiling",
};
#[cfg(not(any(feature = "rs1", feature = "curtain", feature = "bar", feature = "ceiling")))]
pub(crate) const DEVICE: DeviceConfig = DeviceConfig {
    manufacturer: DEVICE_MANUFACTURER,
    name: "MyrtIO ESP32",
    model: "ESP32",
    id: "myrtio_esp32_unknown",
    hostname: "myrtio-esp32-unknown",
};

#[cfg(feature = "rs1")]
pub(crate) const LIGHT: LightConfig = LightConfig {
    led_count: 26,
    skip_leds: 2,
    color_correction: 0xFFAA78,
    brightness_min: 10,
    brightness_max: 100,
    temperature_max_kelvin: 6500,
    temperature_min_kelvin: 1500,
};
#[cfg(feature = "curtain")]
pub(crate) const LIGHT: LightConfig = LightConfig {
    led_count: 26,
    skip_leds: 0,
    color_correction: 0xFFAA78,
    brightness_min: 10,
    brightness_max: 180,
    temperature_max_kelvin: 6500,
    temperature_min_kelvin: 1500,
};
#[cfg(feature = "bar")]
pub(crate) const LIGHT: LightConfig = LightConfig {
    led_count: 85,
    skip_leds: 1,
    color_correction: 0xFFAA78,
    brightness_min: 10,
    brightness_max: 250,
    temperature_max_kelvin: 6500,
    temperature_min_kelvin: 1500,
};
#[cfg(feature = "ceiling")]
pub(crate) const LIGHT: LightConfig = LightConfig {
    led_count: 23,
    skip_leds: 1,
    color_correction: 0xFFAA78,
    brightness_min: 10,
    brightness_max: 100,
    temperature_max_kelvin: 6500,
    temperature_min_kelvin: 1500,
};
#[cfg(not(any(feature = "rs1", feature = "curtain", feature = "bar", feature = "ceiling")))]
pub(crate) const LIGHT: LightConfig = LightConfig {
    led_count: 6,
    skip_leds: 0,
    color_correction: 0xFFFFFF,
    brightness_min: 0,
    brightness_max: 100,
    temperature_max_kelvin: 6500,
    temperature_min_kelvin: 1500,
};

pub(crate) const STORAGE: StorageConfig = StorageConfig {
    write_debounce_ms: 5000,
};

pub(crate) const FIRMWARE: FirmwareConfig = FirmwareConfig {
    version: env!("BUILD_VERSION"),
};

#[macro_export]
macro_rules! led_gpio {
    ($p:expr) => {
        $p.GPIO25
    };
}
