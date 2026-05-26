# Iter-143 Wave 2 MCP Restart Receipt

**Date:** 2026-05-19 (iter-143 wave 2 follow-up to tasks #536/#537)
**Goal:** Restart `dinoforge-mcp` (FastMCP HTTP server on `127.0.0.1:8765`) to pick up the WGC capture backend integration — multi-tier `game_screenshot` chain + new `game_screenshot_wgc` tool — and verify the new tools are reachable via the MCP protocol.

---

## Verdict: PASS

The MCP server was successfully restarted and the new tool surface from the iter-143 wave 2 WGC integration is live:

- `game_screenshot_wgc` tool is **registered and reachable** via `tools/list`
- `game_screenshot` tool description now references the **multi-tier fallback chain** (WGC -> GameControlCli) and cites `docs/proposals/wgc-capture-backend-design.md` / task #536
- Total tool count: **43** (consistent with the iter-143 surface)

---

## Pre-Restart State

```
$ curl http://127.0.0.1:8765/health
{"status":"ok","server":"dinoforge-mcp","version":"0.13.0"}
```

Running MCP process before kill (via WMI):

```
ProcessId   : 44392
Name        : python.exe
CommandLine : "C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe" -m dinoforge_mcp.server --http --port 8765 --host 127.0.0.1
```

Source-side confirmation that the new tool exists in `server.py`:

- `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py:415` declares `async def game_screenshot_wgc(...)`
- `server.py:1484` exposes the health endpoint with `version: 0.13.0` (server.py does not bump the published version string for this wave; behavioral surface is what changed)

---

## Restart Procedure

1. **Killed PID 44392** via `Stop-Process -Id 44392 -Force`. Verified no `python.exe` process matching `*dinoforge_mcp*` remained after 3s sleep.
2. **Relaunched** detached:
   ```
   Start-Process -FilePath "C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe" `
                 -ArgumentList "-m","dinoforge_mcp.server","--http","--port","8765","--host","127.0.0.1" `
                 -WorkingDirectory "C:\Users\koosh\Dino\src\Tools\DinoforgeMcp" `
                 -WindowStyle Hidden
   ```
   New PID observed: **181404** (python.exe).
3. Polled `/health` until ready. Server was reachable on attempt 0 of the polling loop (cold-start time ~9-12s end-to-end).

> Note: The repo also has `UserPromptSubmit` / `SessionStart` hooks that invoke `scripts/start-mcp.ps1 -Action start -Detached`, so the server would have been re-spawned on the next prompt anyway. The manual relaunch here was performed within the same wall-clock window to minimize MCP downtime.

---

## Post-Restart State

```
$ curl http://127.0.0.1:8765/health
{"status":"ok","server":"dinoforge-mcp","version":"0.13.0"}
```

Version string unchanged (server.py did not bump `0.13.0` -> `0.14.0` for this wave). The new behavior is the tool surface, not the version literal.

---

## MCP Handshake + tools/list

Probe script: `scripts/diag/_mcp-tools-probe.ps1` (temporary, removed at session end). It follows the FastMCP streamable-HTTP pattern from `scripts/diag/probe-menu-click.ps1`:

1. `POST /mcp` with `initialize` -> captured session id `3341d94a151a4d7e91e0baab2fd4ede8` from the `Mcp-Session-Id` header.
2. `POST /mcp` with `notifications/initialized` (mcp-session-id header attached).
3. `POST /mcp` with `tools/list`. SSE-framed; the `data:` line was parsed to JSON.

### Result Highlights

```
TOOL_COUNT=43
HAS_game_screenshot_wgc=True
```

### `game_screenshot` description (excerpt — full text returned by tools/list)

> Capture a screenshot of the game window using a multi-tier fallback chain.
>
> Tier order (per `docs/proposals/wgc-capture-backend-design.md`, task #536):
> 1. WGC (Windows.Graphics.Capture, via bare-cua-native) — foreground-independent, survives hung Unity / DXGI exclusive fullscreen / non-focused windows. 5s timeout; falls through silently on failure.
> 2. GameControlCli "screenshot" — named pipe -> Unity ScreenCapture.CaptureScreenshot (GPU backbuffer). Highest-fidelity path but BLOCKS when the game hangs.
> 3. (Future) Last-resort PrintWindow / GDI via HiddenDesktopBackend.
>
> The returned dict includes a `backend` field ("wgc" | "game_control_cli") indicating which tier succeeded, in addition to the original keys produced by GameControlCli (success, path, error, etc.).
>
> Args: `output_path`, `pipe_name` (only used by GameControlCli fallback tier).

### Full tools/list payload (names only)

```
game_status
game_wait_world
game_wait_for_world
game_resources
game_get_resources
game_screenshot                  <-- updated description (multi-tier)
game_screenshot_wgc              <-- NEW tool from iter-143 wave 2
game_query_entities
game_ui_tree
game_click_button
game_load_scene
game_start
game_dismiss
game_catalog
game_launch
game_launch_test
game_launch_vdd
asset_validate
asset_import
asset_optimize
asset_build
pack_validate
pack_build
pack_list
game_get_stat
game_apply_override
game_get_component_map
game_reload_packs
game_verify_mod
game_dump_state
game_input
game_ui_automation
game_analyze_screen
game_wait_and_screenshot
game_navigate_to
catalog_keys
catalog_bundles
log_tail
log_swap_status
log_bepinex
log_debug_log
log_packs_loaded
notify_hmr
```

Full JSON payload of `tools/list` was persisted to `$env:TEMP\DINOForge\mcp-tools-list.json` for this run.

---

## Process Summary

| Stage | Action | Result |
|---|---|---|
| Pre-check | `GET /health` | 200 OK, version 0.13.0, PID 44392 |
| Kill | `Stop-Process -Id 44392 -Force` | Process gone after 3s |
| Relaunch | `Start-Process python -m dinoforge_mcp.server ...` | New PID 181404, hidden window |
| Health re-check | `GET /health` (with 2s poll) | 200 OK on first attempt (server warm-up ~9-12s) |
| MCP handshake | initialize -> notifications/initialized | session 3341d94a... |
| tools/list | tools/list call | 43 tools, includes `game_screenshot_wgc` |

No process orphans, no port conflicts, no auth/handshake failures. The UserPromptSubmit hook did not race with the manual relaunch within this window.

---

## Verdict (restated)

**PASS.** The MCP server is running the iter-143 wave 2 code: `game_screenshot` is now the multi-tier WGC-first orchestrator, and `game_screenshot_wgc` is exposed as a dedicated tool. Both are reachable via `tools/list` over the FastMCP streamable-HTTP protocol on `http://127.0.0.1:8765/mcp`.
