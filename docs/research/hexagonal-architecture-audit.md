# Hexagonal Architecture Audit: Civis Crate Structure

Scope:
- `crates/` workspace crates and their crate roots.
- `civ-watch` is a binary crate, so its root is `crates/watch/src/main.rs` rather than `lib.rs`.

## Executive Summary

The workspace is partially aligned with hexagonal architecture, but the boundaries are not clean enough for a strict ports-and-adapters model.

What is good:
- Several crates are already pure domain libraries: `civ-diffusion`, `civ-genetics`, `civ-planet`, `civ-laws`, `civ-economy`, `civ-protocol-3d`, and most of `civ-build`.
- `civ-research` already defines a port-style trait (`LlmClient`) and keeps the core validator/cache logic separate from its optional adapter module.
- `civ-mod-host` is mostly an adapter boundary around archive/file/signature/wasm concerns.

What is not clean:
- `civ-engine` depends directly on adapter-style crates (`civ-mod-host`, `civ-save-db`) and on several feature crates that are not clearly ports.
- `civ-watch` is doing too much: HTTP server, filesystem persistence, mod registry operations, remote fetching, simulation orchestration, and several domain transforms live in one binary.
- `civ-research` exposes a good port, but the optional `firepass` implementation is still a direct HTTP adapter embedded in the same crate rather than split behind a cleaner adapter boundary.
- `civ-agents` is not rendering code, but it does contain renderer-facing data and comments. I did not find a rendering adapter violation there.

## How I classified crates

### Domain crates
Pure or mostly pure computation, stable data structures, and no direct I/O:
- `civ-diffusion`
- `civ-genetics`
- `civ-planet`
- `civ-laws`
- `civ-economy`
- `civ-protocol-3d`
- `civ-species`
- `civ-build` is mostly domain, but it depends on `civ-voxel` and exposes authoring helpers.

### Port crates
Crates that define boundaries via traits or abstract interfaces:
- `civ-research` defines `LlmClient`.
- `civ-tactics` exposes `OperationalLayer`, which is a useful port-like abstraction.

### Adapter crates
Concrete implementations that touch I/O, process boundaries, or external systems:
- `civ-save-db`
- `civ-infra`
- `civ-mod-host`
- `civ-server`
- `civ-watch`
- `civ-research`'s `firepass` feature module

## Findings

### 1. `civ-engine` is too high-level for a core domain crate

Evidence:
- `civ-engine` depends on `civ-mod-host` and `civ-save-db` in `crates/engine/Cargo.toml`.
- It also re-exports adapter-owned types and functions from `civ_mod_host` in `crates/engine/src/lib.rs`.

