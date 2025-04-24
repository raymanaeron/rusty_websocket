#!/bin/bash

set -e # Exit immediately if a command exits with a non-zero status

# === Parse arguments ===
# Default build mode is debug
MODE="debug"
CARGO_FLAG=""
# Check if the first argument is "--release" and set mode accordingly
if [ "$1" == "--release" ]; then
    MODE="release"
    CARGO_FLAG="--release"
fi
# Set the target directory based on the build mode
TARGET="target/$MODE"

# Display the build mode and output path
echo "[BUILD] Mode set to $MODE"
echo "[BUILD] Output path: $TARGET"

# Build the Rust project using Cargo
echo "Building projects..."
cargo build --manifest-path server/Cargo.toml $CARGO_FLAG

# Copy the web project files to the target directory
echo "Copying web project to $TARGET/web..."
mkdir -p "$TARGET/web" # Ensure the target directory exists
cp -r server/web/* "$TARGET/web" # Copy all files from the web directory

# Indicate that the build and copy process completed successfully
echo "Build and copy completed successfully."
