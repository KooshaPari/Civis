# Game E2E (Visual Regression)

This directory contains visual E2E tests for the Civis game client.

## What This Tests

This is a **GAME** test, not a simulation test. It verifies:

1. **Main Menu** — renders, buttons clickable, title screen visible
2. **World Setup** — scenario picker loads, presets available
3. **WorldGen** — terrain generation completes, transitions to Playing
4. **Playing** — game renders, HUD visible, simulation runs
5. **Outcome** — game-over/victory overlay appears when triggered

## Screenshots

- `baselines/` — reference screenshots for visual regression
- `.game-e2e/captures/` — current run captures (gitignored)

## Running Locally

```bash
# With GPU
bash scripts/game-e2e.sh

# Headless (CI mode)
bash scripts/game-e2e.sh --headless

# Update baselines
bash scripts/game-e2e.sh --headless --update-baselines
```

## CI

Runs on push/PR when `clients/bevy-ref/`, `crates/engine/`, or `crates/server/` change.

Three jobs:
1. `game-flow` — builds, launches, captures screenshots via Xvfb
2. `visual-regression` — compares captures against baselines
3. `update-baselines` — manual trigger to refresh baselines

## Requirements

- Linux: `xvfb`, `xdotool`, `imagemagick`, `libasound2-dev`, `libudev-dev`
- macOS: Metal backend (no Xvfb needed)
- Windows: DX12 backend (no Xvfb needed)
