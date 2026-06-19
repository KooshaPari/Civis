# Branch Consolidation Audit — 2026-06-10

**Scope:** every branch under `refs/heads` and `refs/remotes/origin` (object-database only; no worktree directories touched, no `E:/` paths enumerated).
**Method:**
1. `git for-each-ref --format='%(refname:short) %(objectname:short) %(committerdate:short)' refs/heads refs/remotes/origin` (172 refs).
2. For each: `git merge-base --is-ancestor <ref> origin/main` (origin/main tip = `af913fb25`).
3. Cross-ref: `gh pr list --state open --json number,title,headRefName,baseRefName,isDraft,state,url` (13 open PRs).
4. Authoritative verdicts supplied by task brief (squash-merge / wave1 / tactics-fog-of-war / terrain-fragmentation).

**Verdict legend**

| Tag | Meaning |
|-----|---------|
| `MERGED-DELETE-CANDIDATE` | Branch tip is an ancestor of `origin/main`; safe to delete. |
| `SUPERSEDED` | Work landed via squash/rebase on a different SHA; evidence supplied. |
| `ACTIVE` | Backed by an open PR (334–356) or current critical-path lane. |
| `UNKNOWN-PARK` | Local-only WIP / parked, not yet classified; no current PR. |

---

## Known verdicts (authoritative, from task brief)

| Branch | Verdict | Evidence |
|--------|---------|----------|
| `fix/tactics-fog-of-war-wire-in` | **SUPERSEDED** | Landed via PRs #310–#313 / #331. Byte-identical `crates/tactics/` on main. |
| `fix/terrain-fragmentation` | **SUPERSEDED** | All 4 fixes on `af913fb25`; verdict 2026-06-10. |
| `fix/terrain-fragmentation-ship` | **SUPERSEDED** | All 4 fixes on `af913fb25`; verdict 2026-06-10. |
| `fix/terrain-fragmentation-ship-2` | **SUPERSEDED** | All 4 fixes on `af913fb25`; verdict 2026-06-10. |
| `fix/terrain-fragmentation-v2` | **SUPERSEDED** | All 4 fixes on `af913fb25`; verdict 2026-06-10. |
| `fix/terrain-fragmentation-v3` | **SUPERSEDED** | All 4 fixes on `af913fb25`; verdict 2026-06-10. |
| `fix/client-terrain-fragmentation-ship` | **SUPERSEDED** | All 4 fixes on `af913fb25`; verdict 2026-06-10. |
| `feat/civis-wave1-emergence` | **MERGED** | Squash-merged via #333 → `af913fb25` (current `origin/main` tip). |
| `merge-333-arena` | **MERGED-DELETE-CANDIDATE** | Refers to PR #333 merge commit `8d2c8c17`; pre-squash. |

---

## MERGED-DELETE-CANDIDATE (local)

Tip is an ancestor of `origin/main` (`af913fb25`). Safe to delete; no PR open.

