# Functional Requirements — CivLab

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-25
**Traces to:** PRD.md v1.0 (CivLab epics E1–E6)

---

## Categories

| Code | Domain |
|------|--------|
| CORE | Core simulation engine and tick loop |
| ECON | Economy, markets, and Joule accounting |
| PROTO | Multi-client protocol (WebSocket, binary frames) |
| REPLAY | Deterministic replay and audit trail |
| API | Research API and scenario system |
| CLIENT | Client implementations and integration |

---

## FR-CORE-001: Fixed-Timestep Tick Loop

**Priority**: SHALL
**Description**: The simulation engine SHALL execute a fixed-timestep tick loop at 100 ms per tick with no jitter under nominal load.
**Acceptance Criteria**:
- [ ] 10,000 ticks execute in under 2 minutes headless on commodity hardware
- [ ] Tick interval jitter measured < 1 ms (P99) under no-client load
- [ ] Tick counter monotonically increases; no tick is skipped or duplicated
**Traces to**: E1.1
**Status**: Partial

---

## FR-CORE-002: ECS Entity Model

**Priority**: SHALL
**Description**: The engine SHALL use an ECS (Entity Component System) architecture with defined entity types: cells, buildings, agents, and institutions.
**Acceptance Criteria**:
- [ ] All four entity types creatable and addressable by stable entity ID
- [ ] Component queries complete in O(n) over entity count with cache-friendly memory layout
- [ ] Entity IDs survive serialization/deserialization round-trips unchanged
**Traces to**: E1.2
**Status**: Partial

---

## FR-CORE-003: Deterministic Transition Phase

**Priority**: SHALL
**Description**: The deterministic transition phase SHALL produce bit-identical output for identical inputs, using fixed-point arithmetic with no floating-point state mutations.
**Acceptance Criteria**:
- [ ] Replay of any recorded run from event log produces identical state hashes at every tick
- [ ] No f32/f64 types used in state-mutating paths; fixed-point types enforced by type system
- [ ] Determinism test suite runs on every CI commit and blocks merge on failure
**Traces to**: E1.4
**Status**: Partial

---

## FR-CORE-004: Stochastic Event Phase with Seeded RNG

**Priority**: SHALL
**Description**: The stochastic event phase SHALL use ChaCha20Rng seeded per run, with all RNG draws logged in the event stream to guarantee reproducibility.
**Acceptance Criteria**:
- [ ] RNG seed stored in run header and recoverable from the .civreplay file
- [ ] Every RNG draw emits a `rng_draw` event with seed state and result
- [ ] Property test: two runs from same seed produce identical stochastic event sequences
**Traces to**: E1.5
**Status**: Planned

---

## FR-CORE-005: Policy Evaluation Phase

**Priority**: SHALL
**Description**: The engine SHALL execute a policy evaluation phase each tick that transforms current state into control signals governing production, allocation, and taxation.
**Acceptance Criteria**:
- [ ] Policy evaluation runs before production and allocation in tick ordering
- [ ] Policy results are pure functions of current state (no side effects)
- [ ] Policy override via Scenario API injects custom policy functions without engine modification
**Traces to**: E1.3
**Status**: Planned

---

## FR-CORE-006: Multi-Client Command Queue

**Priority**: SHALL
**Description**: The engine SHALL maintain a priority-ordered command queue that serializes commands from multiple clients before each tick boundary.
**Acceptance Criteria**:
- [ ] Admin commands have priority over player commands; player priority over research
- [ ] FIFO ordering enforced within the same priority tier
- [ ] Commands submitted after tick cutoff are deferred to the next tick
**Traces to**: E1.7
**Status**: Planned

---

## FR-CORE-007: Tick Budget Enforcement

**Priority**: SHALL
**Description**: The engine SHALL target a tick processing budget of 14 ms on commodity hardware and emit a warning log entry when any tick exceeds 16 ms.
**Acceptance Criteria**:
- [ ] P99 tick duration < 14 ms on reference hardware (4-core, 16 GB RAM)
- [ ] Ticks exceeding 16 ms logged with profiling breakdown by phase
- [ ] Performance CI gate blocks merge when P99 regresses beyond 16 ms baseline
**Traces to**: E1.10
**Status**: Planned

---

## FR-ECON-001: Production System

