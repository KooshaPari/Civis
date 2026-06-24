# Civis — actor T-pose + wrong-model (real assets) — root cause + fix plan

**Status:** design (spec-only). **Item:** #95 — actors T-pose and "herd shows soldiers".
**Scope:** `clients/bevy-ref` (Bevy 0.18 reference client). No code changes in this doc.

## 0. TL;DR

Two distinct bugs collapse into one user-visible symptom:

1. **Wrong model dispatch** — `sim_bridge::sync_visible_gameplay`
   (`clients/bevy-ref/src/sim_bridge.rs:151-172`) **never reads `ActorVisual`** and
   never calls `gltf_models::actor_scene`. Every spawned entity — humanoid or
   herd — gets the same procedural capsule. The user sees the humanoid rig
   under a "herd" label, hence "herd shows soldiers".
2. **T-pose** — even after the dispatch is fixed, the CC0 rigs on disk have a
   different rig-name and clip-name vocabulary than
   `animation::attach_actor_animation` hard-codes
   (`clients/bevy-ref/src/animation.rs:261-284`). Result: every animation
   lookup returns `None`, the static fallback fires, and the rig freezes in
   its bind pose (T-pose).

Both are real-asset gaps: the `.glb` files are present, but the loader and
the animation binder were written against a different (KayKit Knight) rig
that is not what was actually fetched.

## 1. Evidence (file:fn)

### 1.1 Spawn path is primitive-only — wrong-model dispatch is dead

- `clients/bevy-ref/src/sim_bridge.rs:144-149` builds a `Capsule3d` mesh and a
  `Cuboid` mesh — every actor, every building, every frame.
- `clients/bevy-ref/src/sim_bridge.rs:151-154` queries
  `(&Civilian, &civ_agents::Position3d)` only. **`ActorVisual` is never read.**
- `clients/bevy-ref/src/sim_bridge.rs:166-171` spawns
  `(SimCivilianMarker, Mesh3d(capsule.clone()), MeshMaterial3d(material), Transform::…)`.
  No `SceneRoot`. No `Handle<Scene>`. No `gltf_models::actor_scene` call.
- `clients/bevy-ref/src/sim_bridge.rs:174-189` does the same for buildings
  with `building_mesh` and a faction/type-tinted material — also primitive-only.
- `gltf_models::actor_scene` (`clients/bevy-ref/src/gltf_models.rs:235-240`)
  is the documented dispatch table for `Humanoid` vs `Herd`, but no caller in
  `sim_bridge.rs` invokes it. The `bin/standalone.rs:144-152` closer comment
  ("sim_bridge swaps capsule/cuboid primitives for real Knight/house scenes")
  is aspirational — the swap was never performed.

**Conclusion:** model-by-species/role mapping does not happen at runtime.
Every actor renders as a faction-tinted capsule, which is what the user is
calling "soldiers". The herd entities are not even distinguishable from
humanoids in the render output.

### 1.2 Animation binder cannot find any clip in any fetched rig

`animation::attach_actor_animation` (`clients/bevy-ref/src/animation.rs:218-319`)
walks every named clip in the source glTF and looks up gaits by name from
hard-coded alias lists:

- `idle` (line 262): `["Idle", "Unarmed_Idle", "2H_Melee_Idle", "Idle_B"]`
- `walk` (lines 263-273): `["Walking_A", "Walking_B", "Walking_C", "Walking_D_Skeletons", "Walk"]`
- `run` (lines 274-283): `["Running_A", "Running_B", "Running_C", "Run"]`

These aliases match the **KayKit Knight / Skeleton** packs. They do **not**
match the rigs that `Tools/fetch-cc0-models.ps1` actually landed in
`clients/bevy-ref/assets/models/`:

