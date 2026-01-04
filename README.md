# MyrtIO ESP Light

ESP32 firmware for WS2812 LED strips with MQTT/Home Assistant integration, factory provisioning, and OTA updates.

## Features

- **Factory Provisioning** — Wi-Fi AP mode at `192.168.4.1` with a web UI for initial setup (Wi-Fi credentials, MQTT broker, LED configuration)
- **OTA Updates** — Upload new firmware via HTTP (`POST /api/ota`) without reflashing over serial
- **Home Assistant Integration** — MQTT discovery with full light control (on/off, brightness, RGB color, color temperature 1500-6500K)
- **Built-in Effects** — Static, Rainbow (multiple variants), Aurora, Lava Lamp, Sunset
- **Persistent State** — Light state and device config survive power loss (debounced flash writes)
- **Boot Button** — GPIO0 button toggles between factory and main firmware

## Hardware

| Function       | GPIO |
|----------------|------|
| WS2812 Data    | 25   |
| Boot Button    | 0    |
| Status LED     | 2    |

Maximum supported LED count: **128**

## Quick Start

Requires [just](https://github.com/casey/just) and the ESP-IDF Rust toolchain (`esp-rs`).

```bash
# Build and flash factory firmware (first time)
just run

# Build and flash main app firmware
just run-app

# Build OTA image and upload to device at 192.168.4.1
just ota

# Monitor serial output
just monitor

# Lint and format
just lint
just format
```

See `Justfile` for all available commands, or refer to `AGENTS.md` for detailed development guidelines.

## Firmware Binaries

| Binary                      | Purpose                                      |
|-----------------------------|----------------------------------------------|
| `myrtio-esp-light-factory`  | Factory provisioning (AP + HTTP + OTA)       |
| `myrtio-esp-light-app`      | Main runtime (Wi-Fi STA + MQTT + HA entity)  |

The factory firmware runs when no valid Wi-Fi/MQTT config exists. After provisioning, it uploads the main app via OTA and reboots.

## License

MIT
