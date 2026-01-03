# App variables
APP_MAIN_NAME := "myrtio-esp-light-app"
APP_FACTORY_NAME := "myrtio-esp-light-factory"
APP_TARGET := "xtensa-esp32-none-elf"

# Build paths
TARGET_PATH := "target" / APP_TARGET
APP_RELEASE_PATH := TARGET_PATH / "release" / APP_MAIN_NAME
OTA_PATH := APP_RELEASE_PATH + ".bin"

# Internal variables
PARTITION_TABLE := "partitions.csv"

build-app *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo build \
        --bin {{APP_MAIN_NAME}} \
        {{ARGS}}

build-app-log *ARGS:
    just build-app --features log {{ARGS}}

build-ota *ARGS:
    just build-app --release {{ARGS}}
    @espflash save-image \
        --chip esp32 \
        --partition-table={{PARTITION_TABLE}} \
        {{APP_RELEASE_PATH}} \
        {{OTA_PATH}}

build-factory *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo build \
        --release \
        --bin {{APP_FACTORY_NAME}} \
        {{ARGS}}

run *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh

    cargo run \
        --bin {{APP_FACTORY_NAME}} \
        --release \
        {{ARGS}}

run-app *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo run \
        --bin {{APP_MAIN_NAME}} \
        --release \
        {{ARGS}}

run-app-log *ARGS:
    just run-app --features log {{ARGS}}

ota: build-ota
    @echo "Sending app..."
    @curl -X POST http://192.168.4.1/api/ota \
        --data-binary "@{{OTA_PATH}}"

run-factory-page *ARGS:
    cd factory-page && VITE_MOCK_API=true bun run dev -- {{ARGS}}

lint *ARGS:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo clippy {{ARGS}}

lint-fix *ARGS:
    just lint --fix --allow-dirty {{ARGS}}

monitor:
    espflash monitor

format *ARGS:
    cargo fmt {{ARGS}}