**Priority**: SHALL
**Description**: Buildings SHALL produce goods each tick according to their production rate and input resource availability, writing outputs to per-entity inventory.
**Acceptance Criteria**:
- [ ] Production halts when required inputs are unavailable (no phantom goods)
- [ ] Production rates configurable per building type in scenario YAML
- [ ] Production events emitted to event log with entity ID, good type, and quantity
**Traces to**: E2.1
**Status**: Partial

---

## FR-ECON-002: Joule Energy Conservation

**Priority**: SHALL
**Description**: The Joule energy unit SHALL serve as the universal economic numeraire with the total quantity conserved across all entities each tick.
**Acceptance Criteria**:
- [ ] Property test: sum of all Joules in simulation is invariant across ticks (conservation law)
- [ ] Joule balance tracked per entity and in a global ledger
- [ ] Any tick that violates conservation produces a fatal simulation error with ledger diff
**Traces to**: E2.4
**Status**: Planned

---

## FR-ECON-003: Market Clearing

**Priority**: SHALL
**Description**: The market system SHALL clear bid/ask orders each tick using price discovery to reach supply/demand equilibrium for each traded good.
**Acceptance Criteria**:
- [ ] Grain market clears in MVP; multi-good markets operational in v1
- [ ] Clearing algorithm is deterministic given identical order books
- [ ] Uncleared orders expire after configurable TTL (default: 1 tick)
**Traces to**: E2.3
**Status**: Partial

---

## FR-ECON-004: Taxation and Budget System

**Priority**: SHALL
**Description**: The fiscal system SHALL implement configurable tax policies that transfer Joules from entities to an institutional treasury each tick and impact legitimacy.
**Acceptance Criteria**:
- [ ] Tax rate configurable per institution and good type
- [ ] Tax revenue credited to institution treasury in same tick as collection
- [ ] Legitimacy model decrements citizen satisfaction as a function of effective tax rate
**Traces to**: E2.6, E2.7, E2.8
**Status**: Planned

---

## FR-ECON-005: Allocation Algorithm

**Priority**: SHALL
**Description**: The allocation system SHALL distribute available goods to consumer entities each tick according to priority and need, running after market clearing.
**Acceptance Criteria**:
- [ ] Allocation priority: subsistence goods before luxury goods
- [ ] Unmet allocation needs increment deprivation counters on affected entities
- [ ] Allocation phase completes in O(n log n) over entity count
**Traces to**: E2.5
**Status**: Planned

---

## FR-PROTO-001: RFC 6455 WebSocket Server

**Priority**: SHALL
**Description**: The simulation server SHALL expose a WebSocket server compliant with RFC 6455 on a configurable port accepting at least 10 simultaneous client connections.
**Acceptance Criteria**:
- [ ] Server accepts >= 10 simultaneous WebSocket connections without degradation
- [ ] TLS support via configurable certificate paths required for non-localhost deployments
- [ ] Graceful close handshake sent to all clients on server shutdown
**Traces to**: E3.1
**Status**: Partial

---

## FR-PROTO-002: JSON-RPC 2.0 Message Dispatcher

**Priority**: SHALL
**Description**: The server SHALL dispatch all client messages using the JSON-RPC 2.0 protocol, returning structured results and errors for every request.
**Acceptance Criteria**:
- [ ] All RPC methods return `result` or `error` as specified by JSON-RPC 2.0
- [ ] Unknown method names return error code -32601
- [ ] Batch requests supported per JSON-RPC 2.0 specification
**Traces to**: E3.2
**Status**: Planned

---

## FR-PROTO-003: Client Handshake and Bootstrap

**Priority**: SHALL
**Description**: Connecting clients SHALL complete a handshake establishing identity, role, and initial world snapshot within 2 seconds on local network before receiving tick deltas.
**Acceptance Criteria**:
- [ ] Handshake completes within 2 seconds of connection on local network
- [ ] Bootstrap snapshot includes all entity states at the current tick
- [ ] Client role (admin/player/research) assigned during handshake and enforced on all subsequent commands
**Traces to**: E3.3
**Status**: Planned

---

## FR-PROTO-004: Binary Frame Protocol

**Priority**: SHALL
**Description**: High-frequency game clients SHALL receive tick deltas as zstd-compressed binary frames with a defined frame header.
**Acceptance Criteria**:
- [ ] Binary frame header includes: tick number, frame type, uncompressed size, checksum
- [ ] zstd compression ratio >= 3:1 on typical delta frames
- [ ] 10 game clients at 60 FPS consume <= 10 Mbps aggregate bandwidth
**Traces to**: E3.6
**Status**: Planned

---

## FR-PROTO-005: Snapshot Filtering by Region and Type

