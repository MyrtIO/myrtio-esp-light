import { createApiService } from "../api";
import type { LightConfiguration } from "../model/types";
import {
  configurationDraft,
  uiState,
  systemInfo,
  configurationToDraft,
  draftToConfiguration,
  parseSystemInfo,
  setLoading,
  setDirty,
  setLocked,
  setOtaProgress,
  setSaveSuccess,
  isValidColorHex,
  type ConfigurationDraft,
} from "../store";
import { sleep } from "../utils/sleep";

// DOM elements cache
let elements: {
  form: HTMLFormElement;
  saveBar: HTMLElement;
  saveBtn: HTMLButtonElement;
  loader: HTMLElement;
  buildVersion: HTMLElement;
  buildDate: HTMLElement;
  macAddress: HTMLElement;
  otaBtn: HTMLButtonElement;
  otaFile: HTMLInputElement;
  bootBtn: HTMLButtonElement;
  otaProgress: HTMLElement;
  otaProgressFill: HTMLElement;
  systemBlock: HTMLElement;
};

// API service
const api = createApiService("/api");

/**
 * Initialize the application
 */
export async function initApp() {
  // Cache DOM elements
  elements = {
    form: document.getElementById("config-form") as HTMLFormElement,
    saveBar: document.getElementById("save-bar") as HTMLElement,
    saveBtn: document.getElementById("btn-save") as HTMLButtonElement,
    loader: document.getElementById("header-loader") as HTMLElement,
    buildVersion: document.getElementById("build-version") as HTMLElement,
    buildDate: document.getElementById("build-date") as HTMLElement,
    macAddress: document.getElementById("mac-address") as HTMLElement,
    otaBtn: document.getElementById("btn-ota") as HTMLButtonElement,
    otaFile: document.getElementById("ota-file") as HTMLInputElement,
    bootBtn: document.getElementById("btn-boot") as HTMLButtonElement,
    otaProgress: document.getElementById("ota-progress") as HTMLElement,
    otaProgressFill: document.getElementById("ota-progress-fill") as HTMLElement,
    systemBlock: document.getElementById("system-block") as HTMLElement,
  };

  // Setup subscriptions to stores
  setupStoreSubscriptions();

  // Setup event listeners
  setupEventListeners();

  // Load initial data
  await loadInitialData();
}

/**
 * Setup subscriptions to react to store changes
 */
function setupStoreSubscriptions() {
  // UI state changes
  uiState.subscribe((state) => {
    // Loading spinner
    elements.loader.classList.toggle("_visible", state.loading);

    // Save bar visibility
    elements.saveBar.classList.toggle("_visible", state.dirty);

    // Save button success state
    elements.saveBtn.classList.toggle("_success", state.saveSuccess);
    elements.saveBtn.textContent = state.saveSuccess ? "Сохранено" : "Сохранить";

    // Lock/unlock form
    elements.form.classList.toggle("_disabled", state.locked);
    elements.systemBlock.classList.toggle("_disabled", state.locked);

    // OTA progress
    if (state.otaProgress !== null) {
      elements.otaProgress.classList.add("_visible");
      elements.otaProgressFill.style.width = `${state.otaProgress}%`;
    } else {
      elements.otaProgress.classList.remove("_visible");
      elements.otaProgressFill.style.width = "0%";
    }
  });

  // System info changes
  systemInfo.subscribe((info) => {
    elements.buildVersion.textContent = info.commitHash;
    elements.buildDate.textContent = info.buildDate;
    elements.macAddress.textContent = info.macAddress;
  });

  // Configuration changes - update form fields
  configurationDraft.subscribe((draft) => {
    updateFormFromDraft(draft);
  });
}

/**
 * Setup all event listeners
 */
function setupEventListeners() {
  // Form field changes
  elements.form.addEventListener("input", handleFieldInput);
  elements.form.addEventListener("change", handleFieldChange);

  // Password toggles
  document.querySelectorAll("[data-toggle-password]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const input = btn.previousElementSibling as HTMLInputElement;
      input.type = input.type === "password" ? "text" : "password";
    });
  });

  // Color picker sync
  const colorInput = elements.form.querySelector(
    '[name="color_correction"]'
  ) as HTMLInputElement;
  if (colorInput) {
    colorInput.addEventListener("input", handleColorInput);
    colorInput.addEventListener("change", handleColorChange);
  }

  // Range sliders
  setupRangeSliders();

  // Save button
  elements.saveBtn.addEventListener("click", handleSave);

  // Test color buttons
  document.querySelectorAll("[data-test-color]").forEach((btn) => {
    btn.addEventListener("click", () => handleTestColor(btn.id));
  });

  // OTA button
  elements.otaBtn.addEventListener("click", () => elements.otaFile.click());
  elements.otaFile.addEventListener("change", handleOtaFileSelect);

  // Boot button
  elements.bootBtn.addEventListener("click", handleBoot);
}

/**
 * Load initial configuration and system info
 */