| Branch | Tip SHA | Date |
|--------|---------|------|
| `chore/dependabot-sweep-20260610` | `a8b19bdb` | 2026-06-08 |
| `chore/dependabot-sweep-20260610b` | `af913fb2` | 2026-06-10 |
| `chore/dependabot-sweep-20260610c` | `af913fb2` | 2026-06-10 |
| `chore/gitignore-target-civis-mcp` | `a8b19bdb` | 2026-06-08 |
| `chore/gitignore-target-civis-mcp2` | `a8b19bdb` | 2026-06-08 |
| `chore/trace-2026-06-09-matrix` | `a8b19bdb` | 2026-06-08 |
| `docs/branch-consolidation` | `af913fb2` | 2026-06-10 (this branch — keep until PR merges) |
| `docs/readme-workstate-20260609` | `a8b19bdb` | 2026-06-08 |
| `docs/readme-workstate-20260609b` | `a8b19bdb` | 2026-06-08 |
| `docs/readme-workstate-20260610` | `a8b19bdb` | 2026-06-08 |
| `docs/readme-workstate-20260610c` | `a8b19bdb` | 2026-06-08 |
| `feat/emergence-dashboard` | `a8b19bdb` | 2026-06-08 |
| `feat/holocron-ui-pass` | `af913fb2` | 2026-06-10 |
| `feat/p2-visible-citizens` | `af913fb2` | 2026-06-10 |
| `feat/verify-harness` | `a8b19bdb` | 2026-06-08 |
| `fix/asset-root-fallback` | `af913fb2` | 2026-06-10 |
| `fix/ci-billing-guard-alert-sync` | `a8b19bdb` | 2026-06-08 |
| `fix/client-playable-build-ship` | `a8b19bdb` | 2026-06-08 |
| `fix/client-terrain-fragmentation-ship` | `a8b19bdb` | 2026-06-08 |
| `fix/standalone-modules` | `af913fb2` | 2026-06-10 |
| `gate/main-verify` | `af913fb2` | 2026-06-10 |
| `p-v0-shared-crate` | `9e441865` | 2026-06-05 |
| `p-v0-wsm3d-bridge` | `710ec042` | 2026-06-05 |
| `p-v1-civis-integration` | `9e441865` | 2026-06-05 |
| `perf/ca-dirty-chunk` | `a8b19bdb` | 2026-06-08 |
| `perf/ca-dirty-chunk-fresh` | `a8b19bdb` | 2026-06-08 |
| `perf/ca-dirty-chunk-v2` | `a8b19bdb` | 2026-06-08 |
| `perf/ca-dirty-chunk-v3` | `a8b19bdb` | 2026-06-08 |
| `play/main-verify` | `af913fb2` | 2026-06-10 |
| `play/main-verify-pgg` | `af913fb2` | 2026-06-10 |
| `worktree-agent-ab0ac483e486512e0` | `9124d06f` | 2026-05-28 |

---

## ACTIVE — backed by open PRs (334–356)

| Branch | PR | Draft | Title |
|--------|----|----|-------|
| `chore/dependabot-frontend-2026-06-05` | #334 | No | chore(web): vite 6 + esbuild 0.25 (CVE-2026-39365) |
| `chore/dependabot-sweep-g` | #353 | Yes | chore(deps): dependabot CVE sweep (rmcp + aws-config) |
| `chore/hygiene-20260610` | #355 | Yes | chore(repo): ignore agent scratch debris (bat/py helpers) |
| `chore/launch-ergonomics` | #352 | Yes | chore(launch): encode verified boot incantation |
| `docs/readme-workstate-20260610` | #348 | No | docs(readme): refresh work-state header |
| `docs/traceability-20260610` | #349 | Yes | docs(trace): 2026-06-10 workstream traceability matrix |
| `feat/emergence-dashboard-v2` | #350 | Yes | feat(emergence): scaffold civ-emergence-metrics crate + dashboard design |
| `feat/session-persistence` | #356 | Yes | feat(server,engine): session persistence — autosaver, load-on-launch |
| `feat/verify-harness-v2` | #351 | Yes | feat(verify): civis-cli verify/pixels/census harness (post-wave-1) |
| `fix/governance-gate-cache-bypass` | #337 | No | fix(ci): bypass pr-governance-gate actions/github-script SHA cache |
| `fix/reusable-caller-permissions` | #347 | Yes | fix(ci): align phenoShared caller permissions with reusable requirements |
| `perf/ca-dirty-chunk-g` | #354 | Yes | perf(voxel): dirty-chunk CA stepping + incremental remesh wiring |

Note: `chore/dependabot-rust-2026-06-05` (`97495d76`, 2026-06-05) has no open PR at the time of audit; flag as `ACTIVE` candidate — re-confirm before deletion.

---

## UNKNOWN-PARK (local-only WIP, no open PR)

These are local branches with no open PR. The audit did not finish cross-referencing each; per task brief, list with one-line note and leave for human triage.

