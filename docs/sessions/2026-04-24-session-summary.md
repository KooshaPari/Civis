# Verification Honesty Session — 2026-04-24

Single session, ~3-hour cycle, multiple loop iterations on a 5-minute cron. User flagged that 1.5 months of agent transcripts across Factory Droid / Codex / Claude Code showed identical false-completion claims. Root cause: DINOForge's verification surface conflates *artifact existence* with *artifact validity*. This session opened that up and started fixing it from the bottom.

## What landed (real, on disk, build passes)

### New code
- `src/Tools/DinoforgeMcp/dinoforge_mcp/external_judge.py` (264 LOC) — `KimiJudgeTier` calling Moonshot vision API. **Refuses silent fallback when MOONSHOT_API_KEY missing** (raises `ExternalJudgeUnavailable`). Persists receipts to `docs/proof/judge-receipts/<timestamp>-<sha8>.json` in the repo, never `$env:TEMP`.
- `src/Tools/DinoforgeMcp/tests/test_external_judge.py` (208 LOC, 13 tests passing) — including the load-bearing `test_missing_key_raises` that prevents future silent-fallback regressions.
- `scripts/analysis/enumerate_mock_theater.py` (~100 LOC, stdlib-only) — strict regex-based enumerator for tautology-style tests. Runs in 2 seconds. Replayable.

### Code modifications
- `vision.py` — `analyze_screenshot(external_judge=True, external_judge_optional=False)` calls Kimi first when requested. Disagreement detection between external + local tier sets `disputed=True`. No auto-resolution.
- `server.py` — `game_analyze_screen(external_judge=...)` parameter forwarded. `game_launch(hidden=True)` / `game_launch_test` / `game_launch_vdd` now propagate `IsolationBackendBroken` / `NoWorkingIsolationBackend` errors with actionable messages pointing to the working DINOBox pool. The `asset_build` MCP tool aliased to `asset_prepare_for_unity` and emits a deprecation note in its return dict.
- `isolation_layer.py` — `HiddenDesktopBackend.KNOWN_BROKEN = True`, `launch_process` raises immediately (was silently crashing on D3D11 init), auto-detect chain refuses to fall through to a broken backend.
- `AssetSwapSystem.cs` — `LoadFirstAssetByType<T>` helper added (lines 541-555), call sites use it. Fixes the 0/36 Star Wars unit name-vs-bundle mismatch at the code level.
- `StatModifierSystem.cs` — `_activeModifications` cache field added; `Reapply()` was empty, now re-enqueues cached overrides; `OnUpdate` populates cache on first-attempt success only. Pack hot-reload of stat overrides is now functional.
- `prove-features-gate.ps1` — fails the build if no judge receipt was written in the last 15 minutes when `-ExternalJudge` is requested, OR if the most recent receipt has a `claude-*` / `codex-*` model field. Anthropic-family judges no longer count as "external."

