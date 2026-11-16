#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/update-dist-hashes.sh <VERSION> <ASSETS_DIR>
VERSION="${1:-0.1.0}"
ASSETS_DIR="${2:-release_assets}"

echo "==> Updating package manifests for version v${VERSION} from ${ASSETS_DIR}..."

AMD64_LINUX="${ASSETS_DIR}/todo-linux-amd64.tar.gz"
ARM64_LINUX="${ASSETS_DIR}/todo-linux-arm64.tar.gz"
ARM64_MACOS="${ASSETS_DIR}/todo-macos-arm64.tar.gz"
X86_64_MACOS="${ASSETS_DIR}/todo-macos-x86_64.tar.gz"

get_hash() {
    local file="$1"
    if [ -f "$file" ]; then
        sha256sum "$file" | awk '{print $1}'
    elif [ -f "${file}.sha256" ]; then
        awk '{print $1}' "${file}.sha256"
    else
        echo "0000000000000000000000000000000000000000000000000000000000000000"
    fi
}

HASH_AMD64_LINUX=$(get_hash "$AMD64_LINUX")
HASH_ARM64_LINUX=$(get_hash "$ARM64_LINUX")
HASH_ARM64_MACOS=$(get_hash "$ARM64_MACOS")
HASH_X86_64_MACOS=$(get_hash "$X86_64_MACOS")

# 1. Update AUR PKGBUILD
if [ -f packaging/aur/PKGBUILD ]; then
    sed -i "s/^pkgver=.*/pkgver=${VERSION}/" packaging/aur/PKGBUILD
    if [ "$HASH_AMD64_LINUX" != "0000000000000000000000000000000000000000000000000000000000000000" ]; then
        sed -i "s/^sha256sums_x86_64=.*/sha256sums_x86_64=('${HASH_AMD64_LINUX}')/" packaging/aur/PKGBUILD
        sed -i "s/^sha256sums_aarch64=.*/sha256sums_aarch64=('${HASH_ARM64_LINUX}')/" packaging/aur/PKGBUILD
    fi
    if command -v makepkg >/dev/null 2>&1; then
        (cd packaging/aur && makepkg --printsrcinfo > .SRCINFO)
    fi
    echo "✓ Updated packaging/aur/PKGBUILD and packaging/aur/.SRCINFO"
fi

# 2. Update Homebrew Formula
update_formula() {
    local formula_path="$1"
    if [ -f "$formula_path" ]; then
        sed -i "s/version \".*\"/version \"${VERSION}\"/" "$formula_path"
        echo "✓ Updated version in ${formula_path}"
    fi
}

update_formula Formula/todo.rb
update_formula packaging/homebrew/todo.rb

echo "==> Packaging manifest update complete for v${VERSION}."
