# CivLab mods (CIV-0700)

| Path | Purpose |
|------|---------|
| `manifest.schema.json` | JSON Schema for `manifest.toml` |
| `example-policy/` | Reference PolicyMod (manifest + optional WASM) |
| `example-economic/` | Reference EconomicMod (manifest + optional WASM) |

## Build example mod WASM

```bash
just civis-3d-mod-wasm
just civis-3d-mod-package   # optional example-policy.civmod ZIP
```

Artifacts (`mod.wasm`, `*.civmod`) are gitignored; build via `just` (target: `wasm32-unknown-unknown`).

After `just civis-3d-mod-wasm`, `cargo test -p civ-mod-host example_policy_dir_wasm` and `example_economic_dir_wasm` assert live WASM ticks.

## Verify

```bash
cargo test -p civ-mod-host
just civis-3d-scenario-check
```
