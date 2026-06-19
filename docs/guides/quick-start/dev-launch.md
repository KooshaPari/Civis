# Dev Launch

One-command launch for the Civis Bevy desktop client.

## Play

```powershell
just play
```

What it does:

1. Kills any running `civ-standalone` process gracefully (non-erroring when none found).
2. Builds `civ-bevy-ref` release binary (`--features bevy,egui --bin civ-standalone`).
3. Launches the binary detached; stdout + stderr stream to `.process-compose/logs/civ-standalone.log`.
4. Writes the PID to `.process-compose/pids/civ-standalone.pid`.
5. Prints `Game ready (pid <N>)` and tails the log until the window closes.

## Debug variants

### Debug logging

```powershell
just play-debug
```

Sets `RUST_LOG=info,civ_bevy_ref=debug,wgpu=warn` — verbose game logic, quiet GPU driver noise.

### Full backtrace

```powershell
just play-trace
```

Sets `RUST_LOG=info,civ_bevy_ref=debug,wgpu=warn` and `RUST_BACKTRACE=full` — full Rust panic traces.

## Stop

```powershell
just stop
```

Kills any running `civ-standalone` process.

## Logs

```powershell
just logs
```

Tails `.process-compose/logs/civ-standalone.log` live (`Get-Content -Wait` on Windows, `tail -f` on Linux/macOS).
You can also open the file directly:

```
.process-compose/logs/civ-standalone.log
.process-compose/logs/civ-standalone.err.log
```

## In-game controls

| Input | Action |
|---|---|
| W / A / S / D | Pan camera |
| Z / Space | Move camera up |
| Shift | Move camera down |
| Mouse wheel | Zoom in / out |
| Right-drag | Orbit camera |
| T | Open tech tree |
| G | Open diplomacy panel |
| L | Open event log |
| Esc | Pause / resume |
| 1 / 2 / 3 / 4 | Set simulation speed (pause / 1x / 2x / 4x) |
| Left-click | Select / use active tool |

## Infra dev stack

To start backing services (Postgres, DragonFly, NATS, MinIO, civ-watch) separately:

```powershell
just dev        # start
just dev-stop   # stop
```

Logs for each service live under `.process-compose/logs/`.
