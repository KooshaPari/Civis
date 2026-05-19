# Iter-142 Commit Messages Ready

Pre-drafted heredoc-ready messages for user authorization. Follow repo conventions per `git log --oneline -5`.

## Commit 1: lefthook scope fix
**Files**: `lefthook.yml`

```
chore(hooks): narrow format-check to {staged_files} glob

The format-check hook ran dotnet format on the entire CI.NoRuntime.sln, scanning
pre-existing IL2026 warnings in PackCompiler (Newtonsoft.Json v13 trim-incompat).
This blocked commits that didn't touch PackCompiler.

Fix: replace hardcoded sln target with {staged_files} variable so the hook only
checks C# files actually being committed.

Refs: docs/qa/lefthook_format_check_audit_iter142.md
Refs: docs/qa/il2026_root_cause_iter142.md
```

## Commit 2: #523 EconomyContentLoader IValidatable wiring
**Files**: `src/Tests/EconomyContentLoaderValidationTests.cs` + Economy Registries

```
fix(economy): align test expectations with iter-128 IValidatable wiring (#523)

Economy Registry.Register methods call Validate() and throw ArgumentException on
validation failure (per iter-128 Pattern #95/#210 wiring). Tests expected
InvalidDataException — now updated to match production behavior.

All 8 EconomyContentLoaderValidationTests pass.

Refs: #523, docs/qa/economy_523_restage_verification_iter142.md
```

## Commit 3: game-fix + governance hardening + audit landings
**Files**: GameBridgeServer.cs, Plugin.cs, DINOForge.Runtime.csproj, CLAUDE.md, lefthook.yml, audit docs

```
feat(runtime): restore game launch + iter-142 governance hardening

Game-launch recovery:
- HandleConnect handler in GameBridgeServer (was MethodNotFound)
- Runtime TFM to netstandard2.0 (BepInEx 5.4 Mono CLR 4.0 cannot load net8.0)
- Log rotation at 100MB + BepInEx logger fallback
- Plugin.Awake() diagnostic probes

Governance hardening:
- block-git-stash.ps1 PreToolUse hook (autoroute to branches)
- guard-git-worktree.ps1 boundary protection
- Pattern Catalog #232/#233 (log rotation, obj/ cleanup)
- CLAUDE.md .NET policy: Runtime is netstandard2.0

Verified: Plugin.Awake() fires, GameBridgeServer online, ECS world discovered.

Refs: docs/sessions/iter-142-DECISIONS-SYNTHESIS.md
Refs: #508, #522, #523
```

**Usage**: Copy into `git commit -m "$(cat <<'EOF' ... EOF)"` heredoc when authorized.
