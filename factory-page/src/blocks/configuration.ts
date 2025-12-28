export interface WifiConfiguration {
  ssid: string;
  password: string;
}

export interface MqttConfiguration {
  host: string;
  port: number;
  username: string;
  password: string;
}

export interface LightConfiguration {
  brightness_min: number;
  brightness_max: number;
  led_count: number;
  skip_leds: number;
  color_correction: number;
}

export interface Configuration {
  wifi: WifiConfiguration;
  mqtt: MqttConfiguration;
  light: LightConfiguration;
}

/** Recursively maps all primitive fields to HTMLInputElement */
type ToInputs<T> = {
  [K in keyof T]: T[K] extends object ? ToInputs<T[K]> : HTMLInputElement;
};

export class ConfigurationBlock {
  private inputs: ToInputs<Configuration>;
  private isDirty: boolean = false;
  private form: HTMLFormElement;
  private colorPicker: HTMLInputElement;

  constructor(form: HTMLFormElement, onDirty: () => void) {
    const $ = (name: string): HTMLInputElement => {
      const input = form.querySelector<HTMLInputElement>(`[name=${name}]`);
      if (!input) {
        throw new Error(`Input with name ${name} not found`);
      }
      input.addEventListener("change", () => {
        if (!this.isDirty) {
          onDirty();
        }
        this.isDirty = true;
      });
      return input;
    };

    this.form = form;
    this.inputs = {
      wifi: {
        ssid: $("wifi_ssid"),
        password: $("wifi_password"),
      },
      mqtt: {
        host: $("mqtt_host"),
        port: $("mqtt_port"),
        username: $("mqtt_username"),
        password: $("mqtt_password"),
      },
      light: {
        brightness_min: $("brightness_min"),
        brightness_max: $("brightness_max"),
        led_count: $("led_count"),
        skip_leds: $("skip_leds"),
        color_correction: $("color_correction"),
      },
    };

    // Setup color picker sync
    this.colorPicker = document.getElementById("color_correction_picker") as HTMLInputElement;
    const hexInput = this.inputs.light.color_correction;

    this.colorPicker.addEventListener("input", () => {
      hexInput.value = this.colorPicker.value.toUpperCase();
      hexInput.dispatchEvent(new Event("change"));
    });

    hexInput.addEventListener("input", () => {
      let v = hexInput.value.toUpperCase().replace(/[^#0-9A-F]/g, "");
      if (!v.startsWith("#")) v = "#" + v.replace(/#/g, "");
      hexInput.value = v.slice(0, 7);
      if (/^#[0-9A-F]{6}$/.test(hexInput.value)) {
        this.colorPicker.value = hexInput.value;
      }
    });
  }

  public unlock() {
    this.form.classList.remove("_disabled");
  }

  public lock() {
    this.form.classList.add("_disabled");
  }

  public markClean() {
    this.isDirty = false;
  }

  public setValues(configuration: Configuration) {
    // Format color_correction as #RRGGBB before recursive set
    const hex = "#" + configuration.light.color_correction.toString(16).toUpperCase().padStart(6, "0");
    this.colorPicker.value = hex;
    recursiveSetValues(this.inputs, {
      ...configuration,
      light: { ...configuration.light, color_correction: hex as unknown as number },
    });
  }

  public validate(): boolean {
    const hex = this.inputs.light.color_correction.value;
    if (!/^#[0-9A-F]{6}$/i.test(hex)) {
      alert("Неверный формат цвета. Используйте #RRGGBB");
      return false;
    }
    return true;
  }

  public getValues(): Configuration {
    const values = recursiveGetValues(this.inputs);
    // Parse #RRGGBB back to u32
    const hex = (values.light.color_correction as unknown as string).replace("#", "");
    values.light.color_correction = parseInt(hex, 16) || 0xFFFFFF;
    return values;
  }
}

function recursiveGetValues<T extends object>(inputs: ToInputs<T>): T {
  const values: T = {} as T;
  for (const [key, inputOrObject] of Object.entries(inputs)) {
    if (inputOrObject instanceof HTMLInputElement) {
      const input = inputOrObject as HTMLInputElement;
      if (input.type === "number") {
        values[key as keyof T] = parseInt(input.value) as T[keyof T];
      } else {
        values[key as keyof T] = input.value as T[keyof T];
      }
      continue;
    }
    values[key as keyof T] = recursiveGetValues(
      inputOrObject as ToInputs<T[keyof T]>
    ) as T[keyof T];
  }
  return values;
}

function recursiveSetValues<T extends object>(inputs: ToInputs<T>, values: T) {
  for (const [key, valueOrObject] of Object.entries(values)) {
    if (typeof valueOrObject === "object") {
      recursiveSetValues(inputs[key as keyof T], valueOrObject as object);
      continue;
    }
    const input = inputs[key as keyof T] as HTMLInputElement;
    input.value = valueOrObject as string;
  }
}
