#!/usr/bin/env bash
# build-macos-app.sh — Build Civis as a macOS .app bundle
#
# Produces Civis.app in the repo root, wrapping the Bevy client binary
# with assets, Info.plist, and an optional .icns icon.
#
# Usage:
#   bash scripts/build-macos-app.sh                # release build
#   bash scripts/build-macos-app.sh --debug        # debug build
#   bash scripts/build-macos-app.sh --no-build     # skip cargo, just bundle
#
# Output:
#   Civis.app/
#   Civis.app/Contents/
#     Info.plist
#     MacOS/civ-bevy-window
#     Resources/
#       assets/   (shipped game assets)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

PROFILE="release"
DO_BUILD=1
APP_NAME="Civis"
APP_BUNDLE="${REPO_ROOT}/${APP_NAME}.app"
CONTENTS="${APP_BUNDLE}/Contents"
MACOS_DIR="${CONTENTS}/MacOS"
RESOURCES="${CONTENTS}/Resources"
BINARY_NAME="civ-bevy-window"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)     PROFILE="debug"; shift ;;
        --no-build)  DO_BUILD=0; shift ;;
        *)           echo "Unknown: $1"; exit 1 ;;
    esac
done

step()  { printf '\033[36m[app]\033[0m %s\n' "$*"; }
ok()    { printf '\033[32m[app]\033[0m %s\n' "$*"; }
fail()  { printf '\033[31m[app]\033[0m %s\n' "$*" >&2; }

# Build
if [[ "$DO_BUILD" -eq 1 ]]; then
    step "Building Bevy client ($PROFILE)..."
    cargo build --locked --profile "$PROFILE" \
        -p civ-bevy-ref \
        --bin civ-bevy-window \
        --features bevy,egui,client-bins
    ok "Build complete"
fi

# Resolve binary path
if [[ "$PROFILE" == "release" ]]; then
    SRC_BINARY="target/release/${BINARY_NAME}"
else
    SRC_BINARY="target/debug/${BINARY_NAME}"
fi

if [[ ! -f "$SRC_BINARY" ]]; then
    fail "Binary not found: $SRC_BINARY"
    exit 1
fi

# Create .app bundle structure
step "Creating ${APP_NAME}.app bundle..."
rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR" "$RESOURCES/assets"

# Copy binary
cp "$SRC_BINARY" "${MACOS_DIR}/${BINARY_NAME}"
chmod +x "${MACOS_DIR}/${BINARY_NAME}"

# Copy assets
if [[ -d "clients/bevy-ref/assets" ]]; then
    cp -R "clients/bevy-ref/assets/" "${RESOURCES}/assets/"
    ok "Assets bundled"
else
    step "No assets directory found, skipping"
fi

# Copy icon if present
ICON_SRC="clients/bevy-ref/assets/icon.icns"
if [[ -f "$ICON_SRC" ]]; then
    cp "$ICON_SRC" "${RESOURCES}/icon.icns"
fi

# Generate Info.plist
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
BUILD=$(git rev-parse --short HEAD 2>/dev/null || echo "dev")

cat > "${CONTENTS}/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.civlab.${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${BUILD}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>${BINARY_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>CivLab — Headless deterministic civilization simulation engine</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.games</string>
</dict>
</plist>
PLIST

ok "Info.plist written (v${VERSION}, build ${BUILD})"

# Bundle size
SIZE=$(du -sh "$APP_BUNDLE" | cut -f1)
ok "=== ${APP_NAME}.app bundle ready: ${SIZE} ==="
echo "  Path: ${APP_BUNDLE}"
echo "  Run:  open ${APP_BUNDLE}"
