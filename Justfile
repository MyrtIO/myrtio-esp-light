set dotenv-load := true

CARGO_CONFIG := 'unstable.build-std = ["alloc", "core"]'

build-esp32:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo build --bin myrtio-light-firmware --config '{{CARGO_CONFIG}}'

run-rs1:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo --config '{{CARGO_CONFIG}}' run --bin myrtio-light-firmware --release --features rs1

lint:
    cargo clippy --config .cargo/config_esp32.toml

# Test the myrtio-macros crate (requires separate target dir due to workspace esp config)
test package="":
    cargo test --package unit-tests --target aarch64-apple-darwin

# Build and OTA flash the rs1 device
ota-rs1:
    #!/bin/bash
    source $HOME/export-esp.sh
    set -e

    echo "Building firmware..."
    cargo --config '{{CARGO_CONFIG}}' build --bin myrtio-light-firmware --release --features rs1

    echo "Creating OTA image..."
    mkdir -p target/ota
    espflash save-image --chip esp32 \
        target/xtensa-esp32-none-elf/release/myrtio-light-firmware \
        target/ota/myrtio-light-firmware-rs1.bin

    echo "Starting OTA update..."
    python3 scripts/ota.py --host myrtio-rs1.lan --image target/ota/myrtio-light-firmware-rs1.bin

