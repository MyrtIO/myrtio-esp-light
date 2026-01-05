import { FetchApiService } from "./api/fetch";
import { MockApiService } from "./api/mock";
import { ConfigurationBlock } from "./blocks/configuration";
import { HeaderBlock } from "./blocks/header";
import { SystemBlock } from "./blocks/system";
import "./style.scss";

async function main() {
  const configForm = document.querySelector<HTMLFormElement>("#configForm");
  if (!configForm) {
    throw new Error("Config form not found");
  }
  const systemSection =
    document.querySelector<HTMLDivElement>(".system-section");
  if (!systemSection) {
    throw new Error("System section not found");
  }
  const header = document.querySelector<HTMLDivElement>("#header");
  if (!header) {
    throw new Error("Header not found");
  }

  const api = import.meta.env.VITE_MOCK_API
    ? new MockApiService()
    : new FetchApiService("/api");

  const systemForm = new SystemBlock(systemSection, onFirmwareUpdate, onBoot);
  const headerBlock = new HeaderBlock(header, onConfigurationSave);
  const configurationForm = new ConfigurationBlock(configForm, {
    onDirty: () => headerBlock.showSaveButton(),
    onLightChange: (light) => {
      api.setLightConfiguration(light).catch(console.error);
    },
  });

  // Wire up test color buttons
  const testRed = document.getElementById("test-red");
  const testGreen = document.getElementById("test-green");
  const testBlue = document.getElementById("test-blue");
  const testWhite = document.getElementById("test-white");

  const testColor = (r: number, g: number, b: number, brightness: number) => {
    api.testColor({ r, g, b, brightness }).catch(console.error);
  };

  const testBrightness = 128;

  testRed?.addEventListener("click", () =>
    testColor(255, 0, 0, testBrightness)
  );
  testGreen?.addEventListener("click", () =>
    testColor(0, 255, 0, testBrightness)
  );
  testBlue?.addEventListener("click", () =>
    testColor(0, 0, 255, testBrightness)
  );
  testWhite?.addEventListener("click", () =>
    testColor(255, 255, 255, testBrightness)
  );

  async function onBoot() {
    await api.bootSystem();
    alert("Система запущена, устройство будет доступно в течение 10 секунд");
  }

  async function onConfigurationSave(e: Event) {
    e.preventDefault();
    if (!configurationForm.validate()) return;
    const values = configurationForm.getValues();
    headerBlock.showLoader();
    headerBlock.hideSaveButton();
    try {
      await api.saveConfiguration(values);
    } catch (error) {
      console.error(error);
    }
    headerBlock.hideLoader();
    configurationForm.markClean();
  }

  async function onFirmwareUpdate(file: File) {
    if (!confirm(`Обновить прошивку файлом ${file.name}?`)) {
      systemForm.clearOtaFile();
      return;
    }
    configurationForm.lock();
    systemForm.lock();
    headerBlock.showLoader();
    headerBlock.hideSaveButton();
    headerBlock.showProgressBar();
    headerBlock.setProgress(0);
    try {
      await api.updateFirmware(file, (progress) =>
        headerBlock.setProgress(progress)
      );
    } catch (error) {
      console.error(error);
    }
    headerBlock.hideLoader();
    headerBlock.hideProgressBar();
    alert("Прошивка обновлена, устройство запустится в течение 30 секунд");
  }

  // Load initial state
  const [configuration, system] = await Promise.all([
    api.getConfiguration(),
    api.getSystemInformation(),
  ]);

  configurationForm.setValues(configuration);
  systemForm.setValues(system);

  configurationForm.unlock();
  systemForm.unlock();

  headerBlock.hideLoader();
}

main().catch(console.error);
