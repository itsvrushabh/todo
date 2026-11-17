#!/usr/bin/env bash
set -euo pipefail

# Package macOS .app bundle for Todo
# Usage: ./scripts/package-macos-app.sh [VERSION] [BINARY_PATH] [OUTPUT_DIR]

VERSION="${1:-0.2.0}"
BINARY_PATH="${2:-target/release/todo}"
OUTPUT_DIR="${3:-dist}"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found at $BINARY_PATH, compiling release binary..."
    cargo build --release --bin todo
    BINARY_PATH="target/release/todo"
fi

echo "==> Building macOS .app bundle for Todo v${VERSION}..."

APP_DIR="${OUTPUT_DIR}/Todo.app"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "${BINARY_PATH}" "${APP_DIR}/Contents/MacOS/todo"
chmod +x "${APP_DIR}/Contents/MacOS/todo"

cp packaging/desktop/todo.png "${APP_DIR}/Contents/Resources/icon.png"

cat <<EOF > "${APP_DIR}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>todo</string>
    <key>CFBundleIconFile</key>
    <string>icon.png</string>
    <key>CFBundleIdentifier</key>
    <string>io.github.itsvrushabh.todo</string>
    <key>CFBundleName</key>
    <string>Todo</string>
    <key>CFBundleDisplayName</key>
    <string>Todo</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

cd "${OUTPUT_DIR}"
tar -czvf "todo-macos-app.tar.gz" "Todo.app"
cd - >/dev/null

echo "✓ Created ${OUTPUT_DIR}/Todo.app and ${OUTPUT_DIR}/todo-macos-app.tar.gz"
