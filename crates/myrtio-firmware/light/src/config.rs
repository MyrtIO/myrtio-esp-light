// struct WifiConfig<'a> {
//     ssid: &'a str,
//     password: &'a str,
// }

// struct MqttConfig<'a> {
//     host: &'a str,
//     port: u16,
// }

// struct LightConfig {
//     led_count: usize,
//     led_gpio: AnyPin<'a>,
// }

pub const DEVICE_NAME: &str = "MyrtIO RS Demo";
pub const DEVICE_MANUFACTURER: &str = "Myrt";
pub const DEVICE_MODEL: &str = "ESP32";
pub const DEVICE_ID: &str = "myrtio_rs_demo";

pub const WIFI_SSID: &str = env!("WIFI_SSID");
pub const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

pub const MQTT_HOST: &str = env!("MQTT_HOST");
pub const MQTT_PORT: u16 = 1883;

pub const LIGHT_LED_COUNT: usize = 6;
#[allow(clippy::unreadable_literal)]
pub const LIGHT_COLOR_CORRECTION: u32 = 0xFFAA78;

macro_rules! led_gpio {
    ($p:expr) => { $p.GPIO25 };
}