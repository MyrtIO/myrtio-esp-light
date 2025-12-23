export type MacAddress = [number, number, number, number, number, number];

export interface SystemInformation {
  build_version: string;
  mac_address: MacAddress;
}

export interface WifiConfiguration {
  ssid: string;
  password: string;
}

export interface MqttConfiguration {
  host: string;
  port: number;
  username: string;
  password: string;
}

export interface LightConfiguration {
  brightness_min: number;
  brightness_max: number;
  led_count: number;
  skip_leds: number;
  color_correction: number;
}

export interface Configuration {
  wifi: WifiConfiguration;
  mqtt: MqttConfiguration;
  light: LightConfiguration;
}
