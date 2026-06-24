# FR-CIV-PLANET-040 Geology Seed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic geology seed layer to `civ-planet` that maps planet config to a fixed set of biome regions and surfaces that map on the engine's `SimulationSnapshot`.

**Architecture:** `GeologyMap::seed(&PlanetConfig)` derives `Vec<RegionBiome>` from config fields alone — no tick, no RNG, pure integer arithmetic — so the result is identical for every call with the same input. The engine stores a `GeologyMap` field on `SimulationSnapshot` and derives it once per `snapshot()` call. No dependency inversion is needed because `civ-engine` already depends on `civ-planet`.

**Tech Stack:** Rust stable (workspace edition), `serde` derive, `proptest` for property tests, `cargo test`.

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Create | `crates/planet/src/geology.rs` | `BiomeKind`, `RegionBiome`, `GeologyMap`, `GeologyMap::seed` + `geology_map_is_stable_for_same_planet_config` test |
| Modify | `crates/planet/src/lib.rs` | Add `pub mod geology; pub use geology::{GeologyMap, RegionBiome, BiomeKind};` |
| Modify | `crates/engine/src/engine.rs` | Import `GeologyMap`, add `geology_map: GeologyMap` to `SimulationSnapshot`, populate in `snapshot()` |
| Modify | `crates/engine/src/lib.rs` | Re-export `GeologyMap, RegionBiome, BiomeKind` from `civ_planet` |

---

### Task 1: Write `geology.rs` with failing test first

**Files:**
- Create: `crates/planet/src/geology.rs`

- [ ] **Step 1: Write the failing test skeleton in geology.rs**

  Create `crates/planet/src/geology.rs` with only the test (no impl yet):

  ```rust
  //! FR-CIV-PLANET-040 — deterministic geology seed layer.
  //!
  //! `GeologyMap::seed` is purely config-derived, produces a stable `Vec<RegionBiome>`
  //! for every call with the same `PlanetConfig`, and never touches tick or RNG.

  #![forbid(unsafe_code)]

  use serde::{Deserialize, Serialize};

  use crate::PlanetConfig;

  /// The six canonical biome archetypes for a planet region.
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum BiomeKind {
      /// Open water — radius-derived; large planets have proportionally more ocean.
      Ocean,
      /// Flat grassland and savanna.
      Plains,
      /// Temperate and tropical forest.
      Forest,
      /// High-altitude or tectonic uplift terrain.
      Mountain,
      /// Arid low-humidity terrain.
      Desert,
      /// Cold polar or high-altitude terrain.
      Tundra,
  }

  /// A single region's biome assignment.
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub struct RegionBiome {
      /// Stable integer identifier for this region (0-based).
      pub region_id: u32,
      /// Assigned biome archetype.
      pub biome: BiomeKind,
  }

  /// Deterministic planet-wide geology map derived from [`PlanetConfig`].
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct GeologyMap {
      /// One entry per region; length == `num_regions` passed to `seed`.
      pub regions: Vec<RegionBiome>,
  }

  impl GeologyMap {
      /// Derive a deterministic geology map from `planet_config`.
      ///
      /// # Determinism guarantee
      /// The result depends only on `planet_config` fields; no external state,
      /// no RNG, no tick. Identical inputs always produce identical outputs.
      ///
      /// # Region count
      /// Uses a fixed grid of 16 canonical regions, each assigned a biome based
      /// on its normalised latitude and `radius_km` / `axial_tilt_deg`.
      ///
      /// # Biome model (all integer arithmetic)
      /// Each region `r` in `0..16` is assigned a latitude band in `[-8, +8]`.
      /// - `|band| >= 7` → Tundra (poles)
      /// - `|band| >= 5` → Mountain (sub-polar uplift)
      /// - Ocean fraction = `radius_km / 6_371` clamped to [0, 8] — regions with
      ///   `r < ocean_regions` get Ocean.
      /// - Remaining equatorial band: Desert when `axial_tilt_deg < 10`, Forest
      ///   when `axial_tilt_deg > 30`, else Plains (temperate).
      pub fn seed(planet_config: &PlanetConfig) -> GeologyMap {
          todo!("implement in Task 2")
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::{defaults_earthlike, PlanetConfig};

      /// FR-CIV-PLANET-040 — same PlanetConfig always produces a bit-identical GeologyMap.
      #[test]
      fn geology_map_is_stable_for_same_planet_config() {
          let (planet, _) = defaults_earthlike();
          let a = GeologyMap::seed(&planet);
          let b = GeologyMap::seed(&planet);
          assert_eq!(a, b, "GeologyMap must be identical across two calls with the same PlanetConfig");

          // Sensitivity: changing radius_km by 1 km must produce a different map
          // (ocean fraction changes, verifying config is actually consumed).
          let mut tweaked = planet;
          tweaked.radius_km = planet.radius_km + 1_000;
          let c = GeologyMap::seed(&tweaked);
          // The tweak shifts ocean_regions by at least 1; maps must differ.
          assert_ne!(
              a.regions.iter().map(|r| r.biome as u8).collect::<Vec<_>>(),
              c.regions.iter().map(|r| r.biome as u8).collect::<Vec<_>>(),
              "radius_km delta of 1000 km must change the geology map"
          );
      }
  }
  ```

