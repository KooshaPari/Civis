# Intent: FR-NFR-S-06 — WebSocket frame size

> Date: 2026-09-20
> FR: FR-NFR-S-06
> Epic: FR-NFR-S

## What This FR Captures

The scalability NFR requiring the average binary WebSocket frame
carrying a 1k-citizen snapshot to stay **< 20 KB**. Statement in
`docs/models/civ-sim/TECHNICAL_SPEC.md:2071`.

## User Intent

Large frames dominate bandwidth and inflate broadcast lag for
all observers. A 20 KB cap on the average frame forces the
encoder to use compact representations (delta snapshots,
compact IDs, run-length-encoded event payloads) instead of
shipping raw JSON, and gives a concrete budget for clients to
tune their receive buffer.

## Acceptance Signal

- Unit test asserts
  `BinaryFrame::to_msgpack_bytes(reference_snapshot_1k).len() < 20_000`
  on a canonical 1k-citizen reference snapshot.

## Implementing Code

- `docs/models/civ-sim/TECHNICAL_SPEC.md:2071` — NFR row.

## Test Coverage

- Unit test on `BinaryFrame::to_msgpack_bytes()` with reference
  snapshot (referenced in the NFR table).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-s-06-intent.md` |
| General spec | `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 |