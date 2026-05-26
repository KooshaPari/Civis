#!/usr/bin/env bash
#
# DINOForge Installer for Linux / Steam Deck
# Installs BepInEx 5.4.23.2 and DINOForge into Diplomacy is Not an Option.
#
# Usage:
#   ./install.sh                       # Auto-detect game path
#   ./install.sh /path/to/game         # Manual game path
#   ./install.sh --dev                 # Include dev tools
#   ./install.sh /path/to/game --dev   # Both
#

set -euo pipefail

BEPINEX_VERSION="5.4.23.2"
BEPINEX_URL="https://github.com/BepInEx/BepInEx/releases/download/v${BEPINEX_VERSION}/BepInEx_unix_5.4.23.2.zip"
DINO_APPID=1272320
DINO_DIR_NAME="Diplomacy is Not an Option"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

step()    { echo -e "${CYAN}[*]${NC} $1"; }
success() { echo -e "${GREEN}[+]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
err()     { echo -e "${RED}[-]${NC} $1"; }

GAME_PATH=""
DEV_MODE=false

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --dev) DEV_MODE=true ;;
        *)     GAME_PATH="$arg" ;;
    esac
done

find_steam_path() {
    local paths=(
        "$HOME/.steam/steam"
        "$HOME/.local/share/Steam"
        "$HOME/.steam/debian-installation"
        "$HOME/snap/steam/common/.steam/steam"
    )
    for p in "${paths[@]}"; do
        if [ -d "$p" ]; then
            echo "$p"
            return
        fi
    done
}

get_library_folders() {
    local steam_path="$1"
    local folders=("$steam_path")

    local vdf="$steam_path/steamapps/libraryfolders.vdf"
    if [ ! -f "$vdf" ]; then
        vdf="$steam_path/config/libraryfolders.vdf"
    fi

    if [ -f "$vdf" ]; then
        while IFS= read -r line; do
            local path
            path=$(echo "$line" | grep -oP '"path"\s+"\K[^"]+' || true)
            if [ -n "$path" ]; then
                path="${path//\\\\/\\}"
                folders+=("$path")
            fi
        done < "$vdf"
    fi

    printf '%s\n' "${folders[@]}" | sort -u
}

find_dino_path() {
    local steam_path
    steam_path=$(find_steam_path)
    if [ -z "$steam_path" ]; then
        return
    fi

    step "Steam found at: $steam_path"

    while IFS= read -r lib; do
        local common_path="$lib/steamapps/common/$DINO_DIR_NAME"
        if [ -d "$common_path" ]; then
            echo "$common_path"
            return
        fi

        local acf="$lib/steamapps/appmanifest_${DINO_APPID}.acf"
        if [ -f "$acf" ]; then
            local installdir
            installdir=$(grep -oP '"installdir"\s+"\K[^"]+' "$acf" || true)
            if [ -n "$installdir" ]; then
                local game_path="$lib/steamapps/common/$installdir"
                if [ -d "$game_path" ]; then
                    echo "$game_path"
                    return
                fi
            fi
        fi
    done < <(get_library_folders "$steam_path")
}

echo ""
echo -e "${MAGENTA}========================================${NC}"
echo -e "${MAGENTA}  DINOForge Installer (Linux)${NC}"
echo -e "${MAGENTA}========================================${NC}"
echo ""

# 1. Find game path
if [ -n "$GAME_PATH" ]; then
    if [ ! -d "$GAME_PATH" ]; then
        err "Specified path does not exist: $GAME_PATH"
        exit 1
    fi
    step "Using specified game path: $GAME_PATH"
else
    step "Auto-detecting DINO installation..."
    GAME_PATH=$(find_dino_path || true)
    if [ -z "$GAME_PATH" ]; then
        warn "Could not auto-detect DINO installation."
        read -rp "Enter the full path to the DINO game directory: " GAME_PATH
        if [ ! -d "$GAME_PATH" ]; then
            err "Path does not exist: $GAME_PATH"
            exit 1
        fi
    else
        success "Found DINO at: $GAME_PATH"
    fi
fi

# 2. Install BepInEx
if [ -f "$GAME_PATH/winhttp.dll" ] || [ -f "$GAME_PATH/run_bepinex.sh" ]; then
    warn "BepInEx appears to be already installed. Skipping download."
