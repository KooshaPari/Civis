# Branch Consolidation Plan — 2026-05-30

Read-only audit + safe disposition of local branches in `C:\Users\koosh\Dino`.
Main working tree branch `feat/unityexplorer-devtools-20260528` was NOT mutated.

## Remote state (confirmed clean)

`git ls-remote --heads origin`:
- `refs/heads/main` → `3edcde3a`
- `refs/heads/gh-pages` → `4d25c258`

Remote is still **main + gh-pages only** — no drift. (Many local branches show
`[origin/...: gone]` tracking — those remote refs were deleted in the earlier
cleanup; the local branches re-accumulated from worktree/cursor activity.)

Merge baseline used for "merged?" = `origin/main` = `3edcde3a`.

## Worktree-checked-out branches (LOCKED — never touch)

From `git worktree list`, these branches are checked out in a worktree and
cannot be deleted (git refuses). They are the integration + its inputs or
cursor lanes:

| Branch | Worktree |
|---|---|
| feat/unityexplorer-devtools-20260528 | C:/Users/koosh/Dino (MAIN — current) |
| integration/v0.27.0-reconcile-20260530 | .claude/worktrees/agent-a9e33634f2cf0b6f8 |
| fix/engine-ui-injection-race-20260529 | agent-a1062def00644b4b8 |
| epic027-loading-screen-20260529 | agent-abd49072c218fc6e7 |
| epic027-catalog-20260530 | .claude/worktrees/epic027-catalog |
| worktree-agent-a22f8720f8cf39912 | agent-a22f8720f8cf39912 |
| worktree-agent-a47111efa98b19c95 / a531060 / a75e007 / a7bd824 / ad0622040 / af112dc6 / afaaaa749 | locked worktrees |
| wip/merge-lane-a / -b / -c | cursor worktrees lane-a/b/c |
| agent/wt-gamelaunch / agent/wt-review / main | cursor wt-* worktrees |

## Disposition table

