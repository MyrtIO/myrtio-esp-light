import type {
  Configuration,
  LightConfiguration,
  LightTestRequest,
  SystemInformation,
} from "../model/types";
import type { ApiService, ProgressCallback } from "./interface";
import { Mutex } from "../utils/mutex";
import { sleep } from "../utils/sleep";

export class FetchApiService implements ApiService {
  private readonly baseUrl: string;
  private readonly mutex = new Mutex();

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.endsWith("/") ? baseUrl.slice(0, -1) : baseUrl;
  }

  async updateFirmware(
    file: File,
    onProgress: ProgressCallback
  ): Promise<void> {
    return this.withLock(async () => {
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
    });
  }

  async getConfiguration(): Promise<Configuration> {
    return this.fetchGetJson<Configuration>("/configuration");
  }

  async bootSystem(): Promise<void> {
    return this.withLock(async () => {
      const response = await fetch(`${this.baseUrl}/boot`, {
        method: "POST",
      });
      if (!response.ok) {
        throw new Error(`Failed to boot system: ${response.statusText}`, {
          cause: response.statusText,
        });
      }
    });
  }

  async saveConfiguration(configuration: Configuration): Promise<void> {
    return this.fetchPostJson("/configuration", configuration);
  }

  async setLightConfiguration(light: LightConfiguration): Promise<void> {
    return this.fetchPostJson("/configuration/light", light);
  }

  async testColor(request: LightTestRequest): Promise<void> {
    return this.fetchPostJson("/light/test", request);
  }

  async getSystemInformation(): Promise<SystemInformation> {
    return this.fetchGetJson<SystemInformation>("/system");
  }

  private async fetchGetJson<T>(url: string): Promise<T> {
    return this.withLock(async () => {
      const response = await fetch(`${this.baseUrl}${url}`);
      return response.json();
    });
  }

  private async fetchPostJson(url: string, body: unknown): Promise<void> {
    return this.withLock(async () => {
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
    });
  }

  private async withLock<T>(fn: () => Promise<T>): Promise<T> {
    const unlock = await this.mutex.lock();
    try {
      const result = await fn();
      await sleep(100);
      return result;
    } finally {
      unlock();
    }
  }
}
