# Pattern Catalog v1.0 Roadmap

**Date:** 2026-05-19
**Status:** Draft roadmap for v1.0 stability declaration

## 1) Definition of Pattern Catalog v1.0 Stable

Pattern Catalog v1.0 is stable when the catalog is usable as a production quality quality-gate system, not just a static list. A pattern is considered stable if it is either:

- explicitly **RETIRED** (removed, migration approved, no replacement behavior needed), or
- actively enforced through the CI pipeline with acceptable baseline semantics: **HIGH=0**.

Stability therefore means both governance and enforcement, not just quantity:

- All catalog entries are either `RETIRED` or `CI-gated`.
- No uncategorized `ACTIVE` patterns remain after migration and deprecation review.
- Any CI-gated pattern that cannot yet be fully automated must be held with **HIGH=0** until analyzer support is implemented.
- Tier definitions and promotions must be synchronized with the catalog source-of-truth and diagnostic map.

### v1.0 target gates

- **Tier 1 analyzers:** cover **≥20** patterns
- **Tier 2 analyzers:** cover **≥15** diagnostic families (DF1001–DF1027 scope)
- **Tier 3 analyzers:** cover **≥30 properties** under fuzz/property contracts
- **Non-compliant pattern states:** none, except `RETIRED` or `HIGH=0`-gated backlog entries

## 2) Current State Assessment (snapshot)

| Tier | Current count | Notes |
|---|---:|---|
| Tier 1 | 16 analyzers | Below target (needs +4 to reach v1.0 gate) |
| Tier 2 | 25 analyzers | Within current DF1001–DF1027 range (27 IDs referenced), below full 27 only by 2 |
| Tier 3 | 152 properties | Well above minimum (30) |
| Pattern Catalog | 32+ patterns | Stable baseline currently incomplete relative to v1.0 gate definition |

## 3) Gap Analysis

### 3.1 Pattern-level enforcement gaps

- **Tier 1 gap:** 32+ catalog patterns total vs 16 Tier 1 coverage ⇒ roughly **16+ patterns** are not Tier 1-enforced.
- **Tier 2 gap:** DF1001–DF1027 set implies **27 possible families**; current 25 means **2 missing analyzers/coverage gaps**.
- **Tier 3 gap:** 152 properties are covered, but no public property-matrix exists yet to confirm if high-risk properties in production schemas are all represented.

### 3.2 Required evidence for v1.0 sign-off

For each pattern, maintain an explicit status row:

- Pattern ID / Name
- Tier target (1 / 2 / 3)
- Enforcing artifact (analyzer/class + rule ID)
- CI link + failing example
- Exception rule (if HIGH=0)
- RETIRED decision (if applicable)

A pattern is blocked for v1.0 if any cell is missing.

### 3.3 Priority gap list for execution

- Promote 4 patterns to Tier 1 to reach minimum coverage.
- Investigate the 2 DF1001–DF1027 gap points and either implement analyzers or mark as retired-with-justification.
- Produce an auditable property catalog-to-Tier3 mapping for the 152 properties.

## 4) Promotion Criteria (Tier 0 → Tier 1)

A Tier 0 pattern (regex-only detector) is promoted to Tier 1 only when:

1. **Signal stability:** detector has false-positive/negative characterization on a minimum representative corpus.
2. **Actionability:** each match can map to a concrete, auto-checkable condition and clear fix guidance.
3. **CI determinism:** analyzer can run in deterministic CI mode with bounded execution time.
4. **Remediation path:** every finding is actionable with documented remediation and expected severity.
5. **Governance approval:** owner signs off for severity classification and baseline semantics.

Baseline policy on promotion:

- Start as `HIGH=0` on first CI integration.
- Increase to `HIGH>0` only after one clean release cycle with stable signal and accepted false-positive rates.

## 5) Sunset Criteria (RETIRED vs HIGH=0 maintenance)

### RETIRED
Retire a pattern when one of the following is true:

- Feature/procedure is fully removed from the platform.
- Pattern semantics are replaced by a stronger Tier 1/Tier 2/Tier 3 signal with strict migration coverage.
- Enforcement cost is not feasible and no product impact requires enforcement.
- Security, legal, or build-safety constraints remove the original scope.

### Maintain at HIGH=0
Keep a non-retired pattern at HIGH=0 when:

- Pattern remains behaviorally valid but needs additional false-positive hardening.
- Analyzer exists but still depends on external signal quality.
- Migration/implementation risk is high but bounded and scheduled for next release.

Retired patterns must include a migration note and removal/justification date in the catalog entry.

## 6) v1.1 Stretch (post-v1.0)

### Planned expansion

- Grow Pattern Catalog to **260+ patterns**.
- Complete full DF1001–DF10xx family closure where feasible (including any remaining gaps uncovered during v1.0).

### New quality track

- Add **Tier 4 mutation testing** for a high-value subset:
  - mutation operators for common anti-pattern variants,
  - checker-sensitivity scoring,
  - confidence calibration for Tier 1 and Tier 2 regressions.

### v1.1 readiness definition

- v1.0 stable criteria met first.
- Tier coverage expansion is measured against a frozen list of additions.
- Mutation outcomes are reported in CI and treated as a separate pass/fail gate.

## Decision log (single-source)

- Date: 2026-05-19
- Document intent: publish this as the Pattern Catalog v1.0 acceptance framework.
- Governance: this roadmap is the authoritative source for pattern-state transitions until superseded by v1.1.