**Priority**: SHALL
**Description**: Clients SHALL be able to subscribe to filtered snapshot streams specifying entity type and/or geographic region bounds to reduce per-client bandwidth.
**Acceptance Criteria**:
- [ ] Filter spec transmitted during handshake and updatable via subscription command
- [ ] Server excludes filtered-out entities from delta frames
- [ ] Region filter uses bounding-box spatial query; out-of-bounds entities fully excluded
**Traces to**: E3.8
**Status**: Planned

---

## FR-REPLAY-001: Civreplay Export Format

**Priority**: SHALL
**Description**: The engine SHALL export complete simulation runs to a .civreplay file containing a header, full event log, and SHA-256 checksum for integrity verification.
**Acceptance Criteria**:
- [ ] Header includes: seed, scenario name, engine version, start timestamp
- [ ] Event log is append-only during a run; no events deleted or modified post-write
- [ ] SHA-256 checksum of event log included in footer and verified on load
**Traces to**: E1.8
**Status**: Partial

---

## FR-REPLAY-002: Bit-Identical Determinism Verification

**Priority**: SHALL
**Description**: Loading and replaying a .civreplay file SHALL produce bit-identical state at every tick compared to the original run, verified by state hash comparison.
**Acceptance Criteria**:
- [ ] Replay mode re-executes all events from log without re-sampling RNG
- [ ] State hash compared at each tick; first divergence reported with tick number and state diff
- [ ] Determinism CI gate runs replay verification on every commit and blocks on failure
**Traces to**: E1.9
**Status**: Planned

---

## FR-API-001: Scenario YAML Format and Validation

**Priority**: SHALL
**Description**: Scenarios SHALL be defined in a versioned YAML format specifying map dimensions, initial entity placement, starting conditions, and policy parameters.
**Acceptance Criteria**:
- [ ] YAML schema documented and versioned in the repo
- [ ] Schema validation runs at scenario load; invalid YAML produces descriptive error with field path
- [ ] Example scenario `data/scenarios/starting_settlement.yaml` included and validated in CI
**Traces to**: E5.1
**Status**: Partial

---

## FR-API-002: Python Scenario Runner

**Priority**: SHALL
**Description**: A Python API SHALL allow researchers to load a scenario YAML, run a headless simulation, and inspect state programmatically via a pip-installable package.
**Acceptance Criteria**:
- [ ] `civlab.run_scenario(path, ticks=50)` completes a 50-tick run in under 5 seconds
- [ ] Package installable via `pip install civlab` with all dependencies declared
- [ ] All public API methods have type annotations and docstrings
**Traces to**: E5.2
**Status**: Planned

---

## FR-API-003: Policy Parameter Override

**Priority**: SHALL
**Description**: The research API SHALL support overriding scenario policy parameters (tax rates, production multipliers, allocation weights) without modifying the scenario YAML file.
**Acceptance Criteria**:
- [ ] Override dict passed to `run_scenario` merges with scenario defaults
- [ ] Invalid parameter names raise `ValueError` listing allowed parameters
- [ ] Override values validated against parameter type constraints before simulation start
**Traces to**: E5.3
**Status**: Planned

---

## FR-API-004: Data Export for Analysis

**Priority**: SHALL
**Description**: The research API SHALL export simulation metrics (agent counts, market prices, ledger balances, events) to CSV and JSON formats compatible with Jupyter/matplotlib.
**Acceptance Criteria**:
- [ ] `civlab.export(run_id, format="csv")` writes per-tick metric table
- [ ] JSON export includes full event log with tick timestamps
- [ ] Export completes in under 30 seconds for a 100,000-tick run
**Traces to**: E5.6
**Status**: Planned

---

## FR-CLIENT-001: Bevy Reference Client

**Priority**: SHALL
**Description**: A Bevy-based reference client SHALL connect to the simulation server, render agents and buildings, and support click-to-build interactions at 60 FPS.
**Acceptance Criteria**:
- [ ] Client renders 60 FPS on reference hardware with 1,000 visible entities
- [ ] Click-to-build sends a `build_command` via JSON-RPC and reflects the result within one tick
- [ ] Client reconnects automatically after server restart within 5 seconds
**Traces to**: E6.1
**Status**: Partial

---

## FR-CLIENT-002: Web TypeScript Client

