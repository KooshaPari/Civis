# Merged Fragmented Markdown

## Source: governance/GOVERNANCE_SUMMARY.md

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


---

## Source: governance/QUALITY_GATES.md

# 9-Gate Quality System

The quality gate system enforces code quality through 9 sequential checks. Each gate validates a specific aspect of code quality, from basic syntax to dependency security.

## Gate Overview

| Gate | Name | Tools | Threshold | Fail Behavior |
|------|------|-------|-----------|---------------|
| 1 | Syntax Validation | python3 ast, node --check, go build, bash -n | Zero errors | Blocking |
| 2 | Linting | ruff, oxlint/eslint, golangci-lint, shellcheck, clippy | Zero errors | Blocking |
| 3 | Type Safety | ty/mypy/pyright, tsc, go vet | Zero errors | Blocking |
| 4 | Tests | pytest, vitest/jest, go test, cargo test | All pass | Blocking |
| 5 | Coverage | pytest-cov, go test -cover | >=80% | Blocking |
| 6 | Security | bandit, gosec, gitleaks, npm audit | Zero high/critical | Blocking |
| 7 | Complexity | radon, ruff C901, gocyclo | CC<=10, Cog<=15 | Blocking |
| 8 | Duplication | jscpd | <5% | Blocking |
| 9 | Dependencies | pip-audit, npm audit, govulncheck, cargo-audit | Zero vulnerabilities | Blocking |

## Gate Details

### Gate 1: Syntax Validation (Fast-Fail)

Catches parse errors before any other analysis runs.

- **Python**: `ast.parse()` on each changed `.py` file
- **JavaScript/TypeScript**: `node --check` on each changed file
- **Go**: `go build ./...` for compile errors
- **Rust**: `cargo check` for compile errors
- **Shell**: `bash -n` for syntax validation

**Fix**: Correct the syntax error shown in the output.

### Gate 2: Linting

Enforces code style and catches common mistakes.

| Language | Tool | Config |
|----------|------|--------|
| Python | ruff | `ruff.toml` or `pyproject.toml` |
| JS/TS | oxlint (preferred) or eslint | `oxlintrc.json` or `.eslintrc` |
| Go | golangci-lint | `.golangci.yml` |
| Shell | shellcheck | `.shellcheckrc` |
| Rust | clippy | `clippy.toml` |

**Fix**: Run `ruff check --fix`, `oxlint --fix`, or the appropriate fixer.

### Gate 3: Type Safety

Static type analysis to catch type errors before runtime.

| Language | Tool | Config |
|----------|------|--------|
| Python | ty (preferred), mypy, pyright | `ty-config.toml`, `mypy.ini` |
| TypeScript | tsc | `tsconfig.json` |
| Go | go vet | Built-in |

**Fix**: Add type annotations, fix type mismatches, or update type stubs.

### Gate 4: Tests

Runs the project test suite.

| Language | Tool | Command |
|----------|------|---------|
| Python | pytest | `pytest -q --tb=short` |
| JS/TS | vitest or jest | `npx vitest run` or `npx jest` |
| Go | go test | `go test ./... -count=1` |
| Rust | cargo test | `cargo test --quiet` |

**Fix**: Fix failing tests. Do not skip or disable them.

### Gate 5: Coverage (>=80%)

Ensures adequate test coverage. Threshold is configurable in `quality-gate.yml`.

| Language | Tool | Report |
|----------|------|--------|
| Python | pytest-cov | `--cov --cov-report=term-missing` |
| Go | go test -cover | Built-in |

**Fix**: Add tests for uncovered code paths. Focus on critical business logic first.

### Gate 6: Security

Multi-layer security scanning.

| Layer | Tool | Scope |
|-------|------|-------|
| Secrets | gitleaks | All files (detects API keys, passwords) |
| SAST | bandit (Python), gosec (Go) | Source code analysis |
| Audit | npm audit | Node dependency vulnerabilities |

**Fix**: Remove secrets from code (use env vars), fix SAST findings, update vulnerable deps.

### Gate 7: Complexity (CC<=10, Cognitive<=15)

Prevents overly complex functions that are hard to test and maintain.

