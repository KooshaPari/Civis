# Spec-Only IDs Deferred by P2 Agent-B

This file tracks IDs from the P2 agent-B slice
(`docs/audits/P2-impl-slice/P2-agent-B.md`) that were **deferred**
rather than implemented, because their spec depends on infrastructure
that is not yet present in the workspace.

## FR-CIV-EMERGENCE (15 IDs deferred)

The 15 IDs in `FR-CIV-EMERGENCE-{100..254}` listed under the slice are
all sub-rows of a 155-row `emergent-systems-tracelinks.md` ledger that
maps each of 11 emergent subsystems (civ-linguabridge, civ-factions,
civ-religion, civ-market, civ-urban, civ-climate, civ-econ,
civ-demographics, civ-psyche, civ-legends, civ-ai, civ-culture,
civ-social, civ-diplomacy, civ-laws) to a batch row range. The ledger's
own closing line states:

> The 11-systems × 30-couplings matrix documented above is the **test
> surface** that promotes each of these 158 dormant IDs to `covered`
> status (i.e., spec + code + test triple).

That matrix is **not built** in this worktree — the couplings code,
the per-system batch-row implementations, and the cross-system event
harness they would need do not exist. Implementing each of the 15 IDs
without that matrix would produce untestable stubs, which violates the
"no dormant IDs after this pass" spirit of the P2 fan-out.

### Deferred IDs (15)

| ID | Spec/trace reference | Reason |
|---|---|---|
| FR-CIV-EMERGENCE-100 | `docs/traceability/emergent-systems-tracelinks.md:150` (civ-linguabridge) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-111 | `docs/traceability/emergent-systems-tracelinks.md:151` (civ-factions) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-119 | `docs/traceability/emergent-systems-tracelinks.md:152` (civ-religion) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-124 | `docs/traceability/emergent-systems-tracelinks.md:153` (civ-market) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-132 | `docs/traceability/emergent-systems-tracelinks.md:154` (civ-urban) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-141 | `docs/traceability/emergent-systems-tracelinks.md:155` (civ-climate) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-144 | `docs/traceability/emergent-systems-tracelinks.md:156` (civ-econ) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-151 | `docs/traceability/emergent-systems-tracelinks.md:157` (civ-demographics) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-168 | `docs/traceability/emergent-systems-tracelinks.md:158` (civ-psyche) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-198 | `docs/traceability/emergent-systems-tracelinks.md:159` (civ-legends) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-221 | `docs/traceability/emergent-systems-tracelinks.md:160` (civ-ai) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-236 | `docs/traceability/emergent-systems-tracelinks.md:161` (civ-culture) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-239 | `docs/traceability/emergent-systems-tracelinks.md:162` (civ-social) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-241 | `docs/traceability/emergent-systems-tracelinks.md:163` (civ-diplomacy) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-249 | `docs/traceability/emergent-systems-tracelinks.md:164` (civ-laws) | Requires 11-systems × 30-couplings matrix |

### Next steps for these IDs

1. Build the 11-systems × 30-couplings matrix as a `civ-emergence-couplings`
   crate (or extend `crates/engine/src/emergence_coupling.rs`).
2. Implement each system's batch-row code (`crates/agents`, `crates/social`,
   `crates/diplomacy`, `crates/laws`, `crates/legends`, etc.) against
   the matrix.
3. Re-run the P2 fan-out for these IDs as `BUILD-NEXT` once the matrix
   exists.