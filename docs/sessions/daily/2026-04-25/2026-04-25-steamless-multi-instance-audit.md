# Steamless / Multi-Instance Reality Audit

**Date**: 2026-04-25
**Auditor**: Infra audit subagent (iteration 37, parallel batch of 5)
**Scope**: Steamless launch claims, DINOBox pool, `_TEST` install dir, multi-instance support, second-game-instance launch path
**Verdict**: BROKEN

---

## Executive summary

The DINOForge multi-instance story is "Steam-tolerant" — it relies on the
already-installed Steam DRM by side-launching `Diplomacy is Not an Option.exe`
directly with `single-instance=0` in `boot.config` — **not** "Steam-free". The
documented Steamless / Goldberg integration is purely vaporware: there is no
Steamless binary, no Goldberg wrapper, no DRM-stripping pipeline, and no
end-to-end test that proves a DINOBox launch under any Steam-detached condition.

Worse, the DINOBox pool that is meant to give isolated test instances is
**empty of mod content**: pool boxes are created but `DINOForge.Runtime.dll` is
never deployed into any of them, so every "DINOBox launch" is a vanilla game
launch with no plugin loaded. The `_TEST` boot.config also had `single-instance=false`
(Unity treats the literal string as truthy, so a second instance is rejected),
and `Launch-DINOBox.ps1` defaults to the broken `Hidden=true` HiddenDesktop
backend.

---

## Findings table

| Mechanism | Claim | Reality | Status |
|-----------|-------|---------|--------|
| `boot.config single-instance=0` | Allows a second concurrent game instance | Works on the main install dir; `_TEST` had `single-instance=false` (truthy) | PARTIAL |
| `_TEST` install dir | Independent BepInEx + saves so second instance launches cleanly | Directory exists, plugin deploys, but boot.config wrong until #188 | PARTIAL |
| `playCUA` binary | Cross-platform isolation backend used by `PlayCUABackend` | Binary path exists on disk; auto-detect path drift in `isolation_layer.py` | PARTIAL |
| DINOBox pool | Pre-warmed boxes with deployed mod, ready to launch | Pool dir exists; **zero boxes contain `DINOForge.Runtime.dll`** | BROKEN |
| `Launch-DINOBox.ps1` defaults | Sane defaults for typical dev launch | Defaults to `Hidden=true` (HiddenDesktop, known broken) | BROKEN |
| Steamless integration | DRM-free launch wrapper for headless test | No binary, no script, no test — references only | VAPORWARE |
| Goldberg wrapper | Steam API emulator for offline launch | No binary, no script, no test — references only | VAPORWARE |

---

## Concrete gaps

1. **DINOBox pool boxes have no Runtime.dll deployed.** `New-DINOBoxPool.ps1`
   provisions box directories but never copies `DINOForge.Runtime.dll`,
   `dinoforge_packs/`, or any plugin artifacts into them. Every "DINOBox
   launch" therefore runs vanilla DINO with zero mod surface. This is the
   single largest infra rot — the entire pool is theatre.

2. **`_TEST/boot.config` has `single-instance=false`.** Unity reads the value
   as a literal string. `false` is non-empty → truthy → the second-instance
   guard fires. Correct value is `single-instance=0`. Until fixed the TEST
   instance silently refuses to launch when the main game is already running.

3. **`Launch-DINOBox.ps1` default `Hidden=$true`.** The HiddenDesktop backend
   crashes Unity D3D11 init (no DXGI adapter on hidden desktops). Default
   should be `Hidden=$false` until VDD-tier-1 ships. This makes the documented
   one-liner produce a black-hole launch.

4. **`PlayCUABackend` path drift.** `isolation_layer.py:608` resolves the
   binary path with a `native/` segment that does not exist on disk. Auto-detect
   therefore fails and the backend reports "unavailable" even when playCUA is
   installed.

5. **No Steamless / DRM-strip pipeline.** Documents reference Steamless and
   Goldberg as if they were working integrations. There is no binary, no
   wrapper script, no CI invocation, and no test exercising a Steam-detached
   launch. The claim should be removed or scoped to "future work".

6. **No end-to-end DINOBox launch proof.** No `docs/sessions/` log shows
   `dinoforge_debug.log` entries originating from a DINOBox-launched process.
   Every "verified" launch in the repo is the main install dir, not a pool box.

7. **No multi-instance regression test in CI.** No workflow exercises
   "launch instance A, launch instance B from `_TEST`, verify both have
   independent saves and BepInEx logs". The only test that touches multi-
   instance is a unit test of the path-resolver helper.

---

## Tasks opened from this audit

- **#188 P0** — DINOBox/playCUA/_TEST infra cleanup (5 fixes covering gaps 1-4 above). Closed at source level in iteration 37.
- Future: Steamless/Goldberg work would land as a separate task under Wave 2 architecture if the user re-prioritizes truly Steam-free launch.

User-facing acceptance for #188 lives in `docs/setup/wave-1-acceptance-runbook.md` —
running `New-DINOBoxPool.ps1 -BoxCount 1 -Force` must produce a box with
`DINOForge.Runtime.dll` present, and a launch from that box must produce real
`dinoforge_debug.log` entries.

---

## See also

- `docs/sessions/2026-04-25-infra-pivot-plan.md` — wave-by-wave plan; this audit is input #1
- `docs/sessions/2026-04-25-sandbox-isolation-audit.md` — overlapping concerns (HiddenDesktop, playCUA)
- `docs/sessions/2026-04-25-ci-manifest-gate-audit.md` — CI gate that should reject empty-pool launches
- `docs/sessions/2026-04-25-bridge-bypass-audit.md` — bridge silent-bypass that masks empty-pool launches
- `docs/sessions/2026-04-25-ui-sdk-audit.md` — separate axis, not directly related but part of same iteration-37 batch
- `docs/sessions/2026-04-24-hidden-desktop-ground-truth.md` — ground-truth test that proves HiddenDesktop crashes Unity D3D11

---

## Verdict

DINOBox / `_TEST` / `playCUA` is "**Steam-tolerant** and **almost-working** at
the source level once #188 lands, but **not Steam-free** and **not yet
end-to-end verified by a real launch**". Until the user runs the acceptance
runbook, multi-instance support is honest progress, not verified working.
