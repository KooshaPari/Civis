# CivLab: Product Requirements Document

**Version:** 1.0
**Status:** APPROVED
**Date:** 2026-02-21
**Authors:** CIV Product & Architecture Team

---

## Executive Summary

**CivLab** is a **headless civilization simulation engine** that is simultaneously:
1. **A game** — Player-driven, real-time (or accelerated) city/nation building with RTS mechanics
2. **A research sandbox** — Deterministic, scriptable, with full event logs for analyzing policy and economic systems
3. **A platform** — Client-agnostic protocol enables Bevy, Unreal, Unity, Godot, Web, and custom renderers to attach

CivLab decouples **simulation logic** from **rendering**. The core engine runs headlessly, exposes a protocol (WebSocket JSON-RPC + binary frames), and allows multiple clients to connect simultaneously to the same deterministic world timeline.

**Target Release:** Q3 2026 (MVP) → Q4 2026 (v1)

---

## Product Vision

### Problem Statement

Existing civilization simulators (Dwarf Fortress, Victoria 3, CK3) are **monolithic.** Logic and rendering are entangled. This creates:

| Problem | Impact |
|---------|--------|
| **Single renderer** | Dwarf Fortress can't render in 3D; CK3 in Unreal would require rewriting |
| **Non-deterministic** | Dwarf Fortress saves aren't portable; hard to audit/replay |
| **Limited research access** | Researchers can't hook into policy evaluation or economy (no API) |
| **No multi-view** | Can't run spectator clients, strategic maps, or research dashboards simultaneously |

### CivLab Solution

```
┌─────────────────────────────────────────────────┐
│  CivLab Headless Simulation Core                │
│  (Deterministic, Event-Logged, Replayable)      │
├─────────────────────────────────────────────────┤
│ - Fixed-timestep tick loop (100 ms/tick)        │
│ - Deterministic (ChaCha20Rng, fixed-point, BTreeMap) │
│ - ECS entity model (Rust, cache-friendly)       │
│ - Policy → Production → Trade → Allocation      │
│ - Full event log (audit trail)                  │
└──────────────────┬──────────────────────────────┘
                   │
                   │ WebSocket JSON-RPC + Binary Frames
                   │
        ┌──────────┼──────────┬─────────────┬──────────┐
        ↓          ↓          ↓             ↓          ↓
    ┌─────┐   ┌────────┐ ┌───────┐   ┌────────┐  ┌──────────┐
    │Bevy │   │Unreal  │ │Unity  │   │  Web   │  │ Research │
    │2D   │   │  C++   │ │  C#   │   │TypeScript│ │  API     │
    │Game │   │  Game  │ │ Game  │   │ Browser│  │ (Python) │
    └─────┘   └────────┘ └───────┘   └────────┘  └──────────┘
```

**Key Differences:**
- **Deterministic:** Exact replay from event log (no RNG surprises)
- **Multi-client:** 5+ renderers on same timeline
- **Research-first:** Full API for scenario scripting, policy testing
- **Game-quality:** 60 FPS client rendering, sub-16ms server tick
- **Open-ended:** Dwarf Fortress depth + Victoria 3 political economy

---

## Target Users

| User Type | Motivation | Example Use Case |
|-----------|------------|------------------|
| **Game Developer** | Integrate into Bevy/Unreal/Unity | "I want an RTS backend that handles deep economy; I provide the UI" |
| **Game Designer** | Design scenarios, test policies | "Tweak tax rates and see if rebellion risk increases" |
| **Researcher** | Study policy emergents, economics | "Model historical scenarios (Crusades, Industrial Revolution)" |
| **Streamer/Content** | Play game live; show mechanics | "Let viewers vote on policies; watch simulation evolve" |
| **Educator** | Interactive history/economics | "Students run scenarios, understand cause/effect in trade networks" |

---

## Competitive Analysis

### Analogs & Inspirations

