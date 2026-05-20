2026-05-19

```markdown
### Pattern #530: MSBuild Deploy Target Silently No-ops Under Multi-TFM Project

**Symptom**
- In projects with multiple TFMs, `dotnet` publish/build pipelines may report success while the `Deploy` target does nothing (no `msbuild` errors, no artifact copy).

**Root Cause**
- Build configurations that call `dotnet publish`/`msbuild` without correctly forcing the deploy-capable TFM or toolchain can run a target path that no-ops for the selected TFM.

**Fix**
- Pin the target framework for deploy-like operations when needed (e.g., desktop plugin/runtime helper projects).
- Validate that deployment steps reference explicit `TargetFramework` or a `TargetFrameworks`-aware entry point.
- Prefer project-scoped validation/build commands on known entry projects before solution-wide pipelines.
- Gate release automation on explicit deploy-verification logs and exit with non-zero status when no-ops are detected.

**Governance**
- Do not call generic deploy/publish paths on multi-TFM projects unless the command and target are TFM-explicit.
- For multi-TFM tooling projects, document required deploy TFM and include a check that confirms output artifacts exist post-deploy.
```

```markdown
### Pattern #235: BepInEx Plugin GraphicRaycaster Without EventSystem Guard

**Symptom**
- Runtime UI overlays crash or receive null-reference behavior when `GraphicRaycaster` assumptions are made in game scenes without an `EventSystem`.

**Root Cause**
- Code touches `UnityEngine.EventSystems.EventSystem.current`/`GraphicRaycaster` paths without guarding for absent event system in non-Unity-UI-first scenes.

**Fix**
- Guard all raycast/event-system dependent code with robust null checks:
  - `EventSystem.current == null` should short-circuit interaction.
  - Skip raycast/overlay actions when UI event infrastructure is unavailable.
  - Add telemetry/debug message so scene-setup issues are diagnosable.

**Governance**
- Never call `GraphicRaycaster` or pointer event APIs as unconditional assumptions.
- Add scene-agnostic null guards before any BepInEx UI utility path interacts with UI raycast.
```

```markdown
### Agent Governance (Concurrency Scaling)

#### Concurrency Subsection
- Scale active concurrency up to **10–15** when current workload permits and no single-blocker bottlenecks exist.
- Keep default local concurrency lower during high-risk merges or when dependencies are tightly coupled.
- Only increase concurrently when:
  - tasks are isolated by domain ownership,
  - no shared file conflicts are projected,
  - required external services/systems are healthy.
- Reduce concurrency immediately when:
  - merge conflicts rise,
  - repeated environment failures occur,
  - cross-domain blockers block progress.
```

```markdown
### Codex Reliability Rule — Model and Effort Selection

- For `gpt-5.x-spark` or `gpt-5.4-mini` style light/short-circuit execution paths, prefer `--reasoning-effort low`.
- For tasks that are **outside repository working directory**, **>300 LOC**, and **multi-step**:
  - use `default Haiku` execution profile unless there is a domain-specific requirement to override.
- For smaller, low-complexity in-cwd changes, use the standard profile unless local policy says otherwise.
```

```markdown
### Pattern #233 Update — iter-143 Wave 2 Incident Reference

**incident:** iter-143 wave 2

**Observed failure mode**
- Migration from legacy SDK/TFM targets to preview .NET introduced a deploy breakage where build succeeded but deployment steps did not execute against the intended output.
- Multi-targeting compounded the issue; project-level publish/deploy assumptions silently skipped the intended runtime target.

**Update (reaffirmed)**
- Pattern #233 remains in force and now explicitly requires a **multi-target strategy** during migrations:
  - validate both build and deploy paths per target,
  - lock deploy steps to explicit target framework(s),
  - verify emitted artifacts and install behavior before broader rollout.
- Treat deploy no-op behavior under multi-TFM as a hard failure mode in migration runbooks.
- Link migration checklists and rollout notes to this prerequisite to prevent recurrence.
```
