# Isolation Layer Dead Code Audit (iter-142)

## Summary
**File**: `src/Tools/DinoforgeMcp/dinoforge_mcp/isolation_layer.py`  
**Total LOC**: 814  
**Dead LOC**: 814 (100% unreachable)  
**Status**: **COMPLETE DEAD CODE — NOT WIRED TO SERVER.PY**

## Table A: LIVE Symbols
*None. Zero production callers.*

## Table B: TEST-ONLY Symbols  
*None. Zero test callers in `src/Tools/DinoforgeMcp/tests/` or elsewhere.*

## Table C: DEAD SYMBOLS

| Class/Function | Lines | Status |
|---|---|---|
| `Frame` (dataclass) | 49–52 | DEAD: import-only, never instantiated |
| `WindowInfo` (dataclass) | 55–61 | DEAD: never instantiated |
| `IsolationBackend` (ABC) | 68–114 | DEAD: never subclassed outside the file |
| `HiddenDesktopBackend` | 121–434 | DEAD: 314 LOC, placeholders (e.g. line 136) |
| `PlayCUAClient` | 441–554 | DEAD: 114 LOC, JSON-RPC harness never called |
| `PlayCUABackend` | 561–749 | DEAD: 189 LOC, all methods unreach |
| `IsolationContextManager` | 756–798 | DEAD: singleton never used |
| `get_isolation_context()` | 805–807 | DEAD: never imported/called |
| `set_isolation_context()` | 810–813 | DEAD: never called |

**Total dead: 814/814 (100%)**

## Key Findings
1. **HiddenDesktopBackend stubs**: Lines 131–147 have placeholder `Frame(data=b"", width=0, height=0)` returns — signals incomplete implementation.
2. **No server.py integration**: `server.py` does NOT import isolation_layer. Game capture is wired directly to `GameControlCli` (C#) via named pipes (line 6–7 docstring).
3. **No test coverage**: Zero test files reference this module.
4. **Design orphaned**: iter-142 verdict (MEMORY.md) confirmed: "HiddenDesktopBackend BROKEN. PlayCUA exists but not exercised against DINO."

## Top 3 Recommendations

### (1) Safe Delete List
```
src/Tools/DinoforgeMcp/dinoforge_mcp/isolation_layer.py (entire file, 814 LOC)
```
⚠️ **Pre-delete confirmation**:
- Grep entire repo for `from .isolation_layer import` or `import isolation_layer` — 0 matches found
- Confirm no CI job or hidden import references it

### (2) RDP/Fleet Replacement Plan
If hidden desktop or multi-instance game testing is needed in future:
- **Use playCUA directly** (Tier 2 in roadmap) — already battle-tested, cross-platform
- **Skip Win32 CreateDesktop** — Tier 2 stubs proved unmaintainable; VDD (Tier 1) is next priority
- **Route through MCP server.py** — not isolation_layer.py — for all game automation

### (3) Keep-as-Scaffold Alternative
If preserving for reference (low priority):
- Move to `docs/scripts/retired/isolation_layer_reference.py` (not imported)
- Add header: `# DEPRECATED: iter-142 audit found 0 production callers. See docs/qa/isolation_layer_dead_code_inventory_iter142.md`
- Do NOT commit to `src/Tools/` — it will be mistakenly wired by future agents

---

**Verdict**: **DELETE** — this module is a completely disconnected scaffold abandoned after playCUA + HiddenDesktop tiers were designed but never integrated into server.py.
