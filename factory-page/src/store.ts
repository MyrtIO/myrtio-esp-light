import { Store } from "./utils/store";
import type {
  Configuration,
  SystemInformation,
  ColorOrder,
  MacAddress,
} from "./model/types";

/** Form-friendly configuration draft (color_correction as hex string) */
export interface ConfigurationDraft {
  wifi: {
    ssid: string;
    password: string;
  };
  mqtt: {
    host: string;
    port: number;
    username: string;
    password: string;
  };
  light: {
    brightness_min: number;
    brightness_max: number;
    led_count: number;
    skip_leds: number;
    color_correction: string; // "#RRGGBB"
    color_order: ColorOrder;
  };
}

/** UI state for showing/hiding elements */
export interface UiState {
  loading: boolean;
  dirty: boolean;
  locked: boolean;
  otaProgress: number | null; // null = not uploading, 0-100 = progress
  saveSuccess: boolean;
}

/** Parsed system information for display */
export interface SystemInfoDisplay {
  commitHash: string;
  buildDate: string;
  macAddress: string;
}

// Default values
const defaultConfiguration: ConfigurationDraft = {
  wifi: { ssid: "", password: "" },
  mqtt: { host: "", port: 1883, username: "", password: "" },
  light: {
    brightness_min: 0,
    brightness_max: 255,
    led_count: 60,
    skip_leds: 0,
    color_correction: "#FFFFFF",
    color_order: "grb",
  },
};

const defaultUiState: UiState = {
  loading: true,
  dirty: false,
  locked: true,
  otaProgress: null,
  saveSuccess: false,
};

const defaultSystemInfo: SystemInfoDisplay = {
  commitHash: "------",
  buildDate: "------",
  macAddress: "--:--:--:--:--:--",
};

// Stores
export const configurationDraft = new Store<ConfigurationDraft>(
  defaultConfiguration
);
export const uiState = new Store<UiState>(defaultUiState);
export const systemInfo = new Store<SystemInfoDisplay>(defaultSystemInfo);

// Helper to convert API Configuration to draft format
export function configurationToDraft(config: Configuration): ConfigurationDraft {
  const hex =
    "#" +
    config.light.color_correction.toString(16).toUpperCase().padStart(6, "0");

  return {
    wifi: { ...config.wifi },
    mqtt: { ...config.mqtt },
    light: {
      ...config.light,
      color_correction: hex,
    },
  };
}

// Helper to convert draft back to API Configuration
export function draftToConfiguration(draft: ConfigurationDraft): Configuration {
  const hex = draft.light.color_correction.replace("#", "");
  const colorCorrection = parseInt(hex, 16) || 0xffffff;

  return {
    wifi: { ...draft.wifi },
    mqtt: { ...draft.mqtt },
    light: {
      ...draft.light,
      color_correction: colorCorrection,
    },
  };
}

// Helper to parse system information for display
export function parseSystemInfo(info: SystemInformation): SystemInfoDisplay {
  const [commitHash, buildDateStr] = parseBuildVersion(info.build_version);
  const buildDate = new Date(buildDateStr).toLocaleString();
  const macAddress = formatMacAddress(info.mac_address);

  return { commitHash, buildDate, macAddress };
}

function parseBuildVersion(version: string): [string, string] {
  const hash = version.slice(0, 7);
  const date = version.slice(8);
  return [hash, date];
}

function formatMacAddress(macAddress: MacAddress): string {
  return macAddress
    .map((byte) => byte.toString(16).toUpperCase().padStart(2, "0"))
    .join(":");
}

// UI state helpers
export function setLoading(loading: boolean) {
  uiState.set((s) => ({ ...s, loading }));
}

export function setDirty(dirty: boolean) {
  uiState.set((s) => ({ ...s, dirty }));
}

export function setLocked(locked: boolean) {
  uiState.set((s) => ({ ...s, locked }));
}

export function setOtaProgress(progress: number | null) {
  uiState.set((s) => ({ ...s, otaProgress: progress }));
}

export function setSaveSuccess(success: boolean) {
  uiState.set((s) => ({ ...s, saveSuccess: success }));
}

// Validate color correction format
export function isValidColorHex(hex: string): boolean {
  return /^#[0-9A-Fa-f]{6}$/.test(hex);
}
