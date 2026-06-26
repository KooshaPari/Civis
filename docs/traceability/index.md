# Traceability

- [Implementation Status (workspace vs specs)](/IMPLEMENTATION_STATUS)
- [Coverage audit (946 untraced FR-IDs)](/traceability/COVERAGE_AUDIT)
- [Traceability Matrix (strategic FR-CORE/ECON)](/traceability/TRACEABILITY_MATRIX)
- [3D extension matrix (FR-CIV-*)](/traceability/fr-3d-matrix)
- [Web spectator matrix (FR-CIV-WEB-*)](/traceability/fr-web-matrix)
- [**Emergence systems matrix (FR-CIV-LANG/PSYCHE/POLITY/…)**](/traceability/fr-emergence-matrix)
- [**Non-functional requirements matrix (NFR-CIV-*)**](/traceability/fr-nfr-matrix)
- [Emergent systems tracelinks (coupling DAG)](/traceability/emergent-systems-tracelinks)
- [Event Taxonomy](/traceability/EVENT_TAXONOMY)
- [Planning Gap Closure Matrix](/PLANNING_GAP_CLOSURE_MATRIX)
- [Planning Gap Status](/PLANNING_GAP_STATUS)

## Acceptance-Contract Oracle

Each row in [`fr-emergence-matrix.md`](fr-emergence-matrix.md) and [`fr-nfr-matrix.md`](fr-nfr-matrix.md) includes an **Acceptance Contract** column: a concrete, machine-checkable pass/fail predicate (e.g. “same seed → identical phoneme sequence”, “tick P99 ≤ 80 ms at 10k agents”, “emergence_feed non-empty after 250 ticks”). That column is the **oracle hook** for cheap agent batch-iteration — agents implement or extend the named test pattern until the contract holds, without re-deriving requirements from prose. Status (`traced` / `code-only` / `stub` / `dormant`) tells the oracle whether code, spec, or tick wiring is the expected gap. This spine is the keystone called out in the AAA traceability audit: traceability matrices supply IDs and contracts; tests supply verdicts.
