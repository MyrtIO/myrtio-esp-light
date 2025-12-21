CARGO_CONFIG := 'unstable.build-std = ["alloc", "core"]'
PARTITION_TABLE := 'partitions.csv'

build-app *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo build \
        --bin myrtio-esp-light-app \
        --config '{{CARGO_CONFIG}}' \
        {{ARGS}}

build-ota *ARGS:
    #!/bin/bash
    just build-app --release

    release_path="target/xtensa-esp32-none-elf/release/myrtio-esp-light-app"
    ota_path="$release_path.ota.bin"

    echo "Creating OTA image..."
    espflash save-image \
        --chip esp32 \
        --partition-table={{PARTITION_TABLE}} \
        $release_path \
        $ota_path


build-factory *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo build \
        --release \
        --bin myrtio-esp-light-factory \
        --config '{{CARGO_CONFIG}}' \
        {{ARGS}}

run *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo run \
        --bin myrtio-esp-light-factory \
        --release \
        --config '{{CARGO_CONFIG}}' \
        {{ARGS}}

lint *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo clippy {{ARGS}}

lint-fix *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo clippy --fix --allow-dirty {{ARGS}}
