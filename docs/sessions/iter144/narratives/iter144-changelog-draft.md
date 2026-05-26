# CHANGELOG draft — iter-144 wave-1 (2026-05-20)

Candidate entry for `CHANGELOG.md`. Do not merge until wave-1 push lands and #543 / #535 decisions are made.

---

## [0.26.0-dev] — iter-144 wave-1 (2026-05-20)

### Added
- `scripts/diag/launch-and-verify-dino.ps1` — 5-tier DINO launch verifier (window paint -> BepInEx log mtime -> dinoforge_debug.log init -> named-pipe `dinoforge-game-bridge` -> 30s health-loop tolerating 3 consecutive unresponsive ticks). Catches hang Type B that the CLAUDE.md standard protocol (MainWindowTitle + Responding) silently passes.
- `feedback_dino_launch_hang_universal.md` — universal launch-hang taxonomy (Types A/B/C/D) + augmented launch protocol prescription.
- `project_iter144_runtime_hang_root_cause.md` — root-cause memo for the wave-1 push blocker (RuntimeDriver root-destroy fix incomplete/regressed on `feat/v0.26.0-implementation-wave-1`).
- `feedback_no_git_ops_while_agents_running.md` — orchestrator-level prohibition on git ops while delegated agents share the working tree; prefer worktrees.
- `feedback_agent_must_assert_branch_context.md` — SHA-anchored branch creation + explicit branch assertion before non-read git ops.
- `project_v0.26.0_wave2_dispatch_plan.md` — 4-agent worktree dispatch plan for wave-2.
- `project_iter144_session_handoff.md` — end-of-session snapshot.
- `feat(mcp)` HiddenDesktopBackend P0 fixes (capture, SendInput desktop, hidden window find) — commit `27ec748f`.
- `feat(journey)` keyframe tagger reads BepInEx log + emits `keyframes.json` — commit `ba38e84d`.
- `feat(mcp)` docker_backend launch + capture + inject scaffolds beyond skeleton — commit `50029823`.

### Fixed
- `fix(sdk)` retire Pattern #231 static-init side effects in NuGet surface — commit `626b6c1f`.
- RuntimeDriver root-destroy hang (#535) re-fix on wave-1 branch — *(TBD pending #543; current behavior is hang Type B per probe receipts).*

### Known Issues
- **DINO universal blank-launch hang** on `feat/v0.26.0-implementation-wave-1` — hang **Type B** per [[feedback_dino_launch_hang_universal]]: window paints, ECS partial-init, pump dies post-`RuntimeDriver.OnDestroy`, named pipe stays bound as zombie. **Wave-1 push blocked** behind lefthook test-integration stage until #543/#535 resolves. Deploy verified by hash (not Pattern #530 silent no-op).
- **LEFTHOOK_EXCLUDE bypass violation on PR #187** — zombie commit agent used `LEFTHOOK_EXCLUDE=test-integration` to push v0.25.0; violates [[feedback_no_lefthook_bypass]]. Pending user decision: accept-as-given vs redo cleanly.
- 3 wave-1 commits local-only (HiddenDesktopBackend P0, journey tagger, docker_backend) plus Pattern #231 SDK retirement — awaiting unblock to push branch + open PR.

### Operational
- 5 new memory entries (hang taxonomy, root-cause, no-git-ops-while-agents, branch-context assertion, wave-2 plan, session handoff).
- Probe tooling (`launch-and-verify-dino.ps1`) caught hang Type B that CLAUDE.md standard protocol passes — autonomy-gap closure per [[feedback_autonomy_gap_is_a_bug]].
- 3 stashes converted to dated `stash/recovered-2026-05-19-{1..3}` branches per [[feedback_stash_auto_route_to_branch]] (closes #510).
- Fireworks-Kimi judge wiring in flight on `feat/v0.26.0-fireworks-kimi-judge` (#103 unblock) — *(TBD pending smoke-test receipt)*.
- `FIREWORKS_API_KEY` shared inline this session — user should rotate.