**Priority**: SHALL
**Description**: A TypeScript/React web client SHALL connect to the server via WebSocket and render a strategic map view of simulation state in a browser.
**Acceptance Criteria**:
- [ ] Client renders in Chrome, Firefox, and Safari (latest stable versions)
- [ ] WebSocket transport handles reconnect with exponential backoff
- [ ] React hooks `useSimulationState()` and `useEntityQuery()` documented and covered by tests
**Traces to**: E6.4
**Status**: Planned

---

## FR-CLIENT-003: Client Role Authorization Enforcement

**Priority**: SHALL
**Description**: The server SHALL enforce client role permissions (admin > player > research) and reject out-of-scope commands with a structured error response.
**Acceptance Criteria**:
- [ ] Research clients cannot submit build or policy commands
- [ ] Unauthorized command attempts return JSON-RPC error code -32603 with role information
- [ ] Role enforcement verified by integration tests covering all three role tiers
**Traces to**: E3.7
**Status**: Planned

---

## FR-METRICS: Simulation Metrics

### FR-METRICS-001: Metrics Struct
**Priority**: SHALL
**Description**: The `Metrics` struct SHALL define four f64 fields: `waste_joules` (10% of consumption), `surplus_joules` (energy budget minus consumption, floored at 0), `tyranny_index` (consumption / (budget+1), capped at 1.0), `legitimacy_index` (1.0 - tyranny_index, floored at 0).
**Acceptance Criteria**:
- [ ] `compute(1000.0, 500.0)` returns `waste_joules=50.0`, `surplus_joules=500.0`
- [ ] When `consumption >= budget`, `tyranny_index > 0.9` and `legitimacy_index < 0.1`
- [ ] All fields are `f64`; struct is `Debug`, `Clone`, `Copy`, `Default`
**Traces to:** E5 (Research Sandbox / Policy Analysis)
**Code:** `crates/engine/src/metrics.rs`

### FR-METRICS-002: Metrics Computation
**Priority**: SHALL
**Description**: The `compute(energy_budget_joules: f64, consumption_joules: f64) -> Metrics` function SHALL compute all four metrics in constant time O(1) using only arithmetic operations with no I/O or allocations.
**Acceptance Criteria**:
- [ ] Function has no side effects; identical inputs always produce identical outputs
- [ ] Function is callable from the ECS tick loop without performance regression (P99 < 1 µs)
**Traces to:** FR-CORE-003 (Deterministic Transition Phase)
**Code:** `crates/engine/src/metrics.rs`

### FR-METRICS-003: Fixed-Point Determinism for Metrics
**Priority**: SHALL
**Description**: Tyranny and legitimacy indices SHALL be computed using the fixed-point `Fixed` type (i64 scaled by 10^6) when deterministic cross-platform reproduction is required; float variants are provided for research export only.
**Acceptance Criteria**:
- [ ] Fixed-point and float results agree to within 6 decimal places for identical inputs
- [ ] Replay verification uses fixed-point metrics exclusively
**Traces to:** FR-REPLAY-002 (Bit-Identical Determinism Verification)
**Code:** `crates/engine/src/lib.rs` — `Fixed` type

## FR-CIV-LIFE: Emergent Agent Life-Sim

### FR-CIV-LIFE-001: Needs Decay
**Priority**: SHALL
**Description**: Every living item SHALL carry a `civ_needs::Needs` satisfaction vector (food, water, rest, safety, social, health) that decays deterministically each tick at per-need rates; satisfying a need raises it, clamped to `[0,1]`.
**Traces to:** FR-CORE-003 (Deterministic Transition Phase)
**Code:** `crates/needs/src/lib.rs`

### FR-CIV-LIFE-002: Sickness From Deprivation
**Priority**: SHALL
**Description**: Sustained unmet needs SHALL accumulate a deprivation streak that, past an onset threshold, triggers a seeded-RNG sickness roll; sickness accelerates health-integrity loss.
**Traces to:** FR-CIV-LIFE-001
**Code:** `crates/needs/src/lib.rs` — `tick`

### FR-CIV-LIFE-003: Death From Unmet Needs
**Priority**: SHALL
**Description**: Health integrity SHALL fall to zero (death) under prolonged deprivation/sickness and regenerate when clear; a sated agent never dies, a fully-deprived agent always eventually dies.
**Traces to:** FR-CIV-LIFE-002
**Code:** `crates/needs/src/lib.rs`

### FR-CIV-LIFE-010..014: Utility-AI Daily Path / POI
**Priority**: SHALL
**Description**: Agents SHALL select a daily target via utility scoring over needs against a `PoiRegistry`, choosing the POI serving the highest-pressure need (distance only a tiebreak) and greedily path-stepping toward it, so eat→rest→socialize routines emerge.
**Traces to:** FR-CIV-LIFE-001
**Code:** `crates/agents/src/daily_path.rs`

