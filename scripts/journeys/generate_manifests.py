#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Generate journey manifests from journey metadata"""

import json
import yaml
from pathlib import Path
from datetime import datetime
import sys
import io

# Set stdout encoding to utf-8 for unicode characters
if sys.stdout.encoding != 'utf-8':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

JOURNEYS = [
    {
        "id": "us-f1-1-game-launch",
        "intent": "User launches game and verifies DINOForge mod is loaded",
        "steps": [
            {"slug": "launch-game", "intent": "Launch DINOForge + game"},
            {"slug": "wait-world", "intent": "Wait for ECS world initialization"},
            {"slug": "verify-overlay", "intent": "Verify debug overlay available"},
        ]
    },
    {
        "id": "us-f2-1-unit-spawn",
        "intent": "User spawns unit and verifies asset swap works",
        "steps": [
            {"slug": "launch", "intent": "Launch game"},
            {"slug": "spawn-unit", "intent": "Spawn Clone Trooper unit"},
            {"slug": "verify-swap", "intent": "Verify asset swap applied"},
            {"slug": "verify-health", "intent": "Verify unit health loaded"},
        ]
    },
    {
        "id": "us-f3-1-debug-overlay",
        "intent": "User toggles debug overlay with F9/F10",
        "steps": [
            {"slug": "launch", "intent": "Launch game"},
            {"slug": "press-f9", "intent": "Press F9 to show overlay"},
            {"slug": "verify-overlay", "intent": "Verify overlay displayed"},
            {"slug": "press-f9-again", "intent": "Press F9 to hide overlay"},
        ]
    },
    {
        "id": "us-f4-1-menu-nav",
        "intent": "User navigates menu with arrow keys",
        "steps": [
            {"slug": "launch", "intent": "Launch game"},
            {"slug": "open-menu", "intent": "Open mod menu"},
            {"slug": "nav-down", "intent": "Navigate down"},
            {"slug": "nav-up", "intent": "Navigate up"},
            {"slug": "close-menu", "intent": "Close menu"},
        ]
    },
]

def generate_manifest(journey):
    """Generate manifest structure for a journey"""
    return {
        "id": journey["id"],
        "intent": journey["intent"],
        "recording": f"journeys/{journey['id']}/{journey['id']}.gif",
        "keyframe_count": len(journey["steps"]),
        "passed": True,  # Will be updated by verification
        "steps": [
            {
                "index": i,
                "slug": step["slug"],
                "intent": step["intent"],
                "screenshot_path": f"journeys/manifests/{journey['id']}/frame-{i:03d}.png",
                "description": step.get("description", ""),
                "assertions": {
                    "must_contain": step.get("must_contain", []),
                    "must_not_contain": [],
                    "ocr_required": False
                },
                "annotations": []
            }
            for i, step in enumerate(journey["steps"])
        ],
        "verification": {
            "mode": "pending",
            "timestamp": datetime.now().isoformat()
        }
    }

if __name__ == "__main__":
    manifest_dir = Path("docs/journeys/manifests")

    for journey in JOURNEYS:
        # Create directory
        journey_dir = manifest_dir / journey["id"]
        journey_dir.mkdir(parents=True, exist_ok=True)

        # Generate manifest
        manifest = generate_manifest(journey)
        manifest_file = journey_dir / "manifest.json"

        with open(manifest_file, "w") as f:
            json.dump(manifest, f, indent=2)

        print(f"✓ Created {manifest_file}")
