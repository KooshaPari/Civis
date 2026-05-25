#!/bin/bash
# Deploy DINOForge Runtime + packs to game directory
# Usage: deploy.sh [--runtime-only | --packs-only]

set -e

GAME_DIR=$(grep '<GameDir>' Directory.Build.props | sed 's/.*<GameDir>//' | sed 's/<.*//')
PLUGINS_DIR="$GAME_DIR/BepInEx/plugins"
PACKS_DIR="$GAME_DIR/packs"

echo "Game directory: $GAME_DIR"
echo ""

if [ "$1" != "--packs-only" ]; then
    echo "=== Building Runtime (Release) ==="
    dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release
    echo "Runtime deployed to: $PLUGINS_DIR/DINOForge.Runtime.dll"
    echo ""
fi

if [ "$1" != "--runtime-only" ]; then
    echo "=== Syncing packs ==="
    mkdir -p "$PACKS_DIR"
    cp -r packs/* "$PACKS_DIR/" 2>/dev/null || echo "No packs to sync"
    echo "Packs synced to: $PACKS_DIR"
    echo ""
fi

echo "=== Verifying deployment ==="
if [ -f "$PLUGINS_DIR/DINOForge.Runtime.dll" ]; then
    echo "  Runtime DLL: OK ($(stat -c%s "$PLUGINS_DIR/DINOForge.Runtime.dll" 2>/dev/null || stat -f%z "$PLUGINS_DIR/DINOForge.Runtime.dll" 2>/dev/null) bytes)"
else
    echo "  Runtime DLL: MISSING"
fi

if [ -f "$PLUGINS_DIR/DINOForge.SDK.dll" ]; then
    echo "  SDK DLL: OK"
else
    echo "  SDK DLL: MISSING"
fi

if [ -d "$PACKS_DIR" ]; then
    PACK_COUNT=$(find "$PACKS_DIR" -name "pack.yaml" 2>/dev/null | wc -l)
    echo "  Packs: $PACK_COUNT pack(s) installed"
else
    echo "  Packs: directory missing"
fi

echo ""
echo "Deploy complete. Restart DINO to load changes."
echo "  F9  = Debug Overlay (ECS world state)"
echo "  F10 = Mod Menu (pack management)"
