# Intent: FR-NFR-S-05 — Event log growth rate

> Date: 2026-09-20
> FR: FR-NFR-S-05
> Epic: FR-NFR-S

## What This FR Captures

The scalability NFR requiring the replay/event-log byte growth
rate to stay **< 5 MB/minute** at 1k citizens and 10 ticks/sec.
Statement in `docs/models/civ-sim/TECHNICAL_SPEC.md:2070`.

## User Intent

The replay log is append-only and survives across sessions; if it
grows too fast, a multi-day research run would consume hundreds of
GB on disk and make replay impractical. 5 MB/min is ~7 GB/day,
which is a manageable archival footprint for a long-running
session.

## Acceptance Signal

- Monitor `civlab_event_log_bytes_total` Prometheus counter; a
  recording rule asserts the 60-second rate stays below the 5
  MB/min budget at the canonical 1k-citizen scenario.

## Implementing Code

- `docs/models/civ-sim/TECHNICAL_SPEC.md:2070` — NFR row.

## Test Coverage

- Prometheus recording rule + alert (referenced in the NFR table).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-s-05-intent.md` |
| General spec | `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 |