Relevant lines:
- [`crates/engine/Cargo.toml:37-52`](../../crates/engine/Cargo.toml#L37-L52): engine depends on `civ-mod-host` and `civ-save-db`.
- [`crates/engine/src/lib.rs:39-44`](../../crates/engine/src/lib.rs#L39-L44): `pub use civ_mod_host::{ ... load_manifest, ModHost, ModRegistry, ... }`

Why this violates hexagonal structure:
- The core simulation crate should not know how manifests are loaded, how mods are validated on disk, or how save data is persisted.
- Those are adapter responsibilities. The engine should consume interfaces or already-loaded data.

What should move where:
- Move manifest loading, archive validation, signature verification, and file-based mod registry behavior out of `civ-engine` and keep them in `civ-mod-host`.
- Move save database persistence concerns out of `civ-engine` and keep them in `civ-save-db` or a dedicated persistence adapter crate.
- Keep `civ-engine` focused on simulation state, tick rules, replay/state transitions, and pure simulation helpers.

How to fix:
- Replace direct engine dependencies on `civ-mod-host` and `civ-save-db` with small port traits in the engine or application layer, for example:
  - `ModManifestSource`
  - `ModRegistryPort`
  - `SaveBundleStore`
- Have `civ-watch` or another application crate provide the concrete adapter implementations.
- Keep only data types in the engine if they are required to model the domain, not to perform the I/O.

### 2. `civ-watch` contains both application orchestration and domain logic

Evidence:
- `civ-watch` owns the HTTP server and routing with Axum in `crates/watch/src/main.rs`.
- It directly constructs a `reqwest::Client` and performs remote fetches in the same file.
- It directly manages filesystem persistence and mod cache directories in the same binary.
- It also imports and executes engine-domain behavior such as `spawn_civilian_at`, `tick_movement`, `drift_toward_home`, and `Simulation` assembly.

Relevant lines:
- [`crates/watch/src/main.rs:22-51`](../../crates/watch/src/main.rs#L22-L51): Axum router, handlers, `reqwest::Client`, filesystem access, simulation worker, and domain spawning helpers all live together.

Why this is a problem:
- `civ-watch` should be an application shell, not the owner of simulation policy.
- When HTTP handlers directly create or mutate domain entities, the mapping between API behavior and the simulation core becomes hard to test and reuse.

What should move where:
- Move simulation/business rules out of the HTTP handlers and into `civ-engine` or a dedicated application service crate.
- Keep `civ-watch` responsible for:
  - HTTP routing
  - request parsing
  - response formatting
  - glue code that calls domain/application services
  - filesystem/network adapters

How to fix:
- Introduce a thin application service layer, for example `civ-app` or `civ-watch-app`, that orchestrates:
  - save/load use cases
  - mod install/publish/fetch use cases
  - snapshot generation
- Make handlers delegate to those services instead of embedding the use cases.
- Keep direct `reqwest` and `std::fs` usage behind adapter structs so tests can substitute fakes.

### 3. `civ-research` has the right port, but its HTTP adapter is still embedded

Evidence:
- `civ-research/src/lib.rs` defines `pub trait LlmClient`.
- The optional `firepass-kimi` feature exposes `pub mod firepass`.
- `crates/research/src/firepass.rs` constructs a concrete `reqwest::Client` and performs the HTTP-backed Kimi call.

Relevant lines:
- [`crates/research/src/lib.rs:65-74`](../../crates/research/src/lib.rs#L65-L74): `pub trait LlmClient: Send + Sync`
- [`crates/research/src/firepass.rs:6-24`](../../crates/research/src/firepass.rs#L6-L24): `reqwest::Client`, environment-based config, and request/response handling

Assessment:
- This is mostly compliant at the boundary level because the core crate does not call HTTP directly in the validator/cache path.
- The only weak point is packaging: the adapter lives inside the same crate, so the port and adapter are not physically separated.

How to improve:
- Keep `LlmClient` in `civ-research`.
- Move `firepass` into a sibling adapter crate, or at least a clearly named adapter module if crate splitting is too expensive right now.
- Make the default `civ-research` feature set remain purely domain/port-level.

### 4. `civ-agents` is not rendering code, but it is renderer-facing

Evidence:
- `civ-agents/src/lib.rs` comments mention renderer-facing material slots and color/texture hints.
- The crate depends on ECS and simulation helpers, not graphics APIs.

Relevant lines:
- [`crates/agents/src/lib.rs:45-52`](../../crates/agents/src/lib.rs#L45-L52): comments referencing renderer-visible material slots

Assessment:
- I did not find actual rendering code here.
- This is not a hexagonal violation by itself; it is a domain model carrying presentation-oriented metadata.

What to watch:
- If renderer code starts to land here, move it out into a client adapter crate.
- Keep the crate limited to agent state, spawn logic, movement, and simulation-relevant data.

## Dependency direction check

### Current direction problems

- `civ-engine` depends on adapter-ish crates:
  - `civ-mod-host`
  - `civ-save-db`
- `civ-watch` depends on many domain crates and also performs direct I/O and HTTP.
- `civ-research` keeps a port but bundles the adapter in the same crate.

### What is acceptable

- Domain crates depending on other domain crates are acceptable if the dependency is a true domain concept.
  - Example: `civ-species` -> `civ-genetics`
  - Example: `civ-build` -> `civ-voxel`
  - Example: `civ-agents` -> `civ-diffusion`, `civ-genetics`, `civ-species`, `civ-voxel`
- Adapter crates depending on domain crates are normal.

### What should not happen

- Domain crates depending on storage, HTTP, filesystem, or process adapters.
- The core engine owning persistence or manifest loading.
- HTTP handlers owning simulation policy.

## Current dependency DAG

This is the practical graph implied by `Cargo.toml` and the source imports.

```text
civ-watch
  -> civ-engine
  -> civ-planet
  -> civ-voxel
  -> civ-protocol-3d
  -> civ-server
  -> civ-agents
  -> civ-laws
  -> civ-tactics
  -> civ-mod-host
  -> civ-save-db
  -> axum / reqwest / tokio / tower-http / std::fs

civ-engine
  -> civ-agents
  -> civ-build
  -> civ-diffusion
  -> civ-economy
  -> civ-mod-host
  -> civ-planet
  -> civ-save-db
  -> civ-tactics
  -> civ-voxel
  -> std::fs / serialization / replay / persistence helpers

civ-research
  -> civ-engine
  -> civ-laws
  -> optional reqwest via firepass feature

civ-agents
  -> civ-diffusion
  -> civ-genetics
  -> civ-species
  -> civ-voxel

civ-species
  -> civ-genetics

civ-build
  -> civ-voxel

civ-tactics
  -> civ-voxel

civ-server
  -> protocol / serialization helpers

civ-mod-host
  -> filesystem / zip / wasm / signature / manifest validation

civ-save-db
  -> rusqlite / filesystem / JSON

civ-infra
  -> sqlx / redis / async-nats / minio adapters
```

## Ideal dependency DAG

This is the shape I would aim for if the workspace is reorganized around strict ports and adapters.

```text
civ-watch (application shell / HTTP adapter)
  -> civ-watch-app or a thin orchestration crate
  -> civ-engine
  -> civ-research ports
  -> civ-mod-host adapters
  -> civ-save-db adapters
  -> axum / reqwest / tokio / tower-http / std::fs

civ-engine (pure simulation core)
  -> civ-agents
  -> civ-build
  -> civ-diffusion
  -> civ-economy
  -> civ-planet
  -> civ-tactics
  -> civ-voxel
  -> small port traits only, if needed

civ-research (domain + ports)
  -> civ-laws
  -> port trait(s) such as LlmClient

firepass adapter crate
  -> civ-research ports
  -> reqwest / env / HTTP

save-db adapter crate
  -> persistence port(s)
  -> rusqlite / filesystem / JSON

mod-host adapter crate
  -> manifest/wasm/archive ports or domain-owned manifest types
  -> filesystem / zip / signature / wasm

client crates
  -> engine + protocol crates
  -> renderer/network-specific dependencies only
```

## Recommended refactor order

1. Extract a small application/service layer for `civ-watch` handlers so HTTP routes stop owning simulation and persistence logic.
2. Stop re-exporting `civ-mod-host` and `civ-save-db` from `civ-engine`; replace with narrow ports or move the orchestration outward.
3. Split `civ-research`'s HTTP adapter into a dedicated adapter crate if the feature boundary keeps expanding.
4. Keep `civ-agents` as-is unless actual renderer code appears there.

## Bottom line

The workspace is not far from a hexagonal shape, but the main boundary violation is that `civ-engine` is acting as an application/service layer while `civ-watch` is also carrying domain use-case logic.

The cleanest end state is:
- `civ-engine` as pure simulation core
- `civ-watch` as HTTP/UI shell
- `civ-mod-host` and `civ-save-db` as adapters
- `civ-research` split into domain + LLM port + adapter
- `civ-agents` kept domain-focused