| Game/Engine | What We Admire | What We Do Differently |
|---|---|---|
| **Dwarf Fortress** | Depth, emergent simulation, auditability | Add political economy, deterministic replay, multi-client |
| **Victoria 3** | Economic detail, policy levers, complexity | Add spatial simulation, events, architecture for modding |
| **Crusader Kings III** | Actor networks, diplomacy, emergent narratives | Add economy depth, research API, faster iteration |
| **Factorio** | Production graphs, resource flows, optimization | Add markets, labor, geography, supply chain economics |
| **Terra Nil** | Ecosystem simulation, ecological balance | Add economy and society (urban planning + civilization) |
| **OpenTTY** | Transport networks, logistics, commerce | Add politics, culture, warfare, deeper markets |
| **Influence (Space Colony)** | Sandbox emergence, policy experimentation | Make truly open-ended (no win conditions) |

### CivLab's Unique Value

| Dimension | Status |
|-----------|--------|
| **Headless + Multi-Client** | ✓ Unique in this category |
| **Deterministic Replay** | ✓ Like Dwarf Fortress; stronger (event logs) |
| **Economy as Deep as Victoria 3** | ✓ Joule economy (energy-based), markets, taxes, subsidies |
| **Actor Networks (CK3-style)** | ◐ Roadmap v2 (citizens, factions, dynasties) |
| **Spatial Simulation** | ◐ Roadmap v2 (migration, region-based trade, climate) |
| **Modular Rendering** | ✓ Unique: Bevy/Unreal/Unity all supported |
| **Research API** | ✓ First-class scripting (scenarios, policies) |
| **Sub-16ms Tick** | ✓ 60 FPS client rendering possible |

---

## Feature Matrix

### MVP (Q3 2026)

**Scope:** Proof of concept. Single tick loop. Minimal economy. One client (Bevy demo).

| Feature | Description | Size |
|---------|---|---|
| **Core Tick Loop** | Fixed-timestep (100 ms), deterministic | M |
| **Basic ECS** | Cells, buildings, citizens, institutions | M |
| **Minimal Economy** | Production (basic), inventory, no trading | S |
| **Single Market** | Grain price clearing, bid/ask matching | S |
| **Joule Placeholder** | Energy unit defined; not integrated | XS |
| **WebSocket Server** | JSON-RPC handshake + snapshot broadcast | M |
| **Bevy Client Demo** | Render agents/buildings; click to build | M |
| **Replay Format** | Export run to .civreplay file | M |
| **Test Suite** | Determinism tests, integration tests | M |

**Success Criteria:**
- [ ] Tick loop runs 10,000 ticks in < 2 minutes (headless)
- [ ] Replay determinism test passes (identical input → identical output)
- [ ] Bevy demo runs 60 FPS, agents move, buildings spawn
- [ ] WebSocket server handles ≥ 3 simultaneous clients
- [ ] .civreplay file exports, loads, verifies checksum

### v1 (Q4 2026)

**Scope:** Complete core. Polished economy. Multi-client support. Client libraries.

| Feature | Description | Size |
|---------|---|---|
| **Joule Economy v1** | Full energy accounting, production chaining | L |
| **Market System v1** | Multi-good markets, supply/demand equilibrium | M |
| **Taxation & Budget** | Fiscal policy, spending, legitimacy impact | M |
| **War & Diplomacy (Shadow)** | Basic combat, casualty resolution, alliances | M |
| **Social System (Shadow)** | Citizen mood, rebellion risk, satisfaction | S |
| **Institutions** | Nations, provinces, hierarchies | M |
| **Climate & Geography (Basic)** | Terrain types, fertility, basic weather | M |
| **Multi-Client Support** | Unreal, Unity, Web clients (reference implementations) | L |
| **Snapshot Filtering** | Clients request partial snapshots (bandwidth optimization) | M |
| **Binary Frames** | High-frequency game client protocol | M |
| **Client Libraries** | TypeScript, C++, C# wrappers | L |
| **Scenario API** | Python API for scenario definition/scripting | L |
| **Performance Tuning** | Sub-16ms tick on commodity hardware | M |
| **Documentation** | Full spec, integration guides, examples | M |

**Success Criteria:**
- [ ] 100,000-tick scenario runs in < 20 minutes
- [ ] Joule economy conserves energy (property test: conservation law holds every tick)
- [ ] ≥ 10 simultaneous clients, no race conditions
- [ ] Bevy, Unreal, Unity, Web clients all render same snapshot deterministically
- [ ] Scenario API: can define custom policies, run 50-tick scenario in < 5 sec
- [ ] Replay any v1 run from event log; verify bit-identical