async function loadInitialData() {
  setLoading(true);

  try {
    await sleep(100); // Small delay for server

    const [config, sysInfo] = await Promise.all([
      api.getConfiguration(),
      api.getSystemInformation(),
    ]);

    // Update stores
    configurationDraft.set(configurationToDraft(config));
    systemInfo.set(parseSystemInfo(sysInfo));

    // Unlock UI
    setLocked(false);
  } catch (error) {
    console.error("Failed to load initial data:", error);
  } finally {
    setLoading(false);
  }
}

/**
 * Update form fields from draft
 */
function updateFormFromDraft(draft: ConfigurationDraft) {
  const fieldMap: Record<string, string | number> = {
    wifi_ssid: draft.wifi.ssid,
    wifi_password: draft.wifi.password,
    mqtt_host: draft.mqtt.host,
    mqtt_port: draft.mqtt.port,
    mqtt_username: draft.mqtt.username,
    mqtt_password: draft.mqtt.password,
    led_count: draft.light.led_count,
    skip_leds: draft.light.skip_leds,
    color_order: draft.light.color_order,
    color_correction: draft.light.color_correction,
    brightness_min: draft.light.brightness_min,
    brightness_max: draft.light.brightness_max,
  };

  for (const [name, value] of Object.entries(fieldMap)) {
    const input = elements.form.querySelector(
      `[name="${name}"]`
    ) as HTMLInputElement | HTMLSelectElement | null;
    if (input && input.value !== String(value)) {
      input.value = String(value);
    }
  }

  // Update color preview
  const colorPreview = document.querySelector("[data-color-preview]") as HTMLElement;
  const colorHex = document.querySelector("[data-color-hex]") as HTMLElement;
  if (colorPreview && colorHex) {
    colorPreview.style.background = draft.light.color_correction;
    colorHex.textContent = draft.light.color_correction.toUpperCase();
  }

  // Update range slider display
  updateRangeDisplay(draft.light.brightness_min, draft.light.brightness_max);
}

/**
 * Handle input event on form fields
 */
function handleFieldInput(e: Event) {
  const target = e.target as HTMLInputElement | HTMLSelectElement;
  const name = target.name;
  if (!name) return;

  // Mark as dirty on any input
  if (!uiState.get().dirty) {
    setDirty(true);
  }
}

/**
 * Handle change event on form fields (committed value)
 */
function handleFieldChange(e: Event) {
  const target = e.target as HTMLInputElement | HTMLSelectElement;
  const name = target.name;
  if (!name) return;

  // Update draft store
  updateDraftFromField(name, target.value);

  // Auto-send light changes
  if (isLightField(name)) {
    sendLightConfiguration();
  }
}

/**
 * Update draft store from a single field change
 */
function updateDraftFromField(name: string, value: string) {
  configurationDraft.set((draft) => {
    const newDraft = { ...draft };

    switch (name) {
      case "wifi_ssid":
        newDraft.wifi = { ...draft.wifi, ssid: value };
        break;
      case "wifi_password":
        newDraft.wifi = { ...draft.wifi, password: value };
        break;
      case "mqtt_host":
        newDraft.mqtt = { ...draft.mqtt, host: value };
        break;
      case "mqtt_port":
        newDraft.mqtt = { ...draft.mqtt, port: parseInt(value) || 1883 };
        break;
      case "mqtt_username":
        newDraft.mqtt = { ...draft.mqtt, username: value };
        break;
      case "mqtt_password":
        newDraft.mqtt = { ...draft.mqtt, password: value };
        break;
      case "led_count":
        newDraft.light = { ...draft.light, led_count: parseInt(value) || 0 };
        break;
      case "skip_leds":
        newDraft.light = { ...draft.light, skip_leds: parseInt(value) || 0 };
        break;
      case "color_order":
        newDraft.light = {
          ...draft.light,
          color_order: value as ConfigurationDraft["light"]["color_order"],
        };
        break;
      case "color_correction":
        newDraft.light = { ...draft.light, color_correction: value.toUpperCase() };
        break;
      case "brightness_min":
        newDraft.light = {
          ...draft.light,
          brightness_min: parseInt(value) || 0,
        };
        break;
      case "brightness_max":
        newDraft.light = {
          ...draft.light,
          brightness_max: parseInt(value) || 255,
        };
        break;
    }

    return newDraft;
  });
}

/**
 * Check if field is a light configuration field
 */
function isLightField(name: string): boolean {
  return [
    "led_count",
    "skip_leds",
    "color_order",
    "color_correction",
    "brightness_min",
    "brightness_max",
  ].includes(name);
}

/**
 * Send light configuration to API
 */
async function sendLightConfiguration() {
  const draft = configurationDraft.get();

  // Skip if color is invalid
  if (!isValidColorHex(draft.light.color_correction)) {
    return;
  }

  const config = draftToConfiguration(draft);
  try {
    await api.setLightConfiguration(config.light);
  } catch (error) {
    console.error("Failed to send light configuration:", error);
  }
}

