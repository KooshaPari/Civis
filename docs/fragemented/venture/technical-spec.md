# Venture Technical Spec

Source mirror: `./venture/TECHNICAL_SPEC.md`

# Venture-Autonomy Technical Spec (v1)

## Control Plane Components
1. Workflow orchestrator (durable execution).
2. Agent runtime/router.
3. Tool permission and capability policy engine.
4. Money API (authorization/limits/ledger boundary).
5. Schema registry and event bus.
6. Compliance policy evaluator.

## Core Architectural Pattern
1. Intent -> validated task schema -> workflow DAG.
2. Agent actions only through constrained tools.
3. External effects require policy checks and signed events.
4. Every state transition emits auditable events.

## Artifact Compiler Subsystem
1. IR specs: `SlideSpec`, `DocSpec`, `TimelineSpec`, `AudioSpec`, `BoardSpec`.
2. Build pipeline: spec -> render -> validate -> export.
3. Provenance metadata attached to outputs.

## Money Control Subsystem
1. Default-deny spend model.
2. Intent-scoped authorization and caps.
3. Merchant/category and velocity controls.
4. Idempotent payment events and reconciliation.

## Security Requirements
1. Workload identity and short-lived credentials.
2. Strict tool allowlists per agent role.
3. External-content isolation and prompt-injection defenses.
4. Tamper-evident event logs.
