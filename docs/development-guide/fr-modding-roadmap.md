# FR-CIV-MOD — Modding roadmap (CIV-0700)

**Spec:** [`CIV-0700-modding-api-spec.md`](../specs/CIV-0700-modding-api-spec.md)  
**Maturity audit:** [`fr-ax-dx-ux-maturity-audit.md`](./fr-ax-dx-ux-maturity-audit.md) § P3

## v1 — Manifest only (Sprint D, current)

| Item | Status | Location |
|------|--------|----------|
| Manifest JSON Schema | Done | `mods/manifest.schema.json` |
| Example PolicyMod manifest | Done | `mods/example-policy/manifest.toml` |
| `civ-mod-host` crate | Stub | `crates/mod-host` — load + validate manifest, no-op `tick` |
| Scenario `mods: []` | Placeholder | `crates/engine/src/scenario.rs` — list of mod directory paths |
| Engine hook | Placeholder | `Simulation::register_mod_stubs`, `mod_host.tick()` each sim tick |

**What works today**

- Parse and validate manifests aligned with CIV-0700 §4.1–4.2 (id, description length, api_version, runtime caps).
- Register mod directories from scenario YAML (paths relative to repo root from engine crate).
- CI: `cargo test -p civ-mod-host`.

**What does not work yet**

- WASM load, sandbox, `world_read` / `action_emit` (§5–8).
- `.civmod` bundles, `civlab-sdk`, mod signing (§11–14).
- Policy phase injection, lifecycle events `mod.loaded.v1` (traceability FR-MOD-004).

## v2 — Host registry + policy stub (planned)

1. `ModRegistry` in `civ-mod-host` with capability sets from manifest permissions.
2. Engine Phase 3a callsite (no WASM): log-only policy hook.
3. `mod.loaded.v1` / `mod.error.v1` on replay bus (see EVENT_TAXONOMY).

## v3 — WASM sandbox (planned)

1. `wasmtime` guest load after manifest validation.
2. Determinism scan (§3.4).
3. Example PolicyMod compiled against `civlab-sdk` in CI.

## v4 — Save/load + distribution (planned)

1. Mod state blobs (CIV-1000 §16.3).
2. `.civmod` ZIP packaging.
3. Dev-mode vs signed mods.

## Run / verify

```bash
cargo test -p civ-mod-host --quiet
cargo test -p civ-engine scenario_mods --quiet
```

## Related

- [`AGENTS.md`](../../AGENTS.md) — do not implement full CIV-0700 until `crates/mod-host` exists (now exists as stub).
- [`TRACEABILITY_MATRIX.md`](../traceability/TRACEABILITY_MATRIX.md) — FR-MOD-001..005 rows remain `planned`.