| File | Animations | Clip names actually present | Bone name |
|------|------------|-----------------------------|-----------|
| `civilian.glb` (Quaternius Farmer) | **0** | *(none)* | `CharacterArmature` |
| `herd.glb` (Quaternius Horse) | 13 | `AnimalArmature\|Idle`, `AnimalArmature\|Walk`, `AnimalArmature\|Gallop`, … | `AnimalArmature` |
| `creature_skeleton_minion.glb` | n/a (untouched, not yet routed) | `Skeleton_Minion\|…` | (KayKit) |
| `creature_skeleton_warrior.glb` | n/a | `Skeleton_Warrior\|…` | (KayKit) |

(`asset_paths::CIVILIAN` = `models/civilian.glb` and `asset_paths::HERD` =
`models/herd.glb` — `clients/bevy-ref/src/gltf_models.rs:62-64`.)

For `civilian.glb`: `gltf.named_animations` is empty → `graph.add_clip` is
never called → `GaitNodes::any()` is `false`
(`clients/bevy-ref/src/animation.rs:139-141`) → `is_static = true`
(`clients/bevy-ref/src/animation.rs:295-303`). No animation ever plays, ever.

For `herd.glb`: clips are present but every name is prefixed with
`AnimalArmature|`. `pick(...)` does exact-string match
(`animation.rs:322-324`) — every lookup misses → `GaitNodes::any()` is
`false` → static fallback → T-pose. The rig is a horse; the user sees
a frozen horse (or, in the current broken state, a capsule that the user
labels "soldier").

### 1.3 SceneRoot is never created, so AnimationPlayer is never spawned

Until §1.1 is fixed, no `SceneRoot` ever enters the world. The fallback
diagnostic that the existing `CIVIS_DUMP` already prints makes this
authoritative:

> `clients/bevy-ref/src/scene_dump.rs:234-246` — "players==0 with actors>0
> means actors spawned as PRIMITIVE capsules (no GLTF scene = no
> AnimationPlayer), i.e. the model fallback fired and never got swapped to
> a SceneRoot. That is the real T-pose root cause."

So `CIVIS_DUMP=<path>` already reports `animation.players == 0` and
`animation.t_posed == <actor count>` whenever the model path is broken —
this is the existing programmatic verification we will use post-fix to
prove the rig is now playing a clip (see §4).

### 1.4 Building / decoration spawns have the same gap

`gltf_models::building_scene_for` (`clients/bevy-ref/src/gltf_models.rs:250-266`)
maps `BuildingType → slot` correctly, with passing tests
(`gltf_models.rs:373-407`). But the only caller of any model-spawn helper
is the (still primitive) `sync_visible_gameplay`. Real buildings (and trees,
rocks, carts) are similarly affected — a building type → model fix rides
along with the actor fix.

## 2. Root-cause summary

| Symptom | Mechanism | Evidence |
|---------|-----------|----------|
| Herd entities render as soldiers (humanoid) | `sim_bridge::sync_visible_gameplay` ignores `ActorVisual` and never calls `actor_scene`. All actors share a single capsule + faction tint. | `sim_bridge.rs:144-189` (no `actor_scene` call); `gltf_models.rs:235-240` (dispatch exists but unused) |
| All actors T-pose (stuck in bind pose) | `civilian.glb` has **0** animations; `herd.glb` has 13 but all prefixed `AnimalArmature\|…`; `animation::attach_actor_animation` alias lists do not match either rig. `GaitNodes::any()` is `false` → `is_static = true` → mesh frozen at scene-spawn pose. | `animation.rs:261-284` (alias lists); `animation.rs:139-141, 295-303` (static fallback); parsed glTF: `civilian.glb` animations=`[]`, `herd.glb` anims=`[AnimalArmature|Idle, …Walk, …Gallop, …]` |
| No `AnimationPlayer` exists at all | `SceneRoot` is never spawned (consequence of §1.1). `animation::attach_actor_animation` cannot run because no player entities exist. | `sim_bridge.rs:166-171` (no `SceneRoot`); `scene_dump.rs:234-246` (diagnostic comment) |

## 3. Fix plan

