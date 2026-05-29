# Local Git State Audit

Generated from read-only inspection on 2026-05-28. No stash, branch, or worktree mutations were performed.

## Scope

- Current branch left untouched: `feat/unityexplorer-devtools-20260528`
- Commands inspected: `git stash list`, `git stash show -p --stat` for each stash, `git branch -vv`, `git worktree list`, `git status --short`, and `git log --oneline -5` for local-only branches.

## Snapshot

- Current branch: `feat/unityexplorer-devtools-20260528`
- Worktrees: 7 total, including 3 branch worktrees and 4 agent/worktree checkouts.
- Dirty index/worktree entries: 14 modified tracked files, 1 staged file, 1 untracked directory, and 16 untracked files/paths.
- Stashes: 2
- Local-only branches reviewed: 9

## Stashes

### `stash@{0}` - `lefthook auto backup`
- Age: about 14 hours old (2026-05-28 07:26:23 -0700)
- Contents observed: broad repo-wide WIP, including workflow hardening, new CDN/lazy-load architecture docs, runtime/SDK asset CDN code, CLI cache/telemetry changes, PackCompiler validation enhancements, lockfile refreshes, and test/coverage artifacts.
- Assessment: `NEEDS-INVESTIGATION`
- Why: the stash is very large and spans many unrelated areas, so it is not safe to treat as disposable without comparing against the current branch and other recovered WIPs.

### `stash@{1}` - `WIP on feat/unityexplorer-devtools-20260528: 0cf468b4 feat(devtools): bundle UnityExplorer as optional dev tool`
- Age: about 18 hours old (2026-05-28 03:36:54 -0700)
- Contents observed: very similar broad WIP surface to `stash@{0}`, with runtime/SDK/tooling/doc updates, plus test and coverage churn.
- Assessment: `PRESERVE-AND-MERGE`
- Why: the stash message ties it to the current feature branch, and the payload is clearly nontrivial unique work that has not been reduced to a clean commit on the branch tip yet.

## Local Branches

### `feat/unityexplorer-devtools-20260528`
- Upstream: none
- Age: tip commit is current branch head `72e6703f`; the stash history shows the same feature work is still in flight.
- Reachability: `NOT-reachable-from-origin-main`
- `git log -5` tail:
  - `72e6703f chore(deps): update packages.lock.json with MinVer resolution across all projects`
  - `6c0db874 fix(tests,packs): regen schema snapshot golden + bump framework_version upper bounds to <1.0.0`
  - `e3d5da4a fix(deploy): DeployToGame copies all DINOForge.*.dll, not just Runtime (#942)`
  - `1d26ad01 fix(packs): add badges/ui_theme/settings to schema + escape Spectre markup in pack names`
  - `ac5730f7 docs(claude): slim CLAUDE.md from 74.8k to <40k chars - relocate reference detail to docs/, preserve all governance rules`
- Assessment: `PRESERVE-AND-MERGE`
- Why: it is the current branch and contains work not yet reachable from `origin/main`.

### `feat/v0.26.0-fireworks-kimi-judge`
- Upstream: none
- Age: tip commit is older branch history, but the head is reachable from `origin/main`
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `96e0d69b test(bridge): skip ConnectAsync_WithCustomTimeout flaky with wave-2 timeouts (#543)`
  - `535b8f7a feat(diag): add health + release-readiness probes; iter-143 wave-2 receipts and UI assets`
  - `953ab9bc docs(qa): land iter-143 wave-2 audit reports`
  - `5621122e docs(release): add v0.26.0 forward-looking plans`
  - `8395bc8c docs(release): add v0.25.0 release-prep documents`
- Assessment: `ALREADY-MERGED`
- Why: branch-name is local only, but the tip is already contained in `origin/main`.

### `feat/v0.26.0-implementation-wave-1`
- Upstream: none
- Age: older wave-1 branch history; tip is reachable from `origin/main`
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `782c5dd2 fix(loader,lefthook): restore exception contract + drop over-strict gate (slice 8/8)`
  - `02b5755b chore: v0.26.0 wave-1 mop-up (slice 7/8 final)`
  - `617300c6 test(tests,scripts): pattern-gate verification + skip-guards (slice 6/8)`
  - `15556821 feat(runtime,domains,tools): iter-145 deploy + analyzer mark sweep (slice 5/8)`
  - `463c72a7 feat(sdk,bridge,analyzers): NuGet surface hardening v0.26.0 (slice 4/8)`
- Assessment: `ALREADY-MERGED`
- Why: the branch head is already in `origin/main`.

