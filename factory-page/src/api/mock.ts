import type {
  Configuration,
  LightConfiguration,
  LightTestRequest,
  SystemInformation,
} from "../model/types";
import type { ApiService, ProgressCallback } from "./interface";
import { sleep } from "../utils/sleep";

export class MockApiService implements ApiService {
  async getConfiguration(): Promise<Configuration> {
    console.log(`[mock] getting configuration`);
    await simulateNetworkDelay();
    return {
      wifi: {
        ssid: "MySSID",
        password: "My Password",
      },
      mqtt: {
        host: "mqtt-balancer.lan",
        port: 1883,
        username: "myrtio",
        password: "myrtio",
      },
      light: {
        brightness_min: 10,
        brightness_max: 255,
        led_count: 60,
        skip_leds: 0,
        color_correction: 0xffb0a0,
        color_order: "grb",
      },
    };
  }

  async saveConfiguration(configuration: Configuration): Promise<void> {
    console.log(`[mock] saving configuration`, { configuration });
    await simulateNetworkDelay();
  }

  async setLightConfiguration(light: LightConfiguration): Promise<void> {
    console.log(`[mock] setting light configuration`, { light });
    await simulateNetworkDelay();
  }

  async testColor(request: LightTestRequest): Promise<void> {
    console.log(`[mock] testing color`, { request });
    await simulateNetworkDelay();
  }

  async getSystemInformation(): Promise<SystemInformation> {
    console.log(`[mock] getting system information`);
    await simulateNetworkDelay();
    return {
      build_version: "0d35914-2025-12-23T11:16:09+0000",
      mac_address: [160, 183, 101, 22, 48, 84],
    };
  }

  async updateFirmware(
    file: File,
    onProgress: ProgressCallback
  ): Promise<void> {
    console.log(`[mock] updating firmware from ${file.name}`);
    const totalTime = 5000;
    const stepTime = totalTime / 10;
    for (let i = 0; i <= 100; i += 10) {
      await sleep(stepTime);
      onProgress(i);
    }
  }

  async bootSystem(): Promise<void> {
    console.log(`[mock] booting system`);
    await simulateNetworkDelay();
  }
}

function simulateNetworkDelay(
  min: number = 50,
  delta: number = 200
): Promise<void> {
  const delay = Math.random() * delta + min;
  return sleep(delay);
}
