#!/usr/bin/env bash
# DINOForge Desktop Companion installer (WSL / bash)
# Usage:  curl -fsSL https://raw.githubusercontent.com/KooshaPari/Dino/main/scripts/install-companion.sh | bash
#
# The companion is a native Windows (WinUI 3) application.
# This script installs it from WSL by writing to the Windows filesystem.
# It requires: curl, unzip, and access to /mnt/c (WSL).

set -euo pipefail

REPO="KooshaPari/Dino"
INSTALL_WIN_DIR="${LOCALAPPDATA:-/mnt/c/Users/$USER/AppData/Local}/DINOForge/Companion"

# ── Colour helpers ─────────────────────────────────────────────────────────────
bold="\033[1m"; cyan="\033[36m"; green="\033[32m"; yellow="\033[33m"; red="\033[31m"; reset="\033[0m"
step()  { echo -e "  ${cyan}==>${reset} $*"; }
ok()    { echo -e "  ${green}[OK]${reset} $*"; }
warn()  { echo -e "  ${yellow}[--]${reset} $*"; }
die()   { echo -e "  ${red}[!!]${reset} $*" >&2; exit 1; }

echo -e "\n${bold}DINOForge Desktop Companion Installer${reset}"
echo    "======================================"

# ── Platform check ─────────────────────────────────────────────────────────────
if [[ "$(uname -r)" != *microsoft* && "$(uname -r)" != *WSL* ]]; then
    die "This script is intended for WSL (Windows Subsystem for Linux).\n       The companion is a Windows-only WinUI 3 app.\n       On native Windows, run the PowerShell one-liner instead:\n\n         irm https://raw.githubusercontent.com/KooshaPari/Dino/main/scripts/install-companion.ps1 | iex"
fi

# ── Resolve version ────────────────────────────────────────────────────────────
VERSION="${DF_VERSION:-}"
if [[ -z "$VERSION" ]]; then
    step "Fetching latest release..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
    [[ -n "$VERSION" ]] || die "Could not determine latest version. Set DF_VERSION manually."
fi
VERSION_NUM="${VERSION#v}"
ok "Target version: $VERSION"

# ── Download ───────────────────────────────────────────────────────────────────
ZIP_NAME="DINOForge.Companion-v${VERSION_NUM}-win-x64.zip"
ZIP_URL="https://github.com/$REPO/releases/download/$VERSION/$ZIP_NAME"
TMP_ZIP="/tmp/$ZIP_NAME"

step "Downloading $ZIP_NAME..."
curl -fsSL --progress-bar "$ZIP_URL" -o "$TMP_ZIP" || die "Download failed: $ZIP_URL"
ok "Downloaded $(du -sh "$TMP_ZIP" | cut -f1)"

# ── Verify SHA256 ─────────────────────────────────────────────────────────────
step "Verifying SHA256..."
EXPECTED=$(curl -fsSL "${ZIP_URL}.sha256" 2>/dev/null | awk '{print toupper($1)}') || true
if [[ -n "$EXPECTED" ]]; then
    ACTUAL=$(sha256sum "$TMP_ZIP" | awk '{print toupper($1)}')
    [[ "$EXPECTED" == "$ACTUAL" ]] || die "SHA256 mismatch!\n  Expected: $EXPECTED\n  Actual:   $ACTUAL"
    ok "SHA256 verified"
else
    warn "Could not fetch checksum — skipping verification"
fi

# ── Extract ────────────────────────────────────────────────────────────────────
step "Installing to $INSTALL_WIN_DIR..."
mkdir -p "$INSTALL_WIN_DIR"
unzip -qo "$TMP_ZIP" -d "$INSTALL_WIN_DIR"
rm -f "$TMP_ZIP"
ok "Extracted to $INSTALL_WIN_DIR"

# ── Done ───────────────────────────────────────────────────────────────────────
EXE_WIN_PATH=$(wslpath -w "$INSTALL_WIN_DIR/DINOForge.DesktopCompanion.exe" 2>/dev/null || echo "$INSTALL_WIN_DIR\\DINOForge.DesktopCompanion.exe")
echo -e "\n  ${green}DINOForge Companion $VERSION installed!${reset}"
echo    "  Location : $INSTALL_WIN_DIR"
echo -e "  Launch   : ${bold}${EXE_WIN_PATH}${reset} (run from Windows Explorer or cmd)"
echo -e "\n  ${yellow}On first run: Settings → set Packs Directory to your BepInEx\\dinoforge_packs path${reset}\n"
