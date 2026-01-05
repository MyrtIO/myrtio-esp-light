import type { Configuration, LightConfiguration, LightTestRequest, SystemInformation } from "../models";

export type ProgressCallback = (progress: number) => void;

export interface ApiService {
  getConfiguration(): Promise<Configuration>;
  saveConfiguration(configuration: Configuration): Promise<void>;
  setLightConfiguration(light: LightConfiguration): Promise<void>;
  testColor(request: LightTestRequest): Promise<void>;
  getSystemInformation(): Promise<SystemInformation>;
  updateFirmware(file: File, onProgress: ProgressCallback): Promise<void>;
  bootSystem(): Promise<void>;
}
