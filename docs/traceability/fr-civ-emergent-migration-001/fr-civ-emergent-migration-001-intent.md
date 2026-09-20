# Intent: FR-CIV-EMERGENT-MIGRATION-001 — Cities grow from migration

> FR: FR-CIV-EMERGENT-MIGRATION-001
> Epic: FR-CIV-EMERGENT-MIGRATION
> // Covers: FR-CIV-EMERGENT-MIGRATION-001

## Requirement

Agents evaluate settlement quality and relocate when conditions are
better elsewhere, producing organic population shifts — prosperous
cities grow, starving settlements hollow out.

Each tick, for every agent with `migration_eligible == true`:

1. Compute `home_pressure(home_settlement)` — weighted combination of
   food_per_capita, safety, labor_opportunity, and social_bonds.
2. Sample K random candidate settlements (configurable `sample_size`).
3. For each candidate, compute `candidate_pull(candidate)`.
4. If the best candidate's pull > `home_pressure × migration_threshold`,
   emit a `MigrationEvent` and move the agent.

## Source

- `crates/engine/src/emergent_migration.rs:1` — module-level FR doc.
- `crates/engine/src/emergent_migration.rs:25` — `Traceability` line.
- `MigrationConfig` exposes weights, threshold, sample size, and
  eligibility delay.

## Acceptance

- Population is conserved across a full tick (source decrements, target
  increments).
- An agent cannot migrate to a settlement with zero housing capacity.
- Migration events are deterministic given the same RNG seed.