Three independent passes, in order. Each is small enough to land as a
single PR; #1 and #2 are prerequisites for #3 to be visible.

### 3.1 Wire the model-by-species/role dispatch (fixes "wrong model")

In `clients/bevy-ref/src/sim_bridge.rs::sync_visible_gameplay` (around
lines 151-172), change the civilian loop to:

1. Query `(&Civilian, &civ_agents::Position3d, &civ_agents::ActorVisual)`
   instead of dropping `ActorVisual`.
2. Resolve the visual handle via `gltf_models::actor_scene(&models,
   actor_visual.0, civilian.faction)` — `gltf_models.rs:235-240` already
   does the `Humanoid → civilian` and `Herd → herd` dispatch and is
   covered by the existing tests
   (`actor_scene_routes_humanoid_to_civilian_slot`,
   `actor_scene_routes_herd_to_herd_slot` — `gltf_models.rs:357-370`).
3. `match` on `ModelOrPrimitive`:
   - `Model(scene_root)` → `commands.spawn((SimCivilianMarker,
     scene_root, Transform::from_translation(world_pos)))`. Drop the
     `Mesh3d`/`MeshMaterial3d` pair for the model path. The model has its
     own materials.
   - `Primitive` → keep the existing capsule + faction-tinted material
     path (this is the explicit fallback the doc already promises,
     `gltf_models.rs:13-15`).
4. Do the symmetric change for buildings using
   `building_scene_for(&models, building.building_type)`
   (`gltf_models.rs:250-266`) — already keyed by
   `BuildingType`, already covered by tests
   (`building_scene_for_routes_each_type_to_its_slot` —
   `gltf_models.rs:373-407`).

Add a feature-gated import so the change is no-op when `models` is off:

```rust
#[cfg(feature = "models")]
use crate::gltf_models::{actor_scene, building_scene_for, GameModels, ModelOrPrimitive};
```

`SimBridgePlugin::build` already runs only when the `bevy` feature is on;
the `models` cfg keeps behaviour identical to today on the default build.

### 3.2 Fix animation binder to match the fetched rigs (fixes T-pose)

Two changes inside `clients/bevy-ref/src/animation.rs`:

**(a) Extend the alias lists to cover the Quaternius Farmer / Horse
vocabulary** (`animation.rs:261-284`):

```rust
idle: pick(&named, &[
    "Idle", "Unarmed_Idle", "2H_Melee_Idle", "Idle_B",
    // Quaternius Farmer / Animal packs (no '|Armature|' prefix in JSON,
    // Bevy's named_animations keeps the full GLTF animation.name string).
    "CharacterArmature|Idle", "AnimalArmature|Idle", "AnimalArmature|Idle_2",
]),
walk: pick(&named, &[
    "Walking_A", "Walking_B", "Walking_C", "Walking_D_Skeletons", "Walk",
    "CharacterArmature|Walking", "AnimalArmature|Walk",
]),
run: pick(&named, &[
    "Running_A", "Running_B", "Running_C", "Run",
    "CharacterArmature|Running", "AnimalArmature|Gallop",
]),
```

`gltf.named_animations` is a `HashMap<String, AnimationClip>` keyed by the
*raw* `animation.name` from the GLTF JSON, which for both Quaternius packs
is the full `<Armature>|<Clip>` string. The exact prefixes must be
verified against a real asset dump before merge (see §3.3 below) — the
parser above is the canonical place to add them.

**(b) For the `civilian.glb` "no animations" case, decide policy, do not
silently freeze.** Today the static fallback
(`animation.rs:292-304`) is correct in the sense that it does not panic,
but the rig has 0 clips, so the user sees a bind-pose Farmer standing
still. The fix in §3.1 (using `actor_scene` for civilians) means the rig
*is* spawned; the missing piece is "play something on it" even when the
rig has no clips. Two acceptable options, pick one and document:

