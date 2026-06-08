#!/usr/bin/env bash
set -euo pipefail
# Install DumpTools CLI tool for DINOForge

set -e

REPO="KooshaPari/Dino"
TOOL_NAME="DumpTools"
VERSION="${1:-latest}"

echo "Installing $TOOL_NAME..."

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) RID="linux-x64" ;;
      aarch64) RID="linux-arm64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    INSTALL_PATH="/usr/local/bin"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) RID="osx-x64" ;;
      arm64) RID="osx-arm64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    INSTALL_PATH="/usr/local/bin"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

echo "Platform: $OS $ARCH | RID: $RID"

# Get release info
API_URL="https://api.github.com/repos/$REPO/releases"
if [ "$VERSION" != "latest" ]; then
  API_URL="$API_URL/tags/$VERSION"
else
  API_URL="$API_URL/latest"
fi

echo "Fetching release info from GitHub..."
RELEASE_JSON=$(curl -s "$API_URL")
TAG=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | cut -d'"' -f4)
DOWNLOAD_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url.*DumpTools-$RID" | head -1 | cut -d'"' -f4)

if [ -z "$DOWNLOAD_URL" ]; then
  echo "Error: No binary found for platform: $RID"
  exit 1
fi

echo "Tag: $TAG"
echo "Download URL: $DOWNLOAD_URL"

# Download and extract
TEMP_DIR="/tmp/dinoforge-DumpTools-$$"
mkdir -p "$TEMP_DIR"
cd "$TEMP_DIR"

echo "Downloading..."
curl -sL "$DOWNLOAD_URL" -o DumpTools.zip

echo "Extracting..."
unzip -q DumpTools.zip

# Install
EXE_PATH=$(ls -1 DumpTools* 2>/dev/null | head -1)
if [ -z "$EXE_PATH" ]; then
  EXE_PATH=$(find . -maxdepth 1 -executable -type f | head -1)
fi

sudo cp "$EXE_PATH" "$INSTALL_PATH/DumpTools"
sudo chmod +x "$INSTALL_PATH/DumpTools"

echo "Installed to: $INSTALL_PATH/DumpTools"

# Verify
echo "Verifying installation..."
"$INSTALL_PATH/DumpTools" --version

echo "✓ $TOOL_NAME installed successfully"

# Cleanup
cd /
rm -rf "$TEMP_DIR"