### `refactor/mcp-sonar-cpd-dedupe`
- Upstream: none
- Age: older refactor branch history; tip is reachable from `origin/main`
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `1e32b1a1 refactor(mcp): dedupe GameInputTool via GameInputHelper; expand Sonar CPD`
  - `15f6a4d1 fix(ci): polyglot restore like core CI; skip dup pattern/rust on PR`
  - `16506bec fix(ci): skip duplicate pattern workflows on PR; fix polyglot test restore`
  - `a4683ea5 fix(ci): polyglot global.json restore; changelog dedupe; pattern workflow continue-on-error`
  - `25292a7d fix(ci): use CancellationTokenSource.Cancel on netstandard; polyglot restore; orphan allowlist`
- Assessment: `ALREADY-MERGED`
- Why: the tip commit is already contained in `origin/main`.

### `worktree-agent-a3846aabec020ba7d`
- Upstream: none
- Age: history rooted at older infrastructure bootstrap work
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `6dcc193c chore: commit untracked infrastructure files`
  - `f0c02791 chore: bootstrap FUNDING.yml and trufflehog.yml (#186)`
  - `4403b7a7 chore(governance): add FUNDING.yml`
  - `663f6bda chore: add AGENTS.md and SECURITY.md [skip ci] (#185)`
  - `f8d87bdd ci: add trufflehog secrets scan`
- Assessment: `ALREADY-MERGED`
- Why: the branch points at a commit already reachable from `origin/main`.

### `worktree-agent-a846f691378b3c472`
- Upstream: none
- Age: same infrastructure bootstrap lineage as the other worktree-agent refs
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `6dcc193c chore: commit untracked infrastructure files`
  - `f0c02791 chore: bootstrap FUNDING.yml and trufflehog.yml (#186)`
  - `4403b7a7 chore(governance): add FUNDING.yml`
  - `663f6bda chore: add AGENTS.md and SECURITY.md [skip ci] (#185)`
  - `f8d87bdd ci: add trufflehog secrets scan`
- Assessment: `ALREADY-MERGED`
- Why: branch tip is already in `origin/main`.

### `worktree-agent-ab5df1fe8f361ca51`
- Upstream: none
- Age: same infrastructure bootstrap lineage as the other worktree-agent refs
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `6dcc193c chore: commit untracked infrastructure files`
  - `f0c02791 chore: bootstrap FUNDING.yml and trufflehog.yml (#186)`
  - `4403b7a7 chore(governance): add FUNDING.yml`
  - `663f6bda chore: add AGENTS.md and SECURITY.md [skip ci] (#185)`
  - `f8d87bdd ci: add trufflehog secrets scan`
- Assessment: `ALREADY-MERGED`
- Why: branch tip is already in `origin/main`.

### `worktree-agent-ad246c04efa32c334`
- Upstream: none
- Age: older PR188 recovery line
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `15ba282c Merge pull request #188 from KooshaPari/safety/iter145-recovery-20260523-0432`
  - `c512d24f fix(bridge): Protocol Sonar reliability PR188`
  - `c2b60a29 refactor(packcompiler): dedupe service helpers for Sonar`
  - `6accd6fa fix(runtime): ModMenuPanel explicit list pane layout (match proof gate tests)`
  - `dddc617c chore(sonar): expand CPD exclusions batch-3`
- Assessment: `ALREADY-MERGED`
- Why: the tip is already reachable from `origin/main`.

### `worktree-agent-ae83614d2361217ad`
- Upstream: none
- Age: same infrastructure bootstrap lineage as the other worktree-agent refs
- Reachability: `reachable-from-origin-main`
- `git log -5` tail:
  - `6dcc193c chore: commit untracked infrastructure files`
  - `f0c02791 chore: bootstrap FUNDING.yml and trufflehog.yml (#186)`
  - `4403b7a7 chore(governance): add FUNDING.yml`
  - `663f6bda chore: add AGENTS.md and SECURITY.md [skip ci] (#185)`
  - `f8d87bdd ci: add trufflehog secrets scan`
- Assessment: `ALREADY-MERGED`
- Why: branch tip is already in `origin/main`.

## Dirty State

Tracked changes:
- `lefthook.yml` staged
- 13 modified `packages.lock.json` files across `src/*`

Untracked / new:
- `tools/phenotype-journeys`
- `docs/design/*.md` new design docs
- `docs/qa/assetswap-real-bundles-spec.md`
- `docs/research/*.md` new research docs
- `docs/sessions/native-mods-page-diagnosis.md`
- `docs/specs/v0.27.0-full-conversion-epic.md`
- `docs/specs/v0.27.0/`
- `telemetry-viewer-task.txt`

## Classification Summary

- `PRESERVE-AND-MERGE`: 2
- `ALREADY-MERGED`: 8
- `DISCARDABLE`: 0
- `NEEDS-INVESTIGATION`: 1

## Notes

- The only branch that clearly needs preservation is the current branch plus the unreconciled WIP stash tied to it.
- The `lefthook auto backup` stash is the riskiest item because it spans many files and likely reflects automation-captured work rather than an intentional branch cut.
- No stash was applied, popped, or dropped.
