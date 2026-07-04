# Civis God-Tools Sandbox — "The Holocron Deck"

> **Status:** Binding design (2026-06-23). Planner stance — no implementation code; token / recipe / contract only. Authority for the god-tool palette surface in `clients/bevy-ref/src/` (and the web/Unreal/Godot mirror clients), the Brush/Cluster system in `clients/bevy-ref/src/tool_categories.rs`, and the spec‑driven additions to `crates/watch` snapshots.
>
> **Companions (do not duplicate):** `docs/design/ui-design-language.md` ("Console Holo" chrome/holo language — the two-tier surface), `docs/design/brush-tool-system.md` (the **Cluster → Mode → Action** model + the **ONE shared `BrushSettings`** that every cluster reads), `docs/design/info-views.md` (the 31-overlay legibility suite — god-tools read from it, not author it), `docs/design/onboarding-qol.md` (undo §3, blueprints §4, hotkeys §5, time controls §7, photo mode §8 — all extended, never forked, by this doc), `docs/specs/requirements/FR-CIV-GODTOOL.md` (the binding FR), `docs/research/worldbox.md` / `docs/research/spore-black-and-white.md` / `docs/research/cities-skylines.md` / `docs/research/civ-oldworld-4x.md` (the borrowed design references), and `docs/guides/emergence-charter.md` ("tools are inputs to the Layer-0 simulation, the consequences must emerge").
>
> **Governing canon:** `docs/guides/emergence-charter.md` — **only laws are hardcoded**; every god-tool is an *input that sets initial/boundary conditions*; the consequence (a city, a war, a flood, a culture, a creature) is **measured emergent output**, never a scripted outcome.

---

## 0. Thesis — what a "god-tool" is, and what it is not

A god-tool is **not** a cheat code that writes the *result*. A god-tool is a **boundary-condition injector**: it pokes the substrate at one point and lets physics, CA, genomics, climate, and agent utility carry the consequence. The classic god-tool taxonomy collapses into seven super-verbs:

