# hud
> HUD overlay system for the Civis 3D client.

## Overview

The `hud` crate provides the heads-up display overlay for the Civis 3D client. It renders critical in-game information such as citizen status, resource indicators, minimaps, and contextual tooltips on top of the 3D viewport.

The HUD system is designed to be modular and composable. Each overlay panel is a self-contained widget that can be shown, hidden, or repositioned at runtime. The crate integrates with the ECS rendering pipeline to draw UI elements without interfering with the scene graph.

Performance is a key concern: the HUD must render at 60 fps alongside the 3D world without introducing frame drops. To this end, the crate uses retained-mode rendering and caches GPU textures for static elements.

## Features

- Modular widget system for HUD panels
- Dynamic positioning and layout anchors
- Minimap with real-time city overview
- Citizen status and need indicators
- Resource ticker and alert system
- Theming support via CSS-like style sheets
- Accessibility hooks for screen readers

## Usage

```rust
use hud::*;

let mut hud = HudOverlay::new(&renderer);
hud.add_panel(MinimapPanel::default());
hud.add_panel(CitizenStatusBar::new(citizen_id));
hud.render(&frame, &delta);
```

## Architecture

The crate follows a retained-mode UI pattern. `HudOverlay` owns a list of `Panel` trait objects. Each panel manages its own layout, input handling, and GPU resources. The overlay composites all visible panels into a single render pass that is blended over the 3D scene.

## License

Part of the Civis project (https://github.com/KooshaPari/Civis).
