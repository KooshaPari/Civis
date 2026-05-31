# Unified Brush-Cluster Tool System

> **Status:** Binding tool-architecture design (2026-05-31). Planner-stance spec for the
> UI/UX + Voxel/Material + Infrastructure Leads. Authority for refactoring
> `clients/bevy-ref/src/{tool_categories,material_brush_ui,terraform_brush,spawn_tools}.rs`
> into ONE coherent cluster→mode→action model driven by ONE shared `BrushSettings`.
>
> **Companion to** `docs/specs/tool-design-directives.md` (terraform/material superset
> directives) and `docs/design/ui-design-language.md` ("Console Holo" chrome/holo language).
>
> **Planner stance (per repo CLAUDE.md):** this carries the data model, state, acceptance
> criteria, and load-bearing pseudocode ONLY. No Rust implementation. The implementer maps
> these onto the existing modules, extending — never duplicating — them.

---

## 0. The thesis — why the current tools feel fragmented, and the fix

Today there are **four uncoordinated tool surfaces**:

1. `tool_categories.rs` — a rich `Category`/`SubTool` taxonomy + `ActiveSubTool`, but it maps
   everything down to the coarse `SpawnTool` enum and has **no brush-param awareness**.
2. `terraform_brush.rs` — its own `BrushSettings` (size/strength/falloff/shape/op/family) +
   its own floating "Terraform Brush" window, gated on `SpawnTool::Terraform`.
3. `material_brush_ui.rs` — a *second, parallel* `SelectedMaterial` carrying its **own**
   `size`/`strength` (a duplicate brush surface), its own floating "Material Brush" window,
   gated on `MaterialPaintArmed`.
4. `spawn_tools.rs` — the coarse `SpawnTool` enum + click→request emit + spawn/select/destroy.

The result: two competing brush-param resources, three floating windows that can overlap, no
single notion of "what cluster is active and what does a click do", and the toolbar↔panel
relationship is undefined. **Brush parameters are duplicated** (`terraform_brush::BrushSettings`
vs `material_brush_ui::SelectedMaterial`), violating the directives' "ONE brush controller"
mandate and the repo's "Extend, Never Duplicate" rule.

**The fix is a single inversion:** make the **cluster** the top-level selection unit and the
**brush** the universal verb-applier.

- The **toolbar** becomes a row of **clusters** (Material, Terraform, Life, Structure,
  Infrastructure, Disaster, Diplomacy, Policy, Select/Inspect). Selecting a cluster is itself a
  "brush action" selection — it activates that cluster and re-drives one **Brush Control Panel**.
- That **separate Brush Control Panel** shows the active cluster's **grouped action modes** + a
  cluster-specific **selector** (material palette / structure list / organism list / …) + the
  **shared brush parameters**. Nothing is crammed into the toolbar.
- **ONE `BrushSettings` resource** holds every shared param. Each cluster contributes only its
  *mode set* and its *selector payload*; the params surface is identical everywhere.

