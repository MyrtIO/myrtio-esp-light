import type { Configuration, SystemInformation } from "../models";

export type ProgressCallback = (progress: number) => void;

export interface ApiService {
  getConfiguration(): Promise<Configuration>;
  saveConfiguration(configuration: Configuration): Promise<void>;
  getSystemInformation(): Promise<SystemInformation>;
  updateFirmware(file: File, onProgress: ProgressCallback): Promise<void>;
}