| Branch | Tip SHA | Date | Note |
|--------|---------|------|------|
| `audit/terrain-fragmentation-relevance` | `53c5d496` | 2026-06-01 | One-off audit scratch; not merged. |
| `chore/bevy-omniroute-parallel` | `63c07ff0` | 2026-05-27 | WIP parallel-session work. |
| `chore/parallel-session-sync` | `9f41bda2` | 2026-05-25 | Older parallel-session WIP. |
| `chore/regen-quality-manifest-2026-06-05` | `f2e0c2cd` | 2026-06-05 | Quality manifest regen scratch. |
| `chore/tech-debt-sweep` | `a41bebc5` | 2026-05-25 | Tech-debt sweep WIP. |
| `ci/local-first-manifest-verify` | `62fb8aa3` | 2026-05-30 | CI manifest-verify lane. |
| `civis-pbr` | `80dfec52` | 2026-05-31 | PBR art-experiments; parked. |
| `docs/p-p1-kickoff` | `1f5d0d62` | 2026-05-28 | P-P1 kickoff docs. |
| `docs/sync-status-2026-05-28` | `275fcf95` | 2026-05-28 | Old status-sync docs. |
| `feat/astar-obstacle-pathfinding` | `d972ba1a` | 2026-05-25 | A* pathfinding WIP. |
| `feat/civis-bevy-game` | `0f5c0133` | 2026-05-29 | Bevy game scaffold. |
| `feat/civis-life-sim` | `d9699bf9` | 2026-05-29 | Life-sim exploration. |
| `feat/civis-pbr2-triplanar` | `92fd460d` | 2026-06-01 | Triplanar PBR exploration. |
| `feat/civis-theme-fix` | `ea20dbe5` | 2026-06-01 | Theme/UI scratch. |
| `feat/extract-topic2-sim-crates` | `714d02d2` | 2026-06-10 | Active local-only extraction WIP (topic 2 sim-crates). |
| `feat/fog-of-war` | `b603ced7` | 2026-05-25 | Fog-of-war WIP. |
| `feat/frecon005-allocation` | `9f01e138` | 2026-06-10 | Frecon005 allocation WIP. |
| `feat/mcp-verify-tools` | `daaf95ac` | 2026-06-10 | Local MCP tool branch; PR #357 is DRAFT. |
| `feat/p-l1-kickoff` | `72eab2b0` | 2026-06-03 | P-L1 kickoff. |
| `feat/p-p1-fr040-geology` | `f64c44bc` | 2026-05-28 | FR040 geology WIP. |
| `feat/p-w1-bevy-gameplay-026` | `a3837475` | 2026-05-27 | Older Bevy gameplay slot. |
| `feat/p-w1-bevy-item-027` | `76994efa` | 2026-05-29 | Older Bevy item slot. |
| `feat/p-w1-civsave-zst` | `f8cb9ca7` | 2026-05-26 | Civsave zst slot. |
| `feat/p-w1-float-flow` | `6f213dab` | 2026-05-26 | Float flow slot. |
| `feat/p-w1-mod-install` | `3c33ae11` | 2026-05-26 | Mod-install slot. |
| `feat/p-w1-tactics` | `0c3d6ee1` | 2026-05-25 | W1 tactics slot. |
| `feat/p-w1-tactics-002-los` | `aff3035d` | 2026-05-25 | W1 tactics LOS slot. |
| `feat/p-w1-tactics-009` | `f606e9a2` | 2026-05-25 | W1 tactics 009 slot. |
| `feat/p-w1-tactics-010` | `3eab4e6d` | 2026-05-25 | W1 tactics 010 slot. |
| `feat/p-w1-tactics-011` | `29031589` | 2026-05-25 | W1 tactics 011 slot. |
| `feat/p-w1-tactics-012` | `613880e0` | 2026-05-27 | W1 tactics 012 slot. |
| `feat/process-compose` | `a41bebc5` | 2026-05-25 | Process-compose scratch. |
| `feat/tactics-ui` | `55ea726c` | 2026-05-25 | Tactics UI WIP. |
| `feat/war-bridge-los-formation` | `b6ccad48` | 2026-05-25 | War-bridge LOS formation. |
| `feature-extract/wave1-333` | `fa5b18ed` | 2026-06-07 | Pre-#333 extraction branch. |
| `fix/clippy-warnings` | `b29ef1d5` | 2026-05-27 | Clippy WIP. |
| `fix/justfile-check` | `71e8c651` | 2026-05-27 | Justfile-check WIP. |
| `fix/launch-asset-sync` | `db36c608` | 2026-06-09 | Launch-asset-sync WIP. |
| `fix/main-pr-governance-gate-sha` | `fa95ae83` | 2026-06-05 | Governance-gate SHA fix scratch. |
| `fix/pr-333-review` | `ee2e4b1e` | 2026-06-05 | PR #333 review fixes. |
| `fix/pr-governance-gate-set-vs-array` | `c6b28266` | 2026-06-05 | Governance-gate jq fix scratch. |
| `fix/tactics-fog-of-war-wire-in` | `17192ce0` | 2026-05-25 | **SUPERSEDED** (per task brief). |
| `fix/terrain-fragmentation` | `aded9a6b` | 2026-06-09 | **SUPERSEDED** (per task brief). |
| `worktree-agent-a1927ce343b95070a` | `7ab317c4` | 2026-05-30 | Agent worktree scratch. |
| `worktree-agent-a34821fb4f51136a2` | `9da27f29` | 2026-05-30 | Agent worktree scratch. |
| `worktree-agent-a69f408505c278af2` | `39df92ff` | 2026-05-30 | Agent worktree scratch. |
| `worktree-agent-a87f60567a413c821` | `f06ab2bc` | 2026-05-29 | Agent worktree scratch. |
| `wt/actor-y-fix` | `f98959db` | 2026-06-02 | Actor Y-fix WIP. |
| `wt/capability-enforce` | `713df55d` | 2026-05-27 | Capability enforcement WIP. |
| `wt/chunk-seam` | `974e7eaa` | 2026-06-04 | Chunk seam slot. |
| `wt/emergence-spawn` | `416d124b` | 2026-06-03 | Emergence spawn slot. |
| `wt/map-seed` | `a5faca8a` | 2026-06-03 | Map-seed slot. |
| `wt/map2d-ux-2494` | `625d98a3` | 2026-06-05 | Map2D UX #2494 slot. |
| `wt/map2d-zoom` | `f98959db` | 2026-06-02 | Map2D zoom slot. |
| `wt/mod-hot-reload` | `6659cc18` | 2026-05-26 | Mod hot-reload slot. |
| `wt/mod-publish-store` | `471bcfb5` | 2026-05-26 | Mod publish/store slot. |
| `wt/policy-mod-sdk` | `8d46f43e` | 2026-05-27 | Policy-mod SDK slot. |
| `wt/remote-mod-store` | `b74c1cd3` | 2026-05-27 | Remote mod-store slot. |
| `wt/rust-mod-verify` | `f054f71d` | 2026-05-26 | Rust mod-verify slot. |
| `wt/rust-tests` | `5a24aa0d` | 2026-05-26 | Rust tests slot. |
| `wt/save-session-db` | `a7d1e8cb` | 2026-05-27 | Save-session DB slot. |
| `wt/save-slot-rpc` | `30ab1a8f` | 2026-05-26 | Save-slot RPC slot. |
| `wt/session-saved-bus` | `808eb4c9` | 2026-05-27 | Session-saved bus slot. |
| `wt/tools-wire` | `b1b91002` | 2026-06-03 | Tools-wire slot. |
| `wt/ui-design` | `416d124b` | 2026-06-03 | UI design slot. |
| `wt/water-placement` | `f98959db` | 2026-06-02 | Water-placement slot. |
| `wt/web-remote-mod-ui` | `faf0707c` | 2026-05-27 | Web remote-mod UI slot. |
| `wt/web-save-slot-rpc` | `b39a18cc` | 2026-05-26 | Web save-slot RPC slot. |

