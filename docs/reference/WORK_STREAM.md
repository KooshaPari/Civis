# CIV Work Stream — Active Implementation Items

**Date:** 2026-02-21

**Status:** All CIV specs complete (8 CLOSED). Implementation roadmap extracted from `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/NEXT_STEPS.md`.

---

## P0: Core Engine Foundation

These items form the bedrock for all CIV simulation. Must complete in dependency order.

| Task ID | Description | Depends On | Status | Owner |
|---------|-------------|-----------|--------|-------|
| **P0-1** | Implement Core Simulation Loop (CIV-0001) | Foundation | TODO | CIV Engine |
| | - Tick-based state machine | | | |
| | - Deterministic event ordering (climate → economy → institutions → actors → conflicts) | | | |
| | - Seed logging and RNG state tracking | | | |
| | - Replay contract: same seed → same events | P0-1 | | |
| **P0-2** | Economy Module: Ledger & Market Clearing (CIV-0100) | P0-1 | TODO | CIV Economy |
| | - Double-entry accounting system | | | |
| | - Market clearing algorithm | | | |
| | - Conservation invariant checks | | | |
| | - Ledger reconciliation hooks | | | |
| **P0-3** | Spatial Representation: Two-Zoom LOD (CIV-0101) | P0-1, P0-2 | TODO | CIV Spatial |
| | - Macro-scale district model | | | |
| | - Micro-scale individual actors | | | |
| | - LOD transition logic | | | |
| | - Spatial queries (neighbor detection, resource flow) | | | |
| **P0-4** | Climate Module: Energy Accounting (CIV-0102) | P0-1, P0-2 | TODO | CIV Climate |
| | - Energy conservation equation | | | |
| | - Supply-stress metrics | | | |
| | - Integration points to economy (energy supply constraints) | | | |
| | - Deterministic weather events | | | |
| **P0-5** | Institutions & Citizen Lifecycle (CIV-0103) | P0-1 | TODO | CIV Actors |
| | - Actor lifecycle (birth, education, career, retirement, death) | | | |
| | - Institutional state machine (formation, change, dissolution) | | | |
| | - Time-series citizen metrics storage | | | |
| | - Dependency propagation (age cohorts affect labor supply, etc.) | | | |
| **P0-6** | Mathematical Foundations: Minimal Constraint Set Theorem (CIV-0104) | Foundation | TODO | CIV Theory |
| | - Implement constraint solver for deterministic state | | | |
| | - Idempotency validator | | | |
| | - Replay determinism proofs | | | |
| **P0-7** | Geopolitical Dynamics: War, Diplomacy, Shadow Networks (CIV-0105) | P0-1, P0-3, P0-5 | TODO | CIV Geo |
| | - Conflict resolution model | | | |
| | - Diplomatic stance tracking | | | |
| | - Alliance formation mechanics | | | |
| | - Shadow network (covert operations) model | | | |
| **P0-8** | Citizen Agency: Social, Ideology, Health, Insurgency (CIV-0106) | P0-1, P0-3, P0-5 | TODO | CIV Social |
| | - Ideology system (preference vectors) | | | |
| | - Health system (epidemics, mortality) | | | |
| | - Insurgency mechanics (grievance → rebellion) | | | |
| | - Social cohesion metrics | | | |

| **P0-9** | Resource Allocation: Joule Economy System (CIV-0107) | P0-1, P0-2 | TODO | CIV Economy |
| | - Joule accumulation mechanics (agents earn work capacity) | | | |
| | - Goal-based allocation framework | | | |
| | - Pluggable allocator interface (Market, Plan, Joule) | | | |
| | - Conservation invariant validation | | | |

**P0 Exit Gate:** All modules implemented, tested, and determinism verified. Run:
```bash
task quality           # Pass all linters, tests, complexity checks
task spec:validate     # All specs have required sections
task traceability:check # FR→Spec→Code links verified
```

---

## P1: Venture Platform Integration

These items integrate CIV with Venture control-plane. Depends on P0 completion.

| Task ID | Description | Depends On | Status | Owner |
|---------|-------------|-----------|--------|-------|
| **P1-1** | CIV Event Export: Economy Events → Venture EventEnvelopeV1 | P0-1, P0-2, Venture P0-1 | TODO | CIV-Venture Integ |
| | - Map `economy.market_cleared.v1` events | | | |
| | - Map `economy.transfer_booked.v1` events | | | |
| | - Bind to `workflow_id`, `trace_id`, `policy_bundle_id` | | | |
| | - Emit immutable event logs | | | |
| **P1-2** | CIV Policy.Evaluate Tool Integration | P0-1, P0-2, Venture P0-4 | TODO | CIV-Venture Integ |
| | - Wrap `civ.policy.evaluate(state, context)` as Venture tool | | | |
| | - Define rate limits: 10 EAU/call, max 100 calls/workflow | | | |
| | - Tool allowlist entry with timeout SLA | | | |
| **P1-3** | Institutional Change Audit Trail | P0-5, Venture P0-5 | TODO | CIV-Venture Integ |
| | - Map `institution.created/disbanded/merged/split` events → compliance cases | | | |
| | - Audit drill: recover full institution evolution from event log | | | |
| **P1-4** | Cost Model: CIV Energy → Venture Spend Quotas | P0-2, P0-4, Venture P0-2 | TODO | CIV-Venture Finance |
| | - Map energy conservation equation to budget model | | | |
| | - Peak-shaving mechanics → spend velocity controls | | | |
| | - Cost estimate validation (&plusmn;5% accuracy) | | | |

