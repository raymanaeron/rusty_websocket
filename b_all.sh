#!/bin/bash

set -e

# === Parse arguments ===
MODE="debug"
CARGO_FLAG=""
if [ "$1" == "--release" ]; then
    MODE="release"
    CARGO_FLAG="--release"
fi
TARGET="target/$MODE"

echo "[BUILD] Mode set to $MODE"
echo "[BUILD] Output path: $TARGET"

echo "Building projects..."
cargo build --manifest-path server/Cargo.toml $CARGO_FLAG

echo "Copying web project to $TARGET/web..."
mkdir -p "$TARGET/web"
cp -r server/web/* "$TARGET/web"

echo "Build and copy completed successfully."