### FR-CIV-LIFE-020..025: Mercantile Resource Stocks
**Priority**: SHALL
**Description**: Individual and collective (settlement/faction) resource stocks SHALL track integer-conserved goods with production/consumption, surplus/deficit, comparative advantage, and mutually-beneficial trade (conserving total goods across actors).
**Traces to:** FR-ECON-001, FR-ECON-005
**Code:** `crates/economy/src/stocks.rs`

### FR-CIV-LIFE-030..035: Emergent Clusters (Join/Leave)
**Priority**: SHALL
**Description**: Collectives SHALL emerge as `ClusterId` membership from deterministic single-link co-location clustering; agents JOIN on net-positive `MembershipPayoff` and LEAVE when payoff drops below threshold — replacing hardcoded `faction: u32` over time.
**Traces to:** FR-CIV-LIFE-020
**Code:** `crates/agents/src/cluster.rs`

---

## FR-CIV-EMERGENT: R&D Consolidated Backlog (2026-05-31)

**Authority:** `docs/design/emergent-systems-spec.md` · **Canon:** `docs/guides/emergence-charter.md`

### FR-CIV-CA — Cellular automaton physics & thermo (`crates/voxel`, `crates/laws`)

- **FR-CIV-CA-001** — `fluid_ca` SHALL extend in place (no salva/falling_sand fork) with `saturation: u8` alongside temperature on dense `CaGrid`.
- **FR-CIV-CA-002** — `MaterialProps` SHALL carry TPT-*model* fields (heat conduct 0–255, melt/boil/freeze, latent heat, phase targets) as data tables, not GPL code imports.
- **FR-CIV-CA-003** — Percolation pass SHALL hold water below field capacity (capillary lock) and allow flow only when saturation exceeds capacity, including small upward capillary transfer.
- **FR-CIV-CA-004** — Evaporation pass SHALL spawn vapor from `p_evap ∝ max(0,T−T_evap)` with RNG, subtract latent heat, and diffuse/condense vapor stochastically.
- **FR-CIV-CA-005** — Thermo pass SHALL conduct heat via 6-neighbor stencil with α=f(HeatConduct) keeping α·n<0.5 stable and apply phase transitions ±latent heat at thresholds.
- **FR-CIV-CA-006** — Boundary flux SHALL expose per-face `BoundaryMode` ghost neighbors (`Vacuum` sink, `Inflow{material,rate,temp}`, `Closed` reflect) without storing ghost cells in the grid.
- **FR-CIV-CA-007** — Sea-level pass SHALL equalize liquids toward global `sea_level` without a pressure solve; powders SHALL settle at angle-of-repose with wet-powder repose boost from saturation.
- **FR-CIV-CA-008** — CA stepper SHALL use dirty-chunk bottom-up sweep with double-buffer compatible with 256³ resident chunks.
- **FR-CIV-CA-009** — MVP resident window SHALL run integrated CA in `engine::phase_voxel` on at least one 256³ chunk with abiogenesis suitability inputs from solvent/energy fields.
- **FR-CIV-CA-010** — Tests SHALL assert basin flat-fill, unsupported-solid fall, and phase-change smoke on a fixed micro-fixture (RNG streams logged, not bit-identical mandated).

### FR-CIV-SPECIES — Semi-param morphology & spawn modes (`crates/species`, `crates/genetics`)

- **FR-CIV-SPECIES-010** — Phenotype render SHALL map gene vector → glTF base archetype (biped/quad/avian) + morph-target weights using SMPL-style `base + Σ gene·morph` without a hand-rolled mesh deformer.
- **FR-CIV-SPECIES-011** — Per-bone `Transform.scale` SHALL encode limb length/girth from genome; `StandardMaterial` SHALL apply HSV tint/emissive from genes.
- **FR-CIV-SPECIES-012** — Accessory child entities SHALL attach when gene thresholds cross configured gates (canonical presets provide default thresholds).
- **FR-CIV-SPECIES-013** — Canonical mode SHALL load named race seeds (human + fantasy exemplars) as preset genome + base mesh + divergence dial 0..1, not scripted outcome paths.
- **FR-CIV-SPECIES-014** — Primitive mode SHALL expose `spawn_organism { genome, cradle_state, … }` with no species enum table.
- **FR-CIV-SPECIES-015** — Seed base meshes SHALL ship as MakeHuman CC0 exports with offline Mixamo rig; SMPL basis remains an optional upgrade path only.
- **FR-CIV-SPECIES-016** — Clients (Bevy first) SHALL display morphed agents at 60 FPS for ≥100 visible instances on reference hardware.
- **FR-CIV-SPECIES-017** — Same `Dna` input SHALL yield identical morph weights across render clients (pure `express` mapping; sim RNG elsewhere allowed).

