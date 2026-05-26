# Unit tests (Pester)

Offline PowerShell contract tests under `tests/unit/*.ps1`. No game required for most cases.

**Run all unit tests** (reliable on Windows; globs are not expanded for `Invoke-Pester`):

```powershell
pwsh -File scripts/qa/run-unit-pester.ps1
```

Requires **Pester 3.x** (`Install-Module Pester -RequiredVersion 3.4.0 -Scope CurrentUser`). Pester 5 is not supported by these tests yet.

| Script | Spec |
|--------|------|
| `Test-BootConfigSingleInstance.ps1` | SPEC-005 `single-instance=0` on launch-game boot.config paths |
| `Test-CaptureFeatureClips.ps1` | SPEC-003 `capture-feature-clips.ps1` script contract |
