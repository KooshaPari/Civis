# Live Bridge / Journey Evidence Blocker Report

**Date:** 2026-05-23  
**Observed at:** 2026-05-23T15:36:43-07:00 (initial failure)  
**Re-verified at:** 2026-05-23T20:40:37-07:00  
**Status:** **PARTIAL** — bridge response path restored; minimal journey evidence collected (not full phenotype record pipeline)

## Result

The named-pipe bridge now answers `GameControlCli status` and `screenshot` end-to-end. Minimal journey evidence (status receipt + one in-game screenshot) was collected via automated smoke.

Full phenotype journey manifests (`phenotype-journey record` / multi-step keyframe sets) are **not** yet produced by this run — treat visual acceptance as pending until a richer capture pass.

## What was checked (initial — blocked)

| Check | Command | Result |
|---|---|---|
| Named pipe presence | `Test-Path \\.\pipe\dinoforge-game-bridge` | `True` |
| MCP health endpoint | `Invoke-RestMethod http://127.0.0.1:8765/health` | `{"status":"ok","server":"dinoforge-mcp","version":"0.13.0"}` |
| Direct bridge status | `dotnet run --project src\Tools\GameControlCli\GameControlCli.csproj -- status` | Connects to pipe, then times out waiting for a live response |

## Re-verification (bridge restored)

| Check | Command | Result |
|---|---|---|
| Named pipe presence | `Test-Path \\.\pipe\dinoforge-game-bridge` | `True` |
| MCP health endpoint | `Invoke-RestMethod http://127.0.0.1:8765/health` | `{"status":"ok","server":"dinoforge-mcp","version":"0.13.0"}` |
| Direct bridge status | `dotnet run --project src\Tools\GameControlCli\GameControlCli.csproj -- status` | **OK** — `Running: True`, `World ready: True`, entity count ~49150 |
| Live journey smoke | `pwsh -File scripts/qa/live-bridge-journey-smoke.ps1` | **PASS** (`overall_pass: true`) |

## Evidence paths

| Artifact | Path |
|---|---|
| Smoke receipt (step log + timestamps) | `docs/qa/evidence/live-bridge-journey_2026-05-23/smoke-receipt.json` |
| Bridge status screenshot | `docs/qa/evidence/live-bridge-journey_2026-05-23/bridge-status-screenshot.png` |
| Repeatable smoke script | `scripts/qa/live-bridge-journey-smoke.ps1` |

`DINO_GAME_PATH` was set during collection (`G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe`). The smoke script still runs pipe/MCP/status checks when it is unset; screenshot capture requires a live bridge response (typically with the game running).

## Original blocker (resolved for bridge I/O)

`GameControlCli` successfully opens `dinoforge-game-bridge`, writes the request frame, and then fails while waiting for the response:

- `System.OperationCanceledException` from `GameClient.ReadFramedMessageAsync`
- fallback response returned: `{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Bridge error"}}`

**Current state:** status and screenshot complete in ~250–350 ms per request when the game bridge plugin is loaded and the world is running.

## Current live-evidence gap

The remaining gap in this workspace is operator timing, not a missing code path:

- MCP config in [`.claude/mcp-servers.json`](../../.claude/mcp-servers.json) now injects `DINOFORGE_PIPE_NAME=dinoforge-game-bridge`
- `GameClientOptions.PipeName` and the CLI default already resolve to `dinoforge-game-bridge`
- The Python MCP wrapper now falls back to `dinoforge-game-bridge` when a configured pipe is absent and the default pipe exists
- Live pipe presence checks in this session returned `False` for both `\\.\pipe\dinoforge-game-bridge` and `\\.\pipe\dinoforge_game`

Operator action: launch the game with the bridge plugin loaded, then re-run `game_status` against the default pipe after confirming the current instance exposes it.

## Evidence commands

```powershell
# Automated smoke (preferred)
pwsh -File scripts/qa/live-bridge-journey-smoke.ps1

# Manual spot checks
Test-Path -LiteralPath "\\.\pipe\dinoforge-game-bridge"
Invoke-RestMethod -Uri "http://127.0.0.1:8765/health" -Method Get -TimeoutSec 5
dotnet run --project src\Tools\GameControlCli\GameControlCli.csproj -- status
dotnet run --project src\Tools\GameControlCli\GameControlCli.csproj -- screenshot docs\qa\evidence\live-bridge-journey_2026-05-23\manual.png
```

## Actionable next step

Run a multi-step phenotype journey capture (`tools/phenotype-journeys` record/verify or GameControlCli `demo`) and promote screenshots through [Phenotype Journey Visual Acceptance Gate](phenotype-journey-visual-acceptance.md) once lighting/terrain evidence meets pass criteria.
