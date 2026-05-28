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

# Validate game path
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

# Download latest release
echo ""
echo "Downloading UnityExplorer from GitHub..."

TEMP_ZIP=$(mktemp)
trap "rm -f $TEMP_ZIP" EXIT

RELEASE_URL="https://github.com/sinai-dev/UnityExplorer/releases/download/4.8.2/UnityExplorer.BepInEx.Mono.zip"

# Try to fetch latest release dynamically
if command -v curl &> /dev/null; then
    LATEST_RELEASE=$(curl -s "https://api.github.com/repos/sinai-dev/UnityExplorer/releases/latest" 2>/dev/null || echo "")
    if [ -n "$LATEST_RELEASE" ]; then
        ASSET_URL=$(echo "$LATEST_RELEASE" | grep -o '"browser_download_url": "[^"]*BepInEx.Mono[^"]*"' | head -1 | cut -d'"' -f4)
        if [ -n "$ASSET_URL" ]; then
            RELEASE_URL="$ASSET_URL"
        fi
    fi
fi

echo "Downloading from: $RELEASE_URL"

if command -v curl &> /dev/null; then
    curl -L -o "$TEMP_ZIP" "$RELEASE_URL" 2>/dev/null || {
        echo "✗ Download failed. Please check your internet connection."
        exit 1
    }
elif command -v wget &> /dev/null; then
    wget -O "$TEMP_ZIP" "$RELEASE_URL" 2>/dev/null || {
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

echo "✓ Downloaded successfully"

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
