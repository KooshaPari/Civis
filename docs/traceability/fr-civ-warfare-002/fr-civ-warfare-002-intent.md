# Intent: FR-CIV-WARFARE-002 -- Doctrine evolution from battle outcomes

> Date: 2026-09-19
> FR: FR-CIV-WARFARE-002
> Epic: FR-CIV-WARFARE

## User Intent

Faction combat doctrine is not hand-authored — it **evolves** from the
real battle outcomes the simulation produces. Each faction rolls its
`DoctrineLibrary` forward one GA generation per scoring cycle, using
accumulated shooter/target/voxels-removed counts as fitness signal.

### What This FR Achieves

`crates/tactics/src/doctrine_evolution.rs` exposes:

- `accumulate_faction_stats(faction_id, engagements)`
  -> `FactionEngagementStats { engagements_as_shooter, engagements_as_target, voxels_removed }`.
- `evolve_doctrine_from_battle(library, faction_id, engagements, rng, generation_index)`
  -> runs a full score+evolve cycle: accumulates stats → re-scores every
  doctrine candidate with `score_doctrine_fitness` → advances one GA
  generation via `evolve_doctrine` using a `ChaCha8Rng` seeded from
  `(faction_id, generation_index)`.

## Acceptance Signal

- `cargo test -p tactics doctrine_evolution` passes.
- Same `(faction_id, generation_index)` seed → identical evolved library.
- `voxels_removed` is proportional to the engagement `radius_voxels`
  (blast-radius heuristic).

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/tactics/` |
| Module | `crates/tactics/src/doctrine_evolution.rs:1` |
