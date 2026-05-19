# NuGet Version Alignment Audit (Iter-142)

**VERSION file**: `0.25.0-dev`

## Version Matrix

| Project | PackageId | Cited Version | Matches VERSION? | TargetFramework |
|---------|-----------|---------------|------------------|-----------------|
| SDK | DINOForge.SDK | 0.18.0 | **NO** | netstandard2.0 |
| Bridge.Protocol | DINOForge.Bridge.Protocol | 0.24.0 | **NO** | netstandard2.0 |
| Bridge.Client | DINOForge.Bridge.Client | 0.24.0 | **NO** | netstandard2.0 |
| InstallerLib | DINOForge.Tools.Installer | 0.20.0 | **NO** | net6.0 |
| Templates | DINOForge.Templates | 0.18.0 | **NO** | net11.0 |
| Warfare | DINOForge.Domains.Warfare | 0.18.0 | **NO** | netstandard2.0 |
| Economy | DINOForge.Domains.Economy | 0.18.0 | **NO** | netstandard2.0 |
| Scenario | DINOForge.Domains.Scenario | 0.18.0 | **NO** | netstandard2.0 |
| UI | DINOForge.Domains.UI | 0.18.0 | **NO** | netstandard2.0 |

## Drift Analysis

**HIGH SEVERITY** — 9 of 9 NuGet-published projects have hardcoded versions that do NOT match the VERSION file (0.25.0-dev).

**Version clusters detected:**
- `0.18.0` — SDK, Templates, Warfare, Economy, Scenario, UI (6 projects, likely last v0.18.0 release)
- `0.24.0` — Bridge.Protocol, Bridge.Client (2 projects, noted in CHANGELOG as v0.24.0 bump)
- `0.20.0` — InstallerLib (singleton outlier)

## release.yml Wiring

**Current approach** (lines 47-62, 143-184):
- Extracts version from Git tag (`v0.25.0` → `0.25.0`)
- Passes `-p:PackageVersion=$env:PACKAGE_VERSION` override at pack time
- **This override bypasses hardcoded csproj versions** and correctly packs using the tag version

**Verdict**: `release.yml` is **correctly configured**. The workflow will override all hardcoded csproj versions with the tag version at pack time. The misalignment exists in the source files but poses **NO RISK** to the release — the CI gate re-stamps packages.

## Pre-Tag Action List

Since `release.yml` overrides at pack time, **NO action is required** before v0.25.0 tag.

However, for **code cleanliness and clarity**:
- Update all 9 csproj files to `<PackageVersion>0.25.0</PackageVersion>` before tag
- This removes the drift and makes the source tree accurate for local developers/audits
- Optional; does not block the v0.25.0 release (CI override is effective)

## Recommendation

**For v0.25.0 go-live**: OK. release.yml override is working.
**For code hygiene**: Create prep PR bumping all 9 projects to 0.25.0 before tag fires (low effort, high clarity).
