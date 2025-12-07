set dotenv-load := true

build-esp32:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo build --config .cargo/config_esp32.toml

run-rs1:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo run --release --features rs1

lint:
    cargo clippy --config .cargo/config_esp32.toml

# Test the myrtio-macros crate (requires separate target dir due to workspace esp config)
test-macros:
    cd crates/myrtio-macros && CARGO_TARGET_DIR=/tmp/myrtio-macros-test cargo test -Z build-std=panic_abort --target {{arch()}}-apple-darwin
