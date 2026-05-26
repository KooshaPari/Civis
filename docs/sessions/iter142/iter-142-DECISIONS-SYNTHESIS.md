# Iter-142 Decisions Synthesis

**Date**: 2026-05-18  
**Phase 0 status**: HandleConnect deploy COMPLETE (verified in deployed DLL at 18:55:53 UTC after false-deploy caught at 17:35) — pending user relaunch verification.  
**Context**: Four concurrent iter-142 audits (HiddenDesktop wiring, lefthook, TIER 1 spec, v0.25.0 scope) merged into user decision points.

---

## Three User Decision Points

### Decision A: #523 Commit Blocker — Lefthook IL2026 (`lefthook_format_check_audit_iter142.md`)

**Problem**: `format-check` hook scans entire `src/DINOForge.CI.NoRuntime.sln` regardless of `glob: "**/*.cs"` filter, blocking #523 commit due to pre-existing PackCompiler IL2026 warnings unrelated to staged changes.

**Recommendation**: Apply **Option 1 (1-line fix)**

```yaml
# C:\Users\koosh\Dino\lefthook.yml, line 19 (BEFORE) — verified by lefthook_fix_target_verification_iter142.md
run: dotnet format src/DINOForge.CI.NoRuntime.sln --verify-no-changes

# AFTER (replace with)
run: dotnet format {staged_files} --verify-no-changes
```

**Details**:
- `{staged_files}` is the standard lefthook variable expansion (https://github.com/evilmartians/lefthook/blob/master/docs/global-options.md#staged_files)
- Narrows scope to only C# files staged in commit #523
- No tool changes; `dotnet format` supports file-list input

**Risk**: LOW  
**Effort**: 5 minutes  
**Unblocks**: #523 commit → #524 verification → merge fix/handle-connect-iter142 → v0.25.0 tag chain

---

### Decision B: TIER 1 Fast-Track (Steamless + MockSteamworksNet) (`tier1_deploy_target_spec_iter142.md` + `tier1_spec_verification_iter142.md`)

**Status**: Spec written. **VERIFIED accurate** (all 4 MSBuild properties exist in csproj; verification: `tier1_spec_verification_iter142.md:7–16`).

**What**: Add `DeployMockSteamworksNet` target to `src/Tools/MockSteamworksNet/MockSteamworksNet.csproj` (28 XML lines from `tier1_deploy_target_spec_iter142.md:56–87`).

**Syntax verified against**:
- `$(GameInstalled)` ✓ DINOForge.Runtime.csproj:11-12
- `$(DeployToGame)` ✓ DINOForge.Runtime.csproj:13
- `$(BepInExDir)` ✓ DINOForge.Runtime.csproj:18 (used)
- `AfterTargets="Build"` ✓ DINOForge.Runtime.csproj:247, 267, 290
- `SkipUnchangedFiles="true"` ✓ DINOForge.Runtime.csproj:256, 278, 297

**Activation**: 
```powershell
dotnet build src/Tools/MockSteamworksNet/MockSteamworksNet.csproj -p:DeployToGame=true -p:DeployMockSteamworks=true
```

**Effort**: 6–8 hours (research + verification already done)  
**Unblocks**: #98, #101, #103, #425 in principle  
**Next gate**: User authorization to drop the 28-line XML into MockSteamworksNet.csproj

---

### Decision C: HiddenDesktopBackend / isolation_layer.py Cleanup (`hidden_desktop_wire_up_audit_iter142.md` + `isolation_layer_dead_code_inventory_iter142.md`)

**Finding**: **814 LOC dead code across the entire `isolation_layer.py` file** (broader audit superseded earlier 315-LOC scope; 315 was just `HiddenDesktopBackend` alone).

**Component breakdown** (per `isolation_layer_dead_code_inventory_iter142.md`):
- `HiddenDesktopBackend` — 314 LOC
- `PlayCUAClient` — 114 LOC (JSON-RPC harness never called)
- `PlayCUABackend` — 189 LOC (all methods unreachable)
- `IsolationContextManager` + helpers — 43 LOC (singleton never instantiated)
- `Frame`, `WindowInfo` data models — import-only
- **Total: 100% unreachable**, zero production/test callers

**Root cause**: Module designed as tier-fallback strategy but never wired into `server.py`. Game capture routes directly to `GameControlCli` (C#) via named pipes; isolation layer was architectural planning that never shipped.

**Actual launch path** (server.py:467–472):
```
hidden=True → _launch_on_vdd() [VDD fallback] → _launch_hidden() [PS1 Win32 CreateDesktop]
hidden=False → subprocess.Popen [primary desktop]
```

**Options**:
1. **Delete entirely** (814 LOC removed) — 30 minutes, safe (verified zero callers)
2. **Retire to `docs/scripts/retired/`** preserving for reference — 15 minutes
3. **Keep** as future scaffold for cross-platform isolation
4. **Replace** with RDP fleet implementation (out of scope for v0.25.0)

**Recommendation**: **Defer to v0.26.0** unless TIER 1 lands first. Decision B (MockSteamworks) takes priority for release cycle. Either option 1 (delete) or option 2 (retire) is appropriate — both eliminate the dead surface from import scope.

---

## Critical Path to v0.25.0 Tag

**MUST Land Before Tag** (`v0_25_0_scope_triage_iter142.md:9–16`):

| Task | Effort | Blocks |
|------|--------|--------|
| #523 EconomyContentLoader regression (fix in flight: a7eb4ac4f96342a56) | 0.5h | SDK pre-release validation |
| #524 PreToolUse hooks smoke test (settings.json safety critical) | 1h | Governance hardening |
| Merge fix/handle-connect-iter142 → main (51 commits, 282-file intersection, 3-phase explicit merge) | 2h | Game recovery + HandleConnect deployment + v0.25.0 tag |

**Subtotal**: 3.5 hours  
**Release-ready trigger**: All (MUST) + merge phase-3 complete → git tag v0.25.0 + release.yml auto-fire

**Nice to Land** (low priority): #269 (Pattern #96 analyzer), #515 (CI path fixes)  
**Defer to v0.26.0**: #101 (Star Wars render), #103 (Kimi runbook), #505 (Pattern #231 static-init), #507/#510/#512 (branch cleanup)

---

## Recommended Sequencing for Next Session

1. **Apply lefthook fix** (Decision A) — user authorizes → commit #523
2. **Verify #523 + #524** passing locally
3. **Merge fix/handle-connect-iter142 → main** (phase-3 explicit merge: GameClient.cs, JsonRpcMessage.cs, VERSION)
4. **Test post-merge** (SDK validation, game launch sanity)
5. **Tag v0.25.0** and fire release.yml
6. **Evaluate Decision B** (TIER 1 fast-track for v0.26.0 sprint if timeline permits)
7. **Decision C cleanup** can wait until TIER 1 lands (deferred to iter-143+)

---

**Iter-142 Audit Docs**: All four source reports live in `docs/qa/` and `docs/proposals/` for long-term reference.