**P1 Exit Gate:** CIV events flow through Venture event bus. Compliance can trace institutional changes and policy decisions.

---

## P2: Visualization & Artifacts

These items model CIV simulation outputs as Venture artifacts (timelines, dashboards, org charts).

| Task ID | Description | Depends On | Status | Owner |
|---------|-------------|-----------|--------|-------|
| **P2-1** | Define CivSimulationArtifact IR Type | P1-1, Venture P0-3a | TODO | CIV-Venture Artifact |
| | - Create `TimelineSpec` for simulation narrative export | | | |
| | - Create `BoardSpec` for economic dashboard | | | |
| | - Create custom IR type for institutional org chart | | | |
| | - All artifacts include `content_hash`, `inputs_hash`, `policy_bundle_id` | | | |
| **P2-2** | Deterministic Artifact Build Contract | P2-1, Venture P0-3b | TODO | CIV-Venture Artifact |
| | - Idempotency key: `hash(ir_hash, toolchain_version, policy_bundle_id, surface)` | | | |
| | - Cache layer: bytewise-identical replay | | | |
| | - Provenance signing for all artifact builds | | | |
| **P2-3** | Simulation Output Export Pipeline | P0-1, P2-1, P2-2 | TODO | CIV Export |
| | - On simulation completion: auto-export artifacts | | | |
| | - Bind artifacts to simulation run (workflow_id, trace_id) | | | |
| | - Versioned artifact storage with provenance | | | |

**P2 Exit Gate:** All CIV simulation outputs are modeled as Venture artifacts. Artifacts are deterministic, auditable, and versioned.

---

## P3: Polish & Hardening

These items improve observability, performance, and incident readiness.

| Task ID | Description | Depends On | Status | Owner |
|---------|-------------|-----------|--------|-------|
| **P3-1** | Performance Tuning | P0 complete | TODO | CIV Perf |
| | - Profile large simulations (100k+ agents) | | | |
| | - Optimize economy market clearing | | | |
| | - Optimize spatial queries | | | |
| **P3-2** | Observability & Logging | P0 complete | TODO | CIV Ops |
| | - Structured JSON logging for all events | | | |
| | - Metrics: tick latency, event count, memory usage | | | |
| | - Dashboard integration with Venture compliance | | | |
| **P3-3** | Documentation & Examples | P0 complete | TODO | CIV Docs |
| | - Walkthrough of small simulation (10 agents, 100 ticks) | | | |
| | - Determinism testing guide | | | |
| | - CIV→Venture integration examples | | | |
| **P3-4** | Incident Playbooks | P1-3, Venture P2 complete | TODO | CIV Ops |
| | - Replay determinism failure recovery | | | |
| | - Policy evaluation timeout handling | | | |
| | - Energy conservation violation detection | | | |

**P3 Exit Gate:** CIV is production-ready with full observability and incident response procedures.

---

## Open Questions

These items require decision owners before implementation can proceed.

| Q# | Question | Spec Location | Impacts | Owner | Due |
|---|----------|---|---------|-------|-----|
| **Q5** | Climate Model Coupling to Economy | CIV-0102, CIV-0100 | P0-2, P0-4 implementation order | CIV Engine | Before P0 |
| | How tightly should climate energy flows couple to economy? Tick-by-tick or decoupled causality? | | | | |
| **Q6** | Institutional Change Propagation Lag | CIV-0103, CIV-0105 | P0-5, P0-7 state machine | CIV Actors | Before P0 |
| | How many ticks delay between institution formation and effect on actor behavior? | | | | |
| **Q7** | CIV Simulation Artifact IR Mapping | CIV-0001, CIV-0100 vs. TRACK_A | P2 artifact design | CIV-Venture Integ | Before P2 |
| | Should CIV simulation outputs (timelines, dashboards) be modeled as Venture artifacts? | | | | |
| **Q8** | CIV Policy.Evaluate Tool Rate-Limiting | TECHNICAL_SPEC, TRACK_C | P1-2 budget model | CIV-Venture Integ | Before P1 |
| | Is `civ.policy.evaluate` rate-limited per-call or per-workflow? What's the SLA? | | | | |

---

## Status Legend

- **TODO** — Not started
- **IN_PROGRESS** — Active work
- **BLOCKED** — Waiting on dependency or decision
- **DONE** — Complete and merged

---

## How to Use This Work Stream

### Claim a Task
Update the `Status` column to `IN_PROGRESS` and add your name as owner.

### Submit Work
Run quality gates before finalizing:
```bash
task quality           # All tests, linters, specs, docs
task traceability:check # FR→Spec→Code linkage (if applicable)
```

### Mark Complete
Update status to `DONE` and link to PR/commit.

### Escalate Blockers
If blocked on a decision, update the corresponding Open Question row with your blocker.

---

## Cross-Track Coordination

**Venture platform dependencies:** See `/Users/kooshapari/temp-PRODVERCEL/485/kush/parpour/NEXT_STEPS.md` (Part 2: Venture P0-P3 tasks)

**Key sync points:**
- **Week 1 (Day 2):** Review EventEnvelopeV1 schema with Venture Platform team
- **Week 1 (Day 4):** Align event payload structure (EventEnvelopeV1 + CIV economy events)
- **Week 1 (Day 6):** Full integration test: `money.authorization.decided.v1` + `economy.market_cleared.v1`
- **Week 2 (Day 9):** Resolve Q7 (CIV artifact IR mapping)
- **Week 2 (Day 12):** Cost model validation (energy conservation test)

---

**Last Updated:** 2026-02-21
**Next Review:** When P0 exits gate or blockers identified
