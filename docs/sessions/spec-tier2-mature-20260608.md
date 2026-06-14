# spec-tier2-mature-20260608

Pointer for the Tier-2 Mature acceptance contract.

## Rule

Tier-2 passes only if Tier-1 already passes and the mature evidence bundle includes:

- 3+ BDD features or equivalent scenarios
- mutation score at or above the mature floor
- a chaos test that rejects at least one tampered input
- a performance baseline for the relevant path
- a load-test threshold that is explicitly declared and met

## Evidence List

- Canonical spec: [docs/specs/SPEC-TIER2-MATURE.md](../specs/SPEC-TIER2-MATURE.md)
- Tier-1 prerequisite: [docs/specs/SPEC-TIER1-MVP.md](../specs/SPEC-TIER1-MVP.md)
- Tier-1 session pointer: [docs/sessions/spec-tier1-mvp-20260608.md](spec-tier1-mvp-20260608.md)
- Tier-2 scenario surface: [docs/user-journeys-tier2.md](../user-journeys-tier2.md)
- Mutation evidence: [docs/mutation-score/index.md](../mutation-score/index.md), [docs/mutation-score/latest.json](../mutation-score/latest.json), [docs/sessions/mutation-bridge-20260608.md](mutation-bridge-20260608.md)
- Chaos evidence: [docs/sessions/chaos-test-add-20260608.md](chaos-test-add-20260608.md)
- Perf evidence: [docs/sessions/perf-baseline-20260608.md](perf-baseline-20260608.md)
- Load-test evidence: [docs/sessions/load-test-skeleton-20260608.md](load-test-skeleton-20260608.md)
- Traceability anchor: [docs/specs/traceability-matrix.md](../specs/traceability-matrix.md)

## Notes

- This is a docs-only pointer for the June 8, 2026 mature contract.
- The session note records the evidence surfaces; it does not claim a new runtime verification run.
