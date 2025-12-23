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
    recursiveSetValues(this.inputs, configuration);
  }

  public getValues(): Configuration {
    return recursiveGetValues(this.inputs);
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