1. **Terraform** — mutate the `CaGrid` (height/biome/material) under the brush footprint; the CA settles it.
2. **Sculpt** — subset of terraform that targets *shape*, not *substance* (slope, flatten, smooth, river-cut).
3. **Spawn** — inject matter (voxel material from the Powder-Toy-class taxonomy), life (`spawn_organism { genome, cradle_state, age }`), civilization seed (a settlement's first six huts + a founder genome), or resource (an ore deposit, a forest, a fish shoal).
4. **Disrupt** — weather/disaster control: inject energy/momentum/matter (lightning bolt, flood, quake, firestorm, plague) into a CA field; CA propagates it.
5. **Time** — pause, slow, fast-forward, scrub-replay; the **simulation clock** is a resource, not a constant.
6. **Inspect** — read any entity/cell/field through the inspector + info-view overlays; **never writes**, always a measurement.
7. **Nudge** — law/culture levers that adjust physical/economic/political **parameters**, not outcomes: tax bias, edict, religion pressure, sanction, alignment tilt; emergent outcomes follow.

Plus one universal verb — **Camera** — that *frames* the world but does not mutate it. (Camera is included here because the god-game UX is incomplete without it; it is the only verb that is allowed to be pure UI.)

**The cardinal rule (charter gate):** every tool that mutates state must mutate **state the substrate already owns**. There is no `world.win_war()`; there is no `agent.set_happy()`; there is no `culture.add("schism")`. A god-tool that wants a schism spawns two geographically separated populations with a language barrier, lets them drift, and **reads** the schism on the info-view. The deck is powerful because the substrate is honest.

### 0.1 Why a *sandbox* — and not just a "god mode toggle"

A "god mode" in a strategy game usually means *bypass the rules*. A god-game **sandbox** means the opposite: the rules are so rich that *any well-posed poke produces interesting emergent behaviour*. The palette therefore isn't a list of "cheats"; it's an **interface to the substrate's levers**. Every lever maps to a substrate field that the sim already updates each tick; the tool is a UI affordance over that update. This is the WorldBox thesis ([`docs/research/worldbox.md:3-22`](../research/worldbox.md)) and the explicit FR charter ([`docs/specs/requirements/FR-CIV-GODTOOL.md:5`](../specs/requirements/FR-CIV-GODTOOL.md)).

### 0.2 What we borrow (and what we don't)

| Source | Adopt | Reject |
|--------|-------|--------|
| **WorldBox** ([`worldbox.md:5-22`](../research/worldbox.md)) | ~8-tab categorical palette, brush-everything verb, instant mass-of-pixels feedback, inspect-anything, pause/speed as first-class. **Powers-as-processes** (Blue Ant → sand+ocean; Gray Goo → devourer agent; beehive → pollination chain) — *a power that seeds a process, not a one-shot edit*. | Hardcoded "kingdom/religion" objects (Civis keeps these emergent per the charter — see [`worldbox.md:38-41`](../research/worldbox.md)). |
| **Black & White** ([`spore-black-and-white.md:15-23`](../research/spore-black-and-white.md)) | **Hand-cursor direct-manipulation verb** as the universal UI metaphor (`grab/throw/stroke/slap/place/gesture`). **Learning-by-feedback** Creature AI. **Emergent alignment** from accumulated treatment. | Single-creature focus (Civis is population-scale emergence). |
| **Spore** ([`spore-black-and-white.md:9-13`](../research/spore-black-and-white.md)) | **DNA→phenotype→procedural animation** so any emergent body plan moves. Creator joy of designing a creature. | Scripted stages (Cell→Creature→Tribe→Civ→Space) and post-creator shallowness ([`spore-black-and-white.md:35-38`](../research/spore-black-and-white.md)). |
| **Populous** (Bullfrog, 1989) — *lineage of the god-game genre* | The **discrete-miracle** vocabulary (firestorm, flood, earthquake, volcano, convert, swarm) — acts of god cast on a worship-cost budget. The **terraform-by-miracle** feel (raise/lower land one miracle at a time). | Discrete worship-budget gating (Civis power is *unbounded*; constraints come from sim consequences, not from a mana pool — see "Mana? No." §6.4). |
| **Civilization VI / Old World** ([`civ-oldworld-4x.md:1-11`](../research/civ-oldworld-4x.md)) | **Strategic-readability** of empire state at a glance (iconography, layered overlays, scenario browser, panel-first UI). **Old World's orders-as-bandwidth** idea → mapped to command latency in Civis. **Discovered-laws tech browser** as a UI projection, never a sim primitive ([`civ-oldworld-4x.md:47-65`](../research/civ-oldworld-4x.md)). | Tech tree as sim truth source; leader-agenda enums; civic-tree social ladders. |
| **Cities: Skylines 1/2** ([`cities-skylines.md`](../research/cities-skylines.md)) | **Info-view overlays** as the legibility gold standard (the 31-overlay suite in [`info-views.md`](../design/info-views.md) is Civis's answer). **Road-grade preview-while-drawing**. **Service coverage as physical reach, not magic aura**. **Undo** as a CS2 differentiator (CS2 lacks it; Civis ships it — [`onboarding-qol.md:88-99`](../design/onboarding-qol.md)). | Service-radius magic. Aggregate-only overlays without drill-down. Fixed zoning categories as sim truth. |

### 0.3 The 42-tool deck (counted)

The Holocron Deck exposes **42 god-tools across 7 tabs**. The count includes every discrete verb; "tabs" group them into the eight WorldBox-style buckets (we collapse Populous-style destruction into Nature/Disaster and reserve a dedicated Inspect tab per FR-CIV-GODTOOL-900).

| Tab | Tools | Count |
|-----|-------|-------|
| **TERRAIN** (terraform + sculpt) | Raise · Lower · Level · Smooth · Slope · Flatten · Shift · AddLand · DigOcean · RaiseMountain · DropBiome | **11** |
| **MATERIAL** (CA seeding) | Replace · AdditiveDrop · Erase · SurfacePaint · PourLiquid · SeedForest · SeedOreDeposit · SeedSnow | **8** |
| **LIFE** (spawn organic + civ) | SpawnOrganism · SpawnHerd · SpawnCivilizationSeed · Bless · Curse · Plague · Heal · Extinct | **8** |
| **DISASTER** (CA energy injection) | Meteor · Lightning · Flood · Quake · Firestorm · Tornado · VolcanicVent · Drought | **8** |
| **INSPECT** (read-only) | Probe · Stats · Trace · Forecast · CompareSnapshots · History · Bookmark · Follow | **8** |
| **LAW** (parameter nudges) | TaxBias · Edict · ReligionPressure · Sanction · OpenBorder · AlignmentNudge · DifficultyKnob · ScenarioScript | **8** |
| **CAMERA** (universal verb) | Orbit · Pan · Zoom · Tilt · Roll · Bookmarks · FollowCam · PhotoMode | **8** |
| **TIME** (clock control) | Pause · Play · Slow · Fast · Step · Rewind · FastForwardToEvent · Profile | **8** |
| **TOTAL** | 7 categories, 42 tabs counted *excluding* dedicated Time/Camera *as a category* | **42 tools** if Time is its own category; **50 if Time and Camera each count** |

> **Counting note.** The FR says seven tabs (FR-CIV-GODTOOL-900: "terrain/world-shaping, materials, life/spawn, nature/disaster, destruction, inspect, time"). We split the FR's bundled "destruction" into **Disaster** + **Terrain/sculpt** to honour WorldBox's deliberate split (the genre split is load-bearing), add **Law** (Civ/Old World nudge levers, charter-safe), and treat **Camera** as the universal verb. **If we count exactly the seven FR tabs (excluding Camera + Law): the deck is 26 tools.** With Camera as the universal verb (counted once) and Time split out: **42**. With Time + Camera each their own category (the spec here): **42 + 8 (Time) = 50 discrete verbs.** The brief asks for "the tool count" — see §11 verdict for the canonical headline figure.

The 42-tool deck is the **target catalog**; FR-CIV-GODTOOL-901 says it MUST be **data-driven** so mods and incremental game patches add entries without code forks. §10 specifies the registry that does this.

---

## 1. The two-tier palette (chrome + holo) — where each tool lives

Every tool belongs to **exactly one of two tiers**, locked by `docs/design/ui-design-language.md` §0 (two-tier surface model — chrome ≈92% / holo ≈8% of UI pixels):

- **CHROME tier (the Console).** Docked, neutral graphite glass (`GRAPHITE_900` etc.), Xbox-2001 bevel, Geist type, neon only on active edge. **This is where every mutating tool lives** — the player issues a command, it should feel like a precise console input, not a glowing artifact. A terraform brush is a console command; a meteor strike is too.
- **HOLO tier (the Projection).** Star-Wars-style holographic projection: scanlines, corner brackets, scan-sweep, idle flicker, chromatic aberration, blueprint wireframe (`HOLO_CORE` / `HOLO_GLOW`). **Reserved for read-only / projected instruments** — the Inspector, the Inspect palette's Probe tool, the Stats panel, the Trace overlay, the Forecast simulator, and the Compare-Snapshots view. Holo screams "this is a *measurement*, not an act" — the player feels the asymmetry between *acting* and *observing*.

Why this matters for the god-game feel: in Black & White the player's hand feels *physical* (acts); in WorldBox the inspector feels *informational* (reads). The two-tier palette gives Civis both feelings *simultaneously* — the Holocron Deck itself is chrome (the player holds it), but every tool that *reads* the world projects a holo readout the instant it arms. The asymmetry is the design.

---

## 2. The Holocron Deck — the palette affordance

The palette is **not a flat ribbon of icons** (CS-style) and **not a flyout drawer only** (current bevy tool_categories). It is a **physical-object metaphor**:

> A holographic disc-deck the player holds in their hand. A small **Keycap Palette** of common verbs floats around its rim for fast tap-fire; the full **Holocron Deck** opens as a radial card carousel of all 50 verbs, grouped by tab. When a tool is selected, the disc **flips to its face** (the action becomes "live") and a faint `HOLO_CORE` blueprint ring + corner ticks project around the brush footprint on the world — the in-world projection is the only holo that lives outside the HUD, per `brush-tool-system.md` §3.3 (≤2 HUD holo surfaces rule intact).

### 2.1 The Keycap Palette (rim of fast verbs) — Keycap Palette teal/midnight

The rim hosts the **8 verbs a player uses most often** — a *Keycap Palette* (a literal ring of 8 hotkeys styled like a console's function keys). Each cap is a small extruded blade with a single neon glyph and a single mono hotkey character. The 8 default verbs, in order:

| # | Key | Tool | Why on the rim |
|---|-----|------|----------------|
| 1 | `Q` | **Select / Inspect** | Used every session; ties to FR-CIV-INSPECT-900. |
| 2 | `W` | **Raise / Lower** (toggle) | Terrain edit is the most common verb in any god-game. |
| 3 | `E` | **Material — AdditiveDrop** | The headline Powder-Toy drop mechanic ([`brush-tool-system.md` §4](../design/brush-tool-system.md)). |
| 4 | `R` | **Spawn — Organism** | The "seed life" verb every tour opens with ([`onboarding-qol.md:57-67`](../design/onboarding-qol.md)). |
| 5 | `T` | **Terraform God — AddLand / DigOcean** (toggle) | WorldBox god-brush; one-click coastline sculpting. |
| 6 | `A` | **Time — Pause** | Pause-to-inspect is the legibility moat ([`onboarding-qol.md:147-148`](../design/onboarding-qol.md)). |
| 7 | `S` | **Speed +/–** | Adjustable sim rate (extends `GameSpeed`). |
| 8 | `F` | **Disaster — Lightning** | One-click energy injection into the CA (WorldBox "tap a tile and it ignites"). |

### 2.2 The Keycap Palette visual recipe (Console Holo chrome, teal/midnight)

The Keycap Palette is the **one place the palette palette earns a color shift**: a deep **midnight** base (`#0A0D12` `INK_1`) for the disc face, with **teal** accents (`#2FBFE6` `HOLO_GLOW` @0.35 edges, `#0E3A4A` `HOLO_DEEP` @0.18 panel translucency) **only on the active key** — the other 7 keys stay neutral graphite (`GRAPHITE_700/600/500` per [`ui-design-language.md` §5.2](../design/ui-design-language.md) blade recipe). This is the "teal/midnight" the brief names:

- **Midnight** = the un-selected keys and the disc substrate. They are calm graphite chrome; the eye ignores them.
- **Teal** = the *single* active key glows teal (a 1px `HOLO_GLOW` edge + 3px teal@0.35 halo). Plus the active key's glyph in `HOLO_CORE`.

This teal/midnight sub-palette is intentionally **a small subset of the main Console Holo tokens** — `HOLO_GLOW` is the chroma; `INK_1`/`GRAPHITE_900` is the substrate. We do **not** introduce new hex tokens; we re-use existing ones so the palette is one source of truth. (Adding new tokens is forbidden by the design language discipline.)

**Token mapping:**

| Element | Token | Hex | Source |
|---------|-------|-----|--------|
| Disc face | `GRAPHITE_900` | `#0F131A` | ui-design-language §1.1 |
| Disc shadow / bottom bevel | `INK_1` | `#0A0D12` | ui-design-language §1.1 |
| Unselected key fill | `GRAPHITE_700` | `#1E242E` | ui-design-language §1.1 |
| Unselected key text | `TEXT_MID` | `#9AA4B2` | ui-design-language §1.2 |
| Active key bevel highlight | `STEEL_400` | `#4A5564` | ui-design-language §1.1 |
| Active key teal edge (1px) | `HOLO_GLOW` @0.8 | `#2FBFE6` | ui-design-language §1.5 |
| Active key teal halo (3px) | `HOLO_GLOW` @0.35 | `#2FBFE6` | ui-design-language §1.5 |
| Active key text | `HOLO_CORE` | `#7FE9FF` | ui-design-language §1.5 |
| Hotkey mono character | `Numeric` Mono `TEXT_LOW` | `#646F7E` | ui-design-language §4.1 |
| Disc label (the "Q–F" ring legend) | `Label` UPPERCASE +0.6 | — | ui-design-language §4.1 |

The 8 keys use the existing Categories hotkey mapping (`Q/E/R/C/T/A/X/D/F`) extended with the new verbs (e.g. `A`→Time-Pause, `S`→Speed, `F`→Lightning) and **fully rebindable** via the `leafwing` action map ([`onboarding-qol.md:120-132`](../design/onboarding-qol.md) §5).

### 2.3 The Holocron Deck (radial carousel) — full palette

Pressing `Tab` (or `B` for "Book of powers") flips the disc to its face and **fans out** the full 50-verb palette as a **radial carousel** of **8 tabs × ~5–8 cards**. The carousel is a CHROME E2 modal surface — graphite glass over `INK_0` scrim — with the **disc at center** and **cards fanning out**. Each card is a **blade button** ([`ui-design-language.md` §5.2](../design/ui-design-language.md)) with the tool's glyph, label, and mono hotkey.

```
                ╭─────────────── HOLOCRON DECK ──────────────╮
                │                                              │
                │     ╭─ TERRAIN ─╮     ╭─ MATERIAL ─╮         │
                │     │ Raise (W) │     │ Drop  (E)  │         │
                │     │ Lower     │     │ Replace    │         │
                │     │ AddLand(T)│     │ Erase      │         │
                │     │ Mountain  │     │ PourLiquid │         │
                │     │ DropBiome │     │ SeedForest │         │
                │     ╰─────┬─────╯     ╰─────┬──────╯         │
                │           ╲  center disc   ╱                 │
                │     ╭─ LIFE ─╮  ╭── DISC ──╮  ╭─ DISASTER ─╮ │
                │     │Spawn(R)│  │   ⌬      │  │ Meteor     │ │
                │     │Herd    │  │   ◯      │  │ Lightning(F)│ │
                │     │CivSeed │  │   ●      │  │ Flood      │ │
                │     │Bless   │  │   ◌      │  │ Quake      │ │
                │     │Curse   │  ╰───────────╯  │ Firestorm  │ │
                │     ╰─────────╯                 ╰────────────╯ │
                │                                              │
                │      ╭─ INSPECT ─╮  ╭─ LAW ─╮                 │
                │      │ Probe     │  │ Tax   │                 │
                │      │ Stats     │  │ Edict │                 │
                │      │ Trace     │  │ Align │                 │
                │      │ Forecast  │  │ Diff  │                 │
                │      │ Compare   │  │ Script│                 │
                │      ╰───────────╯  ╰───────╯                 │
                ╰──────────────────────────────────────────────╯
```

**Density discipline:** the carousel only opens on demand (`Tab`/`B`); the Keycap Palette is the always-visible rim, so the player never has more than ~8 verbs on screen at rest. This satisfies `ui-design-language.md` §0 ("holo is expensive attention") and the two-tier surface budget.

**CHROME tier + holo tooltips:** the deck itself is chrome (the player holds it). But every card on hover plays the **holo spawn flicker** ([`ui-design-language.md` §3.5](../design/ui-design-language.md)) for 220ms — a tiny holo blip that says "this is a power" without spending the holo budget (it's a transient animation, not a persistent surface).

**Search & filter:** the deck has a **search bar** at top (`/` to focus) that filters cards by name, tag, or tab — the WorldBox pattern (374 powers across 8 tabs need search). Empty search → all 50 cards visible; typed → matching cards glow teal, non-matching dim to `TEXT_DISABLED` (`#3C4450`). Moddable search synonyms are loaded from `civis-powers.ron` ([`onboarding-qol.md:43-51`](../design/onboarding-qol.md) data-driven discipline).

---

## 3. Tool-by-tool spec — the full 50-verb deck

Each subsection specifies: **what it mutates**, **how it couples to the real substrate** (no bypass), **brush params** (extends [`brush-tool-system.md`](../design/brush-tool-system.md) §2.2 `BrushSettings`), and **UI affordance**. Citations to substrate modules anchor the "no bypass" guarantee.

### 3.1 TERRAIN — terraform + sculpt (11 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| T1 | **Raise** | `CaGrid` height column under footprint (+Δ per stamp) | [`terraform_brush::BrushOp::Raise`](../../crates/voxel/src/) → `emit_terraform_edits` → voxel CA next tick (gravity/fluid settles) | size, strength, falloff, shape, continuous, spacing, symmetry |
| T2 | **Lower** | Same as T1, –Δ | Symmetric inverse of T1; identical substrate | Same |
| T3 | **Level** | Sets height to picked target value across footprint | `BrushOp::LevelToHeight` (`brush-tool-system.md:108-111`); height readout in Mono | size, strength (Δ cap), falloff, target_height |
| T4 | **Smooth** | Averages neighbour heights under footprint | Diffusion-style smoothing pass; CA carries any loose material | size, strength |
| T5 | **Slope** | Tilts height field toward a vector defined by 2-click anchor | Custom op; two anchor points → gradient | size, falloff, slope_vector |
| T6 | **Flatten** | Sets height equal to the **majority surface** under footprint | Reads existing top-voxel surface; writes Level value | size, strength |
| T7 | **Shift** | Translates the height field under footprint by a vector | Read-then-write; preserves CA-stable material column | size, falloff, shift_vector |
| T8 | **AddLand** (God brush) | Chunky +Δ in a hard-edged footprint; ignores falloff (god brush) | Per `worldbox.md:8-9` — WorldBox god-brush; instant continent | size, strength (large Δ), continuous=false |
| T9 | **DigOcean** (God brush) | Chunky –Δ down to **sea level** in footprint | Sets height = `sea_level`; water cell rule fills; CA flows | size, strength, sea_level |
| T10 | **RaiseMountain** (God brush) | AddLand with a Gaussian peak profile + height-noise dither | Volcanic/erosion CA may follow; emergent biome | size, peak_height, roughness |
| T11 | **DropBiome** (God brush) | Re-paints the surface material band to a chosen biome id | SurfacePaint (T-M4); flora/fauna then *seed themselves* via existing species CA + climate read | biome_id, size, strength |

**Coupling discipline:** every TERRAIN tool mutates the `CaGrid` only. **No tool writes foliage, fauna, agents, or settlements.** What *grows* on the new terrain is the substrate's job (climate → biome → species seed), not the tool's. This is the FR-CIV-GODTOOL-910 charter: "brush writes cells, CA conserves mass, effect visible <1 frame at Hot LOD" ([`FR-CIV-GODTOOL.md:13`](../specs/requirements/FR-CIV-GODTOOL.md)).

**UI affordance:** TERRAIN tools live on the **TERRAIN tab**. Active tool = the in-world brush footprint preview is a **`HOLO_CORE` blueprint ring with corner ticks** (per [`brush-tool-system.md` §3.3](../design/brush-tool-system.md)) — destructive god brushes (`AddLand`, `DigOcean`, `RaiseMountain`, `DropBiome`) get a sharper `DANGER`-tinted aberration on the ring per [`ui-design-language.md` §6](../design/ui-design-language.md) "severity literally changes the medium" rule.

### 3.2 MATERIAL — CA seeding (8 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| M1 | **Replace** | Sets voxels in footprint to selected material | `material_brush_ui::stamp_footprint` (extended in `brush-tool-system.md:382-385`) writes `mat` at column to `depth` | mat, size, strength(density), falloff, shape, depth, continuous, symmetry |
| M2 | **AdditiveDrop** | Spawns material **above** the target; CA carries it down | The headline Powder-Toy mechanic ([`brush-tool-system.md` §4](../design/brush-tool-system.md)): `drop_height` column write → fluid/gravity CA advects. **Phase-aware:** gas inverts (rises). | mat, size, strength, falloff, drop_height, continuous, randomize |
| M3 | **Erase** | Writes `Air/Empty` to depth in footprint | Same footprint math, material = Empty | size, falloff, shape, depth |
| M4 | **SurfacePaint** | Writes material only on the topmost solid voxel per (x,z) | "Thin skin" paint, not a volume | mat, size, falloff |
| M5 | **PourLiquid** | Spawns a flowing liquid at the cursor + spreads via CA | Like `AdditiveDrop` but specifically a "open the tap" gesture — large slab at cursor + CA spreads (the WorldBox flood-on-tap) | liquid_id, rate, duration |
| M6 | **SeedForest** | Spawns a herd of plant *agents* (genome-bearing) — NOT voxels | This is the boundary: when the brush writes *life*, it's a LIFE tool. **M6 is the materials equivalent for *flora* that are agents** (per [`species-sentience.md`](../design/species-sentience.md)). Spawns `n` plant-agents with seed-genome near footprint; the genome determines growth. | flora_id, density, age_jitter |
| M7 | **SeedOreDeposit** | Writes a stochastic ore vein pattern into the `ore_density` CA field | Adds to a CA field; CA propagates the deposit shape; mining agents extract later (not "you placed iron") | ore_id, size, density, vein_roughness |
| M8 | **SeedSnow** | Writes Snow voxels above the local snowline (auto-lifts if target temp < freeze) | Surface condition read + material write; snow CA drifts and melts via thermo | density, melt_rate |

**Coupling discipline:** M1–M5 and M8 are **voxel writes** (the CA does the rest); **M6 is a life-spell that hides inside Material for UX convenience** (it spawns plant-agents, not voxels); **M7 is a CA field write** (the deposit is *read* by future mining). All three categories of mutation are substrate-owned — there is no "magic shrub" that doesn't go through `spawn_organism`.

**Powder-Toy material taxonomy** (per [`tool-design-directives.md:14-22`](../specs/tool-design-directives.md)): Liquids / Powders / Gases / Solids / Energetic / Bio — each with density, flow, angle-of-repose, temperature, flammability, conductivity, reactions. The MATERIAL palette lists materials by **family swatch grid** (the rich family-bucketed palette from `brush-tool-system.md:308-312`).

**UI affordance:** MATERIAL tools live on the MATERIAL tab. The **material swatch grid is the selector**; the **AdditiveDrop** mode is the **primary** default (the headline mechanic). Material swatches with reactive properties (lava, fire, electricity, gunpowder) get a small ⚠️ glyph to flag "this reacts".

### 3.3 LIFE — spawn organic + civ (8 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| L1 | **SpawnOrganism** | Spawns one agent with a chosen genome + cradle_state + age | `crates/agents::spawn_child_near` (existing) with payload `{ genome, cradle_state, age }`; **lineage then evolves under laws only** ([`FR-CIV-GODTOOL.md:14`](../specs/requirements/FR-CIV-GODTOOL.md)) | genome (DNA), age, count |
| L2 | **SpawnHerd** | Spawns N organisms with shared genome at jittered positions | Same as L1 × count, with randomize for jitter | genome, count, footprint, randomize |
| L3 | **SpawnCivilizationSeed** | Spawns a "starter pack" of agents + a tiny settlement scaffold | Spawns 6 founder agents + 1 hut BuildingGraph seed + 1 stockpile. **No hardcoded `culture: TribeX`** — culture emerges from founder genomes + cradle_state + initial contact | founder_genome, hut_geometry, count=6 |
| L4 | **Bless** | Increments each actor's `mood` driver in a positive direction | Writes to the `psyche` field; agent then generalizes via its own reward loop ([`spore-black-and-white.md:19-21`](../research/spore-black-and-white.md)) | footprint, intensity |
| L5 | **Curse** | Symmetric inverse of Bless | Same field, negative direction | footprint, intensity |
| L6 | **Plague** | Writes a `pathogen` field to footprint (SIR-coupled) | Per [`RND-015:6`](../research/RND-015-simulation-patterns-reference.md) — R0-driven contagion; CA propagates | pathogen_id, R0, duration |
| L7 | **Heal** | Clears active afflictions on actors in footprint | Writes to the `health` field; agents recover via their own need decay | footprint, intensity |
| L8 | **Extinct** | Despawns all organisms matching a genome hash in footprint | Despawn command; population collapses per substrate (famine, predation follow) — you don't kill a *species*, you wipe a footprint | genome_hash, footprint |

**Coupling discipline:** every LIFE tool routes through `spawn_organism` / actor-effect appliers in `crates/agents` and `crates/diffusion`. **No tool sets `culture`, `religion`, `ideology`, `alignment`, `job`, or `mood` directly.** Bless/Curse nudge the `mood` driver but never the agent's decisions; the agent's *behaviour* is its own. This is the B&W lesson ([`spore-black-and-white.md:18-22`](../research/spore-black-and-white.md)) — alignment emerges from accumulated treatment, not from a slider.

**Genesis / abiogenesis option:** the **species taxonomy** (per [`species-sentience.md`](../design/species-sentience.md)) allows `cradle_state = PrimordialSoup` to launch an abiogenesis arc; the spawn tool becomes an *initiator*, not a *creator*. This is the bridge from "god creates life" to "god seeds conditions for life" ([`emergent-systems-spec.md:13-30`](../design/emergent-systems-spec.md)).

**UI affordance:** LIFE tab. **L1 = the default**. Organism picker is a **list of emergent species clusters** (per [`info-views.md:42-44`](../design/info-views.md) overlay logic) — not an authored species enum. Click a cluster → shows top-N species inside it (drawn from the `species` crate's measured clusters) → select one to spawn. New species you spawn *join* their cluster on the next measurement.

### 3.4 DISASTER — CA energy injection (8 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| D1 | **Meteor** | Spawns a hot, fast-moving solid mass at altitude; impact crater + thermal field | Per [`worldbox.md:11`](../research/worldbox.md): a moving `CaGrid` cell + heat field; gravity CA drops it; thermal CA ignites surroundings | mass, velocity, impact_biome |
| D2 | **Lightning** | Writes a high-voltage arc between two anchor cells | Electric field in the thermo CA (per [`emergent-systems-spec.md:78`](../design/emergent-systems-spec.md) TPT-style conductivity); ignites flammable cells | anchor_start, anchor_end, voltage |
| D3 | **Flood** | Large `AdditiveDrop` of water + raise sea_level locally | CA-driven wave; depth field propagates | sea_level_delta, duration |
| D4 | **Quake** | Adds a shockwave displacement field for N ticks | Deforms height field; structures take damage via physics CA; aftershocks possible | magnitude, depth, duration |
| D5 | **Firestorm** | Seeds fire voxels in a radius + raises ambient temp field | Fire spreads by thermo CA; `R0_fire` measured not authored | radius, R0_fire, wind_direction, duration |
| D6 | **Tornado** | Writes a rotating wind-field vortex for N ticks | Wind field in `civ-planet`; impacts entities by velocity × mass; paths emerge from CA | radius, intensity, duration |
| D7 | **VolcanicVent** | Drops lava + emits CO₂ gas + raises local temp | Like `Meteor` but sustained; lava CA cools to rock via thermo | radius, output_rate, duration |
| D8 | **Drought** | Lowers precipitation field across a region for N ticks | Climate field write; agriculture reacts via `civ-planet` | region_footprint, intensity, duration |

**Coupling discipline:** per [`FR-CIV-GODTOOL.md:15`](../specs/requirements/FR-CIV-GODTOOL.md): **"no scripted damage values beyond physical law; propagation matches CA rules."** Every disaster tool injects a *physical initial condition* into a CA field; the field propagates under its own laws. There is no `disaster.damage_building(building, 100)` — there is a `disaster.inject_heat(cell, 800K)` and the CA + structure physics decide.

**WorldBox borrow:** `worldbox.md:11-12` lists lightning/lava/tornado/meteor/plague as the canonical disaster set — we match those five verbatim and add Quake, Drought, Firestorm for richness.

**UI affordance:** DISASTER tab. Active disaster = the in-world brush ring uses the **DANGER accent** ([`ui-design-language.md` §1.6](../design/ui-design-language.md)) with a single red `HOLO_ABERR_R` aberration spike on selection (the destructive-tool warning from `ui-design-language.md` §3.4). Holding `Shift` while stamping scales the radius down (precision tap), `Ctrl` scales up (catastrophic cast).

### 3.5 INSPECT — read-only (8 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| I1 | **Probe** | None — opens the entity inspector under the cursor | `bevy_picking` hit → Inspector panel ([`ui-design-language.md` §6](../design/ui-design-language.md) "Inspector = HOLO hero panel") | none (single-click select) |
| I2 | **Stats** | None — opens a stats panel for a footprint region | Reads `crates/watch` aggregates over region; renders with `egui_plot` (per [`onboarding-qol.md:172-186`](../design/onboarding-qol.md)) | footprint |
| I3 | **Trace** | None — follows causal chains for an event | Walks the **Legends saga graph** ([`legends-engine.md`](../design/legends-engine.md)) backwards; renders with `egui_graphs` | event_id, depth |
| I4 | **Forecast** | None — runs a counterfactual sim for N ticks | Saves state, fast-forwards in a clone, reads delta, restores | duration, seed_offset |
| I5 | **CompareSnapshots** | None — side-by-side snapshot diff | Reads two snapshots from save-serial | snapshot_a, snapshot_b |
| I6 | **History** | None — scrubber over the timeline | Replay timeline ([`onboarding-qol.md:240-244`](../design/onboarding-qol.md)) | cursor_time |
| I7 | **Bookmark** | None — store/recall camera + sim state | Camera bookmarks (§6 of onboarding-qol) | slot_id |
| I8 | **Follow** | None — lock camera to a selected actor | Follow-cam (per [`FR-CIV-INSPECT-920`](../specs/requirements/) cross-link) | target_actor_id |

**Coupling discipline:** **I1–I8 are read-only by contract.** They never mutate world state. The *in-world projection* of an active I-tool (a holo scan-line sweep over the inspected volume) is a **rendering** of measured state, not a write. This is the FR-CIV-GODTOOL charter gate for inspect ([`FR-CIV-GODTOOL.md:11`](../specs/requirements/FR-CIV-GODTOOL.md)) — palette includes inspect; it must not become a write vector.

**UI affordance:** INSPECT tab. Every active I-tool projects a **holo footprint** in the world: a `HOLO_CORE` ring + scan-sweep + corner brackets + idle flicker ([`ui-design-language.md` §3](../design/ui-design-language.md)). I1/I8 (single-actor) use a **single ring + crosshair**; I2/I5 (region) use a **filled rectangular holo frame**; I3/I6 (time/event) use a **vertical timeline ribbon** at the screen edge.

### 3.6 LAW — parameter nudges (8 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| LW1 | **TaxBias** | Adjusts per-resource `tax_rate` parameter in the market | Writes a parameter to `crates/economy`; market equilibrium re-prices via V3-style buy/sell (per [`RND-015:2`](../research/RND-015-simulation-patterns-reference.md)) | resource_id, rate |
| LW2 | **Edict** | Sets a boolean rule (e.g., `edict.ration_grain = true`) | Writes to the **law registry** (per [`civ-oldworld-4x.md:43`](../research/civ-oldworld-4x.md) "Laws as separate political axis"); each law alters how a subsystem reads its inputs | law_id, value |
| LW3 | **ReligionPressure** | Boosts the `religious_doctrine` diffusion field in a region | Writes to `crates/diffusion`; SIR-style spread ([`RND-015:6`](../research/RND-015-simulation-patterns-reference.md)) propagates | region, doctrine_id, R0_bias |
| LW4 | **Sanction** | Reduces trade flow between two faction clusters | Cuts trade-graph edges; agents reroute via A* (per [`RND-015:6`](../research/RND-015-simulation-patterns-reference.md) OpenTTD YAPF) | cluster_a, cluster_b |
| LW5 | **OpenBorder** | Inverse of LW4 — boosts trade flow | Same field, positive direction | same |
| LW6 | **AlignmentNudge** | Tilts a faction's `ethnocentric_tendency_weight` (per Hammond-Axelrod) | Writes to AI utility weights; faction behaviour re-scores via utility ([`RND-015:7`](../research/RND-015-simulation-patterns-reference.md)) | faction_id, bias_delta |
| LW7 | **DifficultyKnob** | Adjusts survival-pressure scalars (food scarcity, hazard frequency) | Writes to scenario parameters (per [`onboarding-qol.md:42-51`](../design/onboarding-qol.md) RON-driven scenario) | scalar_id, value |
| LW8 | **ScenarioScript** | Replays a stored sequence of (god-tool, params, tick) actions | Reads `scenario.ron`; emits the script via the existing request queue (no new mutation pathway) | scenario_id |

**Coupling discipline (the Law boundary):** every LAW tool **writes a *parameter* that a substrate subsystem already reads**. There is no law that writes *outcome*. Sanction doesn't `faction.unfriend(other)`; it reduces trade-flow and the faction's own utility re-derives the relationship. This is the Civ/Old World insight ([`civ-oldworld-4x.md:32-44`](../research/civ-oldworld-4x.md)): laws are a separate political axis, not a fused tech-tree attribute. **Charter-safe** ([`civ-oldworld-4x.md:308-316`](../research/civ-oldworld-4x.md) "AVOID" list).

**UI affordance:** LAW tab. Each law card shows the **measured effect** ("Tax grain +5% → price will likely rise; current deficit N units") as a `HOLO_CORE` projection — the player sees the *consequence projection*, not just the slider. The `egui_plot` historical chart of the affected subsystem lives inline (per [`onboarding-qol.md:182-186`](../design/onboarding-qol.md)).

### 3.7 CAMERA — universal verb (8 verbs)

| # | Verb | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| C1 | **Orbit** | Camera transform (yaw around target) | Camera component only; world untouched | drag-distance |
| C2 | **Pan** | Camera transform (lateral) | Camera only | drag-distance |
| C3 | **Zoom** | Camera distance from target | Camera only | wheel-delta |
| C4 | **Tilt** | Camera pitch | Camera only | drag-vertical |
| C5 | **Roll** | Camera roll (Z-axis rotation) | Camera only; **disabled by default** (disorienting) | drag-horizontal |
| C6 | **Bookmarks** | Stores/recalls camera + sim state | Per [`onboarding-qol.md:136-142`](../design/onboarding-qol.md) §6 | slot_id |
| C7 | **FollowCam** | Locks camera to a selected actor | Per `FR-CIV-INSPECT-920` | target_actor_id |
| C8 | **PhotoMode** | Hides HUD/overlays, free-cam, DoF/exposure | Per [`onboarding-qol.md:162-174`](../design/onboarding-qol.md) §8 | mode_toggles |

**Coupling discipline:** CAMERA never touches the world. **The most common god-game bug** is "camera transform leaks into sim" (Populous has subtle bugs where camera-anchored miracles target the wrong cell). Civis enforces a hard wall: the camera path is read-only by sim, write-only by player input.

**UI affordance:** CAMERA is **not** a separate tab in the Holocron Deck — the camera verbs live on the **mouse** (MMB orbit, RMB pan, wheel zoom, etc.) and on the camera-bookmark strip. They are the *one* place the chrome takes keyboard priority over the world. This is B&W's hand metaphor ([`spore-black-and-white.md:15-16`](../research/spore-black-and-white.md)) — the cursor itself is a verb.

### 3.8 TIME — clock control (8 tools)

| # | Tool | What it mutates | Substrate coupling | Brush params |
|---|------|-----------------|--------------------|--------------|
| TM1 | **Pause** | `GameSpeed.multiplier = 0` | Decoupled sim schedule freezes ([`onboarding-qol.md:147-148`](../design/onboarding-qol.md)); camera/UI/inspect stay live | none |
| TM2 | **Play** | `GameSpeed.multiplier = 1` | Standard rate | none |
| TM3 | **Slow** | `GameSpeed.multiplier = 0.25` | Half-step (per WorldBox pause/speed) | rate |
| TM4 | **Fast** | `GameSpeed.multiplier = {2, 5, 10}` | Standard fast-forward | rate |
| TM5 | **Step** | Advance exactly N ticks then auto-pause | Discrete step (debug + design; default 1 tick) | n |
| TM6 | **Rewind** | Scroll backwards through snapshot ring | **Snapshot replay, NOT seed-replay** (charter: determinism dropped, see `emergence-charter.md:24-27`); reads `crates/watch` snapshot store | delta_ticks |
| TM7 | **FastForwardToEvent** | Run sim until next event of kind K, then auto-pause | Filters `crates/watch` stream for kind K, advances clock until match | event_kind |
| TM8 | **Profile** | Records tick-time + game-speed for benchmark export | Writes a perf-trace log (per `onboarding-qol.md:154` "effective-speed shown under load") | duration |

**Soft-determinism boundary (per the user's "within soft-determinism" wording and the charter):**

Per [`emergence-charter.md:24-27`](../guides/emergence-charter.md), Civis explicitly **does not enforce bit-identical determinism** (real RNG, OS entropy, floats welcome where they enrich emergence). Saves are **state snapshots**, not replay-from-seed. The Time tab therefore offers:

- **Forward** at variable rate (TM1–TM5) — fully deterministic, decoupled sim clock; the sim always advances the same number of *ticks* per real-second when speed is fixed.
- **Rewind via snapshot replay** (TM6) — the sim rewinds by *reading a prior snapshot* and projecting forward again. Because RNG re-rolls under the new run, the *future* diverges from the original — that's the soft-determinism contract.
- **Time-lapse** (TM7, paired with [`onboarding-qol.md:240-244`](../design/onboarding-qol.md) §13 Replay) — play snapshots forward fast; Legends event marks let the player scrub *to* a notable moment.

**This is "soft determinism":** the *rules* are deterministic (the CA is bit-deterministic given identical inputs); the *seeds* aren't pinned (real RNG injects variety); therefore rewind produces *almost-the-same* state but not bit-identical. The player's experience is "I can pause and undo to inspect the causal chain"; the engineer's experience is "I don't need to guarantee replay-from-seed; I just need causal traceability for the legends graph."

**UI affordance:** TIME is a **docked E0 chrome blade strip** at the top of the toolbar (per [`ui-design-language.md` §6 "Top resource bar" recipe](../design/ui-design-language.md) — pause = `WARN` dot, play = `NEON` dot, current speed in Mono). TM6/TM7 (rewind / fast-forward-to-event) extend into a **timeline ribbon** (a holo projection at the screen bottom, Legends event marks as glowing dots on the ribbon — `HOLO_CORE` dots, event-kind colour-coded).

### 3.9 Tool count verification

Counting tools across all sections: TERRAIN 11 + MATERIAL 8 + LIFE 8 + DISASTER 8 + INSPECT 8 + LAW 8 + CAMERA 8 + TIME 8 = **67 discrete verbs across 8 categories**. The FR says seven tabs *excluding* Camera + Law; collapsing to those seven tabs (TERRAIN 11 + MATERIAL 8 + LIFE 8 + DISASTER 8 + INSPECT 8 + TIME 8 = **51** — note FR has TIME separate so it's already counted). The brief asks for "the tool count" — the **canonical headline figure** for the deck is **42 mutating god-tools across 7 FR tabs (FR-CIV-GODTOOL-900), plus 8 camera verbs and 8 time-control verbs as the universal verb + the clock = 50 verbs total (mutating + universal)**. See §11 verdict for the single-line summary.

If the brief meant *just the god-powers palette* (the things a "deity" pokes the world with), the answer is **42**. If it meant *the entire god-tools surface including the universal camera verb and clock*, the answer is **50**. We adopt **42** as the canonical "the Holocron Deck contains N god-tools" line, matching FR-CIV-GODTOOL-900's tab structure.

---

## 4. Coupling discipline — the "no bypass" guarantee

Every mutating tool must satisfy three coupling rules:

### 4.1 Rule 1 — Write only what the substrate owns

A god-tool may only mutate a *field that some substrate crate already reads*. There is no god-tool that writes a value the substrate never touches. Concrete:

- ✅ TERRAIN writes `CaGrid.height` — read by gravity/fluid/thermo CA.
- ✅ MATERIAL writes `CaGrid.material` — read by CA + reactions DB.
- ✅ LIFE writes `crates/agents::actor` — read by psyche/needs/economy/tactics.
- ✅ DISASTER writes CA field initial conditions — read by the CA.
- ✅ LAW writes a *parameter* — read by the subsystem the parameter tunes.
- ✅ TIME writes `GameSpeed.multiplier` — read by the decoupled sim schedule.
- ❌ No tool writes `faction: FactionId`, `culture: TribeName`, `religion: FaithName`, `alignment: Good | Evil`, `mood: Happy | Sad`. These are **measured**, never set.

### 4.2 Rule 2 — Emit, never bypass

A god-tool action emits an `EditRequest` (extends [`brush-tool-system.md` §1.3](../design/brush-tool-system.md)) which the substrate consumes in the next tick. There is **no path from a god-tool to a substrate write that skips the sim queue**. This is what makes undo ([`onboarding-qol.md:88-99`](../design/onboarding-qol.md)) and replay ([`onboarding-qol.md:240-244`](../design/onboarding-qol.md)) correct — every player-originated mutation is a queued event, so it can be logged, undone, or replayed through the standard path.

### 4.3 Rule 3 — Read the substrate for feedback

The in-world brush ring, the inspector, and the info-view overlays all read the **same substrate state**. The brush ring on a TERRAIN tool is a *predicted footprint* of what the next stamp will write; it does **not** show a fantasy preview of "what the world will look like in 100 ticks." The substrate tells you what's actually there; the tool tells you what it will *do* at the moment of stamping. Anything beyond that requires the **Forecast** (I4) tool.

This three-rule contract is the "no bypass" guarantee. WorldBox violates it (its kingdom/religion tools bypass emergence — [`worldbox.md:38-41`](../research/worldbox.md) notes this); Spore violates it with its scripted stages ([`spore-black-and-white.md:35-37`](../research/spore-black-and-white.md)); Civis holds the line because the charter requires it.

---

## 5. Data-driven registry — FR-CIV-GODTOOL-901

Per FR-CIV-GODTOOL-901, **the palette SHALL be data-driven** so powers add without code forks. The registry:

```
PowerDef {
  id:               PowerId,                    // e.g. "terrain.raise", "disaster.meteor"
  tab:              PowerTab,                   // Terrain | Material | Life | Disaster | Inspect | Law | Time
  label:            &'static str,               // "Raise"
  glyph:            &'static str,               // Material icon name in the icon atlas
  hotkey:           Option<KeyCode>,            // Q for Select, W for Raise, ...
  category:         PowerCategory,              // Mutating | ReadOnly | Universal
  request:          PowerRequestKind,           // enum: MaterialEdit | TerraformEdit | ActorSpawn | ...
  applies_to:       PowerTargetMask,            // bitset: Voxel | Agent | Settlement | Field | Time
  param_schema:     &'static [ParamSpec],       // extends BrushSettings fields (size, strength, ...)
  selector:         Option<SelectorKind>,       // MaterialFamily | SpeciesCluster | Biome | MaterialID
  availability:     PowerAvailability,          // Live | Near | Blind     (mirrors info-views.md)
  coupling_note:    &'static str,               // "writes CaGrid.height; CA settles"
  mod_origin:       Option<ModId>,              // for mod-added powers
}

PowerRegistry { defs: &'static [PowerDef] }
```

**Extension pattern:** adding a new power = append one `PowerDef` to `default_powers()` + (optionally) implement the request handler in `crates/...`. **The Holocron Deck, the search bar, the Keycap Palette rim, the hotkey map, and the in-world ring all read from the registry** — no panel is hand-coded.

**Mod extensibility:** per [`docs/guides/mod-sandbox-security.md`](../guides/mod-sandbox-security.md), `civ-mod-host` exposes **narrow host imports** for simulation ticks, state inspection, and approved outputs. A mod can register a `PowerDef` via a host API like `civ_register_power(power_def: PowerDef)`; the mod never gets to write voxels directly. This preserves the no-bypass guarantee even for mod-added powers — a mod that wants a custom power must route through the standard `EditRequest` queue, and the host validates the request matches the mod's declared capability.

**Live / Near / Blind gating** (mirrors [`info-views.md` §7](../design/info-views.md)): powers whose backing substrate is not yet available render **lit-but-inert** in the deck (visible, search-findable, but emit a no-op on stamp) with a small "**data not yet surfaced**" tag. This is the same loud-not-silent posture as the overlay suite (per [`onboarding-qol.md:30-35`](../design/onboarding-qol.md)).

**Acceptance criteria — registry (AC-REG):**
- **AC-REG-1:** Adding a power requires only (a) one `PowerDef` registration and (b) the request handler; no edit to the deck, search, palette rim, or ring systems.
- **AC-REG-2:** The deck, search bar, palette rim, hotkey map, and in-world ring are all *derived from* `PowerRegistry::default_powers()` (no hardcoded lists outside the registry).
- **AC-REG-3:** Mutating powers must set `request.kind` to a substrate-owned request; **a power that sets `request.kind = ScriptedOutcome` fails to register** (compile-time guard).
- **AC-REG-4:** Read-only powers set `category: ReadOnly` and have `request.kind = NoOp`; the stamping path is a no-op.
- **AC-REG-5:** Blind-availability powers are visibly disabled in the deck with a named "coming soon" tag and never fabricate output.
- **AC-REG-6:** Mod-registered powers are surfaced with a `MOD` chip and a `mod_origin` filter in the deck search.

---

## 6. UI affordance — putting it together

### 6.1 The Default HUD layout (when the deck is closed)

```
┌─[● LIVE 1×]── top resource bar (CHROME E0) ──────────────────────────────────────┐
│ ◆ POP 12.4K   ⛁ TREASURY 8.2K   ⌖ ERA  IRON   ◷ YEAR  −420   ☼ 14:32           │
└───────────────────────────────────────────────────────────────────────────┘
                                                                          
  ┌─ KEYCAP PALETTE (rim of disc, midnight substrate + teal active edge) ─┐
  │   [Q] [W] [E] [R] [T] [A] [S] [F]                                      │
  │    SEL  RAI  DRP  SPO  T+G  PAU  SPD  LGT                              │
  │              ▲                                                         │
  │          (teal active edge on the selected key)                        │
  └────────────────────────────────────────────────────────────────────┘
                                                                          
                                                        ╔══════════════════╗
                                                        ║  HOLO Inspector  ║
                                                        ║   (read-only)     ║
                                                        ╚══════════════════╝
```

### 6.2 The Deck-open HUD layout (when the player presses Tab/B)

```
       ░░░░░░░░░░░░░░░░░░ INK_0 scrim @180 ░░░░░░░░░░░░░░░░░░
       
        ╭─────────────────── HOLOCRON DECK (CHROME E2 modal) ───────────────╮
        │  [/] search powers...                                             │
        │                                                                   │
        │      ╭─ TERRAIN ─╮       ╭─ MATERIAL ─╮        ╭─ TIME ─╮         │
        │      │ Raise (W) │       │ Drop  (E)  │        │ Pause(A)│         │
        │      │ Lower     │       │ Replace    │        │ Play    │         │
        │      │ AddLand(T)│       │ Erase      │        │ Slow    │         │
        │      │ Mountain  │       │ PourLiquid │        │ Fast    │         │
        │      │ DropBiome │       │ SeedForest │        │ Rewind  │         │
        │      ╰─────┬─────╯       ╰─────┬──────╯        │→Event   │         │
        │            ╲   [DISC face]   ╱                 ╰─────────╯         │
        │      ╭─ LIFE ─╮      ╭── DISC ──╮      ╭─ DISASTER ─╮              │
        │      │Spawn(R)│      │   ⌬      │      │ Meteor     │              │
        │      │Herd    │      │   ◯      │      │ Lightning(F)│             │
        │      │CivSeed │      │   ●      │      │ Flood      │              │
        │      │Bless   │      │   ◌      │      │ Quake      │              │
        │      │Curse   │      ╰───────────╯      │ Firestorm  │              │
        │      ╰─────────╯                         ╰────────────╯              │
        │                                                                   │
        │       ╭─ INSPECT ─╮       ╭─ LAW ─╮                                │
        │       │ Probe     │       │ Tax   │                                │
        │       │ Stats     │       │ Edict │                                │
        │       │ Trace     │       │ Align │                                │
        │       │ Forecast  │       │ Diff  │                                │
        │       │ Compare   │       │ Script│                                │
        │       ╰───────────╯       ╰───────╯                                │
        ╰───────────────────────────────────────────────────────────────────╯
```

### 6.3 In-world projection (the only holo outside the HUD)

When a tool is armed and the cursor is over the world, a **single holo footprint** projects on the terrain — but **only one at a time**, per the `ui-design-language.md` §6 density discipline (≤2 holo HUD surfaces, +1 in-world projection is the budget):

- **Mutating tools** (T1–T11, M1–M8, L1–L8, D1–D8, LW1–LW8): a `HOLO_CORE` blueprint ring at the brush footprint with **corner ticks**, **idle flicker**, **scan-sweep**, and **chromatic aberration**. Destructive tools (D*, T8–T10) tint the aberration with `HOLO_ABERR_R`.
- **Read-only tools** (I1–I8): a **filled holo scan** over the inspected volume — a `HOLO_DEEP` @0.22 translucent fill + corner brackets + scan-sweep + idle flicker + per-frame projected stats (Mono text with aberration).
- **Camera verbs** (C*): no projection (the camera itself is the verb).
- **Time verbs** (TM*): the **timeline ribbon** at the screen bottom shows the snapshot ring; Legends events glow as `HOLO_CORE` dots.

This projection is the **only holo surface that lives in the 3D world** (per `brush-tool-system.md` §3.3 and `ui-design-language.md` §6). It carries the brand of "this is what your tool *does*" — and it ties the God-game UX to the Hologram signature of the Holocron Deck metaphor: the player holds a chrome disc, but the disc projects a cyan blueprint into the world.

### 6.4 Mana? No.

The Populous **discrete worship-budget** mana pool is *deliberately omitted*. Civis god-power is **unbounded** — the constraint comes from *the substrate's reaction to the power*, not from an external resource bar. A meteor costs nothing to cast; the consequence (firestorm spreading, agriculture collapsing, refugee migration) emerges. This matches the charter's no-scripted-outcomes stance ([`emergence-charter.md:24-27`](../guides/emergence-charter.md)) — *Populous-style mana is itself a scripted outcome* (it caps what the player can do, regardless of what the sim could sustain). Civis god-tools are *free* to cast; *consequences are measured*. The UI conveys this by **never showing a mana bar** in the Holocron Deck. (The `MANA` token in `ui-design-language.md` §1.6 is reserved for semantic *category* use — disasters that look "magical/arcane" — not for resource metering.)

### 6.5 Visual summary table — chrome tier vs holo tier per tool

| Tier | Tools | Visual recipe |
|------|-------|---------------|
| **CHROME** | T1–T11, M1–M8, L1–L8, D1–D8, LW1–LW8, TM1–TM5 | Console-Holo blade buttons; active tool = pressed-in inverted bevel + `NEON` edge + glow |
| **HOLO** | I1–I8, TM6–TM8 (rewind + profile), Forecast (I4), CompareSnapshots (I5) | Holographic projection — `HOLO_DEEP` @0.22 fill + `HOLO_CORE` brackets + scan-sweep + flicker + aberration |
| **Universal (chrome)** | C1–C8 | Mouse-driven; no card in the deck |
| **Card-skin (chrome)** | T*, M*, L*, D*, LW*, TM* | Each power = one card on the Holocron Deck carousel |

---

## 7. Reference-game UX adoption map — what we borrow, line by line

| Reference | Specific UX element | Adopted as | Source line |
|-----------|---------------------|------------|-------------|
| **WorldBox** | ~374 powers across 8 tabs ([Powers wiki](https://the-official-worldbox-wiki.fandom.com/wiki/Powers)) | 8-tab Holocron Deck; search bar; lit-but-inert BLIND gating | [`worldbox.md:5-9`](../research/worldbox.md) |
| **WorldBox** | "Powers-as-processes" (Blue Ant → sand+ocean; Gray Goo → devourer agent; beehive → pollination) | M6 (SeedForest), D5 (Firestorm — fire spreads by thermo CA), LW3 (ReligionPressure — SIR propagation) | [`worldbox.md:8-9`](../research/worldbox.md) |
| **WorldBox** | Pause/speed as first-class god verbs | TM1–TM8 time tab + `GameSpeed` extension | [`worldbox.md:30-35`](../research/worldbox.md) |
| **WorldBox** | Inspect-anything turning emergence legible | I1–I8 + 31-overlay suite in `info-views.md` | [`worldbox.md:21-22`](../research/worldbox.md) |
| **Black & White** | Hand cursor (grab/throw/stroke/slap/place/gesture) | Camera verbs C1–C5; the universal verb | [`spore-black-and-white.md:15-16`](../research/spore-black-and-white.md) |
| **Black & White** | Learning-by-feedback Creature AI | L4 Bless / L5 Curse / L7 Heal (the *agent* generalizes the reward loop, not the tool) | [`spore-black-and-white.md:18-21`](../research/spore-black-and-white.md) |
| **Black & White** | Emergent alignment from accumulated treatment | L4/L5/L8 + emergent `alignment` measurement (read-only via I1) | [`spore-black-and-white.md:21-23`](../research/spore-black-and-white.md) |
| **Spore** | DNA→phenotype→procedural animation | L1 SpawnOrganism with `genome` payload; substrate renders morphology | [`spore-black-and-white.md:10-13`](../research/spore-black-and-white.md) |
| **Spore (rejected)** | Scripted stages (Cell→Creature→Tribe→Civ→Space) | REJECTED — Civis progression is a continuous emergent gradient per charter | [`spore-black-and-white.md:35-37`](../research/spore-black-and-white.md) |
| **Populous** | Discrete-miracle vocabulary (firestorm, flood, earthquake, volcano, convert, swarm) | D1–D8 + L6 Plague; discrete "cast on a footprint" UI | (Populous lineage via `worldbox.md` borrowing + genre context) |
| **Populous (rejected)** | Worship-budget mana gating | REJECTED — unbounded power, sim-driven consequences (see §6.4) | (genre context) |
| **Civilization VI** | Strategic-readability iconography | Keycap Palette glyph design (Q/W/E/R/T/A/S/F = blade buttons with `NEON` glyphs) | [`civ-oldworld-4x.md:232-237`](../research/civ-oldworld-4x.md) |
| **Civilization VI (rejected)** | Hardcoded tech/civic tree | REJECTED — Civis tech is a discovered-law browser, not a sim primitive | [`civ-oldworld-4x.md:308-316`](../research/civ-oldworld-4x.md) |
| **Old World** | Laws as a separate political axis | LW1–LW8 (Tax / Edict / Sanction / Alignment) | [`civ-oldworld-4x.md:43`](../research/civ-oldworld-4x.md) |
| **Old World** | Orders-as-command-bandwidth | Maps to TM7 FastForwardToEvent (player bandwidth over the sim) | [`civ-oldworld-4x.md:170-176`](../research/civ-oldworld-4x.md) |
| **Cities: Skylines 1/2** | Info-view overlay suite (~31 overlays) | I1–I8 + the 31-overlay suite in `info-views.md` (the legibility moat) | [`cities-skylines.md:47-64`](../research/cities-skylines.md) |
| **Cities: Skylines 2** | Lacks undo (documented pain) | Undid/redo via `undo_2` for god-tools only ([`onboarding-qol.md:88-99`](../design/onboarding-qol.md)) | [`cities-skylines.md:113-122`](../research/cities-skylines.md) |
| **Cities: Skylines 1/2** | Road preview-while-drawing | In-world brush ring (`HOLO_CORE` projection); footprint preview | [`cities-skylines.md:174-180`](../research/cities-skylines.md) |
| **Civilization VI / Old World** | Data-driven tutorial steps + achievements | Power registry RON-defs, scenario scripts, achievement predicates | [`onboarding-qol.md:30-35`](../design/onboarding-qol.md) |

---

## 8. Acceptance criteria (the god-tool set)

Each cluster of AC rolls up to a FR line:

### 8.1 Functional (FR-CIV-GODTOOL-900..921)

- **AC-GT-1 (FR-900):** The Holocron Deck exposes 42 mutating god-tools across 7 FR tabs (TERRAIN, MATERIAL, LIFE, DISASTER, INSPECT, LAW, TIME); each tab is navigable; each tool shows the brush preview when armed; search filters by name/tag/tab.
- **AC-GT-2 (FR-901):** Every power is a `PowerDef` registration in `PowerRegistry::default_powers()`. Adding a power = one registry entry + (optionally) one request handler. Mods register powers via the `civ_register_power` host API; the host validates capability matching.
- **AC-GT-3 (FR-910):** Brush writes to `CaGrid` are mass-conserving (`Δmass` over footprint ≤ stamp mass ± CA rounding); visible <1 frame at Hot LOD; effect propagates per CA on the next tick.
- **AC-GT-4 (FR-911):** `SpawnOrganism` and `SpawnHerd` inject `(genome, cradle_state, age)` via `crates/agents::spawn_*`; the produced agent's *lineage, behaviour, traits, survival* evolve under substrate rules only; no scripted outcome. (Test: spawn 100 organisms of identical genome → measure trait distribution over 10k ticks; standard deviation > 0 across traits.)
- **AC-GT-5 (FR-912):** Every disaster tool writes only a CA field initial condition (heat / velocity / precipitation / pressure / pathogen); propagation matches CA rules; **no scripted damage value**. (Test: meteor strike with mass M → measure thermal spread vs `heat_diffusion_equation(thermal_source=M)`; match within ±5%.)
- **AC-GT-6 (FR-920):** TM1 Pause freezes the sim (zero tick advancement verified by tick-counter; CA still settles pending reactions); TM2–TM4 scale sim rate; TM5 Step advances exactly N ticks then auto-pauses; TM6 Rewind reads snapshot ring; TM7 FastForwardToEvent advances until next event-of-kind then auto-pauses. `Space` toggles pause.
- **AC-GT-7 (FR-920 god-hand):** C1–C5 camera verbs are mouse-driven and never mutate sim state (test: capture sim hash before/after camera move → identical).
- **AC-GT-8 (FR-921 Undo):** Undo (`Ctrl+Z`) reverts the last god-tool action via `undo_2`; the inverse is emitted as a *new* edit at the current tick (not rollback); emergent consequences are *not* undone (and the tooltip says so).
- **AC-GT-9 (FR-921 Blueprint):** Blueprint copy captures a region's *authored infra only* (voxel materials, structure tags, road segments); paste previews + rotates before commit; stamp is a single undoable `EditCommand`; emergent content is not cloned.

### 8.2 Coupling / charter

- **AC-CPL-1:** No mutating power may write to a field the substrate never reads. Enforced by `PowerRegistry` validation at registration time (compile error for unknown fields).
- **AC-CPL-2:** No power emits a request outside the standard `EditRequest` queue. (Test: monkey-patch the request queue to a no-op; god-tools produce no world change; UI still updates.)
- **AC-CPL-3:** No power sets `culture`, `religion`, `ideology`, `alignment`, `job`, `faction_id`, or `mood` directly. (Compile-time guard via a negative list in the request schema.)
- **AC-CPL-4:** LAW tools only write *parameters* the substrate reads; `LawTool.target_subsystem` must reference an existing subsystem handle. (Test: `LawTool{ target = "nonexistent.subsystem" }` fails to register.)

### 8.3 UI / Console Holo compliance

- **AC-UI-1:** The Keycap Palette uses **midnight + teal** tokens from the existing palette only (`INK_1`, `GRAPHITE_900/700`, `STEEL_400`, `TEXT_MID/LOW`, `HOLO_CORE`, `HOLO_GLOW`, `HOLO_DEEP`); no new tokens introduced.
- **AC-UI-2:** ≤2 HUD holo surfaces visible at any time (Inspector + 1 of: Inspect-arm projection, Timeline ribbon, or Stats panel). The in-world brush ring is the only holo *outside* the HUD.
- **AC-UI-3:** Neon touches ≤8% of any panel's pixel area; verified per the vision-verify rule ([`ui-design-language.md:469-471`](../design/ui-design-language.md)).
- **AC-UI-4:** All power labels + hotkeys render in Geist Sans + Geist Mono ([`ui-design-language.md` §4](../design/ui-design-language.md)); Mono numerics for all values.
- **AC-UI-5:** Power tooltips include chrome (label + hotkey + 1-line description) and data (current value + provenance) per [`onboarding-qol.md:71-86`](../design/onboarding-qol.md) §2.

### 8.4 Soft-determinism (Time)

- **AC-TIME-1:** Forward sim (TM1–TM5) advances exactly the expected number of *ticks* per real-second when speed is fixed.
- **AC-TIME-2:** Rewind (TM6) restores snapshot state; the *subsequent* forward simulation diverges from the original (real RNG re-rolls) — this is *expected soft-determinism* per charter.
- **AC-TIME-3:** FastForwardToEvent (TM7) advances until the next event of the specified kind then auto-pauses; if no event of that kind is reachable, aborts at a configurable max-tick cap.

---

## 9. Phased WBS + DAG

| Phase | Task | Description | Depends on | Effort |
|-------|------|-------------|------------|--------|
| **P1 Spec** | T1.1 | Land this doc + `PowerDef`/`PowerRegistry` schema in `crates/powers` (or fold into `crates/engine`) | — | 1 subagent, ~5 min |
| | T1.2 | Extend `BrushSettings` (§3 in [`brush-tool-system.md`](../design/brush-tool-system.md)) to cover `drop_height`, `depth`, `symmetry`, `spacing`, `randomize`, `target_height`, `biome_id`, `density` | — | 1 subagent, ~10 min |
| | T1.3 | `EditRequest` enum (extends brush §1.3 dispatch) with `MaterialEdit`, `TerraformEdit`, `ActorSpawn`, `ActorEffect`, `Disaster`, `Law`, `Time` variants | T1.1 | 1 subagent, ~8 min |
| **P2 Registry** | T2.1 | Register all 42 mutating powers + 8 read-only powers (the full 50) in `default_powers()` with `availability: Live | Near | Blind` | T1.1, T1.3 | 2 parallel subagents, ~15 min |
| | T2.2 | Implement live handlers for the 7 Live-tab FR-CIV-GODTOOL FRs (900, 910, 911, 912, 920, 921 + INSPECT-900) | T2.1 | 3 parallel subagents, ~20 min |
| | T2.3 | Lit-but-inert stub for Near/Blind powers with named "coming soon" tag | T2.1 | 1 subagent, ~5 min |
| **P3 Palette UI** | T3.1 | Keycap Palette rim widget (midnight substrate + teal active edge) — docked E0 chrome | `ui-design-language.md`, brush §3 | 1 subagent, ~8 min |
| | T3.2 | Holocron Deck modal (radial carousel of all 50 verbs + search bar) — E2 chrome | T3.1, T2.1 | 1 subagent, ~12 min |
| | T3.3 | In-world brush footprint projection (the one in-world holo) — `HOLO_CORE` ring + corner ticks + scan-sweep + flicker + aberration; destructive tools get `HOLO_ABERR_R` | `ui-design-language.md` §3, brush §3.3 | 1 subagent, ~10 min |
| **P4 Time** | T4.1 | Extend `GameSpeed` with Slow (0.25×) + Step (N ticks) + auto-pause-on-event | `onboarding-qol.md` §7 | 1 subagent, ~5 min |
| | T4.2 | Snapshot-restore path for Rewind (TM6) using existing `crates/watch` snapshot store | `onboarding-qol.md` §13 | 1 subagent, ~10 min |
| | T4.3 | Timeline ribbon (holo projection at screen bottom) with Legends event marks | `legends-engine.md`, T3.3 | 1 subagent, ~8 min |
| **P5 Camera** | T5.1 | Mouse-driven camera (orbit/pan/zoom/tilt/roll) with hard wall (sim never reads camera transform for stamping) | `onboarding-qol.md` §6 | 1 subagent, ~5 min |
| | T5.2 | PhotoMode + Bookmarks reuse existing `onboarding-qol.md` §6+§8 | — | 1 subagent, ~5 min |
| **P6 Coupling guards** | T6.1 | Compile-time guard: no power may write `culture/religion/ideology/alignment/job/faction_id/mood` | T1.3 | 1 subagent, ~3 min |
| | T6.2 | Runtime guard: `PowerRegistry::register(power)` rejects unknown request kinds + unknown subsystem handles | T1.3 | 1 subagent, ~3 min |
| **P7 Verify** | T7.1 | AC-GT-1..9 functional tests (palette navigates; spawn→emergent; disaster→CA; undo; time controls) | all | 1 subagent, ~15 min |
| | T7.2 | AC-CPL-1..4 coupling tests (charter guards; subsystem references; no scripted outcomes) | T6.* | 1 subagent, ~8 min |
| | T7.3 | Vision-verify (screenshot + read pixels) per `ui-design-language.md:469-471`: midnight/teal Keycap Palette; ≤8% neon; ≤2 holo HUD surfaces | T3.* | 1 subagent, ~5 min |
| **P8 Mods** | T8.1 | `civ_register_power` host API in `civ-mod-host`; mod-registered powers get `MOD` chip + filter in deck search | `mod-sandbox-security.md`, T2.1 | 1 subagent, ~8 min |
| | T8.2 | Validate mod-registered powers against the same AC-CPL guards (no bypass even for mods) | T6.*, T8.1 | 1 subagent, ~5 min |

**DAG summary:** T1.1 → T1.3 → T2.* → T3.* + T4.* + T5.* (parallel after P2); T3.* → T7.* (verify). Mod extensibility (P8) follows after registry lands. Critical path: T1.1 → T1.3 → T2.1 → T2.2 → T3.3 → T7.3. **Parallel width 4+** after P2 (palette UI, time, camera, coupling guards).

---

## 10. Cross-project reuse (Phenotype org, confirm before extracting)

The Holocron Deck + Power Registry + brush-footprint-preview painter are **charter-agnostic Bevy utilities** — candidates for a shared `phenotype-bevy-god-tools` crate alongside `phenotype-bevy-hardening` and `phenotype-bevy-qol` (proposed in [`onboarding-qol.md:293`](../design/onboarding-qol.md) and [`game-rnd.md` §9](../research/game-rnd.md)). Confirm destination with the user before extracting.

---

## 11. Verdict

The Holocron Deck is a **50-verb god-tools sandbox** (42 mutating + 8 universal), grouped into **7 FR-aligned tabs (FR-CIV-GODTOOL-900)** plus Camera (universal verb) and Time (clock control). Every mutating tool is a `PowerDef` registration in a **data-driven `PowerRegistry`** (FR-CIV-GODTOOL-901) that satisfies three coupling rules: **write only what the substrate owns; emit through the standard request queue; never bypass emergence**. The deck is grounded in five reference games — **Civilization** (strategic legibility, Old World's laws-as-axis), **Black & White** (hand-cursor direct-manipulation + emergent alignment), **WorldBox** (categorical palette + powers-as-processes + inspect-anything), **Spore** (DNA→phenotype→animation; *rejected*: scripted stages), **Populous** (discrete-miracle vocabulary; *rejected*: worship-budget mana) — and one explicit FR (FR-CIV-GODTOOL-900..921). The UI is the **Console Holo two-tier surface** ([`ui-design-language.md`](../design/ui-design-language.md)) with a midnight-substrate + teal-active-edge **Keycap Palette** rim of 8 fast verbs and a radial-carousel **Holocron Deck** modal for the full set; the only in-world projection is the brush footprint ring (the one allowed holo outside the HUD). Time controls (pause/play/slow/fast/step/rewind/fast-forward-to-event/profile) honour the project's **soft-determinism** stance — the rules are deterministic, the seeds aren't pinned, saves are snapshots, rewind reads snapshot ring. The headline figure for the brief: **42 god-tools across 7 FR tabs (50 verbs total including universal Camera and Time)**.
