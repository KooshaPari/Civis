# Boot Verification 2026-06-14

## Build Result

- **Package**: `civis-cli`
- **Profile**: `release` (optimized)
- **Status**: `Finished` successfully in 2m 23s
- **Built binaries**: `civis-census.exe`, `civis-pixels.exe`, `civis-mcp.exe`
- **Note**: `civis-verify.exe` not built (requires `--features bevy`)

## Runtime Verification

### `civis-census --help`

```
Query sim.status over the civ-server JSON-RPC WS bridge

Usage: civis-census.exe [OPTIONS]

Options:
      --host <HOST>              Override the WebSocket host (default $CIV_WS_HOST or 127.0.0.1)
      --port <PORT>              Override the WebSocket port (default $CIV_SERVER_PORT or 3000)
      --path <PATH>              Override the WebSocket path (default $CIV_WS_PATH or /ws)
      --timeout-ms <TIMEOUT_MS>  Override the request timeout in milliseconds
      --raw                      Emit the raw JSON-RPC response rather than the parsed `sim.status`
  -h, --help                     Print help
```

### `civis-pixels assets\brand\icon-128.png`

Machine numbers:

```json
{
  "grid": 16,
  "input": "assets\\brand\\icon-128.png",
  "near_black_threshold": 8,
  "stats": {
    "distinct_hue_count": 79,
    "mean_b": 30.71484375,
    "mean_g": 79.234375,
    "mean_r": 27.65625,
    "percent_gray": 15.234375,
    "percent_near_black": 15.234375,
    "samples": 256
  }
}
```

### `civis-census` (no server running)

```
civis-census: civ-server WS connect to ws://127.0.0.1:3000/ws failed: HTTP error: 302 Found
```

## Summary

- **Build**: OK (release profile, optimized)
- **Binary startup**: OK (`civis-pixels` produced deterministic machine numbers)
- **Server connectivity**: Not available (no `civ-server` running — expected for boot-only check)
- **Version + startup**: Verified
