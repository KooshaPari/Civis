#!/usr/bin/env python3
"""
DINOForge Game Test Scenarios

Predefined test scenarios for TITAN-inspired automated game testing.
Each scenario is a sequence of steps that exercises different game features.
"""

# Smoke test: Basic game launch and screenshot
SCENARIO_SMOKE = {
    "name": "smoke",
    "description": "Basic game launch + screenshot + mod menu toggle",
    "tags": ["smoke", "sanity"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait", "params": {"seconds": 5}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "F10"}},  # toggle mod menu
        {"action": "wait", "params": {"seconds": 2}},
        {"action": "screenshot", "params": {}},
        {"action": "kill", "params": {}},
    ]
}

# Unit spawn scenario: Spawn units and verify asset swaps
SCENARIO_UNIT_SPAWN = {
    "name": "unit_spawn",
    "description": "Spawn units and verify visual asset swaps",
    "tags": ["units", "assets", "swap"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait_for_world", "params": {}},
        {"action": "wait", "params": {"seconds": 3}},
        {"action": "screenshot", "params": {}},
        {"action": "query_entities", "params": {"component": "Unit"}},
        {"action": "analyze_screen", "params": {}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Escape"}},  # pause
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Escape"}},  # resume
        {"action": "kill", "params": {}},
    ]
}

# Modern warfare pack scenario
SCENARIO_MODERN_WARFARE = {
    "name": "modern_warfare",
    "description": "Modern warfare pack units test",
    "tags": ["packs", "modern", "units"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait_for_world", "params": {}},
        {"action": "wait", "params": {"seconds": 5}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "F10"}},  # mod menu
        {"action": "wait", "params": {"seconds": 2}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Escape"}},  # close mod menu
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "query_entities", "params": {"component": "Unit"}},
        {"action": "screenshot", "params": {}},
        {"action": "kill", "params": {}},
    ]
}

# Star Wars pack scenario
SCENARIO_STARWARS = {
    "name": "starwars",
    "description": "Star Wars Clone Wars pack units test",
    "tags": ["packs", "starwars", "units"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait_for_world", "params": {}},
        {"action": "wait", "params": {"seconds": 5}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "F10"}},  # mod menu
        {"action": "wait", "params": {"seconds": 2}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Escape"}},  # close mod menu
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "query_entities", "params": {"component": "Unit"}},
        {"action": "screenshot", "params": {}},
        {"action": "kill", "params": {}},
    ]
}

# Debug overlay scenario
SCENARIO_DEBUG_OVERLAY = {
    "name": "debug_overlay",
    "description": "Toggle debug overlay and verify entity counts",
    "tags": ["debug", "overlay", "entities"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait_for_world", "params": {}},
        {"action": "wait", "params": {"seconds": 3}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "F9"}},  # debug overlay
        {"action": "wait", "params": {"seconds": 2}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "F9"}},  # toggle off
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "screenshot", "params": {}},
        {"action": "kill", "params": {}},
    ]
}

# Pause menu navigation scenario
SCENARIO_PAUSE_MENU = {
    "name": "pause_menu",
    "description": "Navigate pause menu with arrow keys",
    "tags": ["menu", "navigation", "ui"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait_for_world", "params": {}},
        {"action": "wait", "params": {"seconds": 3}},
        {"action": "input", "params": {"key": "Escape"}},  # pause
        {"action": "wait", "params": {"seconds": 2}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Up"}},  # navigate
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Down"}},  # navigate
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "screenshot", "params": {}},
        {"action": "input", "params": {"key": "Escape"}},  # resume
        {"action": "wait", "params": {"seconds": 1}},
        {"action": "kill", "params": {}},
    ]
}

# Stress test: Long gameplay with periodic screenshots
SCENARIO_STRESS = {
    "name": "stress",
    "description": "Extended gameplay stress test (15 screenshots over 30 seconds)",
    "tags": ["stress", "performance", "endurance"],
    "steps": [
        {"action": "launch", "params": {"hidden": False}},
        {"action": "wait_for_world", "params": {}},
        {"action": "wait", "params": {"seconds": 5}},
        {"action": "screenshot", "params": {}},
        # Repeat screenshot every 2 seconds for 30 seconds
    ] + [
        {"action": "wait", "params": {"seconds": 2}},
        {"action": "screenshot", "params": {}}
    ] * 12 + [
        {"action": "kill", "params": {}},
    ]
}

# All scenarios registry
ALL_SCENARIOS = {
    "smoke": SCENARIO_SMOKE,
    "unit_spawn": SCENARIO_UNIT_SPAWN,
    "modern_warfare": SCENARIO_MODERN_WARFARE,
    "starwars": SCENARIO_STARWARS,
    "debug_overlay": SCENARIO_DEBUG_OVERLAY,
    "pause_menu": SCENARIO_PAUSE_MENU,
    "stress": SCENARIO_STRESS,
}


def get_scenario(name: str) -> dict:
    """Get a scenario by name."""
    if name not in ALL_SCENARIOS:
        raise ValueError(f"Unknown scenario: {name}. Available: {list(ALL_SCENARIOS.keys())}")
    return ALL_SCENARIOS[name]


def list_scenarios() -> list:
    """List all available scenarios."""
    scenarios = []
    for name, scenario in ALL_SCENARIOS.items():
        scenarios.append({
            "name": name,
            "description": scenario.get("description", ""),
            "tags": scenario.get("tags", []),
            "steps": len(scenario.get("steps", []))
        })
    return scenarios


if __name__ == "__main__":
    # CLI: list all scenarios
    import json
    print(json.dumps(list_scenarios(), indent=2))