- [ ] **Step 2: Add `pub mod geology` to lib.rs so it compiles (even as stub)**

  In `crates/planet/src/lib.rs`, after `pub mod weather;`, add:

  ```rust
  pub mod geology;
  pub use geology::{BiomeKind, GeologyMap, RegionBiome};
  ```

- [ ] **Step 3: Run the test to confirm it panics at `todo!`**

  ```
  cargo test -p civ-planet geology_map_is_stable_for_same_planet_config 2>&1
  ```

  Expected: FAIL — `not yet implemented: implement in Task 2`

---

### Task 2: Implement `GeologyMap::seed`

**Files:**
- Modify: `crates/planet/src/geology.rs` — replace `todo!` body with working implementation

- [ ] **Step 1: Replace the `todo!` with the integer-only implementation**

  Replace the body of `GeologyMap::seed`:

  ```rust
  pub fn seed(planet_config: &PlanetConfig) -> GeologyMap {
      const NUM_REGIONS: u32 = 16;
      // Ocean fraction: earth-sized planet (6_371 km) → 8 ocean regions out of 16.
      // Scale linearly; clamp to [0, NUM_REGIONS].
      let ocean_regions = ((planet_config.radius_km as u64 * 8) / 6_371)
          .min(NUM_REGIONS as u64) as u32;

      let mut regions = Vec::with_capacity(NUM_REGIONS as usize);
      for r in 0..NUM_REGIONS {
          // Latitude band in [-8, +8], centre-mapped from region index.
          // r=0 → band=-8 (south pole), r=15 → band=+8 (north pole)
          let band: i32 = -8 + (r as i32 * 16 / (NUM_REGIONS as i32 - 1));

          let biome = if r < ocean_regions {
              BiomeKind::Ocean
          } else if band.unsigned_abs() >= 7 {
              BiomeKind::Tundra
          } else if band.unsigned_abs() >= 5 {
              BiomeKind::Mountain
          } else if planet_config.axial_tilt_deg < 10 {
              BiomeKind::Desert
          } else if planet_config.axial_tilt_deg > 30 {
              BiomeKind::Forest
          } else {
              BiomeKind::Plains
          };

          regions.push(RegionBiome { region_id: r, biome });
      }

      GeologyMap { regions }
  }
  ```

- [ ] **Step 2: Run the test to confirm it passes**

  ```
  cargo test -p civ-planet geology_map_is_stable_for_same_planet_config 2>&1
  ```

  Expected: PASS — `test geology::tests::geology_map_is_stable_for_same_planet_config ... ok`

- [ ] **Step 3: Run all planet tests to check no regressions**

  ```
  cargo test -p civ-planet 2>&1
  ```

  Expected: all tests pass.

- [ ] **Step 4: Commit Task 1+2**

  ```bash
  git -C C:/Users/koosh/Dev/Civis add crates/planet/src/geology.rs crates/planet/src/lib.rs
  git -C C:/Users/koosh/Dev/Civis commit -m "feat(planet): FR-CIV-PLANET-040 add GeologyMap deterministic seed"
  ```

---

### Task 3: Surface `GeologyMap` on `SimulationSnapshot`

**Files:**
- Modify: `crates/engine/src/engine.rs` — add `geology_map` field + populate in `snapshot()`
- Modify: `crates/engine/src/lib.rs` — re-export `GeologyMap, RegionBiome, BiomeKind`

- [ ] **Step 1: Extend the `use civ_planet` import in `engine.rs`**

  In `crates/engine/src/engine.rs`, change:

  ```rust
  use civ_planet::{
      compute_climate, compute_weather, defaults_earthlike, Climate, MoonConfig, PlanetConfig,
      WeatherCell,
  };
  ```

  to:

  ```rust
  use civ_planet::{
      compute_climate, compute_weather, defaults_earthlike, BiomeKind, Climate, GeologyMap,
      MoonConfig, PlanetConfig, RegionBiome, WeatherCell,
  };
  ```

