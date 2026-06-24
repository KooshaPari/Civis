# Spore & Black & White Teardown for Civis Spec Map

## Overview
Two god-game touchstones for Civis's **paths-to-sentience** and **god-hand interaction**:
- **Spore** (Maxis, 2008) — a creature's journey across five stages (Cell → Creature → Tribe → Civilization → Space), with a beloved **creature creator** + **procedural animation**. The reference for "evolution toward sentience" framing — and a cautionary tale about scripting it.
- **Black & White** (Lionhead, 2001) — a deity rules villagers via a **hand cursor** + **gesture-cast powers**, and raises a **Creature** that learns by **reward/punishment**; the player's **alignment** (good/evil) emerges from behavior. The reference for **god-hand UX** and **emergent learning/alignment**.

## Feature & Systems Teardown
### Spore — stage progression & creator
Five discrete stages, each almost a different genre; the **Creature Creator** maps part placement → procedural rig → animation so any body plan moves plausibly. Players design morphology and the engine animates it — directly analogous to Civis DNA→phenotype→animation.

### Spore — paths to sentience (and its flaw)
Spore *frames* the cell-to-space arc as evolution, but the stages are **scripted gates**, not emergent thresholds, and depth collapses after the creator ("wide but shallow"). The fun is front-loaded into the editor.

### Black & White — the hand metaphor
The cursor is a literal **hand** that grabs, throws, strokes, slaps, places, and gesture-draws miracles. Direct, physical, no menus — the purest god-interaction UX.

### Black & White — creature learning
The Creature is an AI that **learns by feedback**: reward an action (stroke) to reinforce, punish (slap) to suppress; it generalizes desires/aversions. Genuinely emergent behavior from a learning rule, not a script.

### Black & White — emergent alignment & worship
Player **alignment** emerges from accumulated choices (no good/evil button); villagers' **belief/worship** responds to how you treat them, feeding your power.

## What it NAILS
- **Spore:** procedural morphology→animation from a part graph; the joy of *designing* an organism.
- **B&W:** the **hand** as a universal direct-manipulation metaphor; **learning-by-feedback** creature AI; **alignment/belief as emergent measures**.

## What to ADOPT for Civis
- **Sentience as a crossable threshold**, measured from accumulated cognitive/genomic traits — NOT scripted stages. `[EMERGENT]` → FR-CIV-PSYCHE (sentience threshold), charter Layer-1.
- **DNA→phenotype→procedural animation** so any emergent body plan moves. `[EMERGENT]` (morphology) + `[UI/QoL]` (rig) → `civ-species` + render.
- **God-hand cursor** (grab/move/drop/gesture) as Civis's primary direct-manipulation verb. `[UI/QoL]` → FR-CIV-GODTOOL-920.
- **Alignment/belief/worship as emergent measures** of how the player+agents act — never an authored slider. `[EMERGENT]` → FR-CIV-PSYCHE-911.
- **Learning-by-feedback** as one viable cognitive substrate on the path to sentience. `[EMERGENT]`.

## What to AVOID
- **Spore's scripted stages + post-creator shallowness** — Civis must keep progression a continuous emergent gradient with depth at every scale, not genre-swapping gates.
- **B&W's single-creature focus** — Civis needs population-scale emergence, not one pet.
- Don't hardcode "good/evil" or "civilized/primitive" enums; keep them measured.

## Bevy / Rust ecosystem notes
Procedural creature animation → skeletal/IK crates atop Bevy's animation graph; god-hand → `bevy_picking` + drag. See [bevy-ecosystem-reference](./bevy-ecosystem-reference.md).

## Sources
- Spore (Wikipedia) — https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- Spore Creature Creator / procedural animation (GDC/Maxis writeups) — https://en.wikipedia.org/wiki/Spore_Creature_Creator
- Black & White (Wikipedia) — https://en.wikipedia.org/wiki/Black_%26_White_(video_game)
- B&W Creature AI / learning — https://en.wikipedia.org/wiki/Black_%26_White_(video_game)#Creature
- Lionhead on B&W design (postmortem coverage) — https://www.gamedeveloper.com/design/the-making-of-black-white-
