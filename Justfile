CARGO_CONFIG := 'unstable.build-std = ["alloc", "core"]'
PARTITION_TABLE := 'crates/myrtio-firmware/light/partitions.csv'

build *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo build \
        --config '{{CARGO_CONFIG}}' \
        {{ARGS}}

run *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo run \
        --config '{{CARGO_CONFIG}}' \
        {{ARGS}}

ota device="" host="" chip="esp32":
    #!/bin/bash
    if test -z "{{device}}"; then
        echo "Device is required"
        exit 1
    fi
    if test -z "{{host}}"; then
        echo "Host is required"
        exit 1
    fi
    echo "Building firmware..."
    just build

    release_path="target/xtensa-esp32-none-elf/release/myrtio-light-firmware"
    ota_path="$release_path.ota.bin"

    echo "Creating OTA image..."
    export ota_dir="target/ota/{{device}}"
    espflash save-image \
        --chip {{chip}} \
        --partition-table={{PARTITION_TABLE}} \
        $release_path \
        $ota_path
    python3 scripts/ota.py --host {{host}} --image $ota_path


lint *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo clippy {{ARGS}}

lint-fix *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo clippy --fix --allow-dirty {{ARGS}}

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

