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

## Upload workflow (civ-watch)

`POST /control/mods/upload` accepts JSON `{ "filename": "my-mod.civmod", "data_base64": "..." }`. The archive is written to `mods/uploads/<sanitized-name>.civmod`, validated as a ZIP with `manifest.toml` (and Ed25519 signature when `author_pubkey_hex` is set), then returned as `{ "ok": true, "source": "mods/uploads/my-mod.civmod" }` for catalog listing and `POST /control/mods/install`.

## Publish workflow (local store)

`POST /control/mods/publish` accepts JSON `{ "source": "mods/uploads/my-mod.civmod" }` (path under repo `mods/`, no `..`). The archive is copied to `mods/publish/<manifest.meta.id>.civmod` and returned as `{ "ok": true, "published_source": "mods/publish/my-mod-id.civmod" }`. List published entries with `GET /control/mods/published` (`id`, `name`, `version`, `source`). Published `.civmod` files appear in the mod catalog and can be installed like uploads.

## Remote fetch registry

`mods/remote-registry.json` gates `POST /control/mods/fetch` when `require_registry` is true. Each entry may set `url_prefix`, optional `mod_id`, `require_signature`, and `allowed_pubkeys` (hex). With `require_registry: false` (default), any `http`/`https` URL is allowed; archives still pass `.civmod` validation and Ed25519 checks when `author_pubkey_hex` and `mod.wasm.sig` are present.

## Verify

```bash
cargo test -p civ-mod-host
cargo test -p civ-mod-host signed_mod --quiet
just civis-3d-scenario-check
```
