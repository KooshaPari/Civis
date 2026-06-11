# Asset Gaps — #95 Wrong Actor Models

**Date:** 2026-06-10  
**Branch:** wip/asset-audit  
**Issue:** Civilians render as armoured knights; Herd actors render as undead skeletons.

---

## Model Inventory

| File | Actual Identity | Animation clips (locomotion relevant) | Suitable for |
|------|----------------|---------------------------------------|-------------|
| `civilian.glb` | KayKit Adventures **Knight** — armoured humanoid w/ sword/shield | `Idle`, `Unarmed_Idle`, `Walking_A/B/C`, `Running_A/B` | ❌ Civilian (wrong visual) / ✅ Soldier/Guard |
| `creature_skeleton_minion.glb` | KayKit Skeletons **Skeleton Minion** — undead skeleton | `Idle`, `Idle_B`, `Walking_A/B/C/D_Skeletons`, `Running_A/B/C` | ❌ Herd (wrong identity) / ✅ Undead enemy |
| `creature_skeleton_warrior.glb` | KayKit Skeletons **Skeleton Warrior** — heavier undead skeleton | Same clip set as minion + `Walking_D_Skeletons` | ❌ Herd (wrong identity) / ✅ Undead enemy |
| `building*.glb` | KayKit Hexagon buildings (house, tower, church, market, tavern, well) | Static (no animations) | ✅ Correct |
| `tree*.glb`, `rock*.glb`, `road.glb` | KayKit Hexagon decorations | Static | ✅ Correct |
| `cart.glb` | Khronos ToyCar (glTF sample asset) | Static | Prop use only |
| `cart_wheelbarrow.glb` | KayKit Hexagon wheelbarrow | Static | Prop use only |

### Animation clip names — current models vs. animation.rs expectations

`animation.rs` `pick()` searches (in priority order):

| Gait slot | Searched names | Found in `civilian.glb`? | Found in `creature_skeleton_minion.glb`? |
|-----------|---------------|--------------------------|------------------------------------------|
| Idle | `Idle`, `Unarmed_Idle`, `2H_Melee_Idle`, `Idle_B` | ✅ `Idle` | ✅ `Idle` |
| Walk | `Walking_A`, `Walking_B`, `Walking_C`, `Walking_D_Skeletons` | ✅ `Walking_A` | ✅ `Walking_A` |
| Run  | `Running_A`, `Running_B`, `Running_C` | ✅ `Running_A` | ✅ `Running_A` |

**Conclusion:** animation.rs resolves clips correctly for **both** current models. The bug is **purely visual identity** — not a code defect. No existing in-repo model is a townsperson or a quadruped animal.

---

## What Is Needed

### 1. CIVILIAN — Low-poly townsperson / villager (humanoid)

**Requirements:**
- Visually: plain civilian clothing, no weapon, no armour. Examples: farmer, merchant, peasant, villager.
- Rig: humanoid biped, skeletal animation.
- Required clip names (animation.rs will pick the first match):
  - Idle → clip named **`Idle`** or `Unarmed_Idle`
  - Walk → clip named **`Walking_A`** (preferred) or `Walking_B`/`Walking_C`
  - Run  → clip named **`Running_A`** (preferred) or `Running_B`/`Running_C`
- If the source model uses different names, a thin rename pass is needed (edit GLB via Blender or `gltf-transform rename`) **or** add the model's actual names to the `pick()` candidate lists in `animation.rs` (one-line addition per name).
- License: CC0 / public domain.

**Recommended free sources (CC0):**

