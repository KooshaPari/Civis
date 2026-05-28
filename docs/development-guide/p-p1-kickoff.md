# P-P1 planet + moon — kickoff

**Phase:** P-P1 (`crates/planet`)
**Depends on:** P-V0 (voxel substrate), P-W1 engine tick plumbing
**Branch suggestion:** `feat/p-p1-planet` off `main` after P-W1 items land

## Already wired

| Link | Location |
|------|----------|
| `PlanetConfig` + `MoonConfig` structs | `crates/planet/src/lib.rs` |
| `compute_climate(tick, planet, moon)` pure fn | `crates/planet/src/lib.rs` |
| `is_daytime` helper | `crates/planet/src/lib.rs` |
| `defaults_earthlike()` factory | `crates/planet/src/lib.rs` |
| Replay schema version stub | `SCHEMA_VERSION = "0.1.0-stub"` |
| FR-CIV-PLANET-000..005 tests | `crates/planet/src/lib.rs` `#[cfg(test)]` block |
| FR matrix entry | `docs/traceability/fr-3d-matrix.md` §Planet |

## FR status (`docs/traceability/fr-3d-matrix.md`)

| FR ID | Title | Status | Next step |
|-------|-------|--------|-----------|
| FR-CIV-PLANET-000 | Schema version stub | implemented | — |
| FR-CIV-PLANET-001 | Day/night cycle deterministic | implemented | — |
| FR-CIV-PLANET-002 | Moon tides deterministic | implemented | — |
| FR-CIV-PLANET-003 | `is_daytime` window | implemented | — |
| FR-CIV-PLANET-004 | Earthlike defaults sane | implemented | — |
| FR-CIV-PLANET-005 | Climate bit-identical across calls | implemented | — |
| FR-CIV-PLANET-010 | Engine integration — `ClimateFrame` fed into engine tick | not started | Wire `Climate` into `crates/engine` tick; expose via snapshot |
| FR-CIV-PLANET-020 | Tide-voxel coupling — coastal voxel height modulated by `tide_offset` | not started | `apply_tide_offset` in voxel layer; test sea-level delta |
| FR-CIV-PLANET-030 | Weather grid — per-cell temperature/precipitation derived from `year_phase` + axial tilt | not started | `WeatherCell` struct; `tick_weather_grid` pure fn |
| FR-CIV-PLANET-040 | Geology seed — stable per-cell rock type seeded from `PlanetConfig.radius_km` | not started | `GeologyMap::seed` deterministic from planet params |
| FR-CIV-PLANET-050 | Protocol `ClimateFrame` — serialisable snapshot emitted on replay bus each tick | not started | Add `ReplayEvent::Climate(ClimateFrame)` to `crates/engine/src/replay.rs` |
| FR-CIV-PLANET-060 | Replay hash — `ClimateFrame` fields included in per-tick hash chain | not started | Fold `ClimateFrame` bytes into existing `ReplayHashChain` |

## Test-first approach per FR

Tests must be written **before** implementation; each item below is the acceptance test that gates the PR slice.

### FR-CIV-PLANET-010 — engine integration

**Test name:** `engine_tick_includes_climate_in_snapshot`

Acceptance criteria:
- Given a `Simulation` constructed with `defaults_earthlike()` planet/moon config
- When `engine.tick()` is called N times
- Then `sim.snapshot().climate` is `Some(Climate { tick: N, .. })` with `day_phase` matching `compute_climate(N, ..)` exactly
- Bit-identity: snapshot climate equals a standalone `compute_climate` call for the same tick

Crate under test: `civ-engine`; no network or I/O.

### FR-CIV-PLANET-020 — tide-voxel coupling

**Test name:** `tide_offset_shifts_coastal_voxel_height`

Acceptance criteria:
- Given a flat coastal voxel column at sea-level baseline `h0`
- When `apply_tide_offset(column, tide_offset)` is called with a positive `tide_offset`
- Then the column's effective water height increases by `tide_offset` (within float tolerance 1e-4)
- When called with negative `tide_offset` the height decreases symmetrically
- Zero `tide_offset` leaves column unchanged (bit-identical to baseline)

Crate under test: `civ-planet` or `civ-voxel` (whichever owns the voxel height API).

### FR-CIV-PLANET-030 — weather grid

**Test name:** `weather_grid_temperature_varies_with_year_phase`

