# scripts/diag/ Inventory Audit (iter-144)

Audit date: 2026-05-20. Compares actual files in `scripts/diag/` against references in CLAUDE.md, `docs/`, and `~/.claude/projects/C--Users-koosh-Dino/memory/`.

## Healthy (exist + referenced)

- `game-state-probe.ps1` (4,962 B) — Autonomous game-state diagnostics (8 probes, JSON output) — referenced in CHANGELOG.md:67, memory/feedback_dino_launch_hang_universal.md:34, memory/project_iter143_session_retrospective.md:64,101
- `git-state-probe.ps1` (2,964 B) — Captures git working-tree + branch + stash + deploy-state as JSON — referenced inline (self-doc) at scripts/diag/git-state-probe.ps1:6,16
- `health-summary.ps1` (6,049 B) — Consolidated git + deploy + game + MCP + detector probe — referenced in docs/qa/codex-headless-reliability-notes.md:15, docs/proof/wgc-backend-verification-plan.md:60, docs/release/v0.25.0-CHANGELOG-snippet.md:6, memory/project_iter143_wave2_session_summary.md:49
- `launch-and-verify-dino.ps1` (13,716 B) — Launch DINO + verify 5 progressive health tiers (tolerates 3 unresponsive ticks) — referenced in memory/project_iter144_runtime_hang_root_cause.md:14,74
- `probe-menu-click.ps1` (10,690 B) — Autonomous E2E probe: verify main-menu UI responds to mouse clicks — referenced in docs/proof/iter-143-mcp-restart-receipt.md:70, docs/sessions/iter-143-autonomy-probe-design.md:5,89
- `release-readiness-check.ps1` (8,344 B) — Single-command release readiness probe for v0.25.0 — referenced in memory/project_iter143_wave2_session_summary.md:49

## Orphans (exist + unreferenced)

None. All six scripts have at least one external reference.

## Vaporware (referenced + missing)

- `scripts/diag/_mcp-tools-probe.ps1` — referenced in docs/proof/iter-143-mcp-restart-receipt.md:70 — NOT ON DISK (doc itself notes "temporary, removed at session end" — acceptable, but reference should be marked historical)
- `scripts/diag/wgc-e2e-test.ps1` — referenced in memory/project_v0.26.0_wave2_dispatch_plan.md:31 as planned WGC E2E artifact — NOT ON DISK (forward-looking plan entry, not yet implemented)

## Recommendation

1. Annotate the `_mcp-tools-probe.ps1` reference in `docs/proof/iter-143-mcp-restart-receipt.md` as "(transient, no longer on disk)" to prevent future readers from hunting for it.
2. Track `wgc-e2e-test.ps1` as an open v0.26.0 deliverable (Dino-v026-wgc-e2e worktree task) — it is a known plan item, not vaporware, but a stub `# TODO` script would prevent reference rot.
3. No orphan cleanup needed — all on-disk scripts have governance trails.
