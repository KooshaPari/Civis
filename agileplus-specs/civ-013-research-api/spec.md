---
spec_id: civ-013
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-05-29
---

# Specification: Research API and Scenario System

**Slug**: civ-013-research-api | **Epic**: E5 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Researchers must be able to define custom scenarios, run headless simulations, override policy parameters, and export results to CSV/JSON without modifying engine code. The `.civreplay` format enables auditability and determinism verification. The research API is what differentiates Civis from other civilization engines and is a first-class product requirement.

## Target Users

- Simulation researchers running policy experiments
- Scenario designers creating YAML-defined starting conditions
- Data analysts exporting simulation metrics to Jupyter/matplotlib
- CI agents running determinism gates

## Functional Requirements

- [ ] **FR-API-001**: Scenario YAML format — versioned schema specifying map dimensions, entity placement, starting conditions, policy parameters; validation at load with field-path error on schema violation; `data/scenarios/starting_settlement.yaml` validates in CI
- [ ] **FR-API-002**: Python scenario runner — `civlab.run_scenario(path, ticks=50)` completes 50-tick run in < 5 s; `pip install civlab` installable with all deps declared; all public API methods have type annotations + docstrings
- [ ] **FR-API-003**: Policy parameter override — override dict passed to `run_scenario` merges with scenario defaults; invalid param names raise `ValueError` listing allowed params; values validated against type constraints before simulation start
- [ ] **FR-API-004**: Data export — `civlab.export(run_id, format="csv")` writes per-tick metric table; JSON export includes full event log; export completes in < 30 s for 100,000-tick run
- [ ] **FR-REPLAY-001**: `.civreplay` export — header (seed, scenario, engine version, start timestamp); append-only event log; SHA-256 checksum in footer; verified on load
- [ ] **FR-REPLAY-002**: Bit-identical replay — loading and replaying `.civreplay` produces identical state at every tick; state hash compared at each tick; first divergence reported with tick number + state diff; CI gate blocks on failure

## Non-Functional Requirements

- Scenario YAML schema versioned in repo; `data/scenarios/starting_settlement.yaml` exists and validates in CI
- Python package: `civlab` with type annotations; `civ-research` crate provides Rust FFI or socket interface
- `.civreplay` format: `ReplayLog` in `crates/engine/src/replay.rs`; append-only during run
- Determinism CI gate runs on every commit; blocks merge on first divergence

## Constraints and Dependencies

- Depends on: FR-CORE-004 (stochastic event phase) for RNG seed logging in replay
- Depends on: FR-CORE-003 (deterministic transition) for bit-identical replay guarantee
- Depends on: `civ-research` crate stubs for Python bridge
- Research clients access server via JSON-RPC with research role

## Acceptance Criteria

- [ ] `civlab.run_scenario("data/scenarios/starting_settlement.yaml", ticks=50)` completes in < 5 s
- [ ] Scenario YAML schema validation rejects missing required fields with field-path error
- [ ] Policy override dict correctly merges with scenario defaults; invalid names raise `ValueError`
- [ ] `civlab.export(run_id, format="csv")` produces per-tick metric table
- [ ] SHA-256 checksum in `.civreplay` footer verified on load
- [ ] Replay CI gate: replaying any CI run produces identical state hashes at every tick

## Implementation Notes

- `crates/engine/src/scenario.rs` and `scenarios/baseline.yaml` already landed
- `civ-research` stubs exist; Python bridge (FFI or socket) not yet implemented
- Spec reference: CIV-0001 Phase 5 (Research API)

## Status

| Story | Status |
|-------|--------|
| E5.1 Scenario YAML format | Partial (`scenario.rs` + `baseline.yaml`; schema validation not gated in CI) |
| E5.2 Python scenario runner | Planned (`civ-research` stubs; no Python package yet) |
| E5.3 Policy overrides | Planned |
| E5.4 Query API | Planned |
| E5.5 Replay inspection | Partial (`ReplayLog` exists; inspection tools not built) |
| E5.6 Data export | Planned |
| E5.7 Scenario benchmarking | Planned |
| E5.8 Acceptance tests | Planned |
| FR-REPLAY-001 .civreplay export | Partial (format defined; SHA-256 checksum footer planned) |
| FR-REPLAY-002 Bit-identical CI gate | Partial (determinism tests exist; CI block on divergence not hardened) |
