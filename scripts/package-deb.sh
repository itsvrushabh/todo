#!/usr/bin/env bash
set -euo pipefail

# Package Linux .deb package for Todo
# Usage: ./scripts/package-deb.sh [VERSION] [ARCH] [BINARY_PATH]

VERSION="${1:-0.2.0}"
ARCH="${2:-amd64}"
BINARY_PATH="${3:-target/release/todo}"
OUTPUT_DIR="${4:-dist}"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found at $BINARY_PATH, compiling release binary..."
    cargo build --release --bin todo
    BINARY_PATH="target/release/todo"
fi

echo "==> Building .deb package for Todo v${VERSION} (${ARCH})..."

BUILD_ROOT=$(mktemp -d)
trap 'rm -rf "$BUILD_ROOT"' EXIT

# Create layout
mkdir -p "${BUILD_ROOT}/DEBIAN"
mkdir -p "${BUILD_ROOT}/usr/bin"
mkdir -p "${BUILD_ROOT}/usr/share/applications"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/512x512/apps"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/scalable/apps"

# Install files
install -Dm755 "${BINARY_PATH}" "${BUILD_ROOT}/usr/bin/todo"
install -Dm644 packaging/desktop/todo.desktop "${BUILD_ROOT}/usr/share/applications/todo.desktop"
install -Dm644 packaging/desktop/todo.png "${BUILD_ROOT}/usr/share/icons/hicolor/512x512/apps/todo.png"
install -Dm644 packaging/desktop/todo.svg "${BUILD_ROOT}/usr/share/icons/hicolor/scalable/apps/todo.svg"

# Write control file
cat <<EOF > "${BUILD_ROOT}/DEBIAN/control"
Package: todo
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Todo Contributors <https://github.com/itsvrushabh/todo>
Description: High-performance cross-platform Todo and Task Management
 Fast, memory-safe task manager featuring Desktop GUI, TUI, CLI, and offline sync.
EOF

mkdir -p "${OUTPUT_DIR}"
DEB_NAME="todo_${VERSION}_${ARCH}.deb"

if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build "${BUILD_ROOT}" "${OUTPUT_DIR}/${DEB_NAME}"
    echo "✓ Created ${OUTPUT_DIR}/${DEB_NAME}"
else
    echo "dpkg-deb not found. Creating tar.gz package staging structure instead."
    tar -czf "${OUTPUT_DIR}/${DEB_NAME}.tar.gz" -C "${BUILD_ROOT}" .
    echo "✓ Created ${OUTPUT_DIR}/${DEB_NAME}.tar.gz"
fi
