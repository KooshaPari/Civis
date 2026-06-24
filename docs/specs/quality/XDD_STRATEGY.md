# Civis XDD Test Strategy

**Scope:** `crates/*/src`, `clients/bevy-ref`, and `docs/specs`.

**Constraint:** This is a docs-only adoption plan. Do not edit `.rs` files or
`Cargo.toml` as part of this document.

## Current Test Posture

The workspace already has a useful foundation:

- Unit and async tests are widespread across the Rust crates.
- `civ-engine`, `civ-server`, `civ-watch`, `civ-mod-host`, and `civ-tactics`
  carry the heaviest test load and cover replay, JSON-RPC, snapshots, modding,
  and tactics behavior.
- `proptest` is already present in several simulation crates and is actively
  used in `civ-engine`, `civ-economy`, and `civ-needs`.
- FR-oriented comments already appear in implementation tests, especially in
  `civ-laws`, `civ-research`, `civ-server`, `civ-tactics`, and `clients/bevy-ref`.
- `docs/traceability/fr-3d-matrix.md` is the authoritative 3D FR matrix and
  should remain the source of FR-to-test naming.

The gap is not lack of tests. The gap is discipline selection: every change
currently tends to look like ordinary Rust unit testing, even when the risk is
contract drift, scenario language drift, behavior ambiguity, or emergent
simulation invariants.

## Discipline Definitions

| Discipline | Use For | Primary Artifact | Rust Tooling |
|---|---|---|---|
| TDD | Small deterministic functions, adapters, validators, pure transforms | Failing unit test before implementation | built-in test, `rstest` |
| BDD | Player/operator-visible workflows and multi-service behavior | Given/When/Then feature scenario | `cucumber-rs` |
| CDD | JSON-RPC, WebSocket, snapshot, save/mod/package wire contracts | Contract fixture and compatibility test | `insta`, serde fixtures |
| SDD | Spec-first FR/NFR implementation | Spec row and acceptance test mapping | docs + traceability matrix |
| DDD | Domain model boundaries and ubiquitous language | Module/crate boundary tests using domain nouns | built-in test, `rstest` |
| PDD | Performance budgets and regression guards | Bench/perf scenario with threshold | Criterion later; existing smoke first |
| Property-based | Invariants over large input space | Generated tests with shrinking | `proptest` |

## Recommended Test Crates

Adopt these deliberately rather than everywhere at once:

- `rstest`: parameterized examples for validators, parsers, edge tables, and
  small deterministic functions. Best for crates with many near-duplicate unit
  tests such as `civ-voxel`, `civ-laws`, `civ-research`, `civ-infra`,
  `civ-watch`, and `civ-server`.
- `proptest`: already in use; standardize it for determinism, conservation,
  serialization, LOD, replay, and graph invariants. Best for `civ-engine`,
  `civ-economy`, `civ-needs`, `civ-voxel`, `civ-planet`, `civ-tactics`,
  `civ-protocol-3d`, `civ-genetics`, `civ-species`, and `civ-traffic`.
- `cucumber-rs`: add sparingly for BDD around user/operator workflows that span
  crates or clients. Best for server/watch/mod/save/client attach flows, not
  low-level math.
- `insta`: snapshot stable JSON/YAML/RON outputs and public wire contracts.
  Best for `civ-server` JSON-RPC catalog/responses, `civ-watch` snapshots,
  `civ-protocol-3d` frames, `civ-engine` replay/save manifests, and
  `civlab-sdk` manifests.

## Workspace Adoption Matrix