### v2 (2027 Q1-Q2)

**Scope:** Deep simulation. Advanced economy. Research-grade features.

| Feature | Description |
|---------|---|
| **Joule Economy v2** | Efficiency losses, energy storage, power grids |
| **Citizen Lifecycle** | Birth, aging, death, education, migration |
| **Trade Networks** | Supply chains, merchant routing, tariffs |
| **War v1** | Full combat systems, unit types, fatigue, strategy |
| **Diplomacy v1** | Treaty negotiation, alliances, betrayal |
| **Culture & Ideology** | Religion spread, cultural resistance, revolutions |
| **Technology Tree** | Research costs, benefits, diffusion |
| **Modding API** | Custom policies, goods, entity types |
| **Godot Client** | Fourth major engine supported |
| **Research Notebook** | Interactive scenario analysis, graphs, export |

### v3+ (2027+)

| Feature | Description |
|---------|---|
| **Multiplayer Persistence** | Multiple players control different nations (same world) |
| **GIS Integration** | Real-world maps; run historical scenarios |
| **Agent AI** | NPC decision-making (trade, war, diplomacy) |
| **Mod Marketplace** | Community scenarios, custom economies |
| **Analytics Dashboard** | Real-time metrics, replay analysis, dashboards |

---

## Non-Functional Requirements

### Performance

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Tick Duration** | ≤ 16 ms | 60 FPS client rendering |
| **Tick Determinism** | 100% (bit-identical) | Auditability, testing |
| **Memory Footprint** | ≤ 2 GB for 1M agents | Commodity laptop |
| **Network Bandwidth** | ≤ 10 Mbps (60 clients @ 60 FPS) | Consumer internet |
| **Startup Time** | ≤ 2 sec (load scenario + handshake) | Responsive UX |

### Scalability

| Dimension | Target | Rationale |
|-----------|--------|-----------|
| **Simulation Size** | ≥ 1,000,000 agents | Large civilizations |
| **Map Size** | 10,000 × 10,000 cells | Terra Nil scale |
| **Simultaneous Clients** | ≥ 10 | Spectator + research dashboards |
| **Scenario Duration** | ≥ 1,000,000 ticks (28 hours sim time) | Multi-generation runs |
| **Historical Replays** | ≥ 100,000 runs cached | Research databases |

### Reliability

| Aspect | Target |
|--------|--------|
| **Uptime** | 99% (during development) |
| **Crash-free Ticks** | 99.99% (failures logged, non-fatal) |
| **Data Integrity** | 100% (no silent corruption) |
| **Replay Correctness** | 100% (determinism verified on every run) |

### Security & Compliance

| Aspect | Target |
|--------|--------|
| **Input Validation** | All commands validated before execution |
| **Authorization** | Admin vs. player vs. research API client roles |
| **Audit Trail** | Full event log (no deletion, append-only) |
| **Data Privacy** | Encryption in transit (WebSocket TLS); at rest (file-based) |

### Determinism & Auditability

| Requirement | Mechanism |
|-------------|-----------|
| **Reproducibility** | ChaCha20Rng seeded per run; all RNG logged |
| **Auditability** | Every state mutation → event + hash |
| **Replayability** | .civreplay format: event log + checksum |
| **Verification** | Replay all ticks; compare state hashes |

---

## Epics

### E1: Core Engine (Tiers: MVP, v1)

**Objective:** Deterministic tick loop with ECS, minimal economy, replay capability.

**User Stories:**
- **E1.1:** Implement fixed-timestep tick loop (100 ms/tick, no jitter)
- **E1.2:** Design ECS entity model (cells, buildings, agents, institutions)
- **E1.3:** Implement policy evaluation phase (transform state → controls)
- **E1.4:** Implement deterministic transition phase (no RNG, fixed-point arithmetic)
- **E1.5:** Implement stochastic event phase (seeded RNG, event generation)
- **E1.6:** Serialize state to JSON snapshot
- **E1.7:** Implement multi-client command queue (priority + FIFO ordering)
- **E1.8:** Export runs to .civreplay format (header + event log + checksum)
- **E1.9:** Verify determinism (replay tests, mandatory CI gate)
- **E1.10:** Performance profiling (tick budget: 14 ms target)

