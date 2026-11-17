#!/usr/bin/env bash
set -euo pipefail

# Build Pure Rust iOS static library and bundle structure
# Targets: aarch64-apple-ios (Physical devices), aarch64-apple-ios-sim (Apple Silicon Simulator)

echo "==> Building Pure Rust iOS target for Todo..."

TARGET="${1:-aarch64-apple-ios}"

if ! rustup target list | grep -q "${TARGET} (installed)"; then
    echo "Adding rustup target ${TARGET}..."
    rustup target add "${TARGET}"
fi

cargo build --release --target "${TARGET}" --lib

BUNDLE_DIR="target/ios/Todo.app"
mkdir -p "${BUNDLE_DIR}"
cp "target/${TARGET}/release/libtodo.a" "${BUNDLE_DIR}/libtodo.a" 2>/dev/null || true
cp packaging/ios/Info.plist "${BUNDLE_DIR}/Info.plist"
cp -r static/icons "${BUNDLE_DIR}/"

echo "==> iOS build artifacts prepared in ${BUNDLE_DIR}"
