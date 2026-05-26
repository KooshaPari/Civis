#!/bin/bash
# Library: Common game checking functions for personal use
# Source this file in scripts: source .claude/commands/lib/game-check.sh

GAME_CLI="dotnet run --project src/Tools/GameControlCli --"

# Get current game status
game_status() {
    $GAME_CLI status
}

# Wait for world ready (useful in build/test pipelines)
game_wait_world() {
    local timeout=${1:-30000}
    $GAME_CLI wait-world
}

# Get resources as structured data (for comparing before/after changes)
game_resources() {
    $GAME_CLI resources
}

# Get entity catalog for a specific category
game_catalog() {
    local category=${1:-""}
    if [ -z "$category" ]; then
        $GAME_CLI catalog
    else
        $GAME_CLI catalog "$category"
    fi
}

# Check if game is running and mod platform is ready
game_is_ready() {
    $GAME_CLI status 2>/dev/null | grep -q "Mod platform ready: True" && return 0 || return 1
}

# Take a screenshot and save to output
game_screenshot() {
    local output=${1:-"screenshot_$(date +%s).png"}
    $GAME_CLI screenshot "$output"
    echo "$output"
}

# Quick health check - all systems
game_health_check() {
    echo "=== Quick Game Health Check ==="

    if ! game_is_ready; then
        echo "✗ Game not ready"
        return 1
    fi

    echo "✓ Game connection OK"
    echo "✓ Mod platform loaded"

    local entity_count=$(game_status | grep "Entity count" | grep -oE "[0-9]+")
    if [ "$entity_count" -gt 0 ]; then
        echo "✓ ECS world has $entity_count entities"
    else
        echo "⚠ No entities loaded"
    fi

    echo "✓ Health check complete"
}

export -f game_status game_wait_world game_resources game_catalog game_is_ready game_screenshot game_health_check
