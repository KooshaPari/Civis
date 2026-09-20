# Intent: FR-CIV-WARFARE-003 -- War-economy drain and attrition

> Date: 2026-09-19
> FR: FR-CIV-WARFARE-003
> Epic: FR-CIV-WARFARE

## User Intent

Sustained war must drain cluster economy, and combat casualties must
reduce population — without any of these drains firing during peacetime.
The exhaustion floor (treasury ≤ 10% of starting) is a key turning-point
signal used by the diplomacy layer to broker peace.

### What This FR Achieves

`crates/tactics/src/war_economy.rs` defines:

- `WAR_ECONOMY_DRAIN_RATE = 0.02` — fraction of treasury drained per
  active-war tick.
- `CASUALTIES_PER_ENERGY_UNIT = 0.001` — population loss per unit of
  estimated casualties.
- `WarEconomyDrain { treasury_drain, population_loss, economically_exhausted }`.
- `compute_war_economy_drain(treasury, casualties, at_war)`
  -> zeros when `at_war == false`; monotonic in `treasury` and
  `casualties` otherwise.

## Acceptance Signal

- `cargo test -p tactics war_economy` passes.
- `compute_war_economy_drain(t, c, false) == 0` for every input.
- `economically_exhausted == true` when remaining treasury ≤
  `treasury / 10` (with a `max(10, ...)` floor).

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/tactics/` |
| Module | `crates/tactics/src/war_economy.rs:1` |
