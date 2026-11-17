#!/usr/bin/env bash
set -euo pipefail

# Build Pure Rust Android APK using cargo-apk
# Prerequisites: Android NDK + cargo install cargo-apk

echo "==> Building Pure Rust Android APK for Todo..."

if ! command -v cargo-apk >/dev/null 2>&1; then
    echo "cargo-apk not found in PATH."
    echo "Install via: cargo install cargo-apk"
    echo "Ensure ANDROID_NDK_ROOT or NDK_HOME is set."
    exit 1
fi

cargo apk build --release --features android

echo "==> Build complete! Output APKs located in: target/release/apk/"