- [ ] **Step 2: Add `geology_map` field to `SimulationSnapshot`**

  In `crates/engine/src/engine.rs`, after the `weather_grid` field in `SimulationSnapshot`:

  ```rust
  /// Deterministic geology map for the planet (FR-CIV-PLANET-040).
  ///
  /// Derived from `PlanetConfig` alone; identical for every tick of the same planet.
  pub geology_map: GeologyMap,
  ```

- [ ] **Step 3: Populate `geology_map` in `snapshot()`**

  In `crates/engine/src/engine.rs`, inside the `SimulationSnapshot { ... }` literal returned by `snapshot()`, after `weather_grid: ...`, add:

  ```rust
  geology_map: GeologyMap::seed(&self.planet),
  ```

- [ ] **Step 4: Re-export from `crates/engine/src/lib.rs`**

  In `crates/engine/src/lib.rs`, after the existing `pub use civ_planet::{Climate, MoonConfig, PlanetConfig};` line, add:

  ```rust
  pub use civ_planet::{BiomeKind, GeologyMap, RegionBiome};
  ```

- [ ] **Step 5: Build to confirm no errors**

  ```
  cargo build -p civ-engine 2>&1
  ```

  Expected: `Finished` with no errors.

- [ ] **Step 6: Run engine tests to confirm no regressions**

  ```
  cargo test -p civ-engine 2>&1
  ```

  Expected: all tests pass.

- [ ] **Step 7: Run full workspace build**

  ```
  cargo build 2>&1
  ```

  Expected: `Finished` with no errors.

- [ ] **Step 8: Run full workspace tests**

  ```
  cargo test 2>&1
  ```

  Expected: all tests pass.

- [ ] **Step 9: Commit Task 3**

  ```bash
  git -C C:/Users/koosh/Dev/Civis add crates/engine/src/engine.rs crates/engine/src/lib.rs
  git -C C:/Users/koosh/Dev/Civis commit -m "feat(engine): surface GeologyMap on SimulationSnapshot (FR-CIV-PLANET-040)"
  ```

---

### Task 4: Branch, push, create PR

**Files:** none (git operations only)

- [ ] **Step 1: Create and checkout the feature branch**

  ```bash
  git -C C:/Users/koosh/Dev/Civis checkout -b feat/p-p1-fr040-geology
  ```

  > Note: if commits were made on an existing branch, `git branch -m` to rename, or cherry-pick onto a fresh branch from main.

- [ ] **Step 2: Push the branch**

  ```bash
  git -C C:/Users/koosh/Dev/Civis push -u origin feat/p-p1-fr040-geology
  ```

- [ ] **Step 3: Create the PR via `gh`**

  ```bash
  gh pr create \
    --repo KooshaPari/Civis \
    --head feat/p-p1-fr040-geology \
    --base main \
    --title "feat(planet): FR-CIV-PLANET-040 geology seed" \
    --body "## Summary
  - Adds \`GeologyMap\`, \`RegionBiome\`, \`BiomeKind\` to \`civ-planet\` with deterministic \`GeologyMap::seed(&PlanetConfig)\`
  - Surfaces \`geology_map: GeologyMap\` on \`SimulationSnapshot\` in \`civ-engine\`
  - Re-exports new types from \`civ-engine\` public API
  - Adds \`geology_map_is_stable_for_same_planet_config\` test with radius_km sensitivity check

  ## Spec coverage
  FR-CIV-PLANET-040 — geology seed layer, config-derived, deterministic, no tick dependency.

  ## Test plan
  - [x] \`cargo test -p civ-planet\` — all tests including new geology stability test
  - [x] \`cargo test -p civ-engine\` — no regressions on snapshot/engine tests
  - [x] \`cargo build\` — full workspace clean build"
  ```

---

## Cross-Project Reuse Opportunities

| Candidate | Shared Location | Impacted Repos | Notes |
|-----------|----------------|----------------|-------|
| `BiomeKind` enum | `phenotype-voxel` or a new `phenotype-biomes` crate | Civis, future WorldSphereMod integration | Extract when a second consumer appears (abstraction-at-2-uses rule) |
| `GeologyMap::seed` integer model | `phenotype-voxel` kernel | Same | Only extract when the voxel kernel needs biome-aware meshing |

No extraction warranted yet — only one consumer exists.
