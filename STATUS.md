# Status

Last updated: 2026-05-28

## Build
Local-first CI active. All quality gates run locally and in CI on push/PR.

## Quality gates
- cargo-deny.yml: Monday 09:00 UTC cron + push/PR + workflow_dispatch
- codeql-rust.yml: Tuesday 04:17 UTC cron + push/PR + workflow_dispatch
- cargo-audit.yml: Wednesday 05:37 UTC cron + push/PR + workflow_dispatch
- pre-commit: client-side (cargo fmt + check + gitleaks)
- branch protection: 1 reviewer required, no force-push, dismiss stale
- quality-manifest: `.ci/quality-manifest.json` tracks per-crate gate thresholds

## Quality manifest
`.ci/quality-manifest.json` is the source of truth for gate thresholds and
enrolled crates. Updated as part of every phase completion.

## Live verification
Local quality gates run via `make quality` (cargo-deny + cargo-audit + clippy + fmt).
CI workflows enabled; billing-blocked status resolved as of 2026-05.

## Phase completion (as of 2026-05-28)
- P-V0 phenotype-voxel kernel: COMPLETE
- P-V1 voxel foundation: COMPLETE
- P-W1 tactical warfare (civ-tactics stub): COMPLETE
- FR-CORE-005/006 hash chain: IMPLEMENTED

## Cross-references
See phenotype-org-governance/SUPERSEDED.md for canonical authority.
See `docs/roadmap/plan-3d-phases.md` for per-phase completion status.
