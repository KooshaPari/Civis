#!/bin/bash
# Check complete game state - status + resources + entities
set -e

GAME_CLI="dotnet run --project src/Tools/GameControlCli --"

echo "=== DINOForge Game State Check ==="
echo ""

echo "[1/4] Checking game connection and status..."
$GAME_CLI status
echo ""

echo "[2/4] Checking ECS world readiness..."
$GAME_CLI wait-world
echo ""

echo "[3/4] Checking resource values..."
$GAME_CLI resources
echo ""

echo "[4/4] Checking entity catalog..."
$GAME_CLI catalog
echo ""

echo "✓ Game state check complete"
