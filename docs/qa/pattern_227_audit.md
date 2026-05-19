# Pattern #227 Audit: Public Async Methods Without CancellationToken

## Definition
**Smell**: Public `async Task<T>` or `async ValueTask<T>` methods with no `CancellationToken ct = default` parameter.

**Why bad**: Callers cannot cooperatively cancel long-running operations. Forces awkward workarounds (Task.Run + thread abort) and prevents graceful shutdown coordination.

**Standard**: All public async APIs should accept optional `CancellationToken` per .NET Framework Design Guidelines.

---

## Detection Script
**Path**: `scripts/ci/audit_missing_ct_param.py` (proposed)
**LOC**: ~65 (Python 3.8+)
**Method**:
1. Walk `src/` excluding bin/, obj/, Tests/, Generated/
2. Find `public async Task/ValueTask` declarations
3. Check for `CancellationToken` parameter
4. Classify severity: HIGH (SDK/Bridge surface), MED (Runtime/Domains), LOW (Tools)
5. Exempt: override signatures, `[Obsolete]`, single-shot (ToString/GetHash/Equals/Validate), inline `// no-ct-ok:` markers
6. Output: CSV file + markdown summary

---

## Audit Results

### Summary Statistics
| Metric | Count |
|--------|-------|
| **Total violations** | 42 |
| **HIGH** | 1 |
| **MED** | 0 |
| **LOW** | 41 |

### Directory Heat-Map
| Directory | Violations | Severity |
|-----------|-----------|----------|
| `Tools/DesktopCompanion/` | 23 | LOW (GUI MVVM) |
| `Bridge/Client/` | 2 | LOW (CLI tools) |
| `SDK/Dependencies/` | 1 | HIGH (published API) |
| `Tools/Cli/` | 16 | LOW (CLI tooling) |

### Top 15 Violations (by file:line)

1. `SDK\Dependencies\PackSubmoduleManager.cs:166` [**HIGH**] `GenerateLockFile` — *NuGet surface*
2. `Bridge\Client\GameProcessManager.cs:50` [LOW] `LaunchAsync`
3. `Bridge\Client\GameProcessManager.cs:105` [LOW] `KillAsync`
4. `Tools\DesktopCompanion\Data\AppConfigService.cs:23` [LOW] `LoadAsync`
5. `Tools\DesktopCompanion\Data\AppConfigService.cs:42` [LOW] `SaveAsync`
6. `Tools\DesktopCompanion\Data\DisabledPacksService.cs:44` [LOW] `SaveAsync`
7. `Tools\DesktopCompanion\Data\ModCatalogService.cs:39` [LOW] `LoadCatalogAsync`
8. `Tools\DesktopCompanion\ViewModels\AssetBrowserViewModel.cs:49` [LOW] `ReloadAsync`
9. `Tools\DesktopCompanion\ViewModels\BrowseViewModel.cs:65` [LOW] `LoadCatalogAsync`
10. `Tools\DesktopCompanion\ViewModels\BrowseViewModel.cs:137` [LOW] `RefreshAsync`
11. `Tools\DesktopCompanion\ViewModels\ConflictViewModel.cs:76` [LOW] `AnalyzeConflictsAsync`
12. `Tools\DesktopCompanion\ViewModels\ConflictViewModel.cs:172` [LOW] `RefreshAsync`
13. `Tools\DesktopCompanion\ViewModels\DashboardViewModel.cs:49` [LOW] `RefreshAsync`
14. `Tools\DesktopCompanion\ViewModels\DebugPanelViewModel.cs:37` [LOW] `RefreshAsync`
15. `Tools\DesktopCompanion\ViewModels\PackListViewModel.cs:48` [LOW] `ReloadAsync`

---

## Tier Classification

**Tier: LOW**
- HIGH count = 1 (threshold 10) ✓
- Total = 42 (threshold 50 = moderate, >150 = endemic) ✓
- **Judgment**: Single HIGH violation in NuGet-published SDK surface (`GenerateLockFile`). Remaining 41 LOW violations concentrated in DesktopCompanion GUI (WinUI MVVM ViewModels) and CLI tooling — acceptable for non-critical paths.

---

## Promotion Judgment

Fix the single HIGH violation (`PackSubmoduleManager.GenerateLockFile`); defer LOW/MED as DF1227 (CLI/GUI CT addition).

**Immediate action**: Add `CancellationToken ct = default` to `GenerateLockFile` signature + thread it through to async call sites (e.g., `await git.SubmoduleUpdateAsync(ct)`).

**Deferred** (DF1227): Batch LOW violations (DesktopCompanion ViewModels, CLI tooling) into a single task — not a release blocker, but improves cancellation support for long pack-load operations (BrowseViewModel.LoadCatalogAsync, etc.).
