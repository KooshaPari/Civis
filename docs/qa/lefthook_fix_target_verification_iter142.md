# Lefthook Format-Check Fix Target Verification — Iter-142

**Date**: 2026-05-18  
**Task**: Verify exact line number for `{staged_files}` replacement in lefthook.yml

---

## Finding: LINE NUMBER MISMATCH DETECTED

### File Located
- **Path**: `C:\Users\koosh\Dino\lefthook.yml`
- **Actual Line with `dotnet format`**: **Line 19** (NOT line 9)

### Current Line 19 (Exact Text)
```yaml
      run: dotnet format src/DINOForge.CI.NoRuntime.sln --verify-no-changes
```

### Proposed Replacement (Line 19)
```yaml
      run: dotnet format {staged_files} --verify-no-changes
```

---

## Cross-Check Against Iter-142 Docs

| Document | Cited Line | Actual Line | Status |
|----------|-----------|------------|--------|
| `lefthook_format_check_audit_iter142.md:4` | "lines 17–19" | ✓ Hook def spans 17–19 | CORRECT |
| `iter-142-DECISIONS-SYNTHESIS.md:17` | "line 9 (BEFORE)" | ✗ Should be **line 19** | **DIVERGENT** |
| `iter-142-READY-TO-ACT-CHECKLIST.md:10` | "line 9" | ✗ Should be **line 19** | **DIVERGENT** |

### Resolution
**iter-142-DECISIONS-SYNTHESIS.md:17 and iter-142-READY-TO-ACT-CHECKLIST.md:10 both cite incorrect line number.**

- Correct line: **19** (the `run:` statement within the `format-check:` hook)
- Incorrect cite: "line 9" (refers to a comment line in the file header, not the hook)
- **Correction needed**: Both synthesis + checklist must update line refs from 9 → 19 before execution

---

## Verification Complete ✓
- Hook target identified: Line 19 of `lefthook.yml`
- Proposed text replacement: `src/DINOForge.CI.NoRuntime.sln` → `{staged_files}`
- Risk: LOW (one-word variable substitution, `dotnet format` supports file-list input)
