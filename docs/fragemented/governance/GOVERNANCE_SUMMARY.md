# CIV Project Governance Summary

**Date:** 2026-02-21

**Status:** Aligned with kush ecosystem standards

This document consolidates all governance policies, quality gates, and traceability requirements for the CIV city simulation engine.

---

## Table of Contents

1. [Specification-Driven Workflow](#specification-driven-workflow)
2. [Quality Governance](#quality-governance)
3. [Determinism & Replay Requirements](#determinism--replay-requirements)
4. [Anti-Drift Rules](#anti-drift-rules)
5. [Cross-Track Integration](#cross-track-integration)
6. [Governance Documents Map](#governance-documents-map)

---

## Specification-Driven Workflow

### Core Process

All substantial changes to CIV must:

1. **Start with specs** — Identify affected specs in `docs/specs/`
2. **Update specs first** — Record assumptions, invariants, design decisions
3. **Implement code** — Link code back to spec IDs/sections via comments
4. **Pass quality gates** — All tests, linters, complexity checks
5. **Brief changelog** — One-line entry explaining the change

### CIV Specification Package (9 Specs)

All specifications are located in `docs/specs/CIV-*.md` and are marked **CLOSED**:

| ID | Title | Summary |
|---|---|---|
| **CIV-0001** | Core Simulation Loop | Foundation: tick-based state transitions, deterministic event ordering, replay contract |
| **CIV-0100** | Economy Spec v1 | Ledger model, market clearing, double-entry accounting, conservation invariants |
| **CIV-0101** | Two-Zoom LOD v1 | Spatial representation, hierarchical level-of-detail (macro/micro), city districts |
| **CIV-0102** | Climate Follow-up v1 | Energy accounting integration, supply-stress metrics, demand-response mechanics |
| **CIV-0103** | Institutions, Time-Series, Citizen Lifecycle v1 | Actor lifecycle model, institutional state machine, time-series citizen metrics |
| **CIV-0104** | Minimal Constraint Set Theorem | Mathematical foundations for determinism, idempotency, and replay semantics |
| **CIV-0105** | War, Diplomacy, Shadow Networks v1 | Geopolitical dynamics, conflict resolution, institutional coalitions |
| **CIV-0106** | Social, Ideology, Health, Insurgency v1 | Citizen agency, emergent conflict, health systems, social cohesion |
| **CIV-0107** | Joule Economy System v1 | Agent-centric resource allocation, joule accumulation, work capacity modeling |

**Required Spec Sections:**
- Summary (one-paragraph overview)
- CIV Sim Integration Notes (how this spec relates to other modules)

Verify with: `task spec:validate` or `task spec:list`

---

## Quality Governance

### Configuration Reference

**File:** `qa-config.json`

| Setting | Value | Scope |
|---------|-------|-------|
| **Test Coverage** | ≥ 80% | All code paths |
| **Cyclomatic Complexity** | ≤ 10 per function | rust (cargo-clippy) |
| **Cognitive Complexity** | ≤ 15 per function | rust (cargo-clippy) |
| **Function Lines** | ≤ 60 max | rust |
| **Code Duplication** | ≤ 8% | Across codebase |

### Quality Gates

**File:** `docs/governance/QUALITY_GATES.md`

Run via `task quality`:

1. `cargo fmt --all` — Code formatting
2. `cargo check` — Compilation check
3. `cargo test` — Unit + integration tests
4. `cargo clippy --workspace --all-targets -- -D warnings` — Lint with warnings-as-errors
5. `task spec:validate` — Spec completeness check
6. `task docs:lint` — Markdown linting

**Before finalizing any work**, run:
```bash
task quality
```

### Enforcement

- **Pre-write hooks:** All markdown must pass linting before commit
- **Suppressions:** Zero new suppressions without inline justification (syntax: `# noqa: CODE -- reason`)
- **Code reviews:** All PRs must have passing quality gates
- **CI integration:** Continuous integration rejects non-conforming commits

---

## Determinism & Replay Requirements

### Core Mandate

**All CIV simulation logic must be deterministic and replayable.**

- **No silent randomness:** All RNG calls must be seeded and logged
- **All external effects:** Side effects must be explicit (database writes, file I/O, API calls)
- **Replay guarantee:** Same input state + same seed → same output state, deterministically
- **See:** `CIV-0001` (Core Simulation Loop) and `CIV-0104` (Minimal Constraint Set Theorem)

### Implementation Practices

1. **Seed state logging** — Every stochastic step logs: `(seed, rng_state_before, decision, rng_state_after)`
2. **Event ordering** — Core loop specifies tick order: climate → economy → institutions → actors → conflicts
3. **Idempotency keys** — All state mutations include `(tick, actor_id, decision_id)` for uniqueness
4. **Audit trail** — All policy decisions emit immutable events to `events/` log

### Testing

- **Replay tests:** Run simulation N times; assert identical event log each time
- **Determinism validator:** Tool to replay simulation from event log; compare states
- **See:** `TRACEABILITY_MATRIX.md` for test→spec linkage

---

## Anti-Drift Rules

**All rules inherited from kush ecosystem global CLAUDE.md.**

### Forbidden Patterns

1. **No Fallbacks** — Never add `try: new() except: old()`
2. **No Legacy Compat** — Never add flags like `if legacy_mode: old() else: new()`
3. **No Silent Failures** — Never swallow exceptions or return defaults
4. **No Backwards Compat** — Zero user debt. All changes are breaking.
5. **No "Just In Case" Code** — Only add code that's immediately needed

### Correct Approach

- **Fail fast, fail loud** — Code should stop and print stack traces, not hide errors
- **Fix root causes** — If something fails, fix it; don't work around it
- **Verify parity** — Before removing code, verify feature parity with new implementation

**Reference:** `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/docs/upstream-governance/thegent/CLAUDE.md` (Section: "FORBIDDEN: Fallbacks...")

---

## Cross-Track Integration

### Venture Platform

CIV integrates with the Venture autonomous agent platform at these points:

| CIV Component | Venture Component | Integration Point | Spec Reference |
|---|---|---|---|
| **CIV-0100 Economy Events** | Venture EventEnvelopeV1 | Event streaming with trace IDs | TRACK_C_CONTROL_PLANE.md |
| **CIV-0103 Institutions** | Venture Compliance Machine | Audit trail for institutional changes | OPS_COMPLIANCE_SPEC.md |
| **CIV-0102 Energy Accounting** | Venture Spend Quotas | Conservation equation maps to budget model | TRACK_B_TREASURY_COMPLIANCE_SPEC.md |
| **CIV-0001 Policy.Evaluate** | Venture Tool Allowlist | Rate-limited tool calls with EAU budget | TRACK_C_CONTROL_PLANE.md |

**Cross-Track Spec Index:** `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/SPECS_INDEX.md`

### Planning Workspace

All implementation work tracked in:
- **Immediate next steps:** `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/NEXT_STEPS.md`
- **CIV work stream:** `docs/reference/WORK_STREAM.md` (this project)

---

## Governance Documents Map

### In This Project

| File | Purpose | Audience |
|------|---------|----------|
| **CLAUDE.md** | Project governance index + guardrails | Developers, agents |
| **AGENTS.md** | Agent mandate for CIV work | Agents, project managers |
| **Taskfile.yml** | Task definitions (cargo, linters, spec checks) | All |
| **qa-config.json** | Quality gate thresholds | CI, linters |
| **QUALITY_GATES.md** | Quality gate definitions (this file) | Developers |
| **GOVERNANCE_SUMMARY.md** | Unified governance reference (you are here) | All |
| **docs/reference/WORK_STREAM.md** | Active work items + status | Project managers, developers |
| **docs/reference/TRACEABILITY_MATRIX.md** | FR → Spec → Code linkage | QA, auditors |

### Upstream (Global/Ecosystem)

| File | Purpose |
|------|---------|
| `docs/upstream-governance/thegent/CLAUDE.md` | Global kush ecosystem rules |
| `/Users/kooshapari/kush/trace/CLAUDE.md` | trace project reference standard |
| `/Users/kooshapari/kush/trace/quality-gate.yml` | Reference quality-gate config |

### Parpour Planning Workspace

| File | Purpose |
|------|---------|
| `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/SPECS_INDEX.md` | Master spec index (CIV + Venture) |
| `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/NEXT_STEPS.md` | Cross-track implementation roadmap |

---

## Execution Checklist

Before pushing code or opening PRs:

- [ ] `task spec:validate` passes
- [ ] `task quality` passes (fmt, check, test, clippy, spec:validate, docs:lint)
- [ ] `task traceability:check` passes (if applicable)
- [ ] Code comments link to affected spec IDs (e.g., `// CIV-0100: Economy.market_clear()`)
- [ ] No new lint suppressions without inline justification
- [ ] No fallback/compat/silent-failure code patterns
- [ ] CHANGELOG.md updated (brief entry)
- [ ] Determinism preserved (no stochastic changes without logging seed state)

---

## Governance Evolution

When work touches a governance domain (quality, testing, determinism, traceability):

1. **Check** existing governance docs (above)
2. **Follow** the established pattern
3. **Propose updates** if governance is missing or unclear
4. **Write a conversation dump** to `docs/research/CONVERSATION_DUMP_YYYY-MM-DD.md` when closing planning work

---

## Quick Links

- **Run quality gates:** `task quality`
- **Check work stream:** `cat docs/reference/WORK_STREAM.md`
- **List specs:** `task spec:list`
- **Validate specs:** `task spec:validate`
- **Parpour planning:** `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/NEXT_STEPS.md`
- **Trace reference:** `/Users/kooshapari/kush/trace/CLAUDE.md`

---

**Last Updated:** 2026-02-21
**Status:** ACTIVE
