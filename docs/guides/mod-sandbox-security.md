# Mod Sandbox Security

`civ-mod-host` executes guest mods through `wasmtime`.

## Security model

- Mods run as WebAssembly guests, not native host code.
- The host controls imports and capabilities.
- Guests should only receive the narrow API surface needed for simulation ticks, state inspection, and approved outputs.
- Anything outside that surface should be treated as unavailable unless explicitly exposed by the host.

## Practical implications

- Mods cannot assume filesystem, network, or process access.
- Side effects must flow through host-defined APIs.
- Determinism matters: the sandbox should keep mod execution reproducible across runs.
- Validation and signing are separate from execution. A signed mod is still constrained by the sandbox.

## Operational guidance

- Keep guest interfaces minimal.
- Audit every new host import for capability creep.
- Prefer explicit allowlists over ambient access.
- Treat mod-host changes as security-sensitive and review them with the same rigor as other runtime boundary code.
