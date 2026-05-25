#!/bin/bash
# Check pack list visibility - status only to verify packs loaded
set -e

GAME_CLI="dotnet run --project src/Tools/GameControlCli --"

echo "=== Pack List Visibility Check ==="
echo "Verifying that loaded packs are visible in the UI..."
echo ""

$GAME_CLI status | grep -A 20 "Loaded packs"

echo ""
echo "If packs are listed above with count > 0, check that:"
echo "  1. F10 menu opens (pack list sidebar should be visible)"
echo "  2. Pack names appear in the left sidebar"
echo "  3. Pack descriptions appear on the right"
