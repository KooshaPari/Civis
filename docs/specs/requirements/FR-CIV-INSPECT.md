# FR-CIV-INSPECT — Inspect-Anything & Tooltips

**Owner:** UI Lead. **Source gap:** Feature Matrix §6/§7 (inspect-anything + tooltips = **BLIND**).
**Reference bar:** WorldBox click-any-entity inspector; Dwarf Fortress unit screen; RimWorld's clear inspect panel (clarity gold standard).
**Emergence note:** **[UI/QoL]** — surfaces measured/emergent per-entity state. Must not invent fields; renders what the sim already computes.

## Requirements

| ID | Requirement | Acceptance Criteria |
|---|---|---|
| FR-CIV-INSPECT-900 | Clicking any world element (voxel, agent, settlement cluster, structure, vehicle) SHALL open a context inspector for that entity. | Pick resolves under cursor; inspector opens <100ms; correct entity identified via `bevy_picking`. |
| FR-CIV-INSPECT-901 | Agent inspector SHALL show identity, species/phenotype summary, age, needs, psyche/mood, current activity, relationships, lineage. | Each field reads the owning crate (`civ-species`, `civ-agents`, psyche FR); BLIND fields shown as "unmeasured" not faked. |
| FR-CIV-INSPECT-902 | Settlement/polity inspector SHALL show emergent membership (cluster overlap, NOT a faction id), population, economy summary, culture/ideology, dominant language. | Membership rendered as overlap set; no `faction:u32` displayed. |
| FR-CIV-INSPECT-903 | Voxel/material inspector SHALL show material, temperature, pressure, mass, phase. | Reads `civ-voxel`/`civ-laws`; values match CA state. |
| FR-CIV-INSPECT-910 | Hover tooltips SHALL appear for any interactive element with a concise summary; full panel on click. | Tooltip <150ms hover delay; non-blocking; keyboard-dismissible. |
| FR-CIV-INSPECT-920 | Inspector SHALL support follow-cam (lock camera to selected agent) and a "trace lineage/history" jump. | Follow toggle smooth; history jump opens chronicle (see FR-CIV-NOTIFY history). |

**Validation:** picking integration test; inspector field-coverage test asserting each field binds to a real source or the explicit "unmeasured" sentinel.
