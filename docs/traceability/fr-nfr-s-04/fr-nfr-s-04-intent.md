# Intent: FR-NFR-S-04 — Command throughput

> Date: 2026-09-20
> FR: FR-NFR-S-04
> Epic: FR-NFR-S

## What This FR Captures

The scalability NFR requiring the server to accept
**> 1,000 commands/sec** without the tick interval lengthening.
Statement in `docs/models/civ-sim/TECHNICAL_SPEC.md:2069`.

## User Intent

A live research session may have dozens of scripted agents issuing
god-mode commands (set policy, declare war, queue research)
rapidly. If a 1 kHz command flood blocks the tick loop, the
simulation visually stalls for observers. Commands must be
absorbed asynchronously into the input queue without contending
with the tick critical path.

## Acceptance Signal

- Stress test floods `command_tx` at 1,000 messages/sec while the
  server runs at the nominal 10 Hz tick rate; tick-time histogram
  is identical to the no-load case.

## Implementing Code

- `docs/models/civ-sim/TECHNICAL_SPEC.md:2069` — NFR row.

## Test Coverage

- Load test with command flood (referenced in the NFR table).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-s-04-intent.md` |
| General spec | `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 |