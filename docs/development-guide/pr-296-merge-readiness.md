# PR #296 merge readiness (`feat/civis-3d-foundation`)

**PR:** https://github.com/KooshaPari/Civis/pull/296  
**Branch:** `feat/civis-3d-foundation`

## Local-first CI (required for merge)

**Billing bypass (repo policy):** Pull requests only run `quality-manifest` and `pr-governance-gate`. All other workflows are `workflow_dispatch` / `push: main` only. Label the PR `local-first-ci` to ignore legacy red checks in the governance gate.

GitHub-hosted checks may fail immediately when org **Actions spending limits** are hit. Treat the committed manifest as the source of truth:

```bash
lefthook run pre-push
# or: pwsh scripts/quality/emit-quality-manifest.ps1
bash scripts/quality/verify-quality-manifest.sh
```

Cloud job **quality-manifest (cloud verify)** only runs `verify-quality-manifest.sh` (no Rust/Node on the runner).

## Gates attested in `.ci/quality-manifest.json`

| Gate | Local command |
|------|----------------|
| `civis_3d_verify` | `just civis-3d-verify` |
| `web_test` | `cd web && npm test` |
| `dashboard_typecheck` | `cd web/dashboard && bun run typecheck` |

Stop `civ-watch` before `cargo build` if Windows locks `civ-watch.exe`.

## P-U1 + waves landed on this branch

- **P-U1:** spawn palette (civilian, vehicle, airport, port, hangar), drag-place, convoy, Godot + web L2 authoring
- **Wave 15–17:** trade routes, save/load, terrain paint, building/humanoid visuals, Unreal module scaffold
- **ADR-009:** web spectator + optional Babylon renderer (`FR-CIV-WEB-007`)

## Optional CI (non-blocking)

| Workflow | Note |
|----------|------|
| `unreal-build.yml` | `continue-on-error: true`; exit `2` without UE on hosted runners |
| `quality-full` | `workflow_dispatch` only — full Rust/web sweep |
| Legacy `quality-gate`, `cargo-deny`, CodeQL | May fail on billing; not required if branch policy uses manifest job only |

## Merge checklist

1. Manifest `git_sha` matches `HEAD` or manifest-only parent (`HEAD^`).
2. `just civis-3d-verify` green locally.
3. Rebase on `main` if needed; resolve conflicts in `Cargo.toml` / docs only.
4. Squash or merge per repo convention; tag follow-up: P-W1 tactics integration PR.

## After merge

- Open child PRs from `main` for **P-W1** (`crates/tactics` ↔ engine damage already wired).
- Finish **Unreal** on a machine with UE 5.4: `clients/unreal-show/scripts/build.ps1`.
- Close or retarget #296 tracking issues for billing-blocked workflows separately.
