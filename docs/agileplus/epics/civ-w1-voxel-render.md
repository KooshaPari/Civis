# CIV-W1 — Voxel Render

**Status:** shipped  
**Wave:** voxel render  
**Primary intent:** establish the voxel-facing render and sandbox interaction surface that makes the world legible and editable.

## FR Trace

- FR-CIV-GODTOOL-910
- FR-CIV-GODTOOL-911
- FR-CIV-GODTOOL-912
- FR-CIV-GODTOOL-920
- FR-CIV-GODTOOL-921

## Stories

| Story | Title | FR coverage |
|---|---|---|
| W1.1 | Material brush writes the voxel field | FR-CIV-GODTOOL-910 |
| W1.2 | Spawn brush seeds DNA-bearing life | FR-CIV-GODTOOL-911 |
| W1.3 | Disaster brush injects physical initial conditions | FR-CIV-GODTOOL-912 |
| W1.4 | Time controls and god-hand cursor | FR-CIV-GODTOOL-920 |
| W1.5 | Undo plus blueprint copy/paste | FR-CIV-GODTOOL-921 |

## Story Breakdown

### W1.1 Material brush writes the voxel field

- Add a radius/strength-based brush for terrain and material editing.
- Keep writes mass-conserving and immediate at the active render tier.
- Expose the selection preview so the player can predict the edit footprint.

### W1.2 Spawn brush seeds DNA-bearing life

- Seed organisms through the same world interaction surface.
- Pass DNA into the sim rather than authoring a scripted outcome.
- Let survival and divergence emerge from downstream systems.

### W1.3 Disaster brush injects physical initial conditions

- Provide lightning, fire, flood, quake, and similar world-state perturbations.
- Model the result as a physics/simulation input, not a scripted damage routine.
- Keep propagation inside the sim rules.

### W1.4 Time controls and god-hand cursor

- Support pause, play, and speed changes.
- Let the player pick up, move, and drop supported entities.
- Preserve the interaction model across the shipped render surface.

### W1.5 Undo plus blueprint copy/paste

- Undo the last world edit deterministically.
- Capture a region as a reusable blueprint.
- Paste the region back into the voxel world with the same shape and tags.
