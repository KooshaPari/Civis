# Lefthook format-check Audit — Issue #523

## Hook Definition
**File**: `C:\Users\koosh\Dino\lefthook.yml` (lines 17–19)

```yaml
    format-check:
      glob: "**/*.cs"
      run: dotnet format src/DINOForge.CI.NoRuntime.sln --verify-no-changes
```

## Root Cause
The hook has a **glob filter** (`**/*.cs`) but **ignores it** in the `run` command. The `run` invocation is hardcoded to the full workspace SLN (`src/DINOForge.CI.NoRuntime.sln`), not the glob-matched staged files. This scans **all C# files in the solution** regardless of what was actually staged, triggering pre-existing IL2026 warnings in PackCompiler that were never modified by the #523 commit.

## Why It Matters
- `glob` is decorative (used for hook triggering only, not command input)
- `run` always processes the full SLN, not just staged changes
- Result: #523 EconomyContentLoader fix blocked by unrelated PackCompiler IL2026 warnings

## 3 Scope-Narrowing Options

### Option 1: Use `{staged_files}` Glob (Recommended)
```yaml
    format-check:
      glob: "**/*.cs"
      run: dotnet format {staged_files} --verify-no-changes
```
**Pros**: Stages-only, minimal scope.  
**Cons**: Requires dotnet format to support file-list input (it does via `--include`).

### Option 2: Skip via Env Var
Add to lefthook.yml:
```yaml
    format-check:
      skip: SKIP_FORMAT_CHECK
      run: dotnet format src/DINOForge.CI.NoRuntime.sln --verify-no-changes
```
Commit cleanly with: `SKIP_FORMAT_CHECK=1 git commit`  
**Pros**: Unblocks immediately without code changes.  
**Cons**: Bypasses check entirely (not ideal for CI).

### Option 3: Scope to a Curated SLN
Create `src/DINOForge.Format.sln` (Domains + SDK only, exclude PackCompiler).  
**Pros**: Permanent, no env var workarounds.  
**Cons**: Maintenance burden (keep SLN in sync).

## Recommendation
**Option 1** unblocks #523 with least risk and is the standard lefthook pattern. Lefthook docs: https://github.com/evilmartians/lefthook/blob/master/docs/global-options.md#staged_files (variable expansion), https://github.com/evilmartians/lefthook/blob/master/docs/global-options.md#skip (skip condition).
