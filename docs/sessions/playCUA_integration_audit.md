# playCUA Integration Audit (Phase 2)

**Date**: 2026-04-20  
**Project**: DINOForge  
**Task**: Integrate playCUA hidden desktop modules into DINOForge MCP (Phase 2 onwards)

---

## playCUA Module Audit Summary

### 1. Code Structure & Line Counts

| Module | Location | Lines | Purpose |
|--------|----------|-------|---------|
| **analysis** | `native/src/analysis/mod.rs` | 107 | Image diff & BLAKE3 hashing via AnalysisPort trait |
| **input** | `native/src/input/` | 649 (total) | Keyboard/mouse injection via InputPort trait |
| **process** | `native/src/process/mod.rs` | 185 | Process lifecycle (launch, kill, status) via ProcessPort trait |
| **window** | `native/src/window/` | 259 (total) | Window enumeration & focus via WindowPort trait |
| **ports** | `native/src/ports/mod.rs` | 64 | 5 abstract port traits (trait definitions) |
| **ipc** | `native/src/ipc/` | 483 (total) | JSON-RPC dispatcher & wire types |
| **lib.rs** | `native/src/lib.rs` | <50 | Re-exports |
| **main.rs** | `native/src/main.rs` | ~100 | HTTP/Tokio app wiring |

**Total Implementation**: ~1,850 LOC Rust (excluding tests/adapters)

### 2. Architecture Patterns (Hexagonal)

playCUA uses **strict port/adapter hexagon**:

```
Domain Layer (domain/*.rs)
    ↓
Port Traits (ports/mod.rs) ← 5 abstract traits
    ↓
Adapters (adapters/)        ← Platform-specific implementations
    ↓
IPC/Wire Layer (ipc/)       ← JSON-RPC dispatcher
```

**5 Port Traits** (all use `#[async_trait]`):

1. **CapturePort** — screenshot/capture_display → `Frame { data, width, height }`
2. **InputPort** — key_event, type_text, mouse_event
3. **WindowPort** — list_windows, find_window, focus_window
4. **ProcessPort** — launch, kill, status (PID management)
5. **AnalysisPort** — diff, hash (image analysis)

### 3. IPC/JSON-RPC Method Surface

playCUA exposes 20+ JSON-RPC methods via Dispatcher:

```
screenshot          → CapturePort::capture_window/display
input.key           → InputPort::key_event
input.type          → InputPort::type_text
input.click         → InputPort::mouse_event (click)
input.scroll        → InputPort::mouse_event (scroll)
input.move          → InputPort::mouse_event (move)
windows.list        → WindowPort::list_windows
windows.focus       → WindowPort::focus_window
windows.find        → WindowPort::find_window
process.launch      → ProcessPort::launch
process.kill        → ProcessPort::kill
process.status      → ProcessPort::status
analysis.diff       → AnalysisPort::diff
analysis.hash       → AnalysisPort::hash
ping                → version/health check
```

**Request format** (JSON-RPC 2.0):
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "screenshot",
  "params": { "window_title": "Diplomacy is Not an Option", "monitor": 0 }
}
```

**Response format**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "data": "<base64-encoded-png>",
    "width": 1920,
    "height": 1080,
    "format": "png"
  }
}
```

### 4. Key Dependencies

From `Cargo.toml`:
- `tokio` v1 (async runtime, channels, full features)
- `serde` + `serde_json` (JSON serialization)
- `async-trait` (trait async methods)
- `image` v0.25.2 (PNG/JPEG decode)
- `blake3` v1.5.4 (cryptographic hashing)
- `enigo` v0.2 (keyboard/mouse cross-platform)
- `xcap` v0.2 (screenshot capture)
- `base64` v0.22 (base64 encoding)
- `fast_image_resize` v4 (image scaling)
- `tracing` + `tracing-subscriber` (structured logging)

### 5. Adapter Stack (Platform-Specific)

