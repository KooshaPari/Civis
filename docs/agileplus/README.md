# Civis AgilePlus Waves

AgilePlus-style epic and story breakdown for the shipped Civis waves.

Scope:

- Voxel render
- Life-sim
- Infrastructure
- Perception
- Scale
- UI

Each epic below maps to the FR catalog in `docs/specs/requirements/` and keeps story IDs local to the wave so the shipped work can be traced without reopening the original implementation notes.

## Epic Index

| Epic | Wave | Status | FR IDs |
|---|---|---|---|
| [CIV-W1](./epics/civ-w1-voxel-render.md) | Voxel render | shipped | FR-CIV-GODTOOL-910, FR-CIV-GODTOOL-911, FR-CIV-GODTOOL-912 |
| [CIV-W2](./epics/civ-w2-life-sim.md) | Life-sim | shipped | FR-CIV-PSYCHE-900, FR-CIV-PSYCHE-901, FR-CIV-PSYCHE-910, FR-CIV-PSYCHE-911, FR-CIV-PSYCHE-912, FR-CIV-PSYCHE-920, FR-CIV-PSYCHE-921 |
| [CIV-W3](./epics/civ-w3-infrastructure.md) | Infrastructure | shipped | FR-CIV-ROAD-900, FR-CIV-ROAD-901, FR-CIV-ROAD-902, FR-CIV-ROAD-910, FR-CIV-ROAD-920, FR-CIV-ROAD-921 |
| [CIV-W4](./epics/civ-w4-perception.md) | Perception | shipped | FR-CIV-INFOVIEW-900, FR-CIV-INFOVIEW-901, FR-CIV-INFOVIEW-910, FR-CIV-INFOVIEW-911, FR-CIV-INFOVIEW-912, FR-CIV-INFOVIEW-913, FR-CIV-INFOVIEW-914, FR-CIV-INFOVIEW-920, FR-CIV-INSPECT-900, FR-CIV-INSPECT-901, FR-CIV-INSPECT-902, FR-CIV-INSPECT-903, FR-CIV-INSPECT-910, FR-CIV-INSPECT-920 |
| [CIV-W5](./epics/civ-w5-scale.md) | Scale | shipped | NFR-CIV-SCALE-900, NFR-CIV-SCALE-901, NFR-CIV-SCALE-902, NFR-CIV-SCALE-910, NFR-CIV-SCALE-920, NFR-CIV-PERF-900, NFR-CIV-PERF-901, NFR-CIV-PERF-902 |
| [CIV-W6](./epics/civ-w6-ui.md) | UI | shipped | FR-CIV-GODTOOL-900, FR-CIV-GODTOOL-920, FR-CIV-GODTOOL-921, FR-CIV-NOTIFY-900, FR-CIV-NOTIFY-901, FR-CIV-NOTIFY-910, FR-CIV-NOTIFY-911, FR-CIV-NOTIFY-920, FR-CIV-NOTIFY-921 |

## Notes

- Story IDs are wave-local and are intended for shipped-work retrospectives, release notes, and follow-on planning.
- The decomposition is intentionally traceable to the requirement catalog, not to code ownership boundaries.
- The scale wave includes NFRs because the shipped render and streaming envelope is part of the delivery contract, even though those items are not `FR-CIV-*`.
