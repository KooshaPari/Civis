# Corrupted UTF-8 Path Recovery — Iter-141 (2026-05-18)

## Summary

**Status**: SUCCESS - All 5 corrupted staged entries removed from index; 1 real-content file recovered and correctly re-staged.

## Background

Branch `safety/iter140-snapshot-2026-05-18` had 5 corrupted UTF-8 path entries in the git index. These were created by an earlier subagent that mishandled Windows path encoding (converting colons and backslashes to octal byte sequences). The corruption manifested as staged entries with names like:
- `C\357\200\272UserskooshDino.githubworkflowssync-over-async-gate.yml`
- `\357\200\272TEMPiter90-closure-gate.log`

The octal bytes `\357\200\272` represent the UTF-8 encoding of the Unicode character U+2032 (PRIME, ′), which Git interpreted literally as part of the filename rather than as a path separator or special character.

## Corrupted Entries (Before)

| File | Size | Content | Disposition |
|------|------|---------|-------------|
| `C\357\200\272UserskooshDino.githubworkflowssync-over-async-gate.yml` | 0 bytes | Empty stub | Unstaged (deleted) |
| `C\357\200\272UserskooshDinoPATTERN_114_SUBAGENT_TASKS.md` | 0 bytes | Empty stub | Unstaged (deleted) |
| `C\357\200\272UserskooshDinodocsqact-threading-allowlist.txt.tmp` | 0 bytes | Empty stub | Unstaged (deleted) |
| `C\357\200\272UserskooshDinotest_run_iter99.log` | 467 KB | Real xUnit test output (466K+) | **RECOVERED** |
| `\357\200\272TEMPiter90-closure-gate.log` | 0 bytes | Empty stub | Unstaged (deleted) |

## Recovery Process

### 1. Extracted Real Content
Used `git cat-file blob <sha>` to extract the 467 KB xUnit test run log from the index (blob SHA: `e05b836babcd157ad10317611f3da9b7c86de518`).

Saved to correct location: `docs/sessions/test_run_iter99.log`

Content verified: xUnit test discovery, execution, and failure logs from test run for `DINOForge.Tests.dll`.

### 2. Removed Corrupted Index Entries
Used `git update-index --index-info` with mode `000000` (delete) entries to remove all 5 corrupted blobs from the index. This bypassed the path-encoding issues that `git reset HEAD` and `git rm` could not handle due to the invalid UTF-8 characters in the filenames.

### 3. Re-staged Correct File
After unstaging all corrupted entries, staged the recovered log file at its correct path:
```bash
git add docs/sessions/test_run_iter99.log
```

## Verification

### Before Cleanup
```
A  "C\357\200\272UserskooshDino.githubworkflowssync-over-async-gate.yml"
A  "C\357\200\272UserskooshDinoPATTERN_114_SUBAGENT_TASKS.md"
A  "C\357\200\272UserskooshDinodocsqact-threading-allowlist.txt.tmp"
A  "C\357\200\272UserskooshDinotest_run_iter99.log"
A  "\357\200\272TEMPiter90-closure-gate.log"
```

### After Cleanup
```
git status --short | grep 357
(no output)

git ls-files --stage | grep -E "(bbd4a2|91652ff|d264df|e05b83|f5afc1d)"
(no output)

git status --short docs/sessions/test_run_iter99.log
A  docs/sessions/test_run_iter99.log
```

**Result**: ✅ All 5 corrupted staged entries removed. 1 real-content file recovered and correctly re-staged.

## Root Cause Analysis

**Likely cause**: A subagent's `Write` tool call (or file system operation) passed an unquoted Windows path with colons (e.g., `C:\Users\koosh\Dino\...`) to git without proper escaping or shell quoting. The path was then interpreted literally by bash, with each character converted to its UTF-8 byte sequence, creating invalid git object paths.

**Contributing factors**:
1. The operation ran in a bash context without proper path escaping for Windows paths.
2. The `Write` tool or subagent did not validate the `file_path` parameter for invalid characters.
3. The path conversion was silent — the subagent did not detect or warn that the file was staged under an invalid name.

## Recommendations for Hardening

### Pre-Tool Use Hook (High Priority)
Add a pre-call validation hook in Claude Code settings to reject file paths that:
1. Contain unescaped backslashes or colons in bash contexts (Windows paths in POSIX shells).
2. Contain control characters (ASCII < 32, Unicode surrogates, etc.).
3. Contain UTF-8 byte sequences that would render as invalid filenames on the platform.

**Implementation sketch** (PowerShell-side pre-hook in settings.json):
```json
{
  "hooks": {
    "beforeToolUse": {
      "validateFilePaths": {
        "enabled": true,
        "rules": [
          "reject if file_path contains unescaped backslash in bash context",
          "reject if file_path contains UTF-8 control bytes (0x00-0x1F, 0x80-0x9F)",
          "reject if file_path violates platform filename rules (Windows: no <>:\"|?*, no trailing dot)"
        ]
      }
    }
  }
}
```

### Post-Commit Hook (Medium Priority)
Add a post-commit hook to detect and warn on filenames containing octal sequences (e.g., `\357\200\272`) or suspicious UTF-8 patterns that suggest encoding corruption.

### Subagent Guidance (High Priority)
Update agent governance in CLAUDE.md to require:
1. Always use absolute paths in the form expected by the shell context (PowerShell paths in PowerShell, POSIX paths in bash).
2. Quote all file paths passed to `Write`, `Read`, `Edit`, or git tools.
3. Never pass raw Windows `C:\...` paths to bash commands without converting to `/c/Users/...` (Git Bash / WSL2) or proper escaping.

## Files Changed

- **Created**: `docs/sessions/test_run_iter99.log` (467 KB xUnit test output)
- **Created**: `docs/sessions/corrupted_paths_recovery_iter141.md` (this file)
- **Git Index**: 5 corrupted entries removed, 1 correct entry re-staged

## Next Steps

1. ✅ All corrupted entries removed from index.
2. ✅ Real content recovered and re-staged.
3. → Await orchestrator/safety-branch-commit agent to finalize the commit on `safety/iter140-snapshot-2026-05-18`.
4. → Implement suggested hooks in `settings.json` to prevent future corruption.
5. → Update CLAUDE.md Agent governance section with path escaping guidance.