/**
 * Handle color input (while dragging)
 */
function handleColorInput(e: Event) {
  const input = e.target as HTMLInputElement;
  const colorPreview = document.querySelector("[data-color-preview]") as HTMLElement;
  const colorHex = document.querySelector("[data-color-hex]") as HTMLElement;

  if (colorPreview && colorHex) {
    colorPreview.style.background = input.value;
    colorHex.textContent = input.value.toUpperCase();
  }

  if (!uiState.get().dirty) {
    setDirty(true);
  }
}

/**
 * Handle color change (committed)
 */
function handleColorChange(e: Event) {
  const input = e.target as HTMLInputElement;
  updateDraftFromField("color_correction", input.value);
  sendLightConfiguration();
}

/**
 * Setup range sliders
 */
function setupRangeSliders() {
  const rangeMin = elements.form.querySelector(
    '[name="brightness_min"]'
  ) as HTMLInputElement;
  const rangeMax = elements.form.querySelector(
    '[name="brightness_max"]'
  ) as HTMLInputElement;

  if (!rangeMin || !rangeMax) return;

  const updateRanges = (source: HTMLInputElement) => {
    let min = parseInt(rangeMin.value);
    let max = parseInt(rangeMax.value);

    // Ensure min doesn't exceed max - 10
    if (min > max - 10) {
      if (source === rangeMin) {
        rangeMin.value = String(max - 10);
        min = max - 10;
      } else {
        rangeMax.value = String(min + 10);
        max = min + 10;
      }
    }

    updateRangeDisplay(min, max);
  };

  rangeMin.addEventListener("input", () => updateRanges(rangeMin));
  rangeMax.addEventListener("input", () => updateRanges(rangeMax));
}

/**
 * Update range slider display
 */
function updateRangeDisplay(min: number, max: number) {
  const valMin = document.querySelector("[data-range-val-min]");
  const valMax = document.querySelector("[data-range-val-max]");
  const fill = document.querySelector("[data-range-fill]") as HTMLElement;

  if (valMin) valMin.textContent = String(min);
  if (valMax) valMax.textContent = String(max);

  if (fill) {
    const percentMin = (min / 255) * 100;
    const percentMax = (max / 255) * 100;
    fill.style.left = `${percentMin}%`;
    fill.style.width = `${percentMax - percentMin}%`;
  }
}

/**
 * Handle save button click
 */
async function handleSave() {
  const draft = configurationDraft.get();

  // Validate color
  if (!isValidColorHex(draft.light.color_correction)) {
    alert("Неверный формат цвета. Используйте #RRGGBB");
    return;
  }

  setLoading(true);

  try {
    const config = draftToConfiguration(draft);
    await api.saveConfiguration(config);

    setDirty(false);
    setSaveSuccess(true);

    // Reset success state after 2 seconds
    setTimeout(() => setSaveSuccess(false), 2000);
  } catch (error) {
    console.error("Failed to save configuration:", error);
    alert("Ошибка сохранения конфигурации");
  } finally {
    setLoading(false);
  }
}

/**
 * Handle test color button click
 */
async function handleTestColor(buttonId: string) {
  const testBrightness = 128;
  const colors: Record<string, { r: number; g: number; b: number }> = {
    "test-red": { r: 255, g: 0, b: 0 },
    "test-green": { r: 0, g: 255, b: 0 },
    "test-blue": { r: 0, g: 0, b: 255 },
    "test-white": { r: 255, g: 255, b: 255 },
  };

  const color = colors[buttonId];
  if (!color) return;

  try {
    await api.testColor({ ...color, brightness: testBrightness });
  } catch (error) {
    console.error("Failed to test color:", error);
  }
}

/**
 * Handle OTA file selection
 */
async function handleOtaFileSelect(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  if (!confirm(`Обновить прошивку файлом ${file.name}?`)) {
    input.value = "";
    return;
  }

  // Lock UI
  setLocked(true);
  setLoading(true);
  setDirty(false);
  setOtaProgress(0);

  try {
    await api.updateFirmware(file, (progress) => {
      setOtaProgress(progress);
    });

    alert("Прошивка обновлена, устройство запустится в течение 30 секунд");
  } catch (error) {
    console.error("Failed to update firmware:", error);
    alert("Ошибка обновления прошивки");
  } finally {
    setLoading(false);
    setOtaProgress(null);
    setLocked(false);
    input.value = "";
  }
}

/**
 * Handle boot button click
 */
async function handleBoot() {
  if (!confirm("Запустить устройство с текущей конфигурацией?")) {
    return;
  }

  setLoading(true);

  try {
    await api.bootSystem();
    alert("Система запущена, устройство будет доступно в течение 10 секунд");
  } catch (error) {
    console.error("Failed to boot system:", error);
    alert("Ошибка запуска системы");
  } finally {
    setLoading(false);
  }
}
