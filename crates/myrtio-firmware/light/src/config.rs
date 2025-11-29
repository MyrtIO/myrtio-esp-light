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

pub const WIFI_SSID: &str = env!("WIFI_SSID");
pub const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
pub const MQTT_HOST: &str = env!("MQTT_HOST");
pub const MQTT_PORT: u16 = 1883;
pub const LIGHT_LED_COUNT: usize = 6;

macro_rules! led_gpio {
    ($p:expr) => { $p.GPIO25 };
}