| Source | Pack | Notes |
|--------|------|-------|
| **Quaternius** | [Ultimate Platformer Pack](https://quaternius.com/packs/ultimateplatformer.html) — `Man_Farmer`, `Man_Merchant` | Correct clip names (`Idle`, `Walking_A`, `Running_A`); same rig family as KayKit |
| **Quaternius** | [Animated Humans](https://quaternius.com/packs/animatedhumans.html) | Separate male/female villager; same clip scheme |
| **KayKit** | [City Characters](https://kaylousberg.itch.io/kaykit-city-characters) | Same studio as current assets; clip naming matches existing pick() lists |
| **Kenney** | [City Kit (Characters)](https://kenney.nl/assets/city-characters) | Low-poly; static (no skeletal animation) — usable only as static mesh fallback |

**Recommended pick:** Quaternius "Animated Humans" farmer/merchant (`Man_Farmer.glb`) — CC0, correct clip names, publicly available at <https://quaternius.com/packs/animatedhumans.html>. Drop as `clients/bevy-ref/assets/models/civilian.glb` (replace current Knight).

---

### 2. HERD — Quadruped animal (non-humanoid fauna)

**Requirements:**
- Visually: a farm or wild animal — sheep, cow, horse, deer, or similar. Must read clearly as "fauna/herd", not a creature or enemy.
- Rig: quadruped or similar non-humanoid skeletal rig.
- Locomotion clips: any recognisable idle + walk names. animation.rs will fall back gracefully to whichever gait clips exist. The `pick()` priority list covers `Idle`, `Walking_A/B/C`, `Running_A/B/C` — at least **`Idle`** must be present to avoid `is_static = true`.
- If the model uses names like `Idle_Sheep` or `Walk`, add those to the `pick()` candidate lists in `animation.rs` (the only code change needed for Herd).
- License: CC0 / public domain.

**Recommended free sources (CC0):**

| Source | Pack | Specific model | Notes |
|--------|------|---------------|-------|
| **Quaternius** | [Animated Animals](https://quaternius.com/packs/animatedanimals.html) | `Sheep.glb`, `Cow.glb`, `Horse.glb` | CC0; clips named `Idle`, `Walk`, `Run` (need one-line addition to `pick()` or a rename) |
| **Quaternius** | [Farm Animals](https://quaternius.com/packs/farmanimals.html) | `Chicken.glb`, `Pig.glb` | CC0; same clip naming as above |
| **KayKit** | [Hexagon pack extras](https://kaylousberg.itch.io/) | No quadruped in current KayKit CC0 packs | Not available |
| **Kenney** | [Animal Pack Deluxe](https://kenney.nl/assets/animal-pack-deluxe) | Various | Static meshes only (no skeletal anim) |

**Recommended pick:** Quaternius `Sheep.glb` from Animated Animals (<https://quaternius.com/packs/animatedanimals.html>) — CC0, quadruped, skeletal.  
Clip name delta: Quaternius animals use `Idle`, `Walk`, `Run` (not `Walking_A`). Add to `animation.rs` `pick()`:
```
walk: pick(&named, &["Walking_A", "Walking_B", "Walking_C", "Walking_D_Skeletons", "Walk"]),
run:  pick(&named, &["Running_A", "Running_B", "Running_C", "Run"]),
```
(Two one-line edits in `animation.rs` — out of scope for this worker per task constraints; noted here for the implementer.)

Drop the chosen file as `clients/bevy-ref/assets/models/herd.glb` (new path) and update `asset_paths::HERD` in `gltf_models.rs` to `"models/herd.glb"`.

---

## Code Change Required in gltf_models.rs (once assets are sourced)

```rust
// In asset_paths module:
pub const CIVILIAN: &str = "models/civilian.glb";   // replace with townsfolk model
pub const HERD: &str = "models/herd.glb";           // replace creature_skeleton_minion
```

The `CIVILIAN` constant can keep the same filename if the new townsperson model is dropped as `civilian.glb` (replacing the Knight). The `HERD` constant should point to a new `herd.glb` to avoid confusion — the skeleton models should be kept for future Undead/Enemy actor kinds.

---

## animation.rs Compatibility Notes

No changes to `animation.rs` are needed if the replacement models carry `Idle` / `Walking_A` / `Running_A` clip names (Quaternius Animated Humans does). If Quaternius animal clips use `Walk`/`Run` (not `Walking_A`/`Running_A`), two candidate strings must be appended to the `pick()` calls for `walk` and `run` in `attach_actor_animation`. This is a 2-line change in `animation.rs`.

The `animation.rs` static fallback (`is_static = true`) will silently kick in for any model with no matching locomotion clips — no panic, but the actor will be frozen. The `Idle` clip must be present.

---

## Summary

- No suitable civilian (townsperson) or herd (quadruped animal) model exists in-repo.
- Both current mis-assigned models (`civilian.glb` = Knight, `creature_skeleton_minion.glb` = Skeleton) animate correctly per `animation.rs` — the bug is visual identity only.
- Sourcing two CC0 models from Quaternius resolves #95 without any model authoring.
- No `gltf_models.rs` path const change is made in this branch (the new asset files do not yet exist on disk); the consts will be updated once the sourced `.glb` files are committed.
