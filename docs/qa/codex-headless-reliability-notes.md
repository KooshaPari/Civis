# Codex CLI Reliability Notes — Iter-143 Wave 2

**Date:** 2026-05-19

## Observations

1. **Workspace-write sandbox redirection**
   - In the workspace-write mode, Codex silently redirected writes outside the active cwd to `C:/Users/CodexSandboxOffline/...`.
   - Those redirected locations were not accessible by the orchestrator.
   - Practical workaround:
     - Keep targets inside the current `cwd`, or
     - Set Codex working directory to the target path’s parent directory before writing.

2. **Timeout on small scripted write**
   - `codex mini` timed out on a simple PowerShell script write task (~200 LOC) to `scripts/diag/health-summary.ps1`.
   - Timeout observed at the 600s wrapper budget.
   - Likely causes:
     - prompt complexity likely triggered internal retries/confusion, and/or
     - internal retry loops consumed wrapper budget.
   - Recommendation:
     - Keep prompts under 1500 chars and use explicit file targets for headless dispatches.

3. **Mixed success across model variants**
   - `Codex spark` and `Codex mini` both completed cleanly on simpler scopes:
     - memory retrospective
     - v0.26.0 plan
     - migration guide
     - release-readiness probe (in progress during reporting)
   - Empirical reliability (Wave 2, 6 dispatches):
     - 4/6 succeeded
     - 1/6 sandboxed-write issue
     - 1/6 timed out

4. **Haiku Agent comparison**
   - `Haiku Agent` was slower per task but did not show timeout symptoms in this wave.

## Delegation decision rule (Wave 2 guidance)

Use **Haiku Agent** when either:
- write target is **outside cwd**, or
- expected scope is **greater than 300 LOC**, or
- task requires **multi-step file operations**.

For all other cases, prefer **Codex spark/mini**.

## Pairs with
- 2026-03-15 memory feedback on scoped DesktopCompanion upgrade/build-gating, especially workflow sensitivity to environment/toolchain limits.
- 2026-03-14 memory feedback on automation/tooling reliability, noting environment/build-path checks and retry/validation guardrails.