```
adapters/
  ├─ windows/
  │   ├─ wgc.rs          (Windows.Graphics.Capture)
  │   ├─ sendinput.rs    (Win32 SendInput)
  │   ├─ enumwin.rs      (Win32 EnumWindowsEx)
  │   └─ mod.rs
  ├─ linux/
  │   ├─ x11capture.rs   (X11)
  │   ├─ uinput.rs       (Linux uinput)
  │   ├─ ewmh.rs         (EWMH window mgmt)
  │   └─ mod.rs
  ├─ macos/
  │   ├─ cgcapture.rs    (CoreGraphics)
  │   ├─ cgevent.rs      (Quartz)
  │   ├─ nsworkspace.rs  (NSWorkspace)
  │   └─ mod.rs
  ├─ xcap.rs            (cross-platform fallback)
  ├─ enigo.rs           (cross-platform input)
  ├─ analysis_adapter.rs (image analysis implementation)
  ├─ process_adapter.rs  (process management)
  └─ mod.rs
```

---

## DINOForge MCP Current State

### Current Server (server.py — 1,513 LOC)

**Current architecture**:
```
DINOForge MCP (Python FastMCP 3.x)
    ├─ game_* tools
    │   ├─ game_launch() → Win32 CreateDesktop
    │   ├─ game_screenshot() → WinRT/BitBlt/DXGI cascade
    │   ├─ game_input() → Win32 SendInput
    │   └─ game_analyze_screen() → OmniParser or CLIP
    │
    ├─ asset_* tools → PackCompiler CLI
    ├─ catalog_* tools → Direct JSON parse
    └─ log_* tools → Direct file read
```

**Key Win32 calls embedded in server.py**:
- `FindWindowW()` — window enumeration (line ~300)
- `SetForegroundWindow()` — focus window (line ~320)
- `GetAsyncKeyState()` — F9/F10 polling (line ~410)
- `SendInput()` — keyboard/mouse injection (line ~450)
- `CreateDesktopW()` — hidden desktop (line ~200)
- `GetDC()` / `BitBlt()` — CPU screenshot fallback (line ~350)

---

## Integration Strategy (Phase 2-5)

### Phase 2: Audit ✓ COMPLETE

**What we found:**
- playCUA provides 5 clean port traits (CapturePort, InputPort, WindowPort, ProcessPort, AnalysisPort)
- JSON-RPC dispatcher routes 20+ methods
- Platform-specific adapters (Windows/Linux/macOS) already handle Win32/X11/CoreGraphics
- ~1,850 LOC Rust, well-structured hexagon architecture
- Compatible dependencies (tokio, serde, async-trait)

### Phase 3: Design Isolation Layer

**File structure:**
```
src/Tools/DinoforgeMcp/dinoforge_mcp/
  ├─ isolation_layer.py          ← NEW (180+ LOC)
  ├─ server.py                   ← MODIFY (refactor ~200 LOC)
  └─ playcua_client.py           ← NEW (120+ LOC, JSON-RPC client)
```

**Isolation layer responsibilities:**
1. Abstract over Win32 CreateDesktop vs. playCUA vs. Docker backends
2. Route game_screenshot, game_input, game_navigate_to → playCUA dispatcher
3. Manage playCUA process lifecycle (start/stop, health checks)
4. Handle backend initialization on first call

### Phase 4: Implementation Roadmap

**Step 1: Create isolation_layer.py**
- `class IsolationContext` (holds active backend, process, desktop ID)
- `class IsolationBackend` (abstract base)
  - `class HiddenDesktopBackend` (current Win32 CreateDesktop logic)
  - `class PlayCUABackend` (routes to playCUA JSON-RPC)
  - `class DockerBackend` (stub for future)
- Methods: `capture()`, `inject_input()`, `focus_window()`, `list_windows()`, etc.