### FR-CIV-ARCH — Procedural architecture (`crates/build`, WFC wrap)

- **FR-CIV-ARCH-001** — 3D tiled WFC solver SHALL wrap `ghx_proc_gen` / `bevy_procedural_tilemaps`; Civis SHALL NOT ship a hand-rolled WFC core.
- **FR-CIV-ARCH-002** — `BuildingGraph` split-grammar SHALL remain the authoritative structural schema; WFC selects voxel tiles, not replaces the graph.
- **FR-CIV-ARCH-003** — Tile-sets and adjacency weights SHALL key off emergent `(culture, era, wealth)` vectors, not a fixed building-type enum.
- **FR-CIV-ARCH-004** — Landmark placements SHALL use jigsaw template pools with deterministic parcel scoring given fixed demand signals.
- **FR-CIV-ARCH-005** — Solved volumes SHALL write into `VoxelWorld` with watertight mesh output verifiable on a golden `BuildingGraph` fixture.
- **FR-CIV-ARCH-006** — Freehand tools SHALL emit graph mutations equivalent to grammar edits (`FR-CIV-BUILD-020` parity).
- **FR-CIV-ARCH-007** — Canonical mode SHALL apply culture/era exemplar tile-set presets; primitive mode SHALL accept raw graph + tile-set IDs.
- **FR-CIV-ARCH-008** — Style histogram tests SHALL detect measurable façade divergence when culture vectors differ holding wealth/era fixed.

### FR-CIV-LANG — Emergent language & scripts (`crates/lang` / `civis-lang`)

- **FR-CIV-LANG-001** — All player-visible strings SHALL store `semantic_key` + per-civ lexeme IDs, never locale-baked bytes in sim state.
- **FR-CIV-LANG-002** — Render path SHALL toggle English (system fonts) vs native civ script atlas per civ/view setting.
- **FR-CIV-LANG-003** — Phonology engine SHALL apply diachronic sound-change rules each generation tick (ASCA vendor or clean-room reimpl after GPL decision).
- **FR-CIV-LANG-004** — Lexicon generation SHALL port Lexifer-style per-phoneme Zipf frequencies and phonotactic cluster tables in-tree.
- **FR-CIV-LANG-005** — Civ split/contact SHALL branch lexicons (clone+diverge) and borrow lexemes through receiver phonotactic filters.
- **FR-CIV-LANG-006** — Orthography SHALL lag phonology; spelling drift SHALL be derivable from rule-history without LLM involvement.
- **FR-CIV-LANG-007** — Glyph generator SHALL emit vector strokes rendered via `ab_glyph`/`cosmic-text` into a per-civ `FontAtlas` (or runtime TTF).
- **FR-CIV-LANG-008** — Storage SHALL retain `(inventory, rule-history, root-delta)` for all civs and materialize full lexicon+glyphs only for viewed civs.
- **FR-CIV-LANG-009** — Background name generation SHALL remain DF-tier cheap; no LLM calls on the language engine hot path.
- **FR-CIV-LANG-010** — GPL isolation: ASCA-derived code SHALL live only in `civis-lang` until legal sign-off or replacement reimpl ships.

### FR-CIV-AUDIO — Emergent soundscape & music (`crates/audio` / `civis-audio`)

- **FR-CIV-AUDIO-001** — Playback stack SHALL wrap `fundsp` + `bevy_procedural_audio` → `bevy_kira_audio`; Civis SHALL NOT hand-roll a mixer/DSP core.
- **FR-CIV-AUDIO-002** — `MusicalTradition` SHALL derive from `(culture_vec, available_materials)` and gate instrument families by material class.
- **FR-CIV-AUDIO-003** — Melody generation SHALL use order-1/2 Markov chains over scale degrees bounded to the tradition's mode/scale.
- **FR-CIV-AUDIO-004** — Rhythm generation SHALL use Euclidean (Bjorklund) patterns parameterized by tradition vectors.
- **FR-CIV-AUDIO-005** — Timbre SHALL map material density/stiffness/size to modal or Karplus-Strong voices in fundsp (no wave-equation sim).
- **FR-CIV-AUDIO-006** — v1 SHALL ship four material archetype instruments (metal/wood/hide/reed) with spatial kira playback.
- **FR-CIV-AUDIO-007** — Mix tree SHALL expose four duckable buses per `docs/design/audio-direction.md` (ambient/score/sfx/ui).
- **FR-CIV-AUDIO-008** — Missing audio assets SHALL warn once and play silence without crashing (locked invariant).

### FR-CIV-LEGENDS — History, rumor drift, cultural register (`crates/legends`)

- **FR-CIV-LEGENDS-001** — Sim SHALL emit structured `HistoricalEvent` records on the watch bus; legends layer SHALL NOT author outcomes.
- **FR-CIV-LEGENDS-002** — Historian agents SHALL re-emit `Rumor`/`Chronicle` from witnessed event subsets only.
- **FR-CIV-LEGENDS-003** — Each retelling hop SHALL mutate rumors (actor swap, amplification, teller psyche/culture tags) with gates from OCEAN traits.
- **FR-CIV-LEGENDS-004** — Prose surface SHALL wrap `tracery` templates with bladeink salience; deity spheres SHALL tag cults/taboos/art references.
- **FR-CIV-LEGENDS-005** — Saga-graph ingest SHALL stay compatible with `docs/design/legends-engine.md` query API.
- **FR-CIV-LEGENDS-006** — Missing producer events SHALL log `legends: gap` warnings and show empty saga with reason, never silent omission.
- **FR-CIV-LEGENDS-007** — Cultural register output SHALL feed literature/historian UI; formal register SHALL remain separate from treaty text (see DIPLO).
- **FR-CIV-LEGENDS-008** — Names in legend nodes SHALL reference `NameRef` from language drift, not hardcoded English strings in the engine.

### FR-CIV-PSYCHE — Mind model & decision stack (`crates/agents`, utility wraps)

- **FR-CIV-PSYCHE-001** — Agents SHALL carry OCEAN(5f), PAD mood (decays to OCEAN baseline), Maslow needs vector, and sparse pairwise `(affinity, trust, familiarity)`.
- **FR-CIV-PSYCHE-002** — Primary action selection SHALL use `bevy_observed_utility` scoring needs×personality; GOAP SHALL use `dogoap` when multi-step plans are required.
- **FR-CIV-PSYCHE-003** — Ritual behaviors MAY use `bevy_behave` BTs; `big-brain` remains fallback only.
- **FR-CIV-PSYCHE-004** — Historian embellishment rates SHALL be a measurable function of psyche traits (testable monotonicity on openness/conscientiousness).
- **FR-CIV-PSYCHE-005** — Psyche state SHALL persist in snapshots; reload restores moods/relationships without re-rolling traits.
- **FR-CIV-PSYCHE-006** — Diplomacy reservation utility SHALL read relationship floats from this component (`FR-CIV-DIPLO-003`).
- **FR-CIV-PSYCHE-007** — No LLM call SHALL compute needs, mood, or utility scores.
- **FR-CIV-PSYCHE-008** — Canonical mode MAY seed OCEAN priors per culture exemplar; primitive mode draws from configured distributions only.

### FR-CIV-DIPLO — War goals, trust, negotiation (`hand-roll`)

- **FR-CIV-DIPLO-001** — Wars SHALL use typed bounded war goals; defenders evaluate goal utility to choose capitulate/counter/fight.
- **FR-CIV-DIPLO-002** — Treaties SHALL feed a Trust ledger that caps Opinion and decays without diplomatic income.
- **FR-CIV-DIPLO-003** — Negotiation SHALL accept offers iff `offer ≥ reservation`, with reservation from power + relationship + war exhaustion.
- **FR-CIV-DIPLO-004** — Concession rounds SHALL follow a Zeuthen/Rubinstein monotonic-concession state machine (testable on scripted fixtures).
- **FR-CIV-DIPLO-005** — Formal register transcripts SHALL store structured clauses independent of LLM wording garnish.
- **FR-CIV-DIPLO-006** — LLM SHALL NOT decide treaty acceptance, trust updates, or war-goal selection.
- **FR-CIV-DIPLO-007** — Polity IDs SHALL reference emergent `ClusterId`, not `faction: u32` enums.
- **FR-CIV-DIPLO-008** — Integration tests SHALL cover accept/reject/counter paths for at least three goal types.

