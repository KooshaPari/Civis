# Example UI Counter Pack

A minimal example demonstrating how to create a custom HUD element in DINOForge.

## Overview

This pack introduces a single HUD element: a **Wave Counter** that displays the current wave number in the top-right corner of the gameplay screen.

## Contents

- `pack.yaml` — Pack manifest defining metadata and HudElement loads.
- `hud_elements/wave-counter.yaml` — Single HudElement definition with position, size, and color overrides.

## How It Works

1. The `pack.yaml` manifest declares that this pack loads HUD elements from `hud_elements/wave-counter.yaml`.
2. The HudElementRegistry reads the wave-counter definition and registers it with:
   - **Position**: `top_right` anchor
   - **Dimensions**: 200×40 pixels
   - **Visible in**: Gameplay only (not pause/main menu)
   - **Colors**: White text (#FFFFFF) on dark background (#1A1A1A)
   - **Opacity**: 95% (slightly transparent)

3. At runtime, the HUD system displays the counter overlay, updating the wave number as game state changes.

## Usage

Load this pack in your game configuration:

```yaml
packs:
  - example-ui-counter
```

Or via the DINOForge CLI:
```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/example-ui-counter
```

## Extension Points

To extend this example:
- Modify `position` to `top_left`, `bottom_left`, `bottom_right`, or `center`.
- Add `visible_in: [gameplay, pause]` to show during pause menu.
- Change `type` to `health_bar`, `minimap`, `unit_portrait`, or `alert_banner`.
- Add custom properties in `color_overrides` for faction-specific theming.

See `schemas/ui-overlay.schema.json` for the complete HudElement schema.
