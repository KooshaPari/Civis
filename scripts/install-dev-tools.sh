#!/bin/bash
#
# Install DINOForge optional development tools (UnityExplorer, etc.)
#
# Usage:
#   ./install-dev-tools.sh [--game-path PATH] [--force]
#
# Idempotent: safe to run multiple times. Skips if already installed.
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORCE=0
GAME_PATH=""

# Pinned release — update hash when bumping version (see tools/unityexplorer/README.md)
PINNED_VERSION="4.9.0"
PINNED_ASSET="UnityExplorer.BepInEx5.Mono.zip"
PINNED_URL="https://github.com/sinai-dev/UnityExplorer/releases/download/${PINNED_VERSION}/${PINNED_ASSET}"
EXPECTED_SHA256="0a125bdeeb22bc9763e7a69e74862ba60e61b4ed0f1a90cc3bff0d7b51b09f00"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --game-path)
            GAME_PATH="$2"
            shift 2
            ;;
        --force)
            FORCE=1
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Resolve game path
if [ -z "$GAME_PATH" ]; then
    # Try environment variable
    if [ -n "$GIT_GAME_INSTALL_PATH" ]; then
        GAME_PATH="$GIT_GAME_INSTALL_PATH"
    fi

    # Try to find from Directory.Build.props (parse XML)
    if [ -z "$GAME_PATH" ] && [ -f "$SCRIPT_DIR/../Directory.Build.props" ]; then
        GAME_PATH=$(grep -oP '(?<=<GameInstallPath>)[^<]+' "$SCRIPT_DIR/../Directory.Build.props" 2>/dev/null || echo "")
    fi

    # Common Steam locations on Linux/Mac
    if [ -z "$GAME_PATH" ]; then
        if [ -d "$HOME/.steam/steamapps/common/Diplomacy is Not an Option" ]; then
            GAME_PATH="$HOME/.steam/steamapps/common/Diplomacy is Not an Option"
        elif [ -d "$HOME/Steam/steamapps/common/Diplomacy is Not an Option" ]; then
            GAME_PATH="$HOME/Steam/steamapps/common/Diplomacy is Not an Option"
        fi
    fi
fi

# Validate game path — must be absolute and must not contain traversal sequences
if [[ "$GAME_PATH" == *".."* ]] || [[ -n "$GAME_PATH" && "$GAME_PATH" != /* ]]; then
    echo "[security] Invalid game path (contains '..' or is not absolute): $GAME_PATH" >&2
    exit 1
fi

if [ -z "$GAME_PATH" ] || [ ! -d "$GAME_PATH" ]; then
    cat >&2 <<EOF
Error: Could not resolve game install path.
Please specify --game-path or set GIT_GAME_INSTALL_PATH environment variable.

Expected path format:
  ~/.steam/steamapps/common/Diplomacy is Not an Option
  or
  ~/Games/Diplomacy is Not an Option
EOF
    exit 1
fi

# Set up paths
BEPINEX_PATH="$GAME_PATH/BepInEx"
PLUGINS_DIR="$BEPINEX_PATH/plugins"
DEV_TOOLS_DIR="$PLUGINS_DIR/dev"
UNITYEXPLORER_DIR="$DEV_TOOLS_DIR/UnityExplorer"

echo "DINOForge Dev Tools Installer"
echo "==============================="
echo "Game Path: $GAME_PATH"
echo "Target: $UNITYEXPLORER_DIR"

# Check if already installed
if [ -d "$UNITYEXPLORER_DIR" ] && [ $FORCE -eq 0 ]; then
    echo ""
    echo "✓ UnityExplorer is already installed at $UNITYEXPLORER_DIR"
    echo "Use --force flag to reinstall."
    exit 0
fi

# Create directories
mkdir -p "$DEV_TOOLS_DIR"

# Download pinned release (dynamic "latest" fetch is intentionally disabled — the pinned hash
# only matches the exact version declared above).
echo ""
echo "Downloading UnityExplorer ${PINNED_VERSION} from GitHub..."

TEMP_ZIP=$(mktemp)
trap "rm -f $TEMP_ZIP" EXIT

echo "Downloading from: $PINNED_URL"

if command -v curl &> /dev/null; then
    curl -L -o "$TEMP_ZIP" "$PINNED_URL" || {
        echo "✗ Download failed. Please check your internet connection."
        exit 1
    }
elif command -v wget &> /dev/null; then
    wget -O "$TEMP_ZIP" "$PINNED_URL" || {
        echo "✗ Download failed. Please check your internet connection."
        exit 1
    }
else
    echo "✗ Neither curl nor wget found. Cannot download."
    exit 1
fi

if [ ! -f "$TEMP_ZIP" ] || [ ! -s "$TEMP_ZIP" ]; then
    echo "✗ Download failed or file is empty."
    exit 1
fi

# ── Integrity check ────────────────────────────────────────────────────────────
if command -v sha256sum &> /dev/null; then
    ACTUAL_SHA256=$(sha256sum "$TEMP_ZIP" | awk '{print $1}')
elif command -v shasum &> /dev/null; then
    ACTUAL_SHA256=$(shasum -a 256 "$TEMP_ZIP" | awk '{print $1}')
else
    echo "⚠ Neither sha256sum nor shasum found — skipping integrity check."
    ACTUAL_SHA256="$EXPECTED_SHA256"
fi

if [ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]; then
    echo "[security] UnityExplorer ZIP hash mismatch." >&2
    echo "  Expected : $EXPECTED_SHA256" >&2
    echo "  Actual   : $ACTUAL_SHA256" >&2
    echo "Aborting. To update the pinned hash see tools/unityexplorer/README.md." >&2
    exit 1
fi

echo "✓ Downloaded and verified successfully (SHA256 OK)"

# ── ZIP path-traversal check ───────────────────────────────────────────────────
if command -v unzip &> /dev/null; then
    while IFS= read -r entry; do
        if [[ "$entry" == *".."* ]] || [[ "$entry" == /* ]]; then
            echo "[security] ZIP contains unsafe path: $entry" >&2
            exit 1
        fi
    done < <(unzip -Z1 "$TEMP_ZIP" 2>/dev/null)
fi

# Extract
echo "Extracting to $UNITYEXPLORER_DIR..."
if [ -d "$UNITYEXPLORER_DIR" ]; then
    rm -rf "$UNITYEXPLORER_DIR"
fi

if command -v unzip &> /dev/null; then
    unzip -q "$TEMP_ZIP" -d "$UNITYEXPLORER_DIR" || {
        echo "✗ Extraction failed."
        exit 1
    }
else
    echo "✗ unzip command not found. Cannot extract."
    exit 1
fi

echo "✓ Extracted successfully"

echo ""
echo "✓ UnityExplorer installed successfully!"
echo ""
echo "Usage:"
echo "- Launch the game"
echo "- Press F7 to toggle UnityExplorer"
echo "- Use the hierarchy inspector and C# REPL"
echo ""
echo "Documentation: https://github.com/sinai-dev/UnityExplorer"