| Metric | Max | Tool |
|--------|-----|------|
| Cyclomatic complexity | 10 | radon (Python), gocyclo (Go) |
| Cognitive complexity | 15 | ruff C901 (Python) |
| Function length | 40 lines | Per-language |

**Fix**: Extract helper functions, reduce nesting, simplify conditionals.

### Gate 8: Duplication (<5%)

Detects copy-paste code that should be refactored.

- **Tool**: jscpd (language-agnostic)
- **Min detection**: 5 lines / 50 tokens
- **Threshold**: 5% of codebase

**Fix**: Extract shared logic into functions or modules. Do not create premature abstractions for <3 occurrences.

### Gate 9: Dependencies

Scans for known vulnerabilities in project dependencies.

| Language | Tool | Scope |
|----------|------|-------|
| Python | pip-audit | PyPI vulnerability database |
| Node | npm audit | npm advisory database |
| Go | govulncheck | Go vulnerability database |
| Rust | cargo-audit | RustSec advisory database |

**Fix**: Update vulnerable dependencies. Pin versions if update breaks compatibility.

## Configuration

### quality-gate.yml

Place in project root to override defaults:

```yaml
thresholds:
  coverage: 80
  cyclomatic_complexity: 10
  cognitive_complexity: 15
  max_function_lines: 40
  duplication_pct: 5
  timeout_per_gate: 60
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `QUALITY_GATE_CONFIG` | `./quality-gate.yml` | Config file path |
| `QUALITY_GATE_FAIL_FAST` | `false` | Stop on first failure |
| `QUALITY_GATE_VERBOSE` | `false` | Show PASS gates |
| `QUALITY_GATE_ALL_FILES` | `false` | Check all files, not just changed |
| `PROJECT_DIR` | Git root | Project root directory |

## Usage

```bash
# Run all gates (changed files only)
./scripts/quality/quality-gate.sh

# Run all gates on all files
QUALITY_GATE_ALL_FILES=true ./scripts/quality/quality-gate.sh

# Fail fast on first gate failure
QUALITY_GATE_FAIL_FAST=true ./scripts/quality/quality-gate.sh

# Verbose output
QUALITY_GATE_VERBOSE=true ./scripts/quality/quality-gate.sh
```

## Integration

### Taskfile

```yaml
tasks:
  quality:
    desc: Run 9-gate quality system
    cmds:
      - ./scripts/quality/quality-gate.sh
  quality:all:
    desc: Run 9-gate quality system on all files
    env:
      QUALITY_GATE_ALL_FILES: "true"
    cmds:
      - ./scripts/quality/quality-gate.sh
```

### Pre-push Hook

```bash
#!/bin/bash
./scripts/quality/quality-gate.sh || exit 1
```

### CI Pipeline

```yaml
quality-gate:
  script:
    - QUALITY_GATE_ALL_FILES=true ./scripts/quality/quality-gate.sh
```


---

## Source: governance/track-c-civ-sim-closure.md

# CIV Track C Implementation Closure

## Objective
Close implementation planning gaps with a delivery roadmap, quality gates, traceability requirements, and change-control policy.

## Roadmap
1. MVP: artifact tree, glossary, theorem chain, Policy DSL v1, world-seed handoff, scheduler contracts.
2. Alpha: climate, economy, war/diplomacy/shadow, social/ideology/health/insurgency specs integrated.
3. Validation: unified objective/Pareto protocol and verification harness.
4. Game Layer: scenario catalog hardening, packaging, and reproducible release process.

## Quality Gates
1. `cargo fmt --all`
2. `cargo check`
3. `cargo test`
4. `cargo clippy --workspace --all-targets -- -D warnings`

## Traceability Requirements
1. Every closed TODO maps to a spec path and requirement ID.
2. Event topics align with `docs/traceability/EVENT_TAXONOMY.md`.
3. Matrix updates required in `docs/traceability/TRACEABILITY_MATRIX.md` when status changes.

## Change-Control
1. Record change in spec and artifact tree first.
2. Assess impact via traceability matrix + event taxonomy.
3. Run quality gates and verification checks.
4. Approve with version bump and dated changelog note.


---
