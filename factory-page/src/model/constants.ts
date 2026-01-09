import type {
  LightConfiguration,
  MqttConfiguration,
  SystemInformation,
  WifiConfiguration,
} from "./types";

export const defaultSystemInformation: SystemInformation = {
  build_version: "",
  mac_address: [0, 0, 0, 0, 0, 0],
};

export const defaultWifiConfiguration: WifiConfiguration = {
  ssid: "",
  password: "",
};

export const defaultMqttConfiguration: MqttConfiguration = {
  host: "",
  port: 1883,
  username: "",
  password: "",
};

export const defaultLightConfiguration: LightConfiguration = {
  brightness_min: 0,
  brightness_max: 255,
  led_count: 60,
  skip_leds: 0,
  color_correction: 0,
  color_order: "grb",
};

export const colorOrderOptions = [
  { value: "grb", label: "GRB" },
  { value: "rgb", label: "RGB" },
  { value: "brg", label: "BRG" },
  { value: "rbg", label: "RBG" },
  { value: "gbr", label: "GBR" },
  { value: "bgr", label: "BGR" },
];
