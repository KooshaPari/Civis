# Intent: FR-DIP-002 — Advanced diplomacy effects

> FR: FR-DIP-002
> Epic: FR-DIP
> // Covers: FR-DIP-002

## Requirement

Four diplomacy effects operate on a minimal `WorldState`:

- `CulturalInfluenceEffect` — border-sharing factions drift culturally.
- `TradeEmbargoEffect` — factions impose trade embargoes, reducing
  efficiency between them.
- `MilitaryAllianceEffect` — factions form military alliances, sharing
  intelligence and receiving combat bonuses.
- `TributeEffect` — dominant factions demand tribute from weaker ones.

## Determinism

All computation is integer-only over `BTreeMap`-backed collections.
Given the same `WorldState`, the same effect produces identical events
and state mutations. No RNG, no floating-point, no wall-clock.

## Source

`crates/diplomacy/src/effects.rs:1` — module-level doc declares
`FR-DIP-002` and lists the four effects.

## Acceptance

- `cargo build -p civ-diplomacy` succeeds.
- Each effect is `Send + Sync` and operates without panicking on an
  empty `WorldState`.
- Effect outputs are deterministic across repeated calls with the
  same `WorldState`.