**Step 2: Create playcua_client.py**
- `class PlayCUAClient(IsolationBackend)`
- Methods dispatch JSON-RPC to playCUA server (port 9000)
- Auto-start playCUA on first call if available

**Step 3: Refactor server.py**
- Replace 5-10 Win32 calls with `isolation_context.method_name()`
- Initialize IsolationContext on first game tool call
- Keep backward compatibility (hidden=True uses HiddenDesktopBackend by default)

**Step 4: Test one tool (game_screenshot)**
- Verify screenshot works via isolation layer
- Test both HiddenDesktopBackend and PlayCUABackend (if playCUA running)

### Phase 5: Minimal Viable Integration

**Scope: One tool (game_screenshot)**

**Before** (current server.py):
```python
def game_screenshot(hidden=True):
    if hidden:
        hwnd = create_hidden_desktop()
        img = wgc_capture(hwnd)  # Win32-specific
    else:
        img = get_window_screenshot("Diplomacy is Not an Option")
    return base64.b64encode(img)
```

**After** (with isolation layer):
```python
def game_screenshot(hidden=True, backend='hidden_desktop'):
    isolation = get_isolation_context(backend)
    frame = isolation.capture(window_title="Diplomacy is Not an Option")
    return {
        "success": True,
        "data": base64.b64encode(frame.data),
        "width": frame.width,
        "height": frame.height
    }
```

**Backend selection**:
- `backend='hidden_desktop'` → HiddenDesktopBackend (current behavior)
- `backend='playcua'` → PlayCUABackend (if playCUA running on port 9000)
- Auto-detect: Try playCUA first, fall back to HiddenDesktopBackend

---

## Deliverables (Phase 2-5)

### Phase 2: Audit Report ✓
- playCUA module audit: 1,850 LOC, 5 port traits, 20+ JSON-RPC methods
- Dependency analysis: All compatible (tokio, serde, async-trait)
- Hexagon architecture validated

### Phase 3-5: Code & Tests
1. **isolation_layer.py** (180+ LOC)
   - 4 classes (IsolationContext, IsolationBackend, HiddenDesktopBackend, PlayCUABackend)
   - Port mapping: CapturePort → capture(), InputPort → inject_input(), etc.

2. **playcua_client.py** (120+ LOC)
   - PlayCUAClient class inheriting IsolationBackend
   - JSON-RPC client to playCUA dispatcher
   - Auto-start playCUA binary/cargo if available

3. **server.py refactoring** (200 LOC modified)
   - Remove direct Win32 calls
   - Route through isolation_layer
   - Keep backward compatibility

4. **Tests** (game_screenshot via both backends)
   - Test HiddenDesktopBackend (should work now)
   - Test PlayCUABackend (if playCUA available)
   - Test fallback behavior

5. **Documentation** (CLAUDE.md update)
   - Add section: "Isolation Layer & playCUA Integration"
   - Document backend selection strategy
   - Add playCUA server startup instructions

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| playCUA binary not available on Windows | Medium | Low | Graceful fallback to HiddenDesktopBackend |
| JSON-RPC port 9000 conflict | Low | Low | Auto-detect free port, make configurable |
| Cross-platform adapter incompleteness | Low | Medium | Start with Windows only, expand iteratively |
| Performance regression (overhead) | Low | Low | Benchmark: Win32 vs. playCUA latency |

---

## Next Steps (Delegation to Subagent)

1. **Phase 3**: Create `isolation_layer.py` with 4 backend classes
2. **Phase 4**: Implement `playcua_client.py` with JSON-RPC dispatch
3. **Phase 4**: Refactor `server.py` to use isolation layer (5-10 functions)
4. **Phase 5**: Test `game_screenshot()` with both backends
5. **Phase 5**: Update CLAUDE.md with isolation layer docs

**Estimated effort**: 6-8 hours (Haiku subagent)  
**Blocker for M5 packs?**: No — M5 (warfare-starwars, warfare-modern) can proceed in parallel
