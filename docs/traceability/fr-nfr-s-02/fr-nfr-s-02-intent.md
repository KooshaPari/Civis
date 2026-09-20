# Intent: FR-NFR-S-02 — WebSocket connection overhead

> Date: 2026-09-20
> FR: FR-NFR-S-02
> Epic: FR-NFR-S

## What This FR Captures

The scalability NFR requiring per-client connection overhead —
WebSocket upgrade + handshake + initial snapshot — to complete
in **< 5 ms** on the reference CI hardware. Statement in
`docs/models/civ-sim/TECHNICAL_SPEC.md:2067`.

## User Intent

When a viewer joins a live session, the perceived "load time" is
the time from hitting Connect to seeing the first frame. A
handshake budget of 5 ms is well under one frame at 10 ticks/sec
(100 ms / tick), so a viewer feels the join is instant.

## Acceptance Signal

- Integration test measures the WebSocket upgrade + initial
  snapshot latency percentile and asserts p95 ≤ 5 ms.

## Implementing Code

- `docs/models/civ-sim/TECHNICAL_SPEC.md:2067` — NFR row.

## Test Coverage

- Integration test with timer (referenced in the NFR table).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-s-02-intent.md` |
| General spec | `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 |