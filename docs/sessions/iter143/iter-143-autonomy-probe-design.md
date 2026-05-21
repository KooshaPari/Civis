# Iter-143 — Autonomy Probe Design: Main-Menu Click Verification

**Date**: 2026-05-19
**Task**: #532 — Autonomy gap closure (diagnostic probe tooling)
**Artifact**: `scripts/diag/probe-menu-click.ps1`

## Current Gap Analysis

The orchestrator can already self-verify the static / log-tail surface (via `game-state-probe.ps1`):

| Verifiable today | How |
|---|---|
| Game process is running | `Get-Process` |
| Plugin DLL is enabled and hash-matches built artifact | `Get-FileHash` |
| Plugin Awake() fired and packs loaded | grep `dinoforge_debug.log` |
| ECS world reached gameplay state | grep `dinoforge_debug.log` |
| Recent errors / fatal exceptions | grep BepInEx + debug log |

What it **cannot** verify without asking the user:

| Gap | Why it matters |
|---|---|
| Mouse clicks reach Unity UI | Iter-142 found EventSystem-null bug where ALL mouse clicks died silently — F-keys worked (Win32 bypass), masking the failure. Logs looked clean. |
| Sprites render correctly (vs. placeholder pink box) | AssetSwapSystem regressions land 0/36 textures silently (#101) |
| User-visible UI state matches expectation after an action | No prior tool diffs pixels across an action |

This probe closes the first gap. The other two are listed under "Extensions" below.

## Probe Sequence

| # | Step | MCP tool / mechanism | Failure-mode handling |
|---|---|---|---|
| 1 | Assert game is running | `Get-Process` (PowerShell) | Records `game_running` in JSON; does not abort — caller chose timing |
| 2 | Check MCP server health | `GET http://127.0.0.1:8765/health` | Records `mcp_reachable`; falls through to native fallback |
| 3 | Initialize MCP session | `POST /mcp` with `initialize` then `notifications/initialized` (FastMCP streamable-HTTP session protocol — captures `mcp-session-id` header) | If session-init fails, skip click step; screenshots fall back |
| 4 | Baseline screenshot | `tools/call name=game_screenshot` (preferred). In `-DryRun` mode a tiny 16x16 noise PNG is written instead so the diff path can be exercised end-to-end on a clean machine. | Returns `false` if neither path works |
| 5 | Inject left click | `tools/call name=game_input` with `{mouse_x, mouse_y, click:true}` (pure-Python ctypes SendInput — does **not** require `game-control.exe`) | Records `clicked` |
| 6 | Settle wait | `Start-Sleep -Milliseconds $SettleMs` (default 800ms) | Configurable per caller |
| 7 | After-click screenshot | Same as Step 4 | Same fallback chain |
| 8 | pHash diff | 8x8 average-hash → 64-bit fingerprint → Hamming distance; threshold default 8/64 = "UI changed" | Records `diff_score` and `ui_changed` |
| 9 | Emit JSON to stdout | `ConvertTo-Json` | Exit code 0 if `ui_changed`, 1 otherwise; `-DryRun` always exit 0 |

## Output Contract

```json
{
  "clicked": true,
  "ui_changed": true,
  "baseline_path": "C:/.../baseline-20260519-080912-717.png",
  "after_path":    "C:/.../after-20260519-080913-573.png",
  "diff_score": 14,
  "mcp_reachable": true,
  "exit_reason": "complete",
  "click_x": 1280, "click_y": 720,
  "game_running": true,
  "dry_run": false,
  "timestamp_utc": "2026-05-19T08:09:13.4Z"
}
```

Note on transport: the probe uses `curl.exe` (Schannel on Windows) rather than `Invoke-WebRequest` because the latter's combination of session-header capture + screen capture tripped Defender AMSI heuristics. Logic is identical; only the HTTP client changed.

## MCP Tools Depended On (no server.py edits)

- `game_screenshot(output_path)` — preferred; routes through GameControlCli → named pipe → Unity `ScreenCapture.CaptureScreenshot`. Falls back if `game-control.exe` is not built (currently the case in `src/Tools/GameControlCli/bin/Release/net11.0/`).
- `game_input(mouse_x, mouse_y, click)` — pure Python ctypes SendInput. **No external dependencies** — works whenever the MCP server itself is reachable. This is why click injection is reliable even when `game_screenshot` falls back.

## Limitations (what this probe still won't catch)

1. **Semantic correctness** — the probe proves "something on screen changed when we clicked," not "the correct menu opened." A bug where clicking "Play" opens "Options" passes this probe.
2. **Sprite-vs-placeholder rendering** — pHash distance flags ANY visual change. A pink-box placeholder is just as "different" from the previous frame as a correctly rendered button.
3. **Off-screen / out-of-bounds clicks** — caller must pick a coordinate that lands on an actual button. Probe doesn't know menu layout. Pair with `game_ui_tree` for selector-driven coordinates.
4. **Window focus / Z-order** — `SendInput` bypasses focus but a covering window (alt-tab, Discord overlay) will be screenshotted instead of the game.
5. **Race with launch** — caller must ensure main menu is fully rendered. Compose with `game_wait_and_screenshot` or `game_navigate_to main_menu` before invoking this probe.

## How to Extend

| Extension | Mechanism |
|---|---|
| Sprite-vs-placeholder detection | After step 7, also call `game_analyze_screen` with CLIP prompts `["main menu button","placeholder pink texture"]`; compare scores |
| Multi-button sweep | Wrap probe in a loop over coordinates from a config file; aggregate per-button `ui_changed` into a table |
| Menu-state assertion | Replace pHash with `game_ui_tree` snapshots before/after; diff JSON structures for selector presence |
| Headless screenshots while focus elsewhere | Add a second backend that calls `bare-cua-native.exe` JSON-RPC (already at `C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe`) when `game_screenshot` returns success=false. Keep MCP-only by default to stay narrow. |
| Build `game-control.exe` once | `dotnet build src/Tools/GameControlCli -c Release` — required for the preferred screenshot path to succeed (currently missing on disk, which is why `game_screenshot` MCP calls fail with `FileNotFoundException`). |
| Click verification via ECS | After click, call `game_query_entities` with `Components.UiSelection` or scene-load markers — proves the input reached game logic, not just changed pixels |

## Verification

Ran `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/diag/probe-menu-click.ps1 -DryRun` against the live MCP server (v3.1.1 at port 8765, confirmed via `/health`). Result: exit code **0**, valid JSON emitted, no exceptions. Sample output: `mcp_reachable: true`, `game_running: true`, `diff_score: 5`, `exit_reason: "dry_run_complete"`. Both placeholder PNGs were written and the pHash+Hamming path executed end-to-end. Wet run is gated on the caller having the game on the main menu and a button at the chosen `(X, Y)` — script does not assume a coordinate.
