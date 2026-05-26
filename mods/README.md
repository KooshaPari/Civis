# CivLab mods (CIV-0700)

| Path | Purpose |
|------|---------|
| `manifest.schema.json` | JSON Schema for `manifest.toml` |
| `example-policy/` | Reference PolicyMod (manifest + optional WASM) |
| `example-economic/` | Reference EconomicMod (manifest + optional WASM) |

## Build example mod WASM

```bash
just civis-3d-mod-wasm
just civis-3d-mod-package       # example-policy.civmod
just civis-3d-mod-package-all   # policy + economic .civmod archives
```

Artifacts (`mod.wasm`, `*.civmod`) are gitignored; build via `just` (target: `wasm32-unknown-unknown`).

After `just civis-3d-mod-wasm`, `cargo test -p civ-mod-host example_policy_dir_wasm` and `example_economic_dir_wasm` assert live WASM ticks.

## Signing (partial, FR-CIV-TACTICS-043)

Production loads verify `mod.wasm` when **both** are present:

- `author_pubkey_hex` in `manifest.toml` (32-byte Ed25519 public key, hex)
- `mod.wasm.sig` beside the manifest (directory) or inside the `.civmod` ZIP (64-byte detached signature)

Unsigned mods (no pubkey, no sig) still load for local iteration.

Packaging today (`scripts/package-example-mod.ps1`) emits `manifest.toml` + `mod.wasm` only; add `mod.wasm.sig` manually or extend the script when publishing signed builds.

## `mod-dev` feature

`cargo test -p civ-mod-host --features mod-dev` (or dependents built with `mod-dev`) **skips** WASM determinism scans and Ed25519 verification so example trees without signatures keep passing CI on dev machines.

Default builds (no `mod-dev`) enforce determinism and verify signatures when pubkey + sig are present.

## Verify

```bash
cargo test -p civ-mod-host
cargo test -p civ-mod-host signed_mod --quiet
just civis-3d-scenario-check
```
