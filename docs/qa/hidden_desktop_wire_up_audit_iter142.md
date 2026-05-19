# HiddenDesktopBackend Call-Graph Audit (Iter-142)

## Verdict: **NOT WIRED** (Dead Code in Live Launch Path)

## Summary

HiddenDesktopBackend (315 LOC, lines 121–435 in `isolation_layer.py`) is **orphan code with zero production callers**. The MCP tool signature accepts `hidden=True` but does NOT invoke the isolation layer. The actual launch path is TIER 3 (naive subprocess spawn on primary desktop).

---

## Evidence

### 1. Server.py Game Launch Path (Line 454–475)

```python
async def game_launch(ctx: Context, hidden: bool = False) -> dict:
    if not GAME_EXE.exists():
        return {...}
    try:
        if hidden:
            vdd_result = await _launch_on_vdd(str(GAME_EXE))  # Try VDD (not isolation layer)
            if vdd_result["success"]:
                return vdd_result
            return await _launch_hidden(str(GAME_EXE), "DINOForge_Agent")  # Direct PS1 script
        subprocess.Popen([str(GAME_EXE)], cwd=str(GAME_DIR))  # <-- LINE 472: TIER 3 (primary desktop)
        return {"success": True, ...}
    except Exception as e:
        return {"success": False, ...}
```

**Key finding**: 
- Line 472: `subprocess.Popen([str(GAME_EXE)], cwd=str(GAME_DIR))` — **NO isolation layer call**. Launches directly on user's current desktop.
- `_launch_hidden()` (lines 205–269) is an **ad-hoc PowerShell Win32 helper**, NOT a call to `HiddenDesktopBackend.launch_process()`.
- `isolation_layer.py` is **never imported** in `server.py` (verified: 0 matches for `from.*isolation` or `import.*isolation`).

### 2. Test Instance Launch Path (Line 479–507)

```python
async def game_launch_test(ctx: Context, hidden: bool = True) -> dict:
    test_dir = _get_test_instance_path()
    test_exe = Path(test_dir) / "Diplomacy is Not an Option.exe"
    ...
    try:
        if hidden:
            return await _launch_hidden(str(test_exe), "DINOForge_Agent_Test")  # PS1 script again
        subprocess.Popen([str(test_exe)], cwd=test_dir)  # <-- TIER 3
        return {"success": True, ...}
```

Same pattern: **direct PS1 helper, not isolation layer**.

### 3. HiddenDesktopBackend Usage Search

Grep results for `HiddenDesktopBackend`:
- Line 121: Class definition
- Line 779: Instantiated in IsolationContextManager.get('hidden_desktop') — **fallback logic never triggered in practice**
- Line 795: Instantiated in `_auto_select()` fallback — **never called by server.py**

**Critical fact**: `isolation_layer.py` is **never imported** in `server.py`. The MCP server has zero dependency on the isolation layer module.

### 4. Bridge Layer (GameProcessManager.cs)

No calls to Win32 desktop isolation. Uses vanilla `Process.Start()` with `UseShellExecute=true` (lines 60–92). No integration with any Python isolation backend.

---

## Live Launch Path (One-Liner)

```
MCP game_launch(hidden=True) → server.py:467 _launch_on_vdd()
    [VDD lookup fail] → server.py:471 _launch_hidden()
    [PS1 Win32Desktop.CreateDesktop helper]
    [Creates hidden desktop via WIN32 API per-call]
    
MCP game_launch(hidden=False) → server.py:472 subprocess.Popen([exe], cwd=GAME_DIR)
    [TIER 3 — Primary desktop, no isolation]
```

Neither path invokes `HiddenDesktopBackend` from `isolation_layer.py`.

---

## Dead Code Summary

**HiddenDesktopBackend** (lines 121–435, 315 LOC):
- `capture_window()` (lines 131–139) — Placeholder stubs
- `capture_display()` (lines 141–147) — Placeholder stubs
- `inject_key()` (lines 149–155) — Async Win32 wrapper, never called
- `type_text()` (lines 157–166) — Async Win32 wrapper, never called
- `mouse_click()` (lines 168–174) — Async Win32 wrapper, never called
- `mouse_scroll()` (lines 176–182) — Async Win32 wrapper, never called
- `list_windows()` (lines 184–191) — Stub (returns empty list)
- `focus_window()` (lines 193–199) — Async Win32 wrapper, never called
- `launch_process()` (lines 201–207) — Delegates to `_launch_hidden()` (never called)
- Win32 helper methods: `_send_key()`, `_send_char()`, `_send_click()`, `_send_scroll()`, `_focus_window()`, `_launch_hidden()` (lines 213–434) — 222 LOC of untested async ctypes bindings

**Caller count**: 0 (outside of unit test stubs in isolation_layer itself).

**PlayCUABackend** (lines 561–749): Also unreferenced in `server.py`, though it at least has concrete RPC implementations (not stubs).

---

## Recommendations

### 1. **Delete HiddenDesktopBackend class (315 LOC)**
   - Reason: Never called, superseded by `_launch_hidden()` PS1 helper in server.py
   - Risk: None — no production callers
   - Effort: Delete lines 121–435 from `isolation_layer.py`

### 2. **Consolidate duplicated Win32 logic**
   - The `_launch_hidden()` PS1 script in server.py (lines 205–269, ~65 LOC) replicates the HiddenDesktopBackend's Win32 desktop creation
   - Extract to a shared `win32_desktop_launch()` helper in a new module `src/Tools/DinoforgeMcp/dinoforge_mcp/win32_helpers.py`
   - Call from both `_launch_hidden()` (server.py) and any future HiddenDesktopBackend replacement

### 3. **Clarify the hidden launch story**
   - Document which backend is actually used: VDD first (configured via `.dinoforge_vdd_index`), fallback to per-call PS1 CreateDesktop
   - Add observability: log to server.py which backend was used (VDD index or PS1 desktop name)
   - Consider renaming `_launch_hidden()` to `_launch_on_createdesktop()` for clarity

### 4. **Evaluate isolation_layer.py's role**
   - Current use: None (server.py doesn't import it)
   - Future: If playCUA integration is ever needed, import `isolation_layer` and call `get_isolation_context('playcua')` for cross-platform input/screenshot services
   - For now: Mark module as "experimental / not yet integrated" in a top-level comment

---

## Verdict Summary

| Metric | Value |
|--------|-------|
| **Wiring Status** | NOT WIRED |
| **Dead Code LOC** | 315 (HiddenDesktopBackend) + 222 (Win32 helpers) = 537 |
| **Actual Launch Method** | TIER 3 (primary desktop) or TIER 1 (VDD) via PS1 script |
| **Isolation Layer Usage** | 0 production callers |
| **User Observation Valid** | ✓ YES — DINO consistently launches on primary desktop because `hidden=True` is ignored (goes to VDD or PS1, neither uses the IsolationBackend abstraction) |

---

## Iter-142 Context

User reported: _"i keep seeing the native launch in prim desktop, not a hidden desktop or user that you can then rdp a windwo to"_

**Root cause confirmed**: The `hidden=True` parameter exists in the MCP signature but is NOT wired to the HiddenDesktopBackend implementation. The actual hidden launch path uses a one-off PS1 script with Win32 CreateDesktop API, which DOES work but is separate from the isolation_layer.py module.

The isolation_layer.py module is aspirational documentation (documented in CLAUDE.md, committed to repo) but not operationalized. It's a 3-tier abstraction (VDD → playCUA → HiddenDesktop) that could be useful for cross-platform test automation, but it's currently orphan code.
