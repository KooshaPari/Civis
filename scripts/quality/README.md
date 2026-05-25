# Quality manifest (local-first CI)

Cloud CI (`quality.yml`) only runs `verify-quality-manifest.sh` — no Rust, Node, or Unreal on the runner. Developers attest gates locally via lefthook `pre-push` → `emit-quality-manifest.ps1` / `.sh`.

## Gate tiers

| Tier | Gates | Required for merge? |
|------|-------|---------------------|
| **Core** | `civis_3d_verify`, `web_test`, `dashboard_typecheck` (or `rust_*` / `godot_test` fallback) | Yes — `status` must be `pass` |
| **Optional (Unreal)** | `unreal_preflight`, `unreal_build` | No — `skip` is OK; omit entirely on machines without UE |

Unreal scripts (optional tier):

| Script | Gate key | When recorded |
|--------|----------|---------------|
| `clients/unreal-show/scripts/verify-unreal-ready.ps1` | `unreal_preflight` | `CIVIS_QUALITY_UNREAL=1` or UE+UBT detected at emit time |
| `clients/unreal-show/scripts/build.ps1` (full, no `-SkipUe`) | `unreal_build` | Same; only runs when `UE_ROOT` or Epic `UE_*` + UBT/`Build.bat` exists |

Agent smoke (fast default): `scripts/agent-smoke.ps1` — Rust tests + offline Unreal preflight.

Full UBT compile (opt-in): `scripts/agent-smoke.ps1 -FullUnreal` — runs `build.ps1` when UE+UBT is available.

## Refresh manifest

```powershell
lefthook run pre-push
# or
pwsh -NoProfile -File scripts/quality/emit-quality-manifest.ps1
```

Optional Unreal gates on a UE machine:

```powershell
$env:CIVIS_QUALITY_UNREAL = '1'
pwsh -NoProfile -File scripts/quality/emit-quality-manifest.ps1
```

## Verify (CI)

```bash
bash scripts/quality/verify-quality-manifest.sh
```
