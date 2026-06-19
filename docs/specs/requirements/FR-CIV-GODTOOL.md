# FR-CIV-GODTOOL — God-Tool Palette & Sandbox Interaction

**Owner:** UI Lead (+ Voxel-Render for brush application). **Source gap:** Feature Matrix §6 (god-tools INCOMPLETE; spawn/inspect/hand BLIND).
**Reference bar:** WorldBox ~374 powers across 8 tabs ([WorldBox Powers wiki](https://the-official-worldbox-wiki.fandom.com/wiki/Powers)) — the gold standard for "poke the world" sandbox feel; Black & White god-hand gesture metaphor. Existing direction in `docs/specs/tool-design-directives.md` (terraform = CS+WorldBox superset; materials = WorldBox×PowderToy).
**Emergence note:** **[UI/QoL]** — tools are *inputs to the Layer-0 simulation*. A god-tool sets initial/boundary conditions (spawn matter, raise terrain, ignite, seed life, drop disaster); the *consequences* must emerge from physics/genomics, never scripted. Spawning a species seeds DNA; what it becomes emerges.

## Requirements

| ID | Requirement | Acceptance Criteria |
|---|---|---|
| FR-CIV-GODTOOL-900 | A categorized god-tool palette SHALL be provided (tabs: terrain/world-shaping, materials, life/spawn, nature/disaster, destruction, inspect, time). | Palette navigable by tab + search; tool selection shows brush preview. |
| FR-CIV-GODTOOL-901 | Tool registry SHALL be data-driven so powers add without code forks (mod-extensible per FR-CIV-MOD). | New power = registry entry + handler binding; mods can register powers. |
| FR-CIV-GODTOOL-910 | Material/terrain brushes SHALL apply to the voxel CA with adjustable radius/strength and immediate, mass-conserving effect. | Brush writes `civ-voxel` cells; CA conserves mass; effect visible <1 frame at Hot LOD. |
| FR-CIV-GODTOOL-911 | Life/spawn tools SHALL seed DNA-bearing organisms; outcome (survival, speciation, sentience) emerges, never scripted. | Spawn injects `civ-genetics` DNA; lineage then evolves under laws only. |
| FR-CIV-GODTOOL-912 | Disaster/nature tools (lightning, fire, flood, quake, etc.) SHALL set physical initial conditions; spread/damage emerges from CA. | No scripted damage values beyond physical law; propagation matches CA rules. |
| FR-CIV-GODTOOL-920 | Time controls (pause/play/speed) and a god-hand cursor (grab/move/drop, B&W-style) SHALL be provided. | Speed scales tick rate deterministically; hand can pick/drop supported entities. |
| FR-CIV-GODTOOL-921 | God-tool actions SHALL support undo and a blueprint copy/paste of a region. | Undo reverts last brush deterministically; blueprint captures + re-stamps a voxel/structure region. |

**Validation:** brush mass-conservation test; spawn→emergent-outcome integration test (no scripted result); undo determinism test.
