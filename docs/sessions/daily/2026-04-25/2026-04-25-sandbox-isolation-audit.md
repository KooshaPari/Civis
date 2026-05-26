# Sandbox / Isolation Reality Audit

**Date**: 2026-04-25
**Auditor**: Infra audit subagent (iteration 37, parallel batch of 5)
**Scope**: All seven documented isolation mechanisms — `_TEST` install dir,
`boot.config single-instance`, HiddenDesktop, PlayCUA, VDD, DockerBackend,
PhenoCompose, separate Windows user account
**Verdict**: MOSTLY BROKEN — 1.5 of 7 mechanisms actually work

---

## Executive summary

DINOForge documents seven sandbox / isolation strategies for running a second
concurrent game instance without polluting the user's main session. Of the
seven, exactly **one and a half work**: the `_TEST` install dir + `boot.config
single-instance=0` combo (and even that is degraded by the boot.config typo
caught in audit #1). Everything else is either **broken** (HiddenDesktop, where
Unity D3D11 init fails on hidden desktops with no DXGI adapter — see ground-
truth test 2026-04-24), **partially-implemented** (PlayCUA, where the
auto-detect path drifts off the on-disk binary), or **purely aspirational**
(VDD, DockerBackend, PhenoCompose, separate-user-session).

The `IsolationContext.get('auto')` helper in `isolation_layer.py` is therefore
a footgun: it claims "tries playCUA, falls back gracefully" but in practice it
returns `NoWorkingIsolationBackend` more often than it returns a working
backend, because the playCUA path-drift bug makes it look unavailable.

---

## Findings table

| Mechanism | Implementation status | Runtime status | Verdict |
|-----------|----------------------|----------------|---------|
| `_TEST` install dir | Real (separate dir + BepInEx) | Worked after #188 boot.config fix | WORKS |
| `boot.config single-instance=0` | Real (Unity reads value) | Works on main; `_TEST` had typo until #188 | PARTIAL |
| HiddenDesktop (Win32 `CreateDesktopW`) | Real (creates desktop) | Unity D3D11 init crashes (no DXGI adapter) | BROKEN |
| PlayCUABackend | Real (Rust binary on disk) | Auto-detect path drift; works only with explicit override | PARTIAL |
| VDD (Virtual Display Driver) | Not implemented | — | VAPORWARE |
| DockerBackend | Stub class only | — | VAPORWARE |
| PhenoCompose (Tier 1/2/3) | External, not integrated | — | VAPORWARE |
| Separate Windows user account | Documented only | No automation, no scripts | VAPORWARE |

Counting strictly: 1 WORKS, 1 PARTIAL (boot.config), 2 partial / broken
implementations (HiddenDesktop, PlayCUA), 4 vaporware. The "1.5 of 7" figure
reflects the boot.config + `_TEST` dir combo as one mechanism counted at 1.0
plus playCUA at 0.5 because it works with explicit override.

---

## Concrete gaps

1. **HiddenDesktop is a known dead end.** Win32 `CreateDesktopW` produces a
   desktop with no GDI / DXGI adapter, so Unity's D3D11 init fails as soon as
   the game tries to enumerate adapters. Confirmed by ground-truth test on
   2026-04-24 (`docs/sessions/2026-04-24-hidden-desktop-ground-truth.md`).
   The HiddenDesktopBackend should be marked BROKEN in code (raise
   `IsolationBackendBroken` on launch) and removed from the auto-detect
   sequence — the user's expectation that `Hidden=true` "just works" is
   actively misleading.

2. **PlayCUA path drift.** `isolation_layer.py:608` builds a path with a
   `native/` segment that the on-disk Rust artifact does not have. Auto-detect
   therefore returns "unavailable" even when the binary is present. Fixed in
   #188.

3. **VDD is referenced as Tier 1 but does not exist.** No driver project, no
   INF file, no installation script. The "future Tier 1" claim is a roadmap
   placeholder, not work-in-progress.

4. **DockerBackend is a stub class with no implementation.** The class exists
   to satisfy the interface; calling its `launch_process` raises `NotImplemented`.
   Documenting it as "Tier 1 future" is fine; documenting it under "Backend
   Implementations" without that qualifier is misleading.

5. **PhenoCompose integration is external observation only.** PhenoCompose
   (KooshaPari/phenocompose) is a real upstream project but DINOForge does not
   import, wrap, or invoke it. The integration is roadmap-only, scheduled for
   v0.24.0+.

6. **Separate Windows user account isolation has zero automation.** Documented
   as "standard approach, no special deps", but no script provisions the user,
   no runbook walks the operator through it, and no MCP tool exposes
   "launch-as-user".

7. **`auto` mode is a footgun.** Because so many backends are
   broken/vaporware, the auto-selection logic is effectively "try playCUA, fail".
   Until at least one Tier 1 backend ships clean, the auto path should default
   to the `_TEST` dir + main desktop combo and document the trade-off.

---

## Tasks opened from this audit

- **#188 P0** (closed at source) — playCUA path-drift fix; `_TEST` boot.config; flip `Launch-DINOBoxInstance.ps1` default `Hidden=$false`.
- **Future**: VDD driver project (Tier 1) — would land as a multi-iteration spike, gated on user prioritization.
- **Future**: DockerBackend implementation — gated on Tier-1-headed Docker game-launch story; not blocking current Wave 1.
- **Future**: PhenoCompose CLI wrapper — planned for v0.25.0+ per existing roadmap.

---

## See also

- `docs/sessions/2026-04-25-infra-pivot-plan.md` — Wave 1 acceptance gates this audit
- `docs/sessions/2026-04-25-steamless-multi-instance-audit.md` — overlapping playCUA + DINOBox findings
- `docs/sessions/2026-04-24-hidden-desktop-ground-truth.md` — empirical proof HiddenDesktop is broken
- `docs/sessions/2026-04-25-ci-manifest-gate-audit.md` — CI does not exercise any of these isolation paths
- `docs/sessions/2026-04-25-bridge-bypass-audit.md` — bridge silent-bypass masks isolation failures

---

## Verdict

Sandbox isolation is **mostly broken**. Until VDD or a real Docker backend
ships, the only honestly-working isolation path is `_TEST` install dir +
`single-instance=0` with the launch happening on the main user desktop. The
documented "Tier 1 (future)" marker is the truth — present-tense isolation
claims should be qualified to "the `_TEST` dir gives saves/BepInEx isolation
but does not give display isolation".