Acceptance criteria:
- Given a `WeatherGrid` seeded from `defaults_earthlike()` planet config
- When `tick_weather_grid(&mut grid, &climate)` is called at `year_phase = 0.0` (northern winter)
- And again at `year_phase = 0.5` (northern summer)
- Then the equatorial cell temperature at summer exceeds winter temperature (tilt-driven delta > 0)
- Grid output is deterministic: same `Climate` input yields bit-identical `WeatherCell` values

Crate under test: `civ-planet`.

### FR-CIV-PLANET-040 — geology seed

**Test name:** `geology_map_is_stable_for_same_planet_config`

Acceptance criteria:
- Given `PlanetConfig { radius_km: 6371, .. }` (earthlike)
- When `GeologyMap::seed(&planet)` is called twice independently
- Then both maps are structurally identical: same `RockType` at every grid coordinate
- When `radius_km` changes by 1 the resulting map differs in at least one cell (sensitivity check)
- No runtime state or RNG re-seed must be required; derivation is purely from config fields

Crate under test: `civ-planet`.

### FR-CIV-PLANET-050 — protocol `ClimateFrame`

**Test name:** `climate_replay_event_round_trips_via_bincode`

Acceptance criteria:
- Given a `ReplayEvent::Climate(ClimateFrame { tick: 42, .. })` constructed from `compute_climate(42, ..)`
- When serialised with `bincode` and deserialised back
- Then the resulting `ClimateFrame` fields are bit-identical to the original
- `ClimateFrame` implements `PartialEq` so equality is assertable in tests

Crate under test: `civ-engine` (replay module).

### FR-CIV-PLANET-060 — replay hash

**Test name:** `replay_hash_chain_differs_when_climate_changes`

Acceptance criteria:
- Given a `ReplayHashChain` built from two tick sequences that are identical except one has a different `tide_offset`
- Then the final hash digest of the two chains differs
- Conversely: two chains with identical `ClimateFrame` sequences produce identical digests
- Hash must remain stable across process restarts (no timestamp or entropy injection)

Crate under test: `civ-engine` (replay hash module).

## First PR slice (recommended)

The slices are ordered by dependency; each one is independently mergeable.

1. **Test:** `FR-CIV-PLANET-010` acceptance test (red) — engine snapshot carries `climate`.
2. **Impl:** Wire `compute_climate` into `engine::tick`; expose on `Snapshot` — turn test green.
3. **Test:** `FR-CIV-PLANET-050` acceptance test — `ClimateFrame` round-trips via bincode.
4. **Impl:** Add `ReplayEvent::Climate` variant + `ClimateFrame` struct to `crates/engine/src/replay.rs`.
5. **Test:** `FR-CIV-PLANET-060` acceptance test — hash chain diverges on climate delta.
6. **Impl:** Fold `ClimateFrame` bytes into `ReplayHashChain`.
7. **Test:** `FR-CIV-PLANET-020` acceptance test — tide offset shifts voxel height.
8. **Impl:** `apply_tide_offset` in the voxel layer; integrate with engine tick.
9. **Test:** `FR-CIV-PLANET-030` acceptance test — weather grid temperature varies with year phase.
10. **Impl:** `WeatherCell` + `tick_weather_grid` pure functions in `crates/planet`.
11. **Test:** `FR-CIV-PLANET-040` acceptance test — geology map stable for same config.
12. **Impl:** `GeologyMap::seed` deterministic seeding from `PlanetConfig`.

## Cross-project reuse opportunities

| Candidate | Target shared location | Impacted repos | Notes |
|-----------|----------------------|----------------|-------|
| `ClimateFrame` serialisation schema | `phenotype-voxel` or a new `civ-protocol` crate | Civis, any future planet-bearing sim | Keep replay protocol types in a protocol crate so Bevy client and server share the same bincode layout |
| `WeatherCell` grid primitive | `phenotype-voxel` voxel substrate | Civis | Weather cells map 1:1 to voxel columns; same grid addressing |
| Deterministic PRNG seeding pattern used in `GeologyMap::seed` | Phenotype org shared utility | All deterministic sim repos | Reusable `seed_from_config` helper avoids duplicate `splitmix64`/`xxhash` wrappers |

## Run

```
cargo test -p civ-planet
cargo test -p civ-engine climate
cargo check -p civ-engine
```
