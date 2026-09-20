# Intent: FR-NFR-S-01 — Max simultaneous WebSocket clients

> Date: 2026-09-20
> FR: FR-NFR-S-01
> Epic: FR-NFR-S

## What This FR Captures

The scalability NFR (not security) requiring the server to sustain
**> 100 concurrent WebSocket clients at 10 ticks/sec** with no
dropped frames. The full statement is in
`docs/models/civ-sim/TECHNICAL_SPEC.md:2066`.

## User Intent

A live simulation session with multiple observers — researchers,
broadcast viewers, replay clients — must hold up under realistic
load. If 100 concurrent viewers breaks the broadcast loop, the
server is unfit for the demo / showcase use cases. The threshold
is "greater than 100" rather than "exactly 100" so we have slack.

## Acceptance Signal

- CI load test spins up 100 concurrent `tokio-tungstenite`
  clients against `civ-server`, ticks at 10 Hz for 60 seconds,
  and asserts no client misses frames.
- Production Prometheus scrape shows
  `civlab_websocket_clients_active >= 100` is achievable.

## Implementing Code

- `docs/models/civ-sim/TECHNICAL_SPEC.md:2066` — NFR row in §10.3
  Scalability.

## Test Coverage

- `tests/load/100_clients.rs` (referenced in the NFR table; CI gate).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-s-01-intent.md` |
| General spec | `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 |