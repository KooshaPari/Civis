# Intent: FR-CIV-INT-001 — Hash-chain integrity primitives

> Date: 2026-09-20
> FR: FR-CIV-INT-001
> Epic: FR-CIV-INT

## What This FR Captures

The hash-chain integrity primitives in
`crates/engine/src/hash_chain.rs`. Specifically the contract that
`chain_advance(state, event_bytes) -> [u8; 32]` returns a 32-byte
hash, that order of advancement matters (non-commutative), and
that `HashChainState::new()` starts the running hash at `GENESIS`.
These guarantees underpin the engine's tamper-evident replay log.

## User Intent

Replay integrity is a load-bearing promise — anyone can verify
that a `.civreplay` archive was produced by a specific, unmodified
build of the engine. The chain must be strictly length-32
(uniform slot size), strictly non-commutative (otherwise
re-ordering is invisible), and start at a fixed genesis (so that
"empty chain" is recognizable).

## Acceptance Signal

- `chain_advance(&GENESIS, &tick_event_bytes(n))` always returns
  `HASH_LEN` (=32) bytes.
- Advancing with two different event-byte sequences from the
  same parent hash produces two different child hashes.
- `HashChainState::new().running_hash == GENESIS`.

## Implementing Code

- `crates/engine/tests/fr_engine_replay_integrity_tests.rs:5` —
  module header lists FR-CIV-INT-001.
- `crates/engine/tests/fr_engine_replay_integrity_tests.rs:113`
  — `fr_civ_int_001_chain_advance_length`.
- `crates/engine/tests/fr_engine_replay_integrity_tests.rs:116`
  — `fr_civ_int_001_chain_advance_order_matters` and
  `fr_civ_int_001_hash_chain_state_new`.

## Test Coverage

- Three dedicated tests in the test module above.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-int-001-intent.md` |
| Implementing crate | `crates/engine/src/hash_chain.rs` |