# SPEC-TIER2-MATURE: Tier-2 Mature Acceptance Contract

**Status**: Draft
**Version**: 1.0
**Date**: 2026-06-08
**Last Updated**: 2026-06-08
**Scope**: spec -> coverage -> autograder pipeline mature acceptance contract

---

## Overview

This specification defines the mature acceptance boundary for the spec -> coverage -> autograder pipeline.

Tier-2 is a strict superset of Tier-1:

- Tier-1 MVP must already pass.
- Tier-2 adds a broader behavioral evidence set, quality gates, and operational proof.
- A Tier-2 pass therefore implies a Tier-1 pass.

This contract stays focused on evidence, not implementation. It does not define orchestration internals, scoring engine mechanics, or release policy plumbing.

---

## Tier-2 Pass Rule

The Tier-2 contract MUST pass when all of the following are true:

1. The Tier-1 MVP contract passes first.
2. The contract includes at least 3 BDD features, or an equivalent set of scenarios with the same traceability strength and behavioral coverage.
3. The mutation score meets or exceeds the mature floor of 85%.
4. At least one chaos test proves the system rejects a tampered input, receipt, bundle, or equivalent corruption vector.
5. A performance baseline exists and records the measured baseline for the relevant path.
6. A load-test threshold exists and the recorded result meets that threshold.
7. The autograder can resolve the full evidence bundle without manual intervention.

If the tier-2 evidence bundle is present but the Tier-1 MVP contract is not satisfied, Tier-2 does not pass.

---

## Tier-2 Required Evidence

The Tier-2 contract requires these evidence categories:

1. `docs/specs/SPEC-TIER1-MVP.md`
2. `docs/specs/SPEC-TIER2-MATURE.md`
3. `docs/sessions/spec-tier1-mvp-20260608.md`
4. `docs/sessions/spec-tier2-mature-20260608.md`
5. `docs/user-journeys-tier2.md` or an equivalent scenario bundle with at least 3 independent acceptance paths
6. A mutation evidence artifact that records the score and pass/fail result
7. A chaos evidence artifact that shows at least one tamper rejection
8. A perf baseline artifact for the relevant path
9. A load-test artifact that declares and satisfies the threshold
10. One traceability row in `docs/specs/traceability-matrix.md` or the repo's canonical traceability table

The evidence can live in multiple docs, but the contract is only satisfied when the autograder can resolve them as one coherent bundle.

---

## Tier-2 Required Tests

The Tier-2 contract requires evidence from the following test categories:

1. At least 3 BDD features, or equivalent scenario coverage with the same behavioral strength.
2. At least one mutation test run that produces a score at or above the mature floor.
3. At least one chaos test that tampers a real payload and proves rejection.
4. At least one performance baseline run for the relevant code path.
5. At least one load-test run with an explicit threshold and a passing result.

The contract does not require every individual test to be a single test method. A scenario bundle, fixture matrix, or test suite may satisfy the requirement if the traceability is explicit.

---

## Acceptance Criteria

- [ ] Tier-1 MVP is a prerequisite and passes first.
- [ ] At least 3 BDD features or equivalent scenarios are documented and traceable.
- [ ] Mutation score is documented at 85% or higher.
- [ ] At least one tamper-oriented chaos test is documented and passes.
- [ ] A performance baseline is recorded for the relevant path.
- [ ] A load-test threshold is recorded and met.
- [ ] The autograder can resolve the full Tier-2 evidence bundle.

---

## Non-Goals

- Defining the acceptance-contract-engine implementation
- Defining the scoring algorithm internals
- Requiring a specific framework for BDD or scenario authoring
- Requiring a specific benchmark library or load-test runner
- Replacing the Tier-1 MVP contract

---

## Notes

This document establishes the mature contract boundary. Tier-2 is intentionally stricter than Tier-1 and inherits Tier-1 as a prerequisite rather than a separate option.