---

## UNKNOWN-PARK (origin / remote-only, no local ref)

| Branch | Tip SHA | Date | Note |
|--------|---------|------|------|
| `origin/chore/codeql-pin-actions-2026-04-27` | `7e2ef2e6` | 2026-04-26 | Old CodeQL pin; remote only. |
| `origin/chore/dependabot-frontend-2026-06-05` | `d06ebe2d` | 2026-06-05 | Same tip as local #334 branch. |
| `origin/chore/dependabot-sweep-g` | `a16b37a6` | 2026-06-10 | Same tip as local #353 branch. |
| `origin/chore/enable-dependabot` | `9b93042f` | 2026-04-23 | Old dependabot enablement; remote only. |
| `origin/chore/expand-codeowners` | `de4c498e` | 2026-04-24 | Old CODEOWNERS expansion; remote only. |
| `origin/chore/hygiene-20260610` | `21207ffa` | 2026-06-10 | Same tip as local #355 branch. |
| `origin/chore/integrate-phenotype-docs` | `8642b7c2` | 2026-03-29 | Old phenotype docs; remote only. |
| `origin/chore/launch-ergonomics` | `4f4de585` | 2026-06-10 | Same tip as local #352 branch. |
| `origin/chore/parallel-session-sync` | `9f41bda2` | 2026-05-25 | Same tip as local. |
| `origin/chore/readme-scaffold-Civis` | `8b6abc5b` | 2026-06-05 | Old scaffold; remote only. |
| `origin/chore/regen-quality-manifest-2026-06-05` | `f2e0c2cd` | 2026-06-05 | Same tip as local. |
| `origin/chore/tech-debt-sweep` | `4ce0f8e5` | 2026-05-25 | Same tip as local (diverged from local copy). |
| `origin/ci/add-release-workflow` | `2ae29341` | 2026-04-24 | Old release workflow; remote only. |
| `origin/ci/cargo-deny-scheduled-scan` | `3e4f0975` | 2026-04-27 | Old deny-scan; remote only. |
| `origin/ci/local-first-manifest-verify` | `62fb8aa3` | 2026-05-30 | Same tip as local. |
| `origin/ci/pin-trufflehog` | `33be8eaa` | 2026-05-28 | Old trufflehog pin; remote only. |
| `origin/cursor/*` (14 branches) | various | 2026-04 → 2026-06 | Cursor IDE agent scratch branches on remote; cluster of governance/security script drafts. Human triage. |
| `origin/docs/p-p1-kickoff` | `1f5d0d62` | 2026-05-28 | Same tip as local. |
| `origin/docs/readme-workstate-20260610` | `8fb87693` | 2026-06-10 | Diverged from local (`a8b19bdb`); #348 is the active head. |
| `origin/docs/sync-status-2026-05-28` | `60a11aab` | 2026-05-28 | Diverged from local (`275fcf95`). |
| `origin/docs/traceability-20260610` | `197b0ed8` | 2026-06-10 | Same tip as local #349 branch. |
| `origin/feat/astar-obstacle-pathfinding` | `d972ba1a` | 2026-05-25 | Same tip as local. |
| `origin/feat/civis-3d-foundation` | `43d72f85` | 2026-05-25 | Old 3D foundation; remote only. |
| `origin/feat/emergence-dashboard-v2` | `c48738a3` | 2026-06-10 | Same tip as local #350 branch. |
| `origin/feat/journey-impl` | `8bce18cd` | 2026-05-02 | Old journey impl; remote only. |
| `origin/feat/p-l1-kickoff` | `9465f4d7` | 2026-05-29 | Diverged from local (`72eab2b0`). |
| `origin/feat/p-w1-bevy-gameplay-026` | `a3837475` | 2026-05-27 | Same tip as local. |
| `origin/feat/p-w1-bevy-item-027` | `76994efa` | 2026-05-29 | Same tip as local. |
| `origin/feat/process-compose` | `eaf8b717` | 2026-05-26 | Diverged from local (`a41bebc5`). |
| `origin/feat/session-persistence` | `9974dce8` | 2026-06-10 | Same tip as local #356 branch. |
| `origin/feat/verify-harness-v2` | `daaf95ac` | 2026-06-10 | Same tip as local #351 branch. |
| `origin/feature/civis-trufflehog` | `940b8c08` | 2026-05-03 | Old trufflehog feature; remote only. |
| `origin/fix/alert-sync-caller-permissions` | `45024d64` | 2026-06-10 | Old alert-sync caller perm fix; remote only. |
| `origin/fix/clippy-warnings` | `b29ef1d5` | 2026-05-27 | Same tip as local. |
| `origin/fix/governance-gate-cache-bypass` | `492657d8` | 2026-06-05 | Diverged from local (`0c849ad0`); #337 is the active head. |
| `origin/fix/justfile-check` | `71e8c651` | 2026-05-27 | Same tip as local. |
| `origin/fix/reusable-caller-permissions` | `2d34691b` | 2026-06-10 | Same tip as local #347 branch. |
| `origin/perf/ca-dirty-chunk-g` | `2af97496` | 2026-06-10 | Same tip as local #354 branch. |

