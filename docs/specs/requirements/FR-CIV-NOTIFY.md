# FR-CIV-NOTIFY — Notifications, Stats Dashboards & Onboarding

**Owner:** UI Lead. **Source gap:** Feature Matrix §7 (notifications, stats panels, onboarding, hotkeys = **BLIND**/INCOMPLETE). The "missing bells & whistles."
**Reference bar:** CS2 alerts + statistics panels; RimWorld letters/alerts + clarity; Songs of Syx graphs at empire scale; Old World event log.
**Emergence note:** **[UI/QoL]** — all of this *surfaces* emergent state; it must not author events. Alerts fire on measured conditions; the chronicle (FR-CIV-PSYCHE-920) is the event source.

## Requirements

| ID | Requirement | Acceptance Criteria |
|---|---|---|
| FR-CIV-NOTIFY-900 | An event/alert feed SHALL surface significant emergent events (disasters, conflicts, foundings, milestones, crises) with severity + camera-jump. | Alerts derive from chronicle/measured thresholds; click → camera jumps to location; dismissible; no authored scripted events. |
| FR-CIV-NOTIFY-901 | Alert thresholds SHALL be data-driven and configurable (e.g., happiness below X, population collapse). | Thresholds in config; firing reproducible under fixed seed. |
| FR-CIV-NOTIFY-910 | Statistics dashboards SHALL present time-series for population, economy, ideology/culture, resources, conflict (via `egui_plot`). | Charts read CIV-0103 time-series; update live; pannable/zoomable; export not required v1. |
| FR-CIV-NOTIFY-911 | Stats SHALL be readable at empire scale (aggregate + drill-down), per Songs of Syx. | Aggregate view + per-region drill-down; renders 100k+-agent aggregates without stalling. |
| FR-CIV-NOTIFY-920 | Onboarding SHALL use progressive disclosure: contextual tooltips, a first-run flow, and discoverable god-tools. | First-run highlights core verbs; tooltips on all tools; skippable. |
| FR-CIV-NOTIFY-921 | A full, rebindable hotkey map SHALL cover camera, tools, overlays, speed, and selection. | Hotkeys documented + rebindable; conflict detection; defaults sane for RTS players. |

**Validation:** alert-fires-on-threshold test (seed-stable); stats render smoke at scale; hotkey rebind round-trip test.
