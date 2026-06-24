# CIV-W6 — UI

**Status:** shipped  
**Wave:** ui  
**Primary intent:** expose the control surface, notifications, and onboarding that make the sim usable.

## FR Trace

- FR-CIV-GODTOOL-900
- FR-CIV-GODTOOL-920
- FR-CIV-GODTOOL-921
- FR-CIV-NOTIFY-900
- FR-CIV-NOTIFY-901
- FR-CIV-NOTIFY-910
- FR-CIV-NOTIFY-911
- FR-CIV-NOTIFY-920
- FR-CIV-NOTIFY-921

## Stories

| Story | Title | FR coverage |
|---|---|---|
| W6.1 | Data-driven god-tool palette | FR-CIV-GODTOOL-900 |
| W6.2 | Time controls and god-hand interactions | FR-CIV-GODTOOL-920 |
| W6.3 | Undo and blueprint reuse in the UI | FR-CIV-GODTOOL-921 |
| W6.4 | Event and alert feed | FR-CIV-NOTIFY-900, FR-CIV-NOTIFY-901 |
| W6.5 | Statistics dashboards | FR-CIV-NOTIFY-910, FR-CIV-NOTIFY-911 |
| W6.6 | Progressive onboarding and tool discovery | FR-CIV-NOTIFY-920 |
| W6.7 | Rebindable hotkey map | FR-CIV-NOTIFY-921 |

## Story Breakdown

### W6.1 Data-driven god-tool palette

- Organize tools into navigable categories.
- Keep tool registration extensible without code forks.

### W6.2 Time controls and god-hand interactions

- Support pause, play, and speed control as first-class UI actions.
- Present the hand cursor workflow consistently.

### W6.3 Undo and blueprint reuse in the UI

- Make undo visible and deterministic to the player.
- Allow copy/paste reuse of world regions.

### W6.4 Event and alert feed

- Surface significant emergent events with severity and camera jump.
- Keep thresholds data-driven.

### W6.5 Statistics dashboards

- Present time-series dashboards for population, economy, ideology, resources, and conflict.
- Make the aggregate view readable at empire scale.

### W6.6 Progressive onboarding and tool discovery

- Teach the first-run flow with progressive disclosure.
- Keep key actions discoverable through contextual help.

### W6.7 Rebindable hotkey map

- Provide a complete hotkey map for camera, tools, overlays, speed, and selection.
- Support rebinds and conflict detection.