---

## Summary counts

| Verdict | Local | Origin |
|---------|-------|--------|
| MERGED-DELETE-CANDIDATE | 31 | 0 |
| SUPERSEDED (terrain-fragmentation family) | 6 | 0 |
| SUPERSEDED (tactics-fog-of-war) | 1 | 0 |
| MERGED via squash (#333 wave1) | 1 (`feat/civis-wave1-emergence`) + 1 (`merge-333-arena`) | 0 |
| ACTIVE (backed by open PR 334–356) | 11 (with #357 draft flagged) | 12 (mirrors) |
| UNKNOWN-PARK | 65 (incl. SUPERSEDED above) | ~46 (incl. 14 cursor/* + same-tips) |

---

## FOLLOW-UP — worktree directory cleanup (DEFERRED)

Worktree directory cleanup (`G:\civis-wt-*`, `E:\civis-wt-*`, `C:\Users\koosh\Dev\civis-wt-*`) is **OUT OF SCOPE** for this audit and must wait for **E: drive replacement**. The E: drive is currently failing; many worktrees are unreachable. The worktree-hook constraint forbids editing non-primary worktrees without explicit user direction. Do not attempt cleanup until the drive is replaced and the user explicitly authorizes it.

Action items after E: drive replacement:
1. `git worktree list --porcelain` against the new drive.
2. Cross-check each worktree dir against this audit's MERGED-DELETE-CANDIDATE list.
3. `git worktree remove <dir>` (NOT `git worktree remove --force`) for each candidate, after confirming the branch has been removed from `refs/heads`.

---

## Notes on method

- All evidence drawn from object database: `refs/heads`, `refs/remotes/origin`, commit SHAs, PR metadata. No filesystem enumeration outside the repo. No `E:/` paths read.
- `git merge-base --is-ancestor` was run against `origin/main` (= `af913fb25`). Branches whose tip is not an ancestor but where the brief asserts SUPERSEDED are tagged SUPERSEDED with the brief's evidence.
- The `feat/civis-wave1-emergence` MERGED verdict is confirmed by `gh pr view 333 --json mergeCommit` (merge commit = `af913fb25`, which is the current `origin/main` tip — i.e. squash-merge).
- `chore/dependabot-rust-2026-06-05` is a local-only branch (no matching origin ref, no open PR) — flagged ACTIVE candidate pending confirmation.