**Acceptance Criteria:** See FR-CIV-CORE-001 through FR-CIV-CORE-020 (in CIV-0001 spec)

**Story Points:** ~13 weeks

---

### E2: Economy & Joule System (Tiers: MVP, v1)

**Objective:** Production, allocation, markets. Joule energy-based accounting.

**User Stories:**
- **E2.1:** Implement production system (buildings produce goods per tick)
- **E2.2:** Implement inventory management (per-entity resources + constraints)
- **E2.3:** Implement market clearing (bid/ask matching, price discovery)
- **E2.4:** Implement Joule accounting (energy as universal numeraire)
- **E2.5:** Implement allocation algorithm (distributes goods to consumers)
- **E2.6:** Implement taxation system (fiscal policy → revenue)
- **E2.7:** Implement budget system (spending, deficits, interest)
- **E2.8:** Implement legitimacy model (policy → happiness → rebellion risk)
- **E2.9:** Property testing (conservation laws hold every tick)
- **E2.10:** Stress testing (market crashes, supply shocks don't corrupt state)

**Acceptance Criteria:** See FR-CIV-ECON-001 through FR-CIV-ECON-015 (not in this doc, but referenced in CIV-0100)

**Story Points:** ~16 weeks

---

### E3: Multi-Client Protocol (Tiers: v1)

**Objective:** WebSocket JSON-RPC, binary frames, integration patterns for game engines.

**User Stories:**
- **E3.1:** Implement WebSocket server (RFC 6455 compliant)
- **E3.2:** Implement JSON-RPC 2.0 message dispatcher
- **E3.3:** Implement client handshake (identity, bootstrap data)
- **E3.4:** Implement command protocol (action validation, queueing)
- **E3.5:** Implement snapshot subscription (streaming + filtering)
- **E3.6:** Implement binary frame format (zstd compression, header)
- **E3.7:** Implement client priority tiers (admin > player > research)
- **E3.8:** Implement snapshot filtering (entity type, region bounds)
- **E3.9:** Implement query API (diagnostic queries for research)
- **E3.10:** Performance testing (10 clients @ 60 FPS, < 10 Mbps)

**Acceptance Criteria:** See FR-CIV-PROTO-001 through FR-CIV-PROTO-015 (in CIV-0200 spec)

**Story Points:** ~12 weeks

---

### E4: War & Diplomacy (Tiers: v1, v2)

**Objective:** Combat, alliances, diplomatic events.

**User Stories:**
- **E4.1:** Implement military units (soldiers, armies, generals)
- **E4.2:** Implement combat resolution (deterministic, fatigue-based)
- **E4.3:** Implement casualty handling (dead units removed, morale impact)
- **E4.4:** Implement alliance system (formal agreements, betrayal risk)
- **E4.5:** Implement war declaration (causes, legitimacy impact)
- **E4.6:** Implement occupied territories (control transfer, occupation cost)
- **E4.7:** Implement truce mechanics (time-limited peace)
- **E4.8:** Acceptance tests (replay battle → identical casualties)

**Acceptance Criteria:** TBD (v1.5 / v2)

**Story Points:** ~14 weeks (v2)

---

### E5: Research API & Scenario System (Tiers: v1, v2)

**Objective:** Scriptable scenario definition, policy testing, data export.

**User Stories:**
- **E5.1:** Implement scenario YAML format (map, initial conditions, policies)
- **E5.2:** Implement Python scenario loader (parse YAML, run simulation)
- **E5.3:** Implement policy parameter overrides (tweak economy knobs)
- **E5.4:** Implement query API (agent counts, market prices, ledger balances)
- **E5.5:** Implement replay inspection tools (event filtering, timeline analysis)
- **E5.6:** Implement data export (CSV, JSON for Jupyter/matplotlib)
- **E5.7:** Implement scenario benchmarking (run 100 variations, compare metrics)
- **E5.8:** Acceptance tests (run historical scenario, verify metric bounds)

**Acceptance Criteria:** TBD

**Story Points:** ~10 weeks

---

### E6: Client Implementations (Tiers: v1, v2)

**Objective:** Reference implementations for major game engines + Web.

**User Stories:**
- **E6.1:** Implement Bevy plugin (ECS integration, asset loading, render loop sync)
- **E6.2:** Implement Unreal C++ plugin (UObject integration, binary frame unpacking)
- **E6.3:** Implement Unity C# plugin (GameObject sync, coroutine-based update)
- **E6.4:** Implement Web TypeScript client (React hooks, WebSocket transport)
- **E6.5:** Implement Godot GDScript plugin (scene tree sync)
- **E6.6:** Example scenarios for each (playable demo per engine)
- **E6.7:** Integration tests (each client renders deterministic snapshot)

**Acceptance Criteria:** See pattern 1-4 in CIV-0200

**Story Points:** ~18 weeks

---

## Platform Strategy

### Deployment

**MVP:** Single-machine headless binary
```bash
civlab-server --scenario data/scenarios/starting_settlement.yaml --port 9876
```

**v1:** Docker container + cloud deployments
```bash
docker run -p 9876:9876 civlab-server:v1.0.0 --scenario scenario.yaml
```

### Roadmap

| Phase | Timeline | Focus |
|---|---|---|
| **MVP** | Q3 2026 | Core tick loop + Bevy demo |
| **v1** | Q4 2026 | Full economy + multi-engine clients |
| **v1.1** | Q1 2027 | Performance optimization, bugfixes |
| **v2** | Q1-Q2 2027 | Deep simulation (citizens, tech trees, culture) |
| **v3** | 2027+ | Multiplayer, GIS, modding, analytics |

---

## Success Metrics

### Product Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Time to First Client** | < 5 min | Developer tutorial |
| **Determinism Pass Rate** | 100% | CI: replay tests per commit |
| **API Stability** | 95% | Breaking changes per release |
| **Performance SLA** | 99% ticks < 16 ms | Production monitoring |
| **Multi-Client Capability** | ≥ 10 simultaneous | Load testing |

### User Adoption Metrics (v2+)

| Metric | Target |
|--------|--------|
| **Game Developer Integrations** | ≥ 3 games shipped with CivLab backend |
| **Research Publications** | ≥ 5 academic papers using CivLab |
| **Community Scenarios** | ≥ 50 user-created scenarios |
| **GitHub Stars** | ≥ 1,000 |
| **Community Discord Members** | ≥ 500 |

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|---|---|---|---|
| **Determinism Bug** | Undermines auditability | Medium | Mandatory replay tests, property-based testing |
| **Performance Regression** | Tick > 16 ms → client stutter | Medium | Continuous profiling, budget allocation |
| **Multi-Client Race Condition** | Silent data corruption | Low | Strict immutability, thread-safety analysis |
| **Protocol Breaking Change** | Clients break on update | Low | Versioning strategy, deprecation period |
| **Scope Creep** | Miss MVP deadline | Medium | Strict feature gating, prioritization |

---

## Business Model (Long-term)

### Open Source (MVP → v2)

CivLab is **open source** (MIT license) to maximize adoption and community contribution.

### Monetization Opportunities (v3+)

| Channel | Model | Example |
|---------|-------|---------|
| **Hosted Service** | SaaS | civlab.io: host scenarios, analytics dashboard |
| **Mod Marketplace** | Marketplace fee | 70/30 split on community-created scenarios |
| **Consulting** | Services | Integration support for game studios |
| **Research Licensing** | Custom licenses | Per-seat license for academic institutions |
| **Enterprise Support** | Support contracts | SLA guarantees, priority support |

---

## References

- **CIV-0001:** Core Simulation Loop spec
- **CIV-0100:** Economy v1 spec
- **CIV-0107:** Joule Economy System spec
- **CIV-0200:** Multi-Client Protocol spec
- **ADR-003:** Deterministic Replay ADR
- **ADR-002:** Joule Economy as Allocator ADR
- **GitHub:** https://github.com/civlab/civlab (coming soon)

---

**Document History:**
- v1.0 (2026-02-21): Initial PRD. MVP, v1, v2 roadmap. Competitive analysis. Success metrics.
