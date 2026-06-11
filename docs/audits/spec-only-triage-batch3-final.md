# Spec-Only Triage — Batch 3 (Final)

Source: SPEC-ONLY rows `121-179` in `docs/audits/fr-matrix.json` (continuing after batches 1-2)

Total rows: `59`
Verdict counts:

- `BUILD-NEXT`: `0`
- `DEFER`: `59`
- `ARCHIVE`: `0`

Cumulative summary across all 3 batches:

- `BUILD-NEXT`: `52`
- `DEFER`: `103`
- `ARCHIVE`: `24`

| # | FR ID | Epic | Spec | Verdict | Notes |
|---|---|---|---|---|---|
| 121 | FR-METRICS-004 | FR-METRICS | `docs/FR.md` | DEFER | Non-blocking metrics instrumentation detail; no visible gameplay closure coupling to current parity benchmark. |
| 122 | FR-METRICS-005 | FR-METRICS | `docs/FR.md` | DEFER | Metric taxonomy completion can follow shipped KPI pipeline; not a 1.0 parity blocker. |
| 123 | FR-MOD-002 | FR-MOD | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Mod lifecycle details are useful but out of scope until mod tick/host hardening already prioritized. |
| 124 | FR-MOD-003 | FR-MOD | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Spec-only mod loader behavior without gameplay dependency; defer behind core replay/mod-host loop completion. |
| 125 | FR-MOD-005 | FR-MOD | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Additional mod management behavior is post-1.0 polish versus active partial-good mod pipeline priorities. |
| 126 | FR-NET-001 | FR-NET | `docs/FR.md` | DEFER | Network hardening item is infra quality work and not one of the parity-close functional gaps. |
| 127 | FR-NET-002 | FR-NET | `docs/FR.md` | DEFER | Net transport cleanup is stabilizing work, not required for current parity feature set. |
| 128 | FR-NET-003 | FR-NET | `docs/FR.md` | DEFER | Additional network behavior can be staged after top-20 gap set is closed. |
| 129 | FR-PERF-001 | FR-PERF | `docs/FR_DETAILED.md` | DEFER | Single-pass perf item is quality debt and not mapped to an explicit top-20 blocker from the benchmark. |
| 130 | FR-PERF-002 | FR-PERF | `docs/FR_DETAILED.md` | DEFER | Performance telemetry item can follow large-world parity stream and first-pass optimizations. |
| 131 | FR-PERF-003 | FR-PERF | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Traceability-only perf behavior is non-blocking and better handled in a performance-hardening pass. |
| 132 | FR-PERF-004 | FR-PERF | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Defer until profiling and regression benchmarks show this as an end-user-facing blocker. |
| 133 | FR-PERF-005 | FR-PERF | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Additional perf instrumentation can follow baseline perf closure in current parity lane. |
| 134 | FR-PROT-001 | FR-PROT | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Protocol details are currently documented-only and can be finalized after core protocol coverage stabilizes. |
| 135 | FR-PROT-002 | FR-PROT | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Not currently required to reach benchmark parity and should be staged after protocol test harness matures. |
| 136 | FR-PROT-003 | FR-PROT | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Follow-up protocol detail; no implementation dependency for near-term closure. |
| 137 | FR-PROT-004 | FR-PROT | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Not blocking active simulation/attach/scenario track; defer for later hardening. |
| 138 | FR-PROT-005 | FR-PROT | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Protocol documentation gap can be addressed with next protocol release cycle. |
| 139 | FR-PROT-006 | FR-PROT | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Low-priority contract wording; no immediate implementation signal. |
| 140 | FR-SESS-001 | FR-SESS | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Session metadata spec detail is a cleanup pass without immediate parity impact. |
| 141 | FR-SESS-002 | FR-SESS | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Session observability improvements are sequencing-friendly and post-top-20 in this slice. |
| 142 | FR-SESS-003 | FR-SESS | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Non-blocking spec item pending existing session model stability. |
| 143 | FR-SESS-004 | FR-SESS | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | UX and storage details can be layered once session lifecycle is stable. |
| 144 | FR-SESS-005 | FR-SESS | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Follow-on session integrity behavior for hardening run, not a first-parity gate. |
| 145 | FR-SESS-006 | FR-SESS | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Session edge-case spec can be deferred to maintenance backlog. |
| 146 | FR-SOCI-001 | FR-SOCI | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Sociocultural behavior expansion is a secondary quality dimension; not required for baseline parity loop. |
| 147 | FR-SOCI-002 | FR-SOCI | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Defer until social systems have baseline implementation and test scaffolding. |
| 148 | FR-SOCI-003 | FR-SOCI | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Adds social model depth without direct blocker status in this benchmark pass. |
| 149 | FR-SOCI-004 | FR-SOCI | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Not required for first-pass parity deliverables. |
| 150 | FR-SOCI-005 | FR-SOCI | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Optional social interaction detail after core loops are complete. |
| 151 | FR-SOCI-006 | FR-SOCI | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Follow-up requirement in social domain; not tied to current top-20 milestone. |
| 152 | FR-TEST-001 | FR-TEST | `docs/FR_DETAILED.md` | DEFER | General testing-spec cleanup can be captured after benchmark-focused test cases are implemented. |
| 153 | FR-THRY-001 | FR-THRY | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Theory/risk modeling docs can be staged; no immediate gameplay deliverable dependency. |
| 154 | FR-THRY-002 | FR-THRY | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Non-blocking conceptual work; defer until core sim and UI parity are stable. |
| 155 | FR-THRY-003 | FR-THRY | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Deferrable modeling refinement not required for first closed-loop feature set. |
| 156 | FR-THRY-004 | FR-THRY | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Documentation and modeling depth can follow first stability pass. |
| 157 | NFR-CIV-ACC-003 | NFR-CIV-ACC | `docs/reference/non-functional-requirements.md` | DEFER | Access-performance detail is non-functional quality hardening; no direct implementation path in current epic plan. |
| 158 | NFR-CIV-DET-004 | NFR-CIV-DET | `docs/reference/non-functional-requirements.md` | DEFER | Determinism requirement text without code has to follow deterministic replay hardening sequence. |
| 159 | NFR-CIV-MAINT-001 | NFR-CIV-MAINT | `docs/reference/non-functional-requirements.md` | DEFER | Maintenance obligations are operational quality work best handled in backlog. |
| 160 | NFR-CIV-MAINT-002 | NFR-CIV-MAINT | `docs/reference/non-functional-requirements.md` | DEFER | Post-release maintenance hardening item; defer until base parity gates are closed. |
| 161 | NFR-CIV-MAINT-003 | NFR-CIV-MAINT | `docs/reference/non-functional-requirements.md` | DEFER | Maintenance visibility work is important but not required for initial production closure. |
| 162 | NFR-CIV-MAINT-004 | NFR-CIV-MAINT | `docs/reference/non-functional-requirements.md` | DEFER | Deferred operational process detail with low priority versus gameplay delivery. |
| 163 | NFR-CIV-MAINT-005 | NFR-CIV-MAINT | `docs/reference/non-functional-requirements.md` | DEFER | Ongoing maintenance policy work can be sequenced after active feature completion. |
| 164 | NFR-CIV-MAINT-006 | NFR-CIV-MAINT | `docs/reference/non-functional-requirements.md` | DEFER | Deferrable requirement clean-up for maintainability; no immediate user-visible impact. |
| 165 | NFR-CIV-PERF-002 | NFR-CIV-PERF | `docs/reference/non-functional-requirements.md` | DEFER | This NFR has no implementation evidence and is lower priority versus FR-level scale/perf gaps. |
| 166 | NFR-CIV-PERF-007 | NFR-CIV-PERF | `docs/reference/non-functional-requirements.md` | DEFER | Performance characteristic in spec-only form; defer until measured bottlenecks require it. |
| 167 | NFR-CIV-PORT-001 | NFR-CIV-PORT | `docs/reference/non-functional-requirements.md` | DEFER | Portability target is strategic but not a closed-loop blocker in this tranche. |
| 168 | NFR-CIV-PORT-002 | NFR-CIV-PORT | `docs/reference/non-functional-requirements.md` | DEFER | Defer support-workload hardening until parity build milestones are met. |
| 169 | NFR-CIV-PORT-003 | NFR-CIV-PORT | `docs/reference/non-functional-requirements.md` | DEFER | Noncritical portability requirement can be sequenced later. |
| 170 | NFR-CIV-REL-001 | NFR-CIV-REL | `docs/reference/non-functional-requirements.md` | DEFER | Reliability constraint text is not yet tied to immediate implementation dependency. |
| 171 | NFR-CIV-REL-002 | NFR-CIV-REL | `docs/reference/non-functional-requirements.md` | DEFER | Defer reliability refinement until active features and tests expose failures in this area. |
| 172 | NFR-CIV-REL-003 | NFR-CIV-REL | `docs/reference/non-functional-requirements.md` | DEFER | Operational resilience work is post-ship hardening in current stream. |
| 173 | NFR-CIV-REL-004 | NFR-CIV-REL | `docs/reference/non-functional-requirements.md` | DEFER | Non-blocking resilience behavior with no current implementation seed. |
| 174 | NFR-CIV-SCALE-001 | NFR-CIV-SCALE | `docs/reference/non-functional-requirements.md` | DEFER | Scale-related NFR text duplicates FR-level scaling concerns already in active BUILD-NEXT scope. |
| 175 | NFR-CIV-SCALE-003 | NFR-CIV-SCALE | `docs/reference/non-functional-requirements.md` | DEFER | Defer to FR scale implementation once FR-SCALE parity backlog is complete. |
| 176 | NFR-CIV-SEC-001 | NFR-CIV-SEC | `docs/reference/non-functional-requirements.md` | DEFER | Security requirement is important but requires architectural follow-up not yet planned for this tranche. |
| 177 | NFR-CIV-SEC-002 | NFR-CIV-SEC | `docs/reference/non-functional-requirements.md` | DEFER | Postpone as non-blocking security hardening relative to immediate parity closure. |
| 178 | NFR-CIV-SEC-003 | NFR-CIV-SEC | `docs/reference/non-functional-requirements.md` | DEFER | Low-priority security detail that can be sequenced with security backlog. |
| 179 | NFR-CIV-SEC-004 | NFR-CIV-SEC | `docs/reference/non-functional-requirements.md` | DEFER | No implementation hook in matrix snapshot; defer and re-triage when policy is anchored to code.
