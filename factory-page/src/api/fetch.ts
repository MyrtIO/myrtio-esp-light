import type { Configuration, LightConfiguration, SystemInformation } from "../models";
import type { ApiService, ProgressCallback } from "./interface";

export class FetchApiService implements ApiService {
  private readonly baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.endsWith("/") ? baseUrl.slice(0, -1) : baseUrl;
  }

  async updateFirmware(
    file: File,
    onProgress: ProgressCallback
  ): Promise<void> {
    // Read file as ArrayBuffer to ensure raw binary is sent (not multipart)
    const arrayBuffer = await file.arrayBuffer();

    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      xhr.open("POST", `${this.baseUrl}/ota`);
      xhr.setRequestHeader("Content-Type", "application/octet-stream");
      xhr.upload.onprogress = (e) => {
        const progress = (e.loaded / e.total) * 100;
        onProgress(progress);
      };
      xhr.onerror = (e) => {
        reject(new Error("Failed to update firmware", { cause: e }));
      };
      xhr.onload = () => {
        if (xhr.status === 200 || xhr.status === 204) {
          resolve();
        } else {
          reject(new Error("Failed to update firmware"));
        }
      };
      xhr.send(arrayBuffer);
    });
  }

  async getConfiguration(): Promise<Configuration> {
    return this.fetchGetJson<Configuration>("/configuration");
  }

  async bootSystem(): Promise<void> {
    let response = await fetch(`${this.baseUrl}/boot`, {
      method: "POST",
    });
    if (!response.ok) {
      throw new Error(`Failed to boot system: ${response.statusText}`, {
        cause: response.statusText,
      });
    }
  }

  async saveConfiguration(configuration: Configuration): Promise<void> {
    return this.fetchPostJson("/configuration", configuration);
  }

  async setLightConfiguration(light: LightConfiguration): Promise<void> {
    return this.fetchPostJson("/configuration/light", light);
  }

  async getSystemInformation(): Promise<SystemInformation> {
    return this.fetchGetJson<SystemInformation>("/system");
  }

  private async fetchGetJson<T>(url: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${url}`);
    return response.json();
  }

  private async fetchPostJson(url: string, body: unknown): Promise<void> {
    const response = await fetch(`${this.baseUrl}${url}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    });
    if (!response.ok) {
      throw new Error(`Failed to post to ${url}: ${response.statusText}`, {
        cause: response.statusText,
      });
    }
  }
}
