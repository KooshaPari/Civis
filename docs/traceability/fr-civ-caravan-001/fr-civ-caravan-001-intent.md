# Intent: FR-CIV-CARAVAN-001 — Emergent caravan and trade route formation

> FR: FR-CIV-CARAVAN-001
> Epic: FR-CIV-CARAVAN
> // Covers: FR-CIV-CARAVAN-001

## Requirement

Trade routes form organically between settlements with complementary
resource profiles. Each tick the system evaluates every settlement pair
and decides whether a caravan is profitable enough to spawn. Caravans
travel along routes, are vulnerable to raiding, and increase trade trust
between settlements on success.

## Source

`crates/engine/src/caravan.rs` — module-level doc on line 3 declares the FR
and the module exposes:

- `SettlementProfile` — resource stock, production, population, safety,
  and active-caravan count.
- `MIN_COMMODITY_GAP` (50 SCALE) — minimum gap to trigger a caravan.
- `MIN_SAFETY_THRESHOLD` (300 SCALE = 30%) — minimum safety for departure.
- `RAIDER_PROBABILITY_PER_TICK` (20 SCALE = 2%) — raider encounter chance.
- `CARAVAN_CAPACITY` (1000 SCALE) — cargo capacity per caravan.
- `MAX_CARAVANS_PER_SETTLEMENT` (3) — concurrency cap.

## Acceptance

- `crates/engine/src/caravan.rs` is reachable from the engine crate and
  its public surface compiles with `cargo build -p civ-engine`.
- A settlement pair whose resource gap exceeds `MIN_COMMODITY_GAP` and
  whose safety exceeds `MIN_SAFETY_THRESHOLD` is eligible to spawn a
  caravan up to `MAX_CARAVANS_PER_SETTLEMENT`.
