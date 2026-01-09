import type { ApiService } from "../api";
import { Store } from "../utils/store";
import {
  defaultMqttConfiguration,
  defaultLightConfiguration,
  defaultSystemInformation,
  defaultWifiConfiguration,
} from "./constants";

export const $systemInformation = new Store(defaultSystemInformation);
export const $wifiConfiguration = new Store(defaultWifiConfiguration);
export const $mqttConfiguration = new Store(defaultMqttConfiguration);
export const $lightConfiguration = new Store(defaultLightConfiguration);

$wifiConfiguration.subscribe((wifi) => {
  console.log("wifi", wifi);
});
$mqttConfiguration.subscribe((mqtt) => {
  console.log("mqtt", mqtt);
});
$lightConfiguration.subscribe((light) => {
  console.log("light", light);
});

export function initializeStores(api: ApiService) {
  api.getConfiguration().then((config) => {
    $wifiConfiguration.set(config.wifi);
    $mqttConfiguration.set(config.mqtt);
    $lightConfiguration.set(config.light);
  });
  api.getSystemInformation().then((info) => {
    $systemInformation.set(info);
  });
}
