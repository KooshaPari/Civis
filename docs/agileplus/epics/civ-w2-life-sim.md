# CIV-W2 — Life-Sim

**Status:** shipped  
**Wave:** life-sim  
**Primary intent:** make agent life legible, emergent, and inspectable from need state through history.

## FR Trace

- FR-CIV-PSYCHE-900
- FR-CIV-PSYCHE-901
- FR-CIV-PSYCHE-910
- FR-CIV-PSYCHE-911
- FR-CIV-PSYCHE-912
- FR-CIV-PSYCHE-920
- FR-CIV-PSYCHE-921

## Stories

| Story | Title | FR coverage |
|---|---|---|
| W2.1 | Agent psyche state and bounded memory | FR-CIV-PSYCHE-900 |
| W2.2 | Need-driven mood recomputation | FR-CIV-PSYCHE-901 |
| W2.3 | Kinship and contact social graph | FR-CIV-PSYCHE-910 |
| W2.4 | Belief and norm diffusion | FR-CIV-PSYCHE-911 |
| W2.5 | Language drift and dialect regions | FR-CIV-PSYCHE-912 |
| W2.6 | Chronicle of significant life events | FR-CIV-PSYCHE-920 |
| W2.7 | LOD-tiered psyche and history simulation | FR-CIV-PSYCHE-921 |

## Story Breakdown

### W2.1 Agent psyche state and bounded memory

- Carry drives, temperament, mood, and memory as emergent state.
- Ensure temperament derives from genetics and history, not a fixed enum table.

### W2.2 Need-driven mood recomputation

- Recompute mood from current need satisfaction and recent conditions.
- Keep the contributing factors inspectable and reproducible.

### W2.3 Kinship and contact social graph

- Form edges from co-location, reproduction, and repeated interaction.
- Let bond/grudge weights evolve rather than hardcoding relationship tiers.

### W2.4 Belief and norm diffusion

- Let culture and ideology drift through the social graph.
- Surface clusters as measured communities, not authored faction labels.

### W2.5 Language drift and dialect regions

- Allow contact networks to produce dialects and creoles.
- Make language variation visible as a spatial/social field.

### W2.6 Chronicle of significant life events

- Record births, deaths, migrations, conflicts, foundings, and first contacts.
- Keep the chronicle queryable for legends-style inspection.

### W2.7 LOD-tiered psyche and history simulation

- Run full detail near active areas and aggregated models elsewhere.
- Preserve deterministic promotion and demotion between tiers.
