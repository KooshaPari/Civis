# playCUA Integration (Phase 3-5) — Implementation Specification

## Overview
Integrate playCUA hidden desktop modules into DINOForge MCP by creating an isolation layer that abstracts backend selection (HiddenDesktop vs. playCUA vs. Docker).

## Deliverables

### 1. Create isolation_layer.py (180+ LOC)

**File**: `src/Tools/DinoforgeMcp/dinoforge_mcp/isolation_layer.py`

Core structure:
- `Frame` dataclass: `{data: bytes, width: int, height: int}`
- `WindowInfo` dataclass: `{hwnd: int, title: str, process_id: int, visible: bool}`
- `IsolationBackend` abstract base class with 9 async methods
- `IsolationContext` singleton for lazy backend initialization
- `HiddenDesktopBackend` (current Win32 logic, refactored from server.py)
- `PlayCUABackend` (JSON-RPC client to playCUA server on port 9000)

### 2. Refactor server.py (200 LOC modified)

Replace direct Win32 calls in these functions:
1. `game_launch()` — Remove CreateDesktop logic, use isolation context
2. `game_screenshot()` — Remove WgcCapture/BitBlt, use isolation context
3. `game_input()` — Remove SendInput, use isolation context
4. `game_navigate_to()` — Inherits isolation context via game_input
5. `game_analyze_screen()` — Inherits via refactored game_screenshot

Pattern:
```python
# OLD (direct Win32)
def game_screenshot(hidden=True):
    hwnd = FindWindowW(None, "Diplomacy is Not an Option")
    img = _wgc_capture(hwnd)
    return base64.b64encode(img)

# NEW (isolation layer)
async def game_screenshot(hidden=True, backend='auto'):
    context = IsolationContext.get(backend)
    frame = await context.capture_window("Diplomacy is Not an Option")
    return {
        "success": True,
        "data": base64.b64encode(frame.data),
        "width": frame.width,
        "height": frame.height
    }
```

### 3. Create test_isolation_layer.py

**File**: `scripts/test_isolation_layer.py`

Tests:
1. `test_hidden_desktop()` — HiddenDesktopBackend on Windows
2. `test_playcua()` — PlayCUABackend (graceful skip if unavailable)
3. `test_auto()` — Auto-selection (prefer playCUA, fallback to HiddenDesktop)

### 4. Update CHANGELOG.md & CLAUDE.md

**CHANGELOG.md**: Add feature entry:
```markdown
## [Unreleased]

### Added
- Isolation layer abstraction for game capture/input backends (HiddenDesktop, playCUA, Docker-ready)
- PlayCUABackend for playCUA JSON-RPC integration
- Backend auto-detection: tries playCUA first, falls back to HiddenDesktop
```

**CLAUDE.md**: Add section under "MCP Bridge":
```markdown
### Isolation Layer & Backend Selection

The DINOForge MCP uses an **isolation layer** to abstract capture/input operations
over multiple backend implementations:

#### Backends

1. **HiddenDesktopBackend** (Windows only)
   - Uses Win32 CreateDesktop for hidden game launches
   - Current implementation moved from server.py
   - Stable, battle-tested

2. **PlayCUABackend** (cross-platform)
   - Routes to playCUA JSON-RPC server (port 9000)
   - Provides cross-platform window/input/capture
   - Graceful fallback if playCUA unavailable
   - Requires `playcua_ci_test` repo cloned and `cargo run` available

3. **DockerBackend** (stub, future)
   - For headless/CI game automation
   - Planned for v0.20+

#### Backend Selection

Default behavior:
```python
# Auto: tries playCUA, falls back to HiddenDesktop
context = IsolationContext.get('auto')

# Explicit:
context = IsolationContext.get('hidden_desktop')  # Windows only
context = IsolationContext.get('playcua')         # Requires playCUA server
```

#### Starting playCUA Server

```bash
cd C:\Users\koosh\playcua_ci_test
cargo run -- --listen 127.0.0.1:9000
```

Or via script (TODO: add to scripts/):
```bash
./scripts/start-playcua.ps1
```
```

---

## Implementation Notes

1. **Error handling**: All RPC calls should raise with descriptive messages
2. **Logging**: Use logger.info/warning/error for backend selection and fallback
3. **Async**: All IsolationBackend methods must be `async def`
4. **Base64**: playCUA returns base64-encoded PNG in response; HiddenDesktop works with raw bytes
5. **Port**: playCUA defaults to 9000; make configurable via env var `PLAYCUA_PORT` if needed
6. **Cleanup**: PlayCUABackend should close aiohttp session on context cleanup
7. **Singleton**: IsolationContext is a singleton (one backend per MCP session)

---

## Testing Checklist

- [ ] isolation_layer.py created (180+ LOC, 4 classes)
- [ ] HiddenDesktopBackend passes test_hidden_desktop()
- [ ] PlayCUABackend (if playCUA available) passes test_playcua()
- [ ] Auto-selection logic works: tries playCUA, falls back to HiddenDesktop
- [ ] server.py refactored (5 functions use isolation context)
- [ ] game_screenshot() tested with both backends
- [ ] All existing game_* tools still work (backward compatibility)
- [ ] CHANGELOG.md updated
- [ ] CLAUDE.md updated with isolation layer section
- [ ] No new repo artifacts left in working directory

---

## Blockers for M5
**None** — M5 packs (warfare-starwars, warfare-modern) can proceed in parallel. This is infrastructure work for Phase 2 onwards game automation.

---

## Files to Create/Modify

| File | Action | LOC | Purpose |
|------|--------|-----|---------|
| isolation_layer.py | CREATE | 180+ | Backend abstraction |
| server.py | MODIFY | ~200 lines | Remove Win32 calls, use isolation |
| test_isolation_layer.py | CREATE | 50+ | Test both backends |
| CHANGELOG.md | MODIFY | ~5 lines | Feature entry |
| CLAUDE.md | MODIFY | ~30 lines | Integration docs |

---

## Estimated Implementation Time
**4-6 hours** (Haiku subagent with detailed spec)
