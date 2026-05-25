# Scenario YAML

Versioned scenario files seed headless simulation runs. The loader lives in `crates/engine/src/scenario.rs` (FR-API-001); the canonical example is [`scenarios/baseline.yaml`](../../scenarios/baseline.yaml).

## Schema version

| Key | Type | Required | Default | Notes |
|-----|------|----------|---------|-------|
| `version` | integer | no | `1` | Must equal [`SCENARIO_SCHEMA_VERSION`](../../crates/engine/src/scenario.rs) (currently **1**). Any other value fails with `UnsupportedVersion`. |

## Identity and world state

| Key | Type | Required | Validation | Effect |
|-----|------|----------|------------|--------|
| `name` | string | yes | Non-empty after trim | Human-readable scenario id (e.g. `baseline`). |
| `tick_start` | integer (u64) | yes | Must parse as unsigned integer | Sets `WorldState.tick` at load via [`Scenario::apply_world_state`](../../crates/engine/src/scenario.rs). |
| `population` | integer (u64) | yes | Must be **> 0** | Sets `WorldState.population`. |

## Economy policy

These fields map to [`PolicyInput`](../../crates/engine/src/policy.rs) and drive per-tick consumption in `phase_economy`:

| Key | Type | Required | Validation | Effect |
|-----|------|----------|------------|--------|
| `base_consumption_joules` | integer (u64) | yes | — | Base joules before scarcity scaling (`f64` in policy). |
| `scarcity_multiplier` | float | yes | Must be **≥ 0** | Multiplier on base consumption: `effective = base × max(0, scarcity_multiplier)`. |

Example from baseline: `base_consumption_joules: 5000000000`, `scarcity_multiplier: 1.0`.

## Mods (`mods`)

| Key | Type | Required | Default | Notes |
|-----|------|----------|---------|-------|
| `mods` | list of strings | no | `[]` | Repo-relative directory paths from the Civis repo root (resolved via `crates/engine/../../`). Each entry should contain a mod `manifest.toml` (see [`mods/example-policy`](../../mods/example-policy)). |

**MVP behavior (CIV-0700 / mod-host):** On [`Scenario::into_simulation`](../../crates/engine/src/scenario.rs), paths are passed to [`Simulation::register_mod_stubs`](../../crates/engine/src/engine.rs). Manifests are loaded into `ModHost`; WASM guests and phase hooks are **not** executed yet. Load failures are logged and skipped so headless runs stay up during mod development.

Example with one mod:

```yaml
version: 1
name: mod-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
mods:
  - mods/example-policy
```

Baseline explicitly sets an empty list:

```yaml
mods: []
```

## Minimal file

Omit `version` to use schema v1; omit `mods` for no mods (same as `mods: []`):

```yaml
name: baseline
tick_start: 0
population: 1000000
base_consumption_joules: 5000000000
scarcity_multiplier: 1.0
mods: []
```

## Loading and errors

Use [`load_scenario(path)`](../../crates/engine/src/scenario.rs) to read and validate a file:

| Error | Cause |
|-------|--------|
| `Io` | File missing or unreadable |
| `Parse` | Invalid YAML or wrong types (serde path included in message, e.g. `tick_start`) |
| `UnsupportedVersion` | `version` ≠ supported schema |
| `Validation` | `name` empty, `population` zero, or `scarcity_multiplier` negative |

Engine tests in `scenario.rs` (`baseline_yaml_parses`, `scenario_mods_loads_example_policy`, validation rejects) are the regression source of truth.

## Related docs

- [Modding roadmap](../development-guide/fr-modding-roadmap.md) — manifest schema and v2 hooks
- [Simulation determinism](determinism.md) — seeds and replay with scenarios