- **A. Acknowledge the static model and move on.** The Farmer with no
  animations stays in bind pose (still *correct* — the user sees a real
  farmer, not a capsule, and not T-pose of a wrong rig). Add a
  `gltf_models::actor_scene` return for `Humanoid` to log a one-time
  `warn!` ("civilian.glb has 0 clips; static Farmer will idle in bind
  pose — TODO(FR-CIV-ASSETS): fetch a rigged Farmer").
- **B. Fetch a rigged CC0 humanoid** as part of §3.3. The Farmer is
  currently a static mesh; Quaternius' "Characters" packs include rigged
  variants (e.g. `Ultimate Animated Character Pack`). Replace the
  `civilian.glb` in `assets/models/PROVENANCE.txt` with one that has
  `Idle` / `Walking` clips, then the existing alias lists already cover
  it.

**Recommendation:** ship A now, schedule B as FR-CIV-ASSETS. The T-pose is
eliminated the moment the herd dispatch is fixed (herd.glb has 13
animations and the new aliases cover them); the civilian zero-clip case
is a separate, isolated, and well-named follow-up.

**(c) Make the alias lists data-driven, not hard-coded** (low-risk,
independent cleanup). Move the four `pick(...)` lists into a
`static GAIT_ALIASES: &[(&str, &[&str])]` const so adding a new rig is a
one-line change. Tests in `animation.rs:419-456` already cover the
degradation logic (`gait_degrades_to_available_clip`); add one test that
asserts `pick(&named, GAIT_ALIASES)["AnimalArmature|Idle"]` resolves.

### 3.3 Real-asset sourcing / placeholder-removal plan

Goal: every `.glb` referenced by `gltf_models::asset_paths` is the CC0 file
the loader expects, and `assets/models/README.md` +
`assets/models/PROVENANCE.txt` agree with `asset_paths` constants in
`gltf_models.rs:60-81`.

| File | Current state | Action |
|------|---------------|--------|
| `models/civilian.glb` (Quaternius Farmer) | Present, 0 animations, bind-pose static | (A) Land as-is + warn-log; (B) swap to rigged `Ultimate Animated Character Pack` Farmer in a follow-up; update PROVENANCE.txt and README.md. |
| `models/herd.glb` (Quaternius Horse) | Present, 13 clips, all `AnimalArmature\|…` | Land as-is after §3.2 aliases land. Verify in CI by parsing the JSON chunk and asserting the alias strings resolve. |
| `models/tree.glb` (KayKit tree_single_A) | Present, no anims (static OK) | Keep; no change. |
| `models/building.glb`, `building_house_B.glb`, `building_market.glb`, `building_church.glb`, `building_tavern.glb`, `models/building_tower.glb`, `models/building_well.glb` | Present, no anims (static OK) | Keep; verify routing in §3.1 tests. |
| `models/creature_skeleton_minion.glb`, `models/creature_skeleton_warrior.glb` | Present, **not yet routed** | Out of scope for #95; the `creature` slot is a future `ActorVisualKind::Creature(role)` variant (separate FR). |
| `models/cart.glb`, `models/cart_wheelbarrow.glb`, `models/rock*.glb`, `models/road.glb`, `models/tree_b.glb`, `models/tree_large.glb` | Present, **not yet routed** | Out of scope for #95; the README already lists these as "expanded variety, not yet referenced". |

**CI enforcement** (no new tooling — extend existing fetch script):

- `Tools/fetch-cc0-models.ps1` must (a) verify each `asset_paths` constant
  resolves to a file on disk, and (b) parse the JSON chunk of every
  rig-with-claimed-animations and assert the names exist in
  `gltf_models::animation::GAIT_ALIASES` (or in the `pick` lists). This
  catches "the alias said `AnimalArmature|Idle` but the file is actually
  named differently" without a runtime run.
- `docs/models/civ-sim/TECHNICAL_SPEC.md` already documents
  "single source of truth" for model paths. Link the new design doc
  (`civ-actor-assets-fix.md`) from the relevant `FR-CIV-3D-*` row in
  `docs/traceability/fr-3d-matrix.md` (one-line edit; not blocking).

## 4. Programmatic verification (CIVIS_DUMP)

The existing `CIVIS_DUMP=<path>` hook
(`clients/bevy-ref/src/scene_dump.rs:39-268`) is the right place. It
already serializes `animation.players` and `animation.playing`
(`scene_dump.rs:234-246`); it is the authoritative source for "is the
actor actually playing a clip", with one caveat from the file's own
header (line 21-25): "NOT AUTHORITATIVE for GLTF scene-graph
instantiation and therefore AnimationPlayer presence. In a headless dump
run (no window/GPU) Bevy does NOT instantiate GLTF SceneRoot child
hierarchies."

Use the **windowed** run, not the headless one:

```sh
CIVIS_DUMP=actors.json CIVIS_DUMP_KEEP_ALIVE=1 cargo run \
  -p civ-bevy-window --features bevy,models,egui
```

`CIVIS_DUMP_KEEP_ALIVE=1` (line 266 of `scene_dump.rs`) is exactly the
flag that runs WINDOWED with full GLTF scene instantiation while still
emitting the JSON. Once §3.1 and §3.2 land, `actors.json` must show:

```jsonc
"animation": {"players": <actor_count>, "playing": <actor_count>, "t_posed": 0}
```

Pre-fix the dump reads `players: 0, t_posed: <actor_count>` (every actor
frozen in bind pose). Post-fix the dump reads `players == actors, t_posed:
0`. Add a CI assertion (or a single `jq` check in
`scripts/agent-smoke.ps1` § "G3D0 visual pass") that
`animation.t_posed == 0` AND `animation.players > 0` after the
`civ-bevy-window` warmup; a regression here is a one-line bisect back to
the alias list or the dispatch table.

The dump does not currently surface **per-entity model handle** (it only
counts players and how many are playing). Extend
`dump_scene_system` (one small `Vec<(u64, String, String)>` loop,
~15 lines) to emit:

```jsonc
"actor_models": [
  {"id": 1, "kind": "Humanoid", "scene": "models/civilian.glb", "playing": true, "clip": "Idle"},
  {"id": 2, "kind": "Herd",     "scene": "models/herd.glb",     "playing": true, "clip": "AnimalArmature|Idle"},
  ...
]
```

That is the missing "wrong-model" probe: a verifier can read this list and
assert no `Herd` entity ever has a `Humanoid` scene, and no
`Humanoid` entity ever has a `Herd` scene. The same loop, joined to
`civ_agents::ActorVisual`, proves the §3.1 dispatch is correct end-to-end
without any pixel inspection.

## 5. Files touched by the fix (no edits in this doc)

- `clients/bevy-ref/src/sim_bridge.rs:144-189` — model-spawn dispatch (§3.1)
- `clients/bevy-ref/src/animation.rs:261-284` — alias lists (§3.2a)
- `clients/bevy-ref/src/animation.rs:73-92` (test surface) — alias coverage
  test (§3.2c)
- `clients/bevy-ref/src/scene_dump.rs:234-246` — per-entity model/clip
  probe (§4)
- `Tools/fetch-cc0-models.ps1` — alias-resolution CI check (§3.3)
- `clients/bevy-ref/assets/models/PROVENANCE.txt` and `README.md` —
  follow-up for rigged-civilian swap (§3.3, option B)

## 6. Out of scope

- Full FR-CIV-0700 (capability enforcement, mod store/publish, hot
  reload) — modding v3 is "partial–good" per AGENTS.md; not in this item.
- `creature_skeleton_minion.glb` / `creature_skeleton_warrior.glb`
  routing — needs a new `ActorVisualKind::Creature(role)` variant
  (follow-up FR).
- Quixel/Megascans asset import for buildings — product-only per
  AGENTS.md, not agent-blocking.
- L5 visual pass (lighting, post-fx) — separate
  `docs/development-guide/fr-l5-visual-pass.md` track.
