# Live Entity-Archetype Survey — #986 (2026-05-31)

Source: live `game-control entities <Component>` queries against the running game
(PID 4412 then a freshly deployed instance), bridge `queryEntities` returns the full
component-type list for up to 100 matching entities. Active skirmish: ~56,500 entities.

## The bug #975 introduced

The #975 archetype guesses for cims/buildings/aerial returned 0 RenderMesh-bearing
entities, so the `RenderMesh AND <archetype>` swap query matched nothing →
`batch complete — 0 succeeded, 30 failed`.

## Ground truth (live counts)

| Probe component | Live count | RenderMesh on same entity? | Verdict |
|---|---|---|---|
| `Components.MeleeUnit` | 4801 | YES (direct) | ground units — already swap |
| `Components.RangeUnit` | 170 | YES (direct) | ground units — already swap |
| `Components.CavalryUnit` | 785 | YES (direct) | renderable |
| `Components.SiegeUnit` | 162 | YES (direct) | renderable — chosen for aerial reskin |
| `Components.Unit` | 4971 | YES | generic unit tag |
| `Components.Worker` | **2** | n/a | WRONG cim guess (#975) — singletons, not cims |
| `Components.Citizen` | **58** | **YES (direct)** | **the real cim archetype** |
| `Components.BuildingBase` | 575 | **NO** | parent only; RenderMesh on CHILD |
| `DINOForge.Runtime.Aviation.AerialUnitComponent` | **0** | n/a | never tagged by DINO (no native fliers) |

### Why ground units swap but buildings did not

A `MeleeUnit`/`RangeUnit`/`Citizen` entity carries `Unity.Rendering.RenderMesh`
**on the same entity** as the class marker, so `RenderMesh AND MeleeUnit` matches.

A `BuildingBase` entity's archetype is:

```
Components.HealthBarReference, Components.Selectable, LocalToWorld, Rotation,
Translation, ArmorData, BuildingBase, Enemy, Health, HealthBase, PriceBase,
ObjectId, Orientation, BuildingCacheState,
Unity.Entities.LinkedEntityGroup,        <-- child references
Unity.Transforms.Child                   <-- this is a parent
```

There is **no RenderMesh on the building parent** — it lives on a child entity
referenced through `LinkedEntityGroup`. So `RenderMesh AND BuildingBase` = 0 matches.

## Fixes (verified live)

1. **cims** → `Components.Citizen` (was `Components.Worker`). Live result:
   `sw-cis-droid-pilot` swapped **51/51**, `result=True`.
2. **buildings** → keep `Components.BuildingBase` archetype but add a
   `LinkedEntityGroup` child-walk (`CollectRenderMeshChildren`) that swaps the
   RenderMesh-bearing children. Live result: `sw-cis-command-center`,
   `sw-cis-droid-factory`, `sw-clone-barracks`, `sw-rep-vehicle-bay`, etc. each
   `swapped 100/100` (resolved 17,579 RenderMesh child entities).
3. **aerial_fighter** → `Components.SiegeUnit` (was non-existent
   `AerialUnitComponent`). DINO has no native fliers, so SW aircraft reskin onto a
   real renderable archetype distinct from infantry.

## Live before/after

- Before (build 1BDC999C): `batch complete — 0 succeeded, 30 failed`.
- After (this fix): warfare-starwars pass `batch complete — 39 succeeded, 13 failed`.
  The 13 remaining failures are 90-byte STUB bundles (warfare-aerial pack +
  `sw-rep-clone-engineer`) with no usable mesh — a pre-existing asset-pipeline gap
  (#101), NOT an archetype-resolution failure. Every REAL bundle now swaps.

## Note on the mesh-substring "refine" log line

cims/buildings still emit a `→ refining by mesh substrings` log line, but
`AssetSwapSystem` nulls `targetMeshSubstrings` whenever an archetype filter is
present (the substrings were hand-guessed and would wrongly reject all entities).
The log is cosmetic; the swap is archetype-authoritative, proven by the 51/51 and
100/100 counts.