| Branch | Last commit | Merged into main? | Action | Reason |
|---|---|---|---|---|
| agent/coderabbit-main-config | 2026-05-26 | no | DEFER | CodeRabbit bot config; verify still needed |
| agent/wt-gamelaunch | 2026-05-27 | merged | KEEP | checked out in cursor worktree |
| agent/wt-merge | 2026-05-27 | merged | DELETE-MERGED | merged, not in a worktree |
| agent/wt-review | 2026-05-27 | merged | KEEP | checked out in cursor worktree |
| analyzers-cpd-iter145 | 2026-05-25 | merged | DELETE-MERGED | superseded, on main |
| docs/ci-workflow-bootstrap | 2026-05-29 | no | DEFER | not on main; verify content vs current ci.yml |
| docs/sonar-pr188-hotspots | 2026-05-25 | merged | DELETE-MERGED | on main |
| epic027-catalog-20260530 | 2026-05-29 | no | KEEP | integration input (worktree, locked) |
| epic027-loading-screen-20260529 | 2026-05-29 | no | KEEP | integration input (worktree, locked) |
| epic027-menu-takeover-20260529 | 2026-05-29 | no | KEEP | v0.27.0 epic027 input being reconciled |
| feat/mods-native-quick-panel-20260530 | 2026-05-30 | no | KEEP | feat/mods-* integration input |
| feat/v0.26.0-fireworks-kimi-judge | 2026-05-19 | merged | DELETE-MERGED | on main |
| feat/v0.26.0-implementation-wave-1 | 2026-05-21 | merged | DELETE-MERGED | on main |
| fix/engine-ui-injection-race-20260529 | 2026-05-29 | merged | KEEP | integration input (worktree, locked) |
| fix/governance-pr188 | 2026-05-25 | merged | DELETE-MERGED | on main |
| fix/handle-connect-iter142 | 2026-05-19 | merged | DELETE-MERGED | on main |
| fix/packloads-gamelaunch-post-merge | 2026-05-27 | merged | DELETE-MERGED | on main |
| fix/per-element-ui-bugs-20260529 | 2026-05-29 | no | KEEP | fix/per-element-* integration input |
| fix/pre-push-unit-tests | 2026-05-27 | merged | DELETE-MERGED | on main |
| fix/prove-features-gate-cleanup | 2026-05-27 | merged | DELETE-MERGED | on main |
| fix/sonar-pr188-blockers | 2026-05-25 | merged | DELETE-MERGED | on main |
| followup/post-pr188-followups | 2026-05-26 | merged | DELETE-MERGED | on main |
| refactor/mcp-sonar-cpd-dedupe | 2026-05-25 | merged | DELETE-MERGED | on main |
| safety/iter140-snapshot-2026-05-18 | 2026-05-18 | no | DEFER | safety snapshot; old but tracks origin/safety — keep as recovery point |
| safety/iter145-recovery-20260523-0432 | 2026-05-26 | no | DEFER | recovery snapshot; tracks remote — keep until v0.27.0 lands |
| snyk-fix-7389a3c59... | 2026-05-28 | no | DEFER | Snyk dep-fix not on main; verify if still relevant |
| snyk-fix-80168d90... | 2026-05-28 | no | DEFER | Snyk dep-fix not on main; verify |
| snyk-fix-9edfd94c... | 2026-05-28 | no | DEFER | Snyk dep-fix not on main; verify |
| stash/recovered-2026-05-19-1 | 2026-05-19 | no | DEFER | recovered stash WIP; tracks remote — preserve |
| stash/recovered-2026-05-19-2 | 2026-05-19 | no | DEFER | recovered stash WIP; preserve |
| stash/recovered-2026-05-19-3 | 2026-05-19 | no | DEFER | recovered stash WIP; preserve |
| wip/integrate-stashes | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/local-dirty-20260528 | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/merge-lane-a / -b / -c | — | — | KEEP | cursor worktree lanes (locked) |
| wip/merged-stashes | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/agent-merge-pr221-wip | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/agent-merge-temp | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/coord-stash | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/main-stash | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/pre-post-pr188-followup-a | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/pre-post-pr188-followup-b | 2026-05-27 | no | DEFER | not on main; small WIP, verify before delete |
| wip/stash/resume-stash | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/temp-stash-for-push | 2026-05-28 | no | DEFER | has thunderstore packaging feat; verify landed |
| wip/stash/wip-before-main-sync | 2026-05-27 | merged | DELETE-MERGED | on main |
| wip/stash/wip-local | 2026-05-27 | merged | DELETE-MERGED | on main |
| worktree-agent-a1062def... | 2026-05-29 | merged | DELETE-MERGED | dup of fix/engine-ui ref, merged, not a worktree branch |
| worktree-agent-a1b611e8... | 2026-05-30 | no | KEEP | #947 starwars bundle input (v0.27.0) |
| worktree-agent-a22f8720... | 2026-05-30 | no | KEEP | integration HEAD (worktree, locked) |
| worktree-agent-a3846aab... | 2026-05-03 | merged | DELETE-MERGED | old infra commit, on main |
| worktree-agent-a3f367ba... | 2026-05-30 | no | KEEP | update-check fix input (v0.27.0) |
| worktree-agent-a47111ef... / a531060 / a75e007 / a7bd824 / a9e33634 / aa86de5 / ab4bee0 / abd49072 / ac32d56 / ad0622040 / af112dc6 / af81dd18 / afaaaa749 | — | mixed | KEEP | locked worktrees OR v0.27.0 inputs at 3edcde3a base |
| worktree-agent-a4a14b79... | 2026-05-30 | no | KEEP | warfare-naval roster input (v0.27.0) |
| worktree-agent-a6a6ee7c... | 2026-05-30 | no | KEEP | aerial content input (v0.27.0) |
| worktree-agent-a846f691... | 2026-05-03 | merged | DELETE-MERGED | old infra commit, on main |
| worktree-agent-ab5df1fe... | 2026-05-03 | merged | DELETE-MERGED | old infra commit, on main |
| worktree-agent-ad246c04... | 2026-05-26 | merged | DELETE-MERGED | PR #188 merge, on main |
| worktree-agent-ae4b7434... | 2026-05-30 | no | KEEP | epic-027 changelog input (v0.27.0) |
| worktree-agent-ae83614d... | 2026-05-03 | merged | DELETE-MERGED | old infra commit, on main |

## Executed this session (DELETE-MERGED, safe, not in any worktree)

See command log below.

## Post-integration cleanup list (delete once v0.27.0 lands on main)

Once `integration/v0.27.0-reconcile-20260530` merges to main and its worktrees
are removed, delete:
- integration/v0.27.0-reconcile-20260530
- epic027-catalog-20260530, epic027-loading-screen-20260529, epic027-menu-takeover-20260529
- feat/mods-native-quick-panel-20260530
- fix/engine-ui-injection-race-20260529, fix/per-element-ui-bugs-20260529
- worktree-agent-a1b611e8 (#947), a3f367ba, a4a14b79, a6a6ee7c, ae4b7434, a22f8720 (#956/#957/#963 inputs)
- all remaining `worktree-agent-*` once their worktrees are pruned (`git worktree prune`)
- the cursor lanes wip/merge-lane-a/-b/-c + agent/wt-* once cursor sessions close

DEFER set to revisit after v0.27.0 (verify content landed, then delete or keep as recovery):
- agent/coderabbit-main-config, docs/ci-workflow-bootstrap
- safety/iter140-snapshot, safety/iter145-recovery (keep as recovery points)
- snyk-fix-* x3, stash/recovered-* x3
- wip/stash/pre-post-pr188-followup-b, wip/stash/temp-stash-for-push
