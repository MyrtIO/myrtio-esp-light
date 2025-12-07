# MyrtIO Lighter

Firmware for light devices controlled by Home Assistant.

## Supported platforms

- `esp32`

## Flash partition layout

This firmware uses a custom partition table (`partitions.csv`) to allocate a dedicated `light_state` data partition for persistent storage of the light state.

| Name        | Type | SubType | Offset     | Size       |
|-------------|------|---------|------------|------------|
| nvs         | data | nvs     | 0x9000     | 0x6000     |
| phy_init    | data | phy     | 0xF000     | 0x1000     |
| factory     | app  | factory | 0x10000    | 0x3E0000   |
| light_state | data | 0x40    | 0x3F0000   | 0x10000    |

The `EspNorFlashStorageDriver` writes only to the `light_state` partition (offset `0x3F0000`, size 64 KiB).

## Flashing

The custom partition table is automatically used when flashing via `cargo run` because `.cargo/config.toml` specifies `--partition-table=crates/myrtio-firmware/light/partitions.csv`.

If flashing manually with `espflash`, make sure to include the partition table:

```bash
espflash flash --monitor --partition-table=crates/myrtio-firmware/light/partitions.csv target/xtensa-esp32-none-elf/release/myrtio-light-firmware
```