| Area | Current Signal | Primary Disciplines | Recommended Tools | Adoption Plan |
|---|---|---|---|---|
| `crates/engine` | Heavy tests; replay, invariants, saves, metrics | SDD, DDD, property, CDD | `proptest`, `insta`, `rstest` | Treat FR rows as acceptance anchors. Property-test determinism and conservation. Snapshot replay/save manifests and hash-chain event shapes. |
| `crates/server` | Large JSON-RPC and WS smoke coverage | CDD, BDD, SDD | `insta`, `cucumber-rs`, `rstest` | Freeze catalog and representative RPC responses with snapshots. Add BDD for reset/spawn/tick/save workflows. Use table-driven tests for validation errors. |
| `crates/watch` | HTTP APIs, terrain, snapshots, mods | CDD, BDD, SDD | `insta`, `cucumber-rs`, `rstest` | Snapshot `sim.snapshot`, terrain, mod catalog, and event feed JSON. Use BDD for mod install/unload/upload and dashboard-facing flows. |
| `crates/mod-host` | WASM, signatures, determinism scan, capability surface | CDD, SDD, property | `insta`, `proptest`, `rstest` | Snapshot determinism reports and permission violations. Property-test manifest/capability combinations. Keep CIV-0700 partial-good scope explicit. |
| `crates/civlab-sdk` | Mod manifest/material/building/events API | CDD, SDD, TDD | `insta`, `rstest` | Snapshot manifest examples and event schema. Use TDD for new SDK validation rules before host integration. |
| `crates/protocol-3d` | Frame schema; `proptest` dependency present | CDD, property | `proptest`, `insta` | Property-test encode/decode and frame bounds. Snapshot canonical frame examples used by Bevy/Godot/Unreal clients. |
| `crates/voxel` | Material, LOD, worldgen, boundary, fluid CA | property, DDD, TDD | `proptest`, `rstest` | Property-test coordinate/LOD composition, dirty ordering, material conservation, and fluid bounds. Use `rstest` for known material phase tables. |
| `crates/planet` | Geology/weather models | property, SDD | `proptest`, `rstest` | Property-test generated climate/geology bounds and seed determinism. Tie public behaviors to FR-CIV-PLANET rows. |
| `crates/tactics` | High test count across LOS/pathfinding/fog/war bridge | property, DDD, SDD | `proptest`, `rstest`, `insta` | Property-test pathfinding and fog invariants. Use table tests for LOS/formation examples. Snapshot combat replay events. |
| `crates/economy` | Market/allocation/stocks property tests | property, DDD, SDD | `proptest`, `rstest` | Expand conservation and bounded-price properties. Use DDD tests that preserve economic nouns: market, allocation, stock, institution. |
| `crates/needs` | Needs model with property tests | property, DDD | `proptest`, `rstest` | Property-test saturation, monotonicity, and clamping. Use fixtures for named need profiles. |
| `crates/agents` | Cluster/daily path behavior | DDD, property, BDD-lite | `proptest`, `rstest` | Property-test path/cluster bounds and LOD consistency. Use scenario-named tests for daily routine behavior. |
| `crates/civ-traffic` | Road/lane/vehicle domain | DDD, property, SDD | `proptest`, `rstest` | Property-test graph connectivity, lane occupancy, and vehicle placement. Map road requirements to tests before new tools. |
| `crates/build` | Building graph/grammar/allocation | DDD, property, SDD | `proptest`, `rstest`, `insta` | Property-test graph invariants. Snapshot representative building graphs. Use FR-CIV-BUILD rows as acceptance criteria. |
| `crates/genetics` | Deterministic mutation/recombination | property, DDD | `proptest`, `rstest` | Property-test allele bounds and seed stability. Table-test known parent/offspring examples. |
| `crates/species` | DNA to phenotype | property, DDD | `proptest`, `rstest` | Property-test phenotype bounds and deterministic mapping. |
| `crates/diffusion` | Adoption curves | property, TDD | `proptest`, `rstest` | Property-test monotonicity and bounded adoption. Table-test canonical S-curve points. |
| `crates/laws` | Schema/version/validator tests | SDD, CDD, TDD | `rstest`, `insta` | Snapshot law schema examples. Use TDD for every new validator rule and FR-CIV-LAWS row. |
| `crates/research` | Cache, replay refusal, client output | CDD, SDD, TDD | `insta`, `rstest`, `proptest` | Snapshot generated card/cache records. TDD every replay/cache rule because failure mode is determinism drift. |
| `crates/infra` | Config helpers plus real DB integration | CDD, TDD | `rstest`, `insta` | Table-test URL/config parsing. Snapshot persistence metadata. Keep real-service tests clearly gated. |
| `crates/save-db` | SQLite save metadata | CDD, TDD | `rstest`, `insta` | Snapshot schema-visible records and migration outputs. Table-test slot/session edge cases. |
| `clients/bevy-ref` | Rendering/UI/client attach tests | BDD, CDD, SDD, PDD | `cucumber-rs`, `insta`, `rstest` | BDD player-visible attach/focus/minimap/event-feed workflows. Snapshot decoded frame-to-UI state, not pixels at first. Add PDD smoke for frame-budget-sensitive systems. |
| `docs/specs` | FR and CIV specs with embedded acceptance language | SDD, BDD, CDD | markdown lint later; no Rust crate | Every new FR should include acceptance criteria, discipline tag, target crate/client, and test name pattern. |

## Discipline Selection Rules

Use the smallest discipline that catches the real failure mode:

- Use TDD when adding a pure rule, parser, validator, or transform.
- Use DDD when a module models a named domain concept and tests should protect
  language and boundaries, not just branches.
- Use SDD when a change claims an FR, NFR, CIV spec section, or traceability row.
- Use CDD when serialized shape matters across crates, clients, saves, mods, or
  replay.
- Use BDD when the behavior crosses a player/operator workflow boundary.
- Use property-based testing when examples are too small to prove the invariant.
- Use PDD when frame time, tick time, memory, or payload size is part of the
  requirement.

## Concrete Adoption Plan

### Phase 1: Normalize Test Labels

Update future tests and specs to use this naming convention:

- `fr_<domain>_<id>__<behavior>` for FR acceptance tests.
- `contract_<surface>__<shape_or_compatibility>` for CDD.
- `prop_<domain>__<invariant>` for property tests.
- `bdd_<workflow>__<outcome>` for BDD-backed scenarios.
- `perf_<surface>__<budget>` for PDD guards.

No mass rename is required. Apply this only when touching a test or adding new
coverage.

### Phase 2: Add Contract Fixtures Before New Surface Area

For any new or changed public shape, require a CDD fixture before implementation:

- JSON-RPC catalog entries and representative responses in `civ-server`.
- `sim.snapshot`, terrain, mod catalog, and event feed JSON in `civ-watch`.
- `Frame3d` and voxel delta examples in `civ-protocol-3d`.
- Replay/save bundle manifests in `civ-engine`.
- Mod manifests and permission reports in `civ-mod-host` / `civlab-sdk`.

Preferred tool: `insta` with JSON/YAML redactions for volatile fields.

### Phase 3: Expand Property Suites Where Bugs Hide

Use `proptest` where one or two examples are weak:

- Determinism: same seed + same commands => same state/hash/replay.
- Conservation: resources, needs, stocks, materials, and guest state do not leak.
- Bounds: coordinates, LOD tiers, weather, market prices, psyche/needs values.
- Graph invariants: roads, building graphs, social/contact graphs, lanes.
- Serialization: encode/decode round-trips preserve canonical values.

Keep property tests local to crates until an invariant crosses a process or
client boundary.

### Phase 4: Add BDD Only for Cross-Boundary Workflows

Do not wrap every unit test in Cucumber. Use `cucumber-rs` for workflows where
plain unit names fail to describe the product behavior:

- Server attach and live tick flow.
- Mod upload/install/unload/publish/fetch.
- Save slot/list/load/autosave ring.
- Bevy reference client focus/minimap/event-feed flows.
- Scenario YAML loading into engine/server/watch behavior.

Feature files should cite FR IDs and specs. Step implementations should call
existing test helpers rather than start bespoke services where possible.

### Phase 5: Add Performance Discipline After Contracts Stabilize

Start PDD with smoke-level budgets before Criterion-level benchmarking:

- Tick budget in `civ-engine`.
- Snapshot generation latency and payload size in `civ-watch`.
- JSON-RPC round-trip smoke in `civ-server`.
- Bevy frame-sensitive systems such as terrain, minimap, event feed, and live
  attach decode.
- Voxel meshing and dirty-queue drain in `civ-voxel` / `civ-protocol-3d`.

Budgets must be explicit, reproducible, and tied to NFR rows such as
`NFR-CIV-SCALE-PERF`.

## Worked BDD Example

Mapped requirement: `FR-CIV-BEVY-023` in
`docs/traceability/fr-3d-matrix.md`, which requires Bevy event-feed toasts and
scrollable log behavior on WebSocket lifecycle changes.

Scenario:

```gherkin
Feature: Bevy live attach event feed

  @FR-CIV-BEVY-023 @client @bdd
  Scenario: WebSocket lifecycle events appear in the Bevy event feed
    Given the Bevy reference client is running in standalone mode
    And the live attach endpoint is configured from the client attach matrix
    When the WebSocket connection reports "connected"
    Then the event feed contains a system event for "connected"
    And the event is visible as a toast
    And the event remains available in the scrollable log
```

Implementation mapping:

| Gherkin Step | Existing Surface | Test Discipline |
|---|---|---|
| Given standalone mode | `clients/bevy-ref/src/bin/standalone.rs`, `clients/bevy-ref/src/lib.rs` | BDD/SDD |
| And endpoint configured | `docs/guides/client-attach-matrix.md`, `live_attach` setup | CDD/BDD |
| When connection reports connected | `clients/bevy-ref/src/live_attach.rs` lifecycle event path | BDD |
| Then system event exists | `clients/bevy-ref/src/event_feed.rs` state assertions | TDD/DDD |
| And toast visible | event feed UI state, not pixel diff initially | BDD |
| And scrollable log retains it | event log retention behavior | BDD/TDD |

Acceptance rule: the BDD scenario passes only if the same lifecycle event is
observable both as immediate feedback and as retained history. A pure unit test
can still cover event insertion, but the BDD scenario protects the FR-level
workflow that users see.

## Spec Authoring Rules

Every new or amended FR/NFR should include:

- Requirement ID.
- Acceptance criteria.
- Target crate/client/docs path.
- Primary discipline: TDD, BDD, CDD, SDD, DDD, PDD, or property.
- Recommended test name pattern.
- Contract fixture path when serialized shape is involved.
- Performance budget when latency, memory, FPS, payload size, or throughput is
  part of the requirement.

## Definition of Done by Change Type

| Change Type | Minimum Test Discipline |
|---|---|
| Pure function or validator | TDD with built-in test or `rstest` |
| Public JSON/YAML/RON/schema shape | CDD with `insta` fixture |
| FR implementation | SDD with traceability matrix row and named acceptance test |
| Cross-crate or user-visible workflow | BDD scenario plus lower-level unit/contract tests |
| Simulation invariant | Property-based test with `proptest` |
| Domain model boundary | DDD tests using domain nouns and crate-local fixtures |
| Performance-sensitive path | PDD smoke or benchmark tied to explicit budget |

## Guardrails

- Do not add Cucumber around low-level deterministic math.
- Do not snapshot unstable fields without redaction.
- Do not accept a new wire shape without a contract fixture.
- Do not claim an FR is implemented unless the test name or traceability row
  makes the requirement discoverable.
- Do not use property tests as a substitute for named acceptance examples.
- Do not make required runtime dependencies optional to make tests pass; fail
  clearly and gate integration tests explicitly.

## First Five Adoption Targets

1. `civ-server`: snapshot JSON-RPC catalog and representative `sim.snapshot`,
   `sim.spawn_entity`, and `save.slot` responses.
2. `civ-watch`: snapshot terrain/mod/save API responses and convert repeated
   status/error cases to `rstest`.
3. `civ-engine`: expand determinism/replay property tests and snapshot save
   bundle manifests.
4. `clients/bevy-ref`: add BDD scenario coverage for FR-CIV-BEVY-023 and
   minimap/focus workflows before visual pixel snapshots.
5. `civ-mod-host`: snapshot determinism reports, permission violations, and
   signed manifest handling before extending CIV-0700 behavior.