### FR-CIV-PBR — Materials & voxel texturing (`clients`, `crates/voxel`)

- **FR-CIV-PBR-001** — Shipped PBR textures SHALL come from CC0 sets (ambientCG, Poly Haven) as complete downloaded sets, not procedural albedo authorship.
- **FR-CIV-PBR-002** — glTF channel wiring SHALL use separate metallic-roughness (G=rough, B=metal) and occlusion (R=AO) maps; ORM sources MAY fan into both without repack.
- **FR-CIV-PBR-003** — Terrain SHALL use wrapped `bevy_triplanar_splatting` driven by mesher `matid` (position+normal+matid, no UV seams).
- **FR-CIV-PBR-004** — Built voxels SHALL use greedy mesh + texture-array atlas where triplanar is unsuitable.
- **FR-CIV-PBR-005** — Distant LOD SHALL fall back to vertex-color shading without breaking nearer PBR layers.
- **FR-CIV-PBR-006** — Albedo SHALL be sRGB; data maps SHALL load linear (`is_srgb=false`).
- **FR-CIV-PBR-007** — Canonical mode SHALL reference exemplar material seed manifests; primitive mode allows per-matid overrides.
- **FR-CIV-PBR-008** — Missing texture files SHALL fail loud in dev builds and degrade to flat tint in player builds with logged warnings.

### FR-CIV-INFRA — Service grid substrate (power, water, coverage) (`crates/civ-traffic`)

- **FR-CIV-INFRA-070** — The service grid substrate SHALL model power, water, and coverage services as typed cells with an adjacency list; `place_source` SHALL flip a cell to `Active` and `coverage_ring` SHALL return every cell within `range` cells (Chebyshev distance) of a query coord.
- **FR-CIV-INFRA-071** — A building at coord `c` SHALL be considered "served" by a service kind iff at least one cell within `range` cells (Chebyshev) of `c` hosts a source of that kind; the substrate SHALL expose `coverage_ring` so the economy and renderer can compute served status without scanning the full grid.
- **FR-CIV-INFRA-072** — `transmit(coord)` SHALL flip every cell reachable from `coord` (BFS over the adjacency list, `BTreeMap` order) to `Outage`; the operation SHALL be idempotent and SHALL NOT cross disconnected components.

### FR-CIV-LLM — Minimal garnish cache (`crates/research`, `crates/ai`)

- **FR-CIV-LLM-001** — Cache key SHALL be `{seed, prompt_hash, model_id, snapshot_hash, output_hash}` and hits SHALL return byte-identical text.
- **FR-CIV-LLM-002** — Prompts SHALL bind to tiny tag structs (event/sphere/theme); raw world state SHALL NOT be sent to models.
- **FR-CIV-LLM-003** — Live sim SHALL target ~0 LLM calls per tick; first-view lazy fetch with permanent cache is allowed.
- **FR-CIV-LLM-004** — Allowed uses: lore rephrase, diplomatic wording garnish, ε tie-break flicker only.
- **FR-CIV-LLM-005** — Forbidden uses: event generation, rumor logic, utility/GOAP, treaty acceptance, trust math, needs/mood (guard tests required).
- **FR-CIV-LLM-006** — Tracery (or template) fallback SHALL serve text when cache misses and policy forbids live calls.

### FR-CIV-SCALE — HW-bounded streaming (`crates/voxel`, `crates/engine`, phenotype-voxel)

- **FR-CIV-SCALE-001** — MVP SHALL support ~0.5 mi² resident with at least one 256³ CA chunk active.
- **FR-CIV-SCALE-002** — Final target SHALL impose no fixed world-size cap; only hardware working-set and disk SHALL bound extent.
- **FR-CIV-SCALE-003** — Streaming SHALL use LOD rings with horizon-fade seams between chunk resolutions.
- **FR-CIV-SCALE-004** — Sim-LOD SHALL run full agent/CA fidelity near interest and statistical gestalt far away without state divergence explosions.
- **FR-CIV-SCALE-005** — Prefetch SHALL queue disk chunk loads from camera/sim velocity vectors.
- **FR-CIV-SCALE-006** — Shared chunk IO contracts SHOULD migrate to phenotype-voxel per cross-project reuse plan.
- **FR-CIV-SCALE-007** — Save format SHALL persist materialized chunk snapshots, not require bit-identical replay from seed alone.
- **FR-CIV-SCALE-008** — Scale tests SHALL report max resident chunks and P99 tick time on reference hardware without silent degradation.
