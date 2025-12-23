type MacAddress = [number, number, number, number, number, number];

export interface SystemInformation {
  build_version: string;
  mac_address: MacAddress;
}

export class SystemBlock {
  private buildVersion: HTMLDivElement;
  private buildDate: HTMLDivElement;
  private macAddress: HTMLDivElement;
  private block: HTMLElement;
  private otaButton: HTMLButtonElement;
  private otaFile: HTMLInputElement;
  private bootButton: HTMLButtonElement;

  constructor(section: HTMLElement, onOta: (file: File) => void, onBoot: () => void) {
    const $ = <T extends HTMLElement = HTMLDivElement>(selector: string): T => {
      const element = section.querySelector<T>(selector);
      if (!element) {
        throw new Error(`Element with selector ${selector} not found`);
      }
      return element;
    };

    this.block = section;
    this.buildVersion = $("#build-version");
    this.buildDate = $("#build-date");
    this.macAddress = $("#mac-address");
    this.otaButton = $("#button-ota");
    this.otaFile = $("#ota-file");
    this.bootButton = $("#button-boot");

    this.otaButton.addEventListener("click", (e) => {
      e.preventDefault();
      this.otaFile.click();
    });

    this.otaFile.addEventListener("change", (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) {
        return;
      }
      onOta(file);
    });

    this.bootButton.addEventListener("click", (e) => {
      e.preventDefault();
      onBoot();
    });
  }

  public setValues(system: SystemInformation) {
    const [commitHash, buildDate] = parseBuildVersion(system.build_version);
    this.buildVersion.textContent = commitHash;
    this.buildDate.textContent = buildDate.toLocaleString();
    this.macAddress.textContent = formatMacAddress(system.mac_address);
  }

  public unlock() {
    this.block.classList.remove("_disabled");
  }

  public lock() {
    this.block.classList.add("_disabled");
  }

  public clearOtaFile() {
    this.otaFile.value = "";
  }
}

function parseBuildVersion(version: string): [string, Date] {
  const hash = version.slice(0, 7);
  const date = version.slice(8);
  return [hash, new Date(date)];
}

function formatMacAddress(macAddress: MacAddress): string {
  return macAddress.map((byte) => byte.toString(16).padStart(2, "0")).join(":");
}