This unifies the four surfaces into: `tool_categories` (cluster registry) + `brush` (the new
unified controller, an *extension* of today's `terraform_brush::BrushSettings`) + `spawn_tools`
(the world-effect appliers). `material_brush_ui` collapses into a *selector widget* inside the
one panel, not a second brush.

---

## 1. The cluster → mode → action data model

Three nested layers. Each layer is a small, data-driven enum/struct so the registry stays
cheap to extend (the existing `tool_categories` discipline).

### 1.1 Layer 1 — Cluster (the toolbar unit)

A **Cluster** is a top-level toolbar slot. Selecting it = a brush-action selection that
activates the cluster and repopulates the Brush Control Panel. Reuses today's `Category`
struct (rename conceptually; keep `Category` as the type to avoid churn) plus a `kind`
discriminant and a `modes` list.

| Cluster | `ClusterKind` | Selector payload | Effect domain |
|---------|--------------|------------------|---------------|
| Select / Inspect | `Select` | none | picks/inspects sandbox entities |
| Material | `Material` | `MaterialId` (rich palette by family) | voxel CA grid |
| Terraform | `Terraform` | optional biome id / target height | terrain/voxel columns |
| Life | `Life` | organism/herd kind | actor spawns |
| Structure | `Structure` | building kind | seated cuboid actors |
| Infrastructure | `Infrastructure` | road kind | drag-drawn polylines |
| Disaster | `Disaster` | disaster kind | destructive area effects |
| Diplomacy | `Diplomacy` | relation kind | faction-state edits (future) |
| Policy | `Policy` | policy kind | polity-state edits (future) |

> Reuse the existing `CATEGORIES` constant verbatim for **order, icons, hotkeys, accent
> mapping, icon_key()**. Extend the `Category` struct with two fields: a `kind: ClusterKind`
> and `modes: &'static [ActionMode]`. The `subtools` field is **superseded** by `modes` +
> the selector (see §1.4 refactor) — keep it during migration, drop it after.

### 1.2 Layer 2 — Action Mode (the grouped verbs in the panel)

An **ActionMode** is one verb within a cluster's action group. This is the "grouped action
modes" the user asked for. It is the *primary* sub-selection inside the Brush Control Panel,
shown as a segmented blade group (UI §3). Modes are organized into **mode groups** (a labelled
sub-row) so a cluster can present e.g. "PLACEMENT" vs "PRECISE" vs "GOD" groups.

```
ActionMode {
    kind:        ActionKind,      // the world-effect verb (see §1.3)
    label:       &'static str,
    icon:        &'static str,
    group:       ModeGroup,       // for sub-row grouping in the panel
}
ModeGroup = Placement | Precise | God | Remove | Surface | Spawn | Effect | Relation
```

**Per-cluster mode catalog** (the action GROUPS the panel renders):

- **Material** (group `Placement` + `Remove` + `Surface`):
  - `Replace` — overwrite target voxels with the selected material (today's stamp).
  - `AdditiveDrop` — **drop from the sky**: spawn the material in a column *above* the target
    and let the fluid/gravity CA carry it down (Powder-Toy / WorldBox). See §4.
  - `Erase` — remove voxels (write Air/Empty) in the footprint.
  - `SurfacePaint` — paint only the topmost solid surface layer (thin skin), not a volume.
- **Terraform** (groups `Precise` + `God`) — reuse today's `BrushOp` exactly:
  - Precise: `Raise`, `Lower`, `LevelToHeight`, `Smooth`, `Slope`, `Flatten`.
  - God: `AddLand`, `DigOcean`, `RaiseMountain`, `DropBiome`.
- **Life** (group `Spawn` + `Effect`): `SpawnOrganism`, `SpawnHerd`, `Bless`, `Curse`,
  `Disease`, `Kill`.
- **Structure** (group `Placement`): `House`, `Farm`, `Workshop`, `Market`, `Wall`, `Tower`,
  `Monument`.
- **Infrastructure** (group `Placement`): `Road`, `Trail`, `Highway`, `Bridge`, `Canal`.
- **Disaster** (group `Effect`): `Meteor`, `Flood`, `Quake`, `Storm`, `Wildfire`, `Plague`.
- **Diplomacy** (group `Relation`): `Alliance`, `War`, `Trade`.
- **Policy** (group `Effect`): `Tax`, `Edict`, `Religion`.
- **Select/Inspect** (group none): `Select`, `Inspect`.

> The existing `SubTool` enum's verb list **is** this mode catalog. Refactor: lift `SubTool`'s
> variants into `ActionKind` (§1.3) and attach each to a cluster via a `modes` table, instead
> of mapping `SubTool→SpawnTool`. The icon/label methods migrate 1:1.

### 1.3 Layer 3 — Action (the resolved world effect)

`ActionKind` is the union of every verb. It is what a brush stamp *resolves to* against the
world. Each `ActionKind` knows (a) which **applier** consumes it and (b) how `BrushSettings`
params translate into the emitted request. This replaces the lossy `SubTool::spawn_tool()`
mapping with a richer dispatch.

```
ActionKind dispatch table (cluster-mode → applier message):
  Material::Replace       -> MaterialEditRequest { mode: Replace,  mat, footprint }
  Material::AdditiveDrop  -> MaterialEditRequest { mode: Drop,     mat, footprint, drop_height }
  Material::Erase         -> MaterialEditRequest { mode: Erase,    footprint }
  Material::SurfacePaint  -> MaterialEditRequest { mode: Surface,  mat, footprint }
  Terraform::*            -> TerraformEditRequest { op, footprint, target_height, biome_id }
  Life::SpawnOrganism|Herd-> SpawnCivilianRequest { position, count(from density) }
  Life::Bless|Curse|Disease|Kill -> ActorEffectRequest { effect, footprint }   (future applier)
  Structure::*            -> PlaceStructureRequest { position, kind }
  Infrastructure::*       -> PlaceRoadRequest { points, kind } (drag) / PlaceStructureRequest (Canal)
  Disaster::*             -> DisasterRequest { kind, footprint }                (future applier)
  Diplomacy::*            -> DiplomacyRequest { kind, target_faction }          (future applier)
  Policy::*               -> PolicyRequest { kind }                             (future applier)
  Select::Select|Inspect  -> SelectEntityRequest { position } / inspect open
```

**Inert-but-lit policy (preserve today's behavior):** any `ActionKind` whose applier message
does not yet exist renders **lit-but-inert** in the panel (no-op click), exactly as
`tool_categories` does with `spawn_tool()==None` today. The dispatch table is the *only* place
that grows when a new applier lands — panel + cluster layout never change.

### 1.4 Refactor of the four modules (extend, never duplicate)

| Module | Today | After |
|--------|-------|-------|
| `tool_categories.rs` | `Category`+`SubTool`+`ActiveSubTool`; maps to `SpawnTool` | **Cluster registry**: keep `CATEGORIES`; add `ClusterKind`, `modes` table, `ModeGroup`. `SubTool` variants become `ActionKind`. `ActiveSubTool`→`ActiveCluster`/`ActiveMode` (§2). |
| `terraform_brush.rs` | owns `BrushSettings`, `BrushOp`, shape/falloff; own window | **`BrushSettings` is promoted to the universal resource** (§2.2). `BrushOp`/`BrushShape`/`BrushFalloff` stay here. The "Terraform Brush" window is **deleted**; its slider/mode widgets move into the one Brush Control Panel as the Terraform cluster's contribution. `TerraformEditRequest` + `emit_*` applier stay. |
| `material_brush_ui.rs` | owns `SelectedMaterial{material,size,strength}` (dup brush); own window; `MaterialPaintArmed` | **De-duplicated**: drop `size`/`strength` from `SelectedMaterial` (now in `BrushSettings`); `SelectedMaterial` keeps **only** `material: MaterialId`. The palette becomes a **selector widget** rendered inside the one panel (not a window). `MaterialFamily` classification + swatches stay. `stamp_sphere`/`paint_into_voxel_grid` evolve into the `MaterialEditRequest` applier (§4). `MaterialPaintArmed` is replaced by "active cluster == Material". |
| `spawn_tools.rs` | coarse `SpawnTool`, click→request, appliers | Keep appliers + cursor/pointer plumbing. `ActiveTool`/`SpawnTool` become a **thin derived view** of `ActiveCluster`+`ActiveMode` (a `From` mapping for back-compat) OR are retired in favor of the new state once all callers move. New `MaterialEditRequest`/`ActorEffectRequest`/`DisasterRequest` messages added here (the appliers' home). |

> **Net:** one brush resource, one panel, one cluster registry, one dispatch table. Two
> floating windows (`Terraform Brush`, `Material Brush`) collapse into the single docked
> Brush Control Panel.

---

## 2. State model — ActiveCluster / ActiveMode / BrushSettings

Three resources are the runtime source of truth. All are Bevy `Resource`s.

### 2.1 Selection state

```
Resource ActiveCluster {
    kind: ClusterKind,            // which toolbar cluster is active
    index: usize,                 // index into CATEGORIES (for layout/accent)
}   // default: Select

Resource ActiveMode {
    kind: ActionKind,             // the picked verb within the active cluster
}   // default: Select::Select

// Per-cluster "last mode" memory so re-entering a cluster restores the user's
// last verb (e.g. Terraform remembers it was on Smooth). A small map:
Resource ClusterModeMemory {
    last_mode: HashMap<ClusterKind, ActionKind>,
}
```

**Invariant:** `ActiveMode.kind` must always belong to `ActiveCluster.kind`'s mode list.
Switching cluster sets `ActiveMode` from `ClusterModeMemory` (or the cluster's first mode).

### 2.2 The shared `BrushSettings` resource (ALL params)

Promote today's `terraform_brush::BrushSettings` to the **single** brush param surface. It
already has size/strength/falloff/shape/op/continuous/target_height/biome_id. Extend it with
the cross-cluster params the user listed ("and more"):

```
Resource BrushSettings {
    // --- core footprint (shared by EVERY cluster) ---
    radius:        f32,          // brush size, world metres   (RADIUS_RANGE)
    strength:      f32,          // op magnitude / fill density (STRENGTH_RANGE)
    falloff:       BrushFalloff, // Linear | Smooth | Hard
    shape:         BrushShape,   // Circle | Square | Diamond
    continuous:    bool,         // continuous-paint vs single-stamp

    // --- "and more" shared params ---
    spacing:       f32,          // min world-distance between stamps along a drag (0 = every frame)
    randomize:     f32,          // 0..1 jitter applied to position/strength/material-pick per stamp
    symmetry:      Symmetry,     // None | MirrorX | MirrorZ | Radial(n)  (god-pen / world-edit symmetry)

    // --- material-cluster params (ignored by other clusters) ---
    drop_height:   f32,          // AdditiveDrop spawn altitude above target (voxels)  [§4]
    depth:         i32,          // volume depth for Replace/Erase (how many voxels down)

    // --- terraform-cluster params ---
    op:            BrushOp,      // resolved from ActiveMode for Terraform
    target_height: f32,          // LevelToHeight
    biome_id:      u8,           // DropBiome

    // --- life/structure-cluster params ---
    density:       f32,          // 0..1 spawn-count multiplier (herd size, sparse scatter)
}
```

**Ranges (constants on `BrushSettings`, extend today's):** `RADIUS_RANGE=(2,96)`,
`STRENGTH_RANGE=(0.25,40)`, `DROP_HEIGHT_RANGE=(1,64)`, `DEPTH_RANGE=(1,32)`,
`SPACING_RANGE=(0,16)`, `RANDOMIZE_RANGE=(0,1)`, `DENSITY_RANGE=(0,1)`.

**Param applicability matrix** (which params each cluster's panel exposes — others are hidden,
not deleted, so the resource stays single):

| Param | Material | Terraform | Life | Structure | Infra | Disaster |
|-------|:-:|:-:|:-:|:-:|:-:|:-:|
| radius | ✔ | ✔ | ✔ | ✔ (footprint preview) | – | ✔ |
| strength | ✔ (density) | ✔ (height Δ) | – | – | – | ✔ (intensity) |
| falloff | ✔ | ✔ | ✔ | – | – | ✔ |
| shape | ✔ | ✔ | ✔ | – | – | ✔ |
| continuous | ✔ | ✔ (non-god) | ✔ | – | – | – |
| spacing | ✔ | ✔ | ✔ | – | ✔ | – |
| randomize | ✔ | ✔ | ✔ | ✔ | – | ✔ |
| symmetry | ✔ | ✔ | – | – | – | ✔ |
| drop_height | ✔ (AdditiveDrop only) | – | – | – | – | – |
| depth | ✔ (Replace/Erase) | – | – | – | – | – |
| target_height | – | ✔ (Level) | – | – | – | – |
| biome_id | – | ✔ (DropBiome) | – | – | – | – |
| density | – | – | ✔ | ✔ | – | – |

`BrushSettings::visible_params(cluster, mode) -> &[ParamKind]` returns the ordered param list
the panel should render. This is the only cluster-awareness the param widgets need.

### 2.3 Brush footprint (shared emit struct)

A `BrushFootprint { center, radius, shape, falloff, strength, symmetry }` value is computed
once per stamp and embedded in every applier request, so Material/Terraform/Disaster appliers
all consume the **same** footprint geometry (the `coverage()`/`apply()` math already in
`terraform_brush` becomes shared).

---

## 3. Brush Control Panel — layout & UX (Console Holo)

A single **docked CHROME E0** panel (per `ui-design-language.md` §6 "Category toolbar" sits in
chrome; brush panel is its companion chrome surface). **Not a floating window** — it is anchored
adjacent to the toolbar so toolbar (clusters) and panel (controls) read as one instrument.

### 3.1 Anchor & relationship to toolbar

```
┌─[● LIVE]── top resource bar (CHROME E0) ──────────────────────────────────┐
└───────────────────────────────────────────────────────────────────────────┘
 ┌─ TOOLBAR (CHROME E0) ─┐   ┌─ BRUSH CONTROL PANEL (CHROME E0) ──────────────┐
 │ [SELECT]              │   │ MATERIAL                          ● live       │ ← header strip
 │ [▟MATERIAL▙] ◀ active │   │ ┌ ACTION ────────────────────────────────────┐ │   (Label, NEON_DIM underline,
 │ [TERRAFORM]          │   │ │ [▟REPLACE▙][ DROP ][ ERASE ][ SURFACE ]     │ │    live-dot since it drives world)
 │ [LIFE]               │   │ └────────────────────────────────────────────┘ │
 │ [STRUCTURE]          │   │ ┌ MATERIAL ──────────────────────────────────┐ │ ← cluster selector
 │ [INFRA]              │   │ │ Liquids  ▓▓▓▓  Powders ▓▓▓  Solids ▓▓▓▓     │ │   (palette / list /
 │ [DISASTER]           │   │ │ (rich WorldBox×PowderToy swatch grid)       │ │    organism set …)
 │ [DIPLOMACY]          │   │ └────────────────────────────────────────────┘ │
 │ [POLICY]             │   │ ┌ BRUSH ─────────────────────────────────────┐ │ ← shared params
 │                      │   │ │ SHAPE   [○][▢][◇]      SIZE  ▟▆▆▆━━━○ 12   │ │   (visible_params filtered)
 │  active = pressed-in │   │ │ FALLOFF [Lin][Smooth][Hard]  STR  ▟▆━━━○ 0.7│ │
 │  inverted bevel +    │   │ │ DROP HEIGHT ▟▆▆━━━○ 24   DEPTH ▟▆━○ 4        │ │
 │  full NEON edge+glow │   │ │ [✔ Continuous]  SPACING ▟━━○ 2  RANDOM ▟━○  │ │
 └──────────────────────┘   │ └────────────────────────────────────────────┘ │
                            └────────────────────────────────────────────────┘
```

The toolbar is the **left vertical blade strip** of clusters (reuse the §6 "Category toolbar"
recipe: blade buttons, active = pressed-in inverted bevel + full `NEON` edge + glow). The Brush
Control Panel sits immediately to its right (or above the toolbar on a horizontal layout),
**re-rendered whenever `ActiveCluster` changes**.

### 3.2 Panel sections (top → bottom)

1. **Header strip** (§5.1) — `UPPERCASE Label` = active cluster name; `NEON_DIM` underline;
   a single `NEON` live-dot since the panel drives live world edits. Right-aligned: a Mono
   readout of the active mode (e.g. `DROP`).
2. **ACTION group** — a labelled `BRUSH`-style sub-section containing the cluster's
   **ActionMode segmented blade group(s)** (§5.6 segmented control). Multiple `ModeGroup`s
   render as separate sub-rows with a tiny `TEXT_LOW` group caption (e.g. `PRECISE` / `GOD`
   for Terraform; `PLACE` / `REMOVE` for Material). Selected mode = pressed-in + `NEON` bottom
   edge. This is the "grouped action modes" requirement.
3. **SELECTOR** — cluster-specific payload picker, only when the cluster has one:
   - **Material:** the rich family-bucketed swatch palette (today's `material_brush_ui`
     `family_section`/`material_swatch`, restyled to the blade/inset-well recipe §5.7;
     selected swatch = 2px `NEON` ring per §5.7 selected-row treatment, not a colored fill).
   - **Structure / Life / Infra / Disaster:** a vertical **inset-well list** (§5.7) of the
     cluster's items with glyph + Label; selected row = 2px `NEON` left edge bar.
   - **Terraform / Select:** no selector (modes are self-sufficient).
4. **BRUSH params** — the shared param widgets, filtered by `visible_params(cluster, mode)`.
   Sliders use the §5.5 recipe (`NEON_DIM→NEON` fill, blade handle, Mono readout tooltip).
   Shape/Falloff are §5.6 segmented controls. `continuous` is a §5.6 boolean toggle.

### 3.3 Holo usage

The panel is **chrome**, not holo (holo budget is reserved for inspector + minimap per §6
density discipline). The **only** holo touch: the live **brush footprint preview projected in
the 3D world** (the cursor ring) MAY adopt a faint `HOLO_CORE` blueprint ring + corner ticks
when a destructive/god mode is armed — a small in-world projection, not a HUD holo surface.
Keep ≤2 HUD holo surfaces rule intact.

### 3.4 Interaction rules (inherited, non-negotiable)

- **Egui owns the pointer first** (`PointerOverUi`): clicking any panel/toolbar widget must
  never fall through to a world stamp. This gate already exists in `spawn_tools`; the unified
  emit path keeps it.
- **Hotkeys:** cluster hotkeys (the existing `Category.hotkey` Q/E/R/C/T/A/X/D/F) select the
  cluster. Mode cycling within a cluster: `[` / `]` (or `Tab`) cycles `ActionMode`.
- **Stamp cadence:** `continuous` + `spacing` govern drag stamping (single-stamp on press if
  `!continuous`; god modes always single chunky stamp — preserve today's `is_god()` rule).
- **Motion:** blade hover 90ms, press 60ms, panel re-skin on cluster switch = the §7 tab-switch
  slide (160ms) applied to the panel content swap.

---

## 4. The Material additive-drop-from-sky mechanic

The headline new mechanic. **`Material::AdditiveDrop`** does not write voxels at the target —
it spawns the chosen material in a **column above** the target footprint and lets the existing
fluid/gravity CA carry it down, so it **piles / flows / splashes** naturally (Powder-Toy sand
pour, WorldBox water/lava drop).

### 4.1 Mechanic spec

```
Given a stamp at footprint F (center c, radius r, shape, falloff) under Material::AdditiveDrop:
  drop_top    = surface_y_at(c) + drop_height        // spawn altitude (BrushSettings.drop_height)
  For each (x,z) cell inside F (weighted by shape coverage × falloff):
    spawn the selected material at voxels [drop_top - k .. drop_top] for a thin slab
    where slab thickness scales with strength (density) and coverage at (x,z).
  The CA then advects/settles these cells on subsequent ticks:
    - powders fall + pile to angle-of-repose
    - liquids fall + spread + seek level
    - gases rise instead (drop "below" semantics inverts for Gas phase — spawn low, let rise)
```

**Key contrast with `Replace`:** `Replace` writes at the target column (instant, deterministic
shape). `AdditiveDrop` writes **above** and defers final placement to the CA — the player gets
emergent piles/puddles, not a stamped blob. This is the "DROP FROM THE SKY" the user specified.

### 4.2 Params that drive the drop

- `drop_height` — how high above the surface the column spawns (higher = more spread/scatter on
  impact). New slider, Material-cluster only.
- `strength` (reused as **density/rate**) — slab thickness per stamp / how much material pours.
- `randomize` — jitters per-cell spawn so the pour looks granular, not a solid block (reuse the
  existing `voxel_dither` hash in `material_brush_ui` as the allocation-free jitter source).
- `continuous` + `spacing` — a held drag becomes a moving "pour spout."
- `shape`/`falloff` — the cross-section of the falling column.

### 4.3 Implementation path (extend existing)

The existing `material_brush_ui::stamp_sphere` already writes a material sphere into
`CaGrid` with a dither. **Extend, don't duplicate:**

- Generalize `stamp_sphere` → a `stamp_footprint(grid, footprint, mode, mat, params, salt)`
  that branches on `MaterialMode { Replace | Drop | Erase | Surface }`:
  - `Replace`: write `mat` in-footprint (today's behavior) to `depth`.
  - `Drop`: write `mat` in the lifted column `[surface+drop_height-slab .. surface+drop_height]`
    over the footprint's XZ cross-section; the CA does the rest.
  - `Erase`: write `Empty/Air` in-footprint to `depth`.
  - `Surface`: write `mat` only on the topmost solid cell per (x,z) column.
- The CA already advects whatever cells exist (the module comment promises "painted water flows,
  sand piles, lava cools" — drop just seeds those cells higher up). **No new CA code needed**;
  drop is purely a *where-to-seed* change.

### 4.4 Acceptance criteria — additive drop

- **AC-DROP-1:** With `AdditiveDrop` + a powder material, a single stamp spawns cells at
  `≈ surface + drop_height` (not at the surface), verifiable by reading grid cells right after
  the stamp.
- **AC-DROP-2:** After N CA ticks the spawned powder has descended and formed a pile whose base
  is wider than the spawn cross-section (angle-of-repose spread) — emergent, not a stamped blob.
- **AC-DROP-3:** With a liquid material, dropped cells fall and spread to seek a level rather
  than holding the spawn column shape.
- **AC-DROP-4:** With a gas material, `AdditiveDrop` spawns low and the cells **rise** (inverted
  drop semantics) — gas pour vs powder/liquid pour differ by phase.
- **AC-DROP-5:** `Replace` mode (same material, same footprint) writes at the target column and
  does NOT lift — proving Drop vs Replace are distinct verbs sharing one selector + params.
- **AC-DROP-6:** `drop_height`, `strength`, `randomize`, `continuous` each measurably change the
  pour (altitude, slab thickness, granularity, spout-while-dragging).

---

## 5. Per-cluster mode → world-effect mapping (the dispatch contract)

How each cluster's modes translate `BrushSettings` + selector into the applier request.

### 5.1 Material → `MaterialEditRequest`
- footprint from `BrushSettings`; `mat` from `SelectedMaterial.material`; `mode` from
  `ActiveMode` (`Replace|Drop|Erase|Surface`); `drop_height`/`depth` from `BrushSettings`.
- Consumed by the (extended) material applier writing into `CaGrid`; CA settles next tick.

### 5.2 Terraform → `TerraformEditRequest` (UNCHANGED applier)
- `op` resolved from `ActiveMode` (the §1.2 Terraform modes map 1:1 to today's `BrushOp`);
  footprint + `target_height` + `biome_id` from `BrushSettings`. Today's `emit_terraform_edits`
  + `effective_strength()` + `sample_delta()` stay as-is — they already implement the
  CS-precise + WorldBox-god superset. **Only the panel sourcing changes** (one panel, not the
  standalone Terraform window).

### 5.3 Life → `SpawnCivilianRequest` (+ future `ActorEffectRequest`)
- `SpawnOrganism`/`SpawnHerd`: emit spawn(s); `density` × footprint area = count (herd =
  many, organism = one). `randomize` scatters positions in-footprint.
- `Bless|Curse|Disease|Kill`: footprint-area actor effects → `ActorEffectRequest` (future
  applier; lit-but-inert until it lands).

### 5.4 Structure → `PlaceStructureRequest` (UNCHANGED applier)
- mode = building kind; single seated placement at the cursor (today's path). `radius` only
  drives a footprint preview; `density` allows scatter-place if >0.

### 5.5 Infrastructure → `PlaceRoadRequest` (drag) / `PlaceStructureRequest` (Canal)
- Road/Trail/Highway/Bridge = today's drag-to-draw polyline (unchanged `accumulate_road_draft`).
  Canal maps to a terraform-dig + water-fill combo (future; lit-but-inert acceptable).

### 5.6 Disaster → `DisasterRequest` (future applier)
- footprint + `strength`(intensity) + `kind`. Until the applier exists, several disasters can
  reuse `DestroyEntityRequest` over the footprint (today's `Kill`→`Destroy` mapping) so the
  cluster is partially live immediately.

### 5.7 Diplomacy / Policy → future appliers
- Faction/polity state edits. Lit-but-inert until backing systems exist (preserve today's
  `tool_categories` inert policy).

---

## 6. Acceptance criteria (system-level)

- **AC-CLUSTER-1:** The toolbar shows all 9 clusters in `CATEGORIES` order; selecting one sets
  `ActiveCluster` and repopulates the Brush Control Panel within one frame.
- **AC-CLUSTER-2:** Exactly **one** Brush Control Panel exists (the two legacy floating windows
  `Terraform Brush` + `Material Brush` are gone); it is a docked CHROME E0 surface anchored to
  the toolbar.
- **AC-CLUSTER-3:** Switching clusters preserves each cluster's last `ActiveMode` via
  `ClusterModeMemory`.
- **AC-MODEL-1:** There is exactly **one** `BrushSettings` resource; `SelectedMaterial` no
  longer carries `size`/`strength` (no duplicate brush surface) — enforced by a test asserting
  `SelectedMaterial` has only a material field.
- **AC-MODEL-2:** `visible_params(cluster, mode)` matches the §2.2 matrix; the panel renders
  only those params per cluster/mode.
- **AC-MODEL-3:** `ActiveMode.kind` always belongs to `ActiveCluster.kind`'s mode list
  (invariant test).
- **AC-MODE-1:** Material exposes the four action modes (`Replace`/`AdditiveDrop`/`Erase`/
  `SurfacePaint`) as a grouped segmented control + the rich family palette selector.
- **AC-MODE-2:** Terraform exposes the Precise group (6 ops) and God group (4 ops) as two mode
  sub-rows; behavior is byte-identical to today's `BrushOp` emit.
- **AC-DROP-1..6:** §4.4 (the additive-drop mechanic).
- **AC-DISPATCH-1:** Every `ActionKind` with an existing applier emits the correct request;
  every `ActionKind` without one renders lit-but-inert (no panic, no world effect).
- **AC-UI-1:** Panel honors `ui-design-language.md`: chrome graphite glass, neon only on
  active mode edge + selected swatch ring (≤8% area), Mono numeric readouts, no holo HUD surface
  added (verified per the vision-verify rule by screenshotting + reading pixels).
- **AC-GATE-1:** `PointerOverUi` still blocks world stamps when the cursor is over the toolbar
  or panel (no click-through).

---

## 7. Refactor plan (phased WBS + DAG)

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| 1 Model | T1.1 | Add `ClusterKind`, `ModeGroup`, `ActionKind`, `ActionMode`; attach `modes` to `Category`; lift `SubTool` verbs into `ActionKind` | — |
| 1 Model | T1.2 | Promote `terraform_brush::BrushSettings` to universal; add `spacing`/`randomize`/`symmetry`/`drop_height`/`depth`/`density` + ranges | — |
| 1 Model | T1.3 | Add `ActiveCluster`/`ActiveMode`/`ClusterModeMemory` resources; invariant + default tests | T1.1 |
| 1 Model | T1.4 | Add `BrushFootprint` + `visible_params(cluster,mode)`; param-matrix test | T1.2, T1.1 |
| 2 Dispatch | T2.1 | New messages `MaterialEditRequest`, `ActorEffectRequest`, `DisasterRequest` in `spawn_tools` | T1.1 |
| 2 Dispatch | T2.2 | Build the `ActionKind`→message dispatch table; inert-but-lit fallback | T2.1, T1.3 |
| 2 Dispatch | T2.3 | De-dup `SelectedMaterial` (drop size/strength); reroute to `BrushSettings`; test | T1.2 |
| 3 Drop | T3.1 | Generalize `stamp_sphere`→`stamp_footprint` with `Replace/Drop/Erase/Surface` modes | T2.3 |
| 3 Drop | T3.2 | Implement additive-drop seeding (lift column; phase-aware gas inversion); AC-DROP tests | T3.1 |
| 4 UI | T4.1 | Build the single docked Brush Control Panel (header + ACTION group + selector + params), Console-Holo styled | T1.4, T2.2 |
| 4 UI | T4.2 | Move material palette into panel as selector widget; delete `Material Brush` window | T4.1, T2.3 |
| 4 UI | T4.3 | Move terraform mode/param widgets into panel; delete `Terraform Brush` window | T4.1 |
| 4 UI | T4.4 | Wire cluster toolbar → `ActiveCluster`; mode segmented group → `ActiveMode`; hotkeys + `[`/`]` cycling | T4.1, T1.3 |
| 5 Verify | T5.1 | Screenshot + read pixels (vision-verify): one panel, ≤8% neon, drop-from-sky visibly piles/flows | T4.* , T3.2 |

**Cross-Project Reuse Opportunities:** `BrushSettings` + `BrushFootprint` + the
shape/falloff coverage math + `stamp_footprint` modes are candidates for the shared
`phenotype-voxel` kernel (a generic "voxel brush" the Civis Bevy client, and any future engine
binding, can reuse). Flag to the user before extracting; until then, keep them in
`terraform_brush.rs` (renamed conceptually to the brush controller home) as the single owner.

---

## 8. Requirements traceability (FR-CIV-BRUSH-*)

| Req ID | Requirement | Satisfied by |
|--------|-------------|--------------|
| FR-CIV-BRUSH-01 | Toolbar is a row of tool CLUSTERS; selecting one activates it + drives the brush panel | §1.1, §3, AC-CLUSTER-1/2 |
| FR-CIV-BRUSH-02 | A SEPARATE Brush Control Panel (not in toolbar) shows the active cluster's controls | §3, AC-CLUSTER-2 |
| FR-CIV-BRUSH-03 | Grouped ACTION MODES per cluster (segmented, by `ModeGroup`) | §1.2, §3.2, AC-MODE-1/2 |
| FR-CIV-BRUSH-04 | Material modes: Replace vs Additive-drop vs Erase vs Surface-paint + rich palette | §1.2, §3.2, §5.1, AC-MODE-1 |
| FR-CIV-BRUSH-05 | Additive-drop = spawn material above target; CA gravity/fluid carries it down | §4, AC-DROP-1..6 |
| FR-CIV-BRUSH-06 | Terraform modes superset (CS precise + WorldBox god) sharing brush params | §1.2, §5.2, AC-MODE-2 |
| FR-CIV-BRUSH-07 | ONE shared `BrushSettings` resource drives every cluster | §2.2, AC-MODEL-1 |
| FR-CIV-BRUSH-08 | Shared brush params: shape/size/strength/falloff/continuous + spacing/randomize/symmetry/drop-height/depth | §2.2, AC-MODEL-2 |
| FR-CIV-BRUSH-09 | `ActiveCluster`/`ActiveMode` state with per-cluster mode memory | §2.1, AC-CLUSTER-3, AC-MODEL-3 |
| FR-CIV-BRUSH-10 | Each cluster maps modes to the real world effect (material/terraform/spawn/…) | §5, AC-DISPATCH-1 |
| FR-CIV-BRUSH-11 | Unify the four fragmented modules; extend-not-duplicate; de-dup the second brush surface | §1.4, §7, AC-MODEL-1 |
| FR-CIV-BRUSH-12 | Panel uses the Console-Holo UI design language | §3, AC-UI-1 |
| FR-CIV-BRUSH-13 | Egui pointer gate preserved (no click-through) | §3.4, AC-GATE-1 |

> AgilePlus linkage: file under an Epic "Unified Brush-Cluster Tools" with Stories per phase
> (§7). Each FR-CIV-BRUSH-* gets a requirement→code→test TraceLink once implementers land tests.