else
    step "Downloading BepInEx $BEPINEX_VERSION..."
    TEMP_ZIP="/tmp/BepInEx_${BEPINEX_VERSION}.zip"

    if command -v curl &>/dev/null; then
        curl -L -o "$TEMP_ZIP" "$BEPINEX_URL"
    elif command -v wget &>/dev/null; then
        wget -O "$TEMP_ZIP" "$BEPINEX_URL"
    else
        err "Neither curl nor wget found. Install one and retry."
        exit 1
    fi
    success "Downloaded BepInEx."

    step "Extracting BepInEx to game directory..."
    unzip -o "$TEMP_ZIP" -d "$GAME_PATH"
    rm -f "$TEMP_ZIP"
    success "BepInEx extracted."

    # Make unix loader executable
    if [ -f "$GAME_PATH/run_bepinex.sh" ]; then
        chmod +x "$GAME_PATH/run_bepinex.sh"
        success "Made run_bepinex.sh executable."
    fi
fi

# 3. Create plugins directory
mkdir -p "$GAME_PATH/BepInEx/plugins"

# 4. Copy Runtime DLL if available
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_DLL="$SCRIPT_DIR/../../../Runtime/bin/Release/netstandard2.0/DINOForge.Runtime.dll"
if [ ! -f "$RUNTIME_DLL" ]; then
    RUNTIME_DLL="$SCRIPT_DIR/../../../Runtime/bin/Debug/netstandard2.0/DINOForge.Runtime.dll"
fi
if [ -f "$RUNTIME_DLL" ]; then
    cp "$RUNTIME_DLL" "$GAME_PATH/BepInEx/plugins/"
    success "Copied DINOForge.Runtime.dll to BepInEx/plugins/"
else
    warn "DINOForge.Runtime.dll not found. Build the Runtime project first."
fi

# 5. Create packs directory
mkdir -p "$GAME_PATH/packs"
EXAMPLE_DIR="$GAME_PATH/packs/example-balance"
if [ ! -d "$EXAMPLE_DIR" ]; then
    mkdir -p "$EXAMPLE_DIR"
    cat > "$EXAMPLE_DIR/pack.yaml" << 'PACKEOF'
id: example-balance
name: Example Balance Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: DINOForge Agents
type: balance
description: An example balance pack demonstrating DINOForge pack structure.
depends_on: []
conflicts_with: []
loads:
  units: []
  buildings: []
PACKEOF
    success "Created example pack at packs/example-balance/"
fi

# 6. Dev mode
if [ "$DEV_MODE" = true ]; then
    step "Installing dev tools..."
    SDK_DLL="$SCRIPT_DIR/../../../SDK/bin/Release/netstandard2.0/DINOForge.SDK.dll"
    if [ ! -f "$SDK_DLL" ]; then
        SDK_DLL="$SCRIPT_DIR/../../../SDK/bin/Debug/netstandard2.0/DINOForge.SDK.dll"
    fi
    if [ -f "$SDK_DLL" ]; then
        cp "$SDK_DLL" "$GAME_PATH/BepInEx/plugins/"
        success "Copied DINOForge.SDK.dll"
    else
        warn "SDK DLL not found."
    fi

    SCHEMAS_SRC="$SCRIPT_DIR/../../../../schemas"
    if [ -d "$SCHEMAS_SRC" ]; then
        mkdir -p "$GAME_PATH/DINOForge/schemas"
        cp -r "$SCHEMAS_SRC"/* "$GAME_PATH/DINOForge/schemas/"
        success "Copied schemas."
    fi
fi

# 7. Verify
echo ""
step "Verifying installation..."
ALL_GOOD=true

check_path() {
    if [ -e "$1" ]; then
        success "$2 ... OK"
    else
        err "$2 ... MISSING"
        ALL_GOOD=false
    fi
}

check_path "$GAME_PATH/BepInEx" "BepInEx directory"
check_path "$GAME_PATH/BepInEx/plugins" "BepInEx plugins directory"
check_path "$GAME_PATH/packs" "Packs directory"

if [ -f "$GAME_PATH/BepInEx/plugins/DINOForge.Runtime.dll" ]; then
    success "DINOForge Runtime DLL ... OK"
else
    warn "DINOForge Runtime DLL ... NOT FOUND (build and copy manually)"
fi

echo ""
if [ "$ALL_GOOD" = true ]; then
    success "DINOForge installation complete!"
    echo "  Game path: $GAME_PATH"
    echo "  Run the game to generate BepInEx config."
else
    warn "Installation has issues. Review the messages above."
fi
echo ""
