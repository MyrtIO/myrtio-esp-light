set dotenv-load := true

build-esp32:
    #!/bin/bash
    source $HOME/export-esp.sh
    cargo build

# Test the myrtio-macros crate (requires separate target dir due to workspace esp config)
test-macros:
    cd crates/myrtio-macros && CARGO_TARGET_DIR=/tmp/myrtio-macros-test cargo test -Z build-std=panic_abort --target {{arch()}}-apple-darwin
