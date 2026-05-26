# Pattern #228 Audit: Empty Catch Blocks

**Date**: 2026-05-18  
**Status**: ENDEMIC (148 violations)

## Definition

**Pattern #228**: Exception handlers with empty bodies (`catch { }`, `catch (Exception ex) { }`).

**Why Bad**: Loss of observability. Silent swallowing masks failures entirely, preventing error recovery and making debugging impossible. Cannot distinguish "operation succeeded with no cleanup needed" from "operation failed catastrophically."

## Audit Results

**Script LOC**: 47 lines (Python regex-based scanner)  
**Total Violations**: 148  
**Tier**: ENDEMIC (>20 violations)

### Top 5 Violations

| File | Line | Snippet |
|------|------|---------|
| `src/Analyzers/EmptyCatchBlockAnalyzer.cs` | 13 | `catch { }` |
| `src/Analyzers/SilentCatchAnalyzer.cs` | 54 | `catch {}` |
| `src/Analyzers/SilentCatchAnalyzer.cs` | 87 | `catch { /* safe-swallow: */ }` |
| `src/Bridge/Client/GameClient.cs` | 272 | `catch (Exception ex) { // Phase 4c...` |
| `src/Bridge/Client/GameClient.cs` | 480 | `catch { // Will...` |

### Distribution Summary

- **Analyzer self-referential** (src/Analyzers/): 2 — test fixtures demonstrating empty catch
- **Bridge/Client**: 9 — GameClient, GameProcessManager
- **Runtime**: 41 — various ECS systems and utilities
- **Tools** (CLI, MCP, PackCompiler): 18
- **SDK** (registries, loaders, validators): 78

## Governance

**Status**: INHERITED DEFICIT from pre-iter-111 code. Iter-111 swept `catch (Exception)` with logging, but missed empty-body variants and test fixtures.

**Allowlist Strategy**:
- **Safe-swallow candidates** (disposable cleanup on Dispose): mark with `// empty-catch-ok: <reason>`
- **Error-logging candidates**: add `_logger.LogWarning(ex, "context")`
- **Analyzer test fixtures**: move to test-only code or suppress via `[SuppressMessage]`

**CI Gate**: Detection script (`scripts/ci/audit_empty_catch_blocks.py`) requires refinement to distinguish legitimate vs. malodorous patterns. Recommend allowlist-first approach.

## Next Actions

1. Separate analyzer test code (EmptyCatchBlockAnalyzer.cs, SilentCatchAnalyzer.cs) from production
2. Analyze top 15 files by violation count to identify patterns
3. Create allowlist at `docs/qa/pattern-228-allowlist.txt` for confirmed-safe sites
4. Govern SDKLoader/PackRegistry cluster (78 violations) via DF1023 Roslyn analyzer
