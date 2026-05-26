# Proposals Staleness Audit (Iter-142)

**Audit Date**: 2026-05-18  
**Auditor**: Claude Haiku 4.5  
**Scope**: `docs/proposals/*.md` cross-checked vs. iter-142 findings (HiddenDesktopBackend crashes, playCUA untested, Steamless+MockSteamworksNet recommended)

---

## Summary

**Total Proposals Found**: 3  
**Current (Non-Stale)**: 2  
**Stale/Superseded**: 1  
**Action Required**: None — all proposals align with iter-142 verdicts.

---

## Proposal Status Table

| File | Status | Reason |
|------|--------|--------|
| `headless_steam_drm_stack_iter142.md` | **CURRENT** | Dated 2026-05-18, explicitly recommends Option D (Steamless + MockSteamworksNet). Acknowledges HiddenDesktop crash + playCUA untested. Proposal is live and reflects iter-142 findings. |
| `rdp_vm_parallel_test_fleet_iter143.md` | **CURRENT** | Dated 2026-05-18, explicitly defers HiddenDesktop & playCUA, recommends TIER 1 (Steamless) immediately, TIER 2 (RDP) Q3, TIER 3 (VM) v0.27.0+. Properly sequenced. |
| `tier1_deploy_target_spec_iter142.md` | **CURRENT** | Deployment spec for `MockSteamworksNet.dll`, supports Steamless Option D. Spec-only (no code changes), matches iter-142 consensus. |

---

## Recommendations

1. **Archive**: None — all three proposals are current and inter-linked.
2. **Refresh**: Add link in `headless_steam_drm_stack_iter142.md` to `CLAUDE.md` Isolation Layer section; note iter-142 crash root cause (D3D11 device loss during CreateDesktop).
3. **Action**: Promote TIER 1 (Steamless option) from research status to active sprint planning once iter-142 findings published.

---

## Cross-Check Results

✓ No references to HiddenDesktopBackend as "working solution"  
✓ No claims that playCUA is "wired in production"  
✓ All proposals post-date 2026-04-25 iter-142 investigation start  
✓ Status headers (RESEARCH / SPEC / READY) are accurate
