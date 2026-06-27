# Traceability

- [Implementation Status (workspace vs specs)](/IMPLEMENTATION_STATUS)
- [Traceability Matrix (strategic FR-CORE/ECON)](/traceability/TRACEABILITY_MATRIX)
- [3D extension matrix (FR-CIV-*)](/traceability/fr-3d-matrix)
- [Web spectator matrix (FR-CIV-WEB-*)](/traceability/fr-web-matrix)
- [Event Taxonomy](/traceability/EVENT_TAXONOMY)
- [Planning Gap Closure Matrix](/PLANNING_GAP_CLOSURE_MATRIX)
- [Planning Gap Status](/PLANNING_GAP_STATUS)


## Update 2026-06-27

- fr-emergence-matrix.md expanded: 16 §3.4 charter + 155 §3.1-3 batch + 4 §3.3 integration = **175 rows total** (was 16)


## CI / automation

- `.github/workflows/audit-fr-coverage.yml` runs `Tools/audit-fr-coverage/audit.sh` on every PR and pushes to `fix/workspace-bevy-ref-holocron-dep`. Coverage regression = workflow failure.