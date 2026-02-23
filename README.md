# CivLab (Scaffold)

CivLab is a deterministic socio-economic simulation scaffold centered on policy-driven world evolution.

## Workspace Layout

- `crates/engine`: deterministic tick/state core
- `crates/policy`: policy interfaces and decision transforms
- `crates/metrics`: measurable outputs (waste/surplus/tyranny/legitimacy)
- `crates/io`: scenario/schema loading boundaries
- `crates/server`: executable entrypoint and orchestration shell
- `schemas/`: policy and scenario schemas
- `scenarios/`: runnable scenario fixtures
- `docs/`: specs, ADRs, governance, architecture

## Quick Start

```bash
cargo check
cargo test
cargo run -p civ-server
```

## Governance

- Spec-first changes in `docs/specs/`
- Deterministic quality gates before merge
- Explicit assumptions/invariants linked to implementation changes