### Documentation
- `docs/TRUTH_TABLE.md` — single-source per-feature ✅/🟡/❌ status with evidence pointers. The acceptance criterion for "is X actually working." 6 update sections capture the iteration history.
- `docs/sessions/2026-04-24-hidden-desktop-ground-truth.md` — verdict: hidden-desktop is fundamentally broken (no DXGI adapter on hidden Windows desktops; Unity D3D11 init crashes before window creation). The lpDesktop wiring was correct but the premise was wrong.
- `docs/sessions/2026-04-24-asset-swap-truth-audit.md` — verdict: 0 of 36 Star Wars unit visual swaps render at runtime; bundle-asset-name vs `visual_asset` YAML key mismatch; tests "pass" because they use `FakeAssetBundle`. Fix landed.
- `docs/sessions/2026-04-24-modplatform-lifecycle-truth-audit.md` — verdict: ModPlatform is REAL end-to-end. 0 stubs, 0 NotImplementedException, every lifecycle stage executes real work.
- `docs/sessions/2026-04-24-stat-modifier-truth-audit.md` — verdict: OnUpdate / ApplyImmediate / Bridge.applyOverride are REAL. Reapply was empty stub (now fixed). Uses `IncludePrefab` correctly.
- `docs/sessions/2026-04-24-mcp-tools-truth-table.md` — 21 of 26 game_* tools are REAL bridge calls, 1 Win32-direct, 4 process launchers, 0 stubs.
- `docs/sessions/2026-04-24-ci-workflow-truth-table.md` — 24 workflows, 0 launch the real game.
- `docs/sessions/2026-04-24-empty-stub-method-catalog.md` — 10 stubs across Runtime/SDK/Domains, all intentional no-ops.
- `docs/sessions/2026-04-24-playcua-vdd-truth-audit.md` — playCUA binary is missing on disk; `scripts/setup-vdd.ps1` referenced in error messages doesn't exist. Both classified as VAPORWARE pending real implementation.
- `docs/sessions/2026-04-24-hot-reload-truth-audit.md` — pack hot-reload + HMR signal watcher are REAL implementations. End-to-end session proof of a reload firing in a running game still missing.
- `docs/test-results/2026-04-24-honest-decomposition.json` — bucket distribution: 35.4% pure logic, 18.1% schema, 24.3% mock-bridge, 7.4% property/fuzz, 11.8% mock-theater (heuristic), 2.9% real game integration.
- `docs/test-results/mock-theater-strict-enumeration.json` — strict count: **6 of 2,536** (heuristic claim of 298 was 50× inflated). All 6 deleted, build passes, enumerator now reports 0.
- README.md — Verification Status block near top (does not bury); claims like "1,269 tests" and "20/20 CI green" replaced with honest characterizations.
- CHANGELOG.md — Unreleased entry under both `Fixed` and `Discovered` headings.
- CLAUDE.md — "Honesty about coverage" subsection under Testing Philosophy citing TRUTH_TABLE.md, mock-theater enumerator, and the spec-driven (not schema-driven) SDD definition.
- `.claude/commands/prove-features.md` — old "VLM Model Selection" section listing only Anthropic-family models replaced with explicit 3-tier judge ladder + disagreement gate + banned-phrases callout.
- Bridge.Client/README.md and Bridge.Protocol/README.md — Requirements section warning external NuGet consumers that a running modded game is needed; corrected Quick Start using real v0.24.0 method names (the previous version referenced types that don't exist).

### Memory
- `~/.claude/projects/.../memory/feedback_self_judging_proof_is_not_proof.md` — feedback memory inherited by future sessions. Banned phrases: "VLM-confirmed", "headlessly verified", "all features proven", "production-ready" (until the verification gap is closed). Mandatory: external judge receipt + replayable command before any "verified" claim.

## Pattern observed (the meta-finding)

Most of DINOForge's *implementation* is real and well-built. The rot is concentrated in three layers:

1. **Verification surface**: tests, judges, CI gates. Self-judging proof bundles. Mock-bridge integration tests that pass while the live game would fail. CI workflows that compile but never launch.
2. **Input data mismatch**: code is correct, but the data it consumes was generated with a different shape. The 0/36 asset swap is the canonical example — bundle internal asset name vs YAML `visual_asset` key.
3. **Recursive vaporware in recovery paths**: when a feature breaks, the error message points to a recovery script that itself doesn't exist. Recursion of the same failure mode at the next level up.

The audit-layer hallucination is real too. The first honest decomposition claimed 298 mock-theater tests; strict enumeration found 6. Even honest audits have heuristic-vs-enumerated gaps. **Every claim must be checked at the level above it.**

## What's still open

- **End-to-end Kimi judge receipts**: requires `MOONSHOT_API_KEY` from the user and one real game launch. Until then, the external judge tier exists but has produced no actual receipts.
- **Real-game CI runner**: 24 workflows, 0 launch the game. Either set up a self-hosted runner with DINO installed, or ship a CHANGELOG note that integration is dev-machine-only.
- **playCUA binary** (task #99): code wired, binary not built. Either build it in `C:\Users\koosh\playcua_ci_test\` if the cargo project exists, or drop the references entirely.
- **VDD setup-vdd.ps1** (task #100): pieces of false reference cleaned up. Writing the actual script is separate work, parked as TBD.
- **AssetSwap render verification** (task #101): code fix landed, build passes; needs a real game launch + Kimi receipt to confirm 0/36 → ~36/36.
- **Pack hot-reload session proof** (task #98): implementation real, but no session log captures a reload firing in a running game with before/after evidence.
- **12 stub bundles in warfare-starwars**: real bundle building requires Unity Editor batch mode. Currently honest about not existing.

## Recommended next actions

1. Set `MOONSHOT_API_KEY` in your shell, then run `prove-features` against any one feature to land the first external judge receipt. That receipt is the proof DINOForge's verification has actually graduated.
2. Once a receipt exists, update README.md's "Verification Status" block to reference it as the example. Concrete artifact > prose.
3. Decide on playCUA: either commit to building the binary, or drop the references entirely from CLAUDE.md and the deprecation messages. The half-state is the worst of both.
4. Decide on real-game CI: a self-hosted runner with DINO + DINOForge-deployed is the real fix. Document explicitly if it's deferred.

## Tasks closed this session

#85, #86, #87, #88, #89, #90, #91, #92, #93, #94, #95, #96, #97, #100, #102 — 15 tasks completed.

## Tasks still open

#98 (hot-reload session proof, needs real game), #99 (playCUA binary decision), #101 (asset swap render verification, needs real game).
