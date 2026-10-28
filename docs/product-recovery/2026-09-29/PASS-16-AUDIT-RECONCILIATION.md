# Civis pass 16 — audit reconciliation and catalog quarantine

Date2026-09-30. Current main `590fad0643eb85cae89edd9e64ed6b991461de6e`; this commit adds only `docs/audits/spec-only-triage-2026-09-29.md` relative to54d57589.

## What the imported audit establishes

The audit adjudicates an existing 205-ID SPEC-ONLY inventory and reports:
-121 REAL-GAP;
-14 IMPLEMENTED-ELSEWHERE;
-7 NAMESPACE-COLLISION;
-23 MISFILED-TEMPLATE;
-13 MISFILED-REPORTING;
-12 MISFILED-DESIGN;
-15 SYNTHETIC.

These counts describe **that audit's inventory**, not the mature product contract. They are not a completion denominator and do not become requirements merely because the audit counted them.

## High-value falsifications to adopt

1. Coverage scanner misses root `FUNCTIONAL_REQUIREMENTS.md`, which contains177 SHALL statements. Any docs-only denominator is incomplete.
2. Tag-on-definition can count dead substrate as implemented; accessibility symbols are cited as concrete examples with no renderer/client consumers.
3. Named test suites in asset specs are absent.
4. The contiguous `FR-CIV-EMERGENCE-100..254` block was range-manufactured and later deleted; its surviving row-counting index falsely claims coverage. **Quarantine this entire generated range from grading until individual authored obligations are independently recovered.**
5. Rust-only scanning misses workflow/YAML implementation.
6. Existing code can contradict authored design, e.g. no-era-regression vs a requirement that adoption collapse can regress era.
7. A claimed `civlab` batch-analysis product surface (run lists/branch-from-tick/comparison/export) has no corresponding binary/run model in current code.

These align directly with this recovery program's rules: source tags are not implementation, generated IDs are not accepted obligations, and reachability matters.

## Authority decisions NOT silently resolved

The audit itself identifies questions that materially change the product contract:
- Is root `FUNCTIONAL_REQUIREMENTS.md` authoritative despite Draft status and an APPROVED PRD with no SHALL/MUST obligations?
- Are `docs/design/*` per-ID acceptance catalogues normative or design proposals?
- Is the historical `civlab` batch-analysis/run-management product still part of Civis, a predecessor product, or out of current scope?

This recovery does not answer those by counting files. Continue archaeology for explicit user/accepted-design evidence. If no decisive evidence exists, these become user-decision blockers before final semantic closure.

## Catalog policy update

Immediately exclude from grading:
- the synthetic155-ID emergence range;
- reserved/report-only IDs;
- template-only rows without authored obligations;
- namespace-colliding IDs until canonical owner resolved;
- dead/unmounted symbol bindings;
- implementation tags unsupported by mounted behavior.

Preserve real authored obligations as candidates with authority state, not automatic accepted requirements.

## Persistence relevance

The audit classifies FR-SAVE-006/009 as implemented-elsewhere, but our candidate-bound tests show state completeness failures outside those byte/hash criteria. This demonstrates why "save integrity requirement implemented" cannot be promoted to "save/load mature". Keep artifact integrity and semantic state completeness separate.
