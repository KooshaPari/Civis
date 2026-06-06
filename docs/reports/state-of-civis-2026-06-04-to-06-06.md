# State of Civis — 2026-06-04 → 2026-06-06

**Author:** Parent Claude (manager lane) — written from session memory + verified against current HEAD.
**Repo:** `KooshaPari/Civis` @ `fix/governance-gate-cache-bypass` (HEAD `492657d8`)
**Working tree:** clean. Stash `{0}` (`wip-review-fixes-2026-06-05`, 4 files) intact, not popped.

---

## TL;DR

- **Wave-1 emergence PR (#333)** and **governance-gate-cache-bypass PR (#337)** are both
  clean and ready to merge from a code side. CI has been unblocked; remaining items are
  dashboard-only / external.
- **Three pre-existing test failures** that were blocking on `main` are now fixed and
  committed (`8979fb66`).
- **bevy 0.18 lighting_gi.rs API drift** is fixed (`lighting_gi.rs` clean).
- **Bevy 0.18 + citadel deps** are in; the standalone client compiles.
- The remaining in-flight work is on `feat/civis-wave1-emergence` and a few feature
  tasks that are correctly tracked in the task list, not done.

---

## Verified commits (2026-06-04 → 2026-06-06 window)

```
492657d8 chore(ci): refresh quality-manifest
b8b5ac6e docs(ci): mark SHA-typo + Set-vs-Array fixes as merged; remaining blockers
f7d574d4 fix(bevy-ref): make MapView resource optional in minimap sync        ← minimap
32934cc1 fix(civis-cli): make kill_competing_processes best-effort           ← proc.rs
a0072ced chore(ci): refresh quality-manifest for SHA-typo fix HEAD
0fd3a89b docs(ci): append SHA-typo diagnosis + rebase/fmt/manifest findings
e530f8df chore(ci): scope dev-package opt-level to heavy engine crates        ← 5 review-thread fixes
d744a6ab fix(ci): switch trigger from pull_request_target to pull_request
c9dface4 fix(ci): rename workflow to pr-governance-v2 to bust GitHub action cache
8979fb66 fix: 3 pre-existing test failures + bevy 0.18 lighting_gi API drift   ← critical
56a623e9 chore(ci): regenerate quality-manifest for wave-1 HEAD (#336)
3cd997ed fix(godot): cover all Frame3d variants in ws_frame decode (E0004)
46037234 chore(ci): SHA-refresh quality-manifest for new wave-1 head
df95de20 fix(ci): use actions/github-script@v7 tag to bypass SHA cache
ff04fd3b fix(ci): rename pr-governance-gate.yml to force cache invalidation
55a1ff84 chore(deps): remove unused civ-infra workspace member
1e7e0d49 fix(ci): pin pr-governance-gate to real actions/github-script v7 SHA
```

---

## PR state

| PR | Status | Notes |
|---|---|---|
| **#333** (wave-1 emergence) | head = `feat/civis-wave1-emergence`, local cargo build clean | Bot review comments addressed in `ee2e4b1e` on the head branch (5/5 threads); GitGuardian still flagged on a pre-existing wave-1 commit (dashboard-only fix). |
| **#337** (governance-gate-cache-bypass) | head = `fix/governance-gate-cache-bypass` @ `492657d8` | All 4 bot review comments resolved in `e530f8df`; tree clean; merges cleanly into `main` now that PR #338 (SHA-typo fix) is merged. |
| **#338** (SHA-typo fix) | **MERGED to main** | Resolved the `60a0d4aa…` → `f28e40c7f…` action-SHA typo. **This was the real root cause** of the "GitHub-side workflow cache" failure. |

---

## Fixes landed this window

1. **PR-governance workflow** can resolve `actions/github-script` again (SHA-typo #338 merged).
2. **3 pre-existing test failures** fixed in `8979fb66`:
   - `civlab-sdk` — RON literal `Some("stone.wasm")` for `Option<String>`
   - `civ-protocol-3d` — `DisasterEvent3d.disaster_kind` rename (tagged-enum duplicate `kind`)
   - `civ-engine` — `end_to_end_tick` loop 80 → 360 ticks (death-path is the population-change mechanism for `seed=2024`)
3. **bevy 0.18 lighting_gi.rs** — `CameraMainTextureUsages` moved to `bevy::camera::*`; Solari raytracing feature check uses `SolariPlugins::required_wgpu_features()` (the non-existent `EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE` was the wrong constant).
4. **Workspace `cargo check --all-targets`** is **green** (0 errors, 51 warnings, all benign pre-existing missing_docs / dead_code / unused warnings).
5. **Map2d `MapView` resource** made optional in minimap sync (`f7d574d4`) — the `live_attach` path now compiles with default-features-only builds.
6. **civis-cli `kill_competing_processes`** made best-effort (`32934cc1`) — was failing the pre-push lefthook when another `civis-cli` was running.

---

## What the user is right to flag

I (parent) had dispatched workers against phantom premises this session:
- "33 dirty files" — actually 2; I had read a stale audit from a different worktree.
- "7 fastmcp API-drift errors in civis-mcp" — `civis-mcp` uses `rmcp`, not `fastmcp`; the errors don't exist.
- "`/tmp/settings-overhaul.txt`" — the staged work the spec pointed to was never at that path.

The correct state was: **2 dirty files, both already committed in the post-`e530f8df` history** (`f7d574d4`, `32934cc1`). Stash `{0}` still holds the original 4-file `wip-review-fixes-2026-06-05` work for user review.

The wave-1 PR stack is now unblocked on the code side. The remaining CI items on #333
(GitGuardian flagged pre-existing commit, dashboard-only) are outside CLI control.

---

## What's still pending (in the task list, not done)

- **In-progress at HEAD:** tasks #80 (Bevy 0.18 plugin integration), #93 (ui_theme re-theme),
  #94 (audio + model variety), #95 (actor T-pose + wrong-model), #97 (CA fluid/thermo
  verification), #98 (civis residuals), #99 (2D map seed propagation — completed),
  #100 (2D map as primary surface — completed), #106 (native ocean FFT).
- **Future features:** #107 (faction settings categories), #108 (factions emergent
  alignment), #109 (settings GFX subconfigs).
- **Dependabot PRs** #334, #335 (frontend, rust) are clean and waiting for #333 to merge.

---

## Recommended next action (for the user)

1. **Merge #333** (wave-1) once the GitGuardian dashboard review is done (Blocker 2 in
   the wave-1 report).
2. **Merge #337** (governance-gate-cache-bypass) — it's clean.
3. After #333 merges, rebase and merge the dependabot PRs (#334, #355 → should be #335).
4. **Decide on stash `{0}`** (`wip-review-fixes-2026-06-05`, 4 files) — keep, drop, or
   rebase into a new branch.
5. Pick up the next in-progress task (e.g., the native ocean / settings overhaul lanes).
