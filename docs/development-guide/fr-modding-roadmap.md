# FR-CIV-MOD — Modding roadmap (CIV-0700)

**Spec:** [`CIV-0700-modding-api-spec.md`](../specs/CIV-0700-modding-api-spec.md)
**Maturity audit:** [`fr-ax-dx-ux-maturity-audit.md`](./fr-ax-dx-ux-maturity-audit.md) § P3

## v1 — Manifest only (Sprint D, current)

| Item | Status | Location |
|------|--------|----------|
| Manifest JSON Schema | Done | `mods/manifest.schema.json` |
| Example PolicyMod manifest | Done | `mods/example-policy/manifest.toml` |
| `civ-mod-host` crate | **Partial (v3)** | `crates/mod-host` — manifest, `.civmod` ZIP, `wasmtime` policy tick |
| `civlab-sdk` guest | **Partial** | `crates/civlab-sdk` — `civlab_policy_tick` export |
| Scenario `mods: []` | Done | `scenarios/baseline.yaml` lists `mods/example-policy` when path validates |
| Engine hook | Stub | `register_mod_stubs`; policy phase at `phase_economy` via `ModHost::tick` |

**What works today**

- Parse and validate manifests aligned with CIV-0700 §4.1–4.2 (id, description length, api_version, runtime caps).
- Register mod directories from scenario YAML (paths relative to repo root from engine crate).
- CI: `cargo test -p civ-mod-host`.

**What does not work yet**

- Full capability API: `world_read` / `action_emit` (§5–8).
- Economic WASM phase hooks; mod signing (§14).
- Replay bus JSON `mod.loaded.v1` (engine has `ReplayEvent::ModLoaded` only).

## v2 — Host registry + policy stub — Done (stub)

| Item | Status | Location |
|------|--------|----------|
| `ModRegistry` | Done (stub) | `crates/mod-host` — `on_policy_phase` filters policy + `write_policy` |
| Engine Phase 3a callsite | Done (stub) | `Simulation::phase_economy` — `tracing::debug!` per log line |
| `mod.loaded.v1` / `mod.error.v1` | Planned | replay bus (EVENT_TAXONOMY) |

**What works today (v2)**

- Log lines `mod:{id}:policy_phase:tick=N` for eligible policy mods each economy phase.
- `ModHost::tick(sim_tick)` delegates to registry (no WASM).

**What does not work yet (v2+)**

- Capability enforcement beyond manifest flags; actual policy writes.
- `mod.loaded.v1` / `mod.error.v1` on replay bus.

## v3 — WASM sandbox (**partial**, 2026-05-25)

| Item | Status | Location |
|------|--------|----------|
| `wasmtime` policy tick | Done | `crates/mod-host/src/wasm_guest.rs` |
| `.civmod` ZIP load | Done | `ModHost::load_civmod_archive` |
| `civlab-sdk` | Done | `crates/civlab-sdk` |
| Example WASM build | Script | `scripts/build-example-policy-wasm.ps1` |
| Determinism scan | Planned | §3.4 |
| CI packaged `.civmod` | Planned | `scripts/package-example-mod.ps1` (optional) |

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
