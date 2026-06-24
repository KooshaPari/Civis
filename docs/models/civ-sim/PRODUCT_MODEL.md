# CivLab Civ-Sim Product Model

**Version:** 0.2.0
**Status:** Active
**Owner:** CivLab Product / Engineering
**Last Updated:** 2026-02-21
**Audience:** Product managers, engineers, researchers, integration partners

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Product Vision and Strategy](#3-product-vision-and-strategy)
4. [Jobs To Be Done (JTBD)](#4-jobs-to-be-done-jtbd)
5. [Product Surfaces and Feature Map](#5-product-surfaces-and-feature-map)
6. [Simulation Domain Coverage](#6-simulation-domain-coverage)
7. [Technical Product Constraints](#7-technical-product-constraints)
8. [Competitive Analysis](#8-competitive-analysis)
9. [Business Model](#9-business-model)
10. [Success Metrics and KPIs](#10-success-metrics-and-kpis)
11. [Risks and Mitigations](#11-risks-and-mitigations)
12. [Roadmap and Milestones](#12-roadmap-and-milestones)
13. [Integration with Parpour/Venture](#13-integration-with-parpourventure)
14. [Governance and Decision Framework](#14-governance-and-decision-framework)

---

## 1. Executive Summary

### 1.1 Product Vision (Expanded)

CivLab is a **headless, deterministic civilization simulation engine** written in Rust. It models the full lifecycle of a society — energy production, resource distribution, institutional legitimacy, demographic change, climate feedback, and armed conflict — as a tick-driven state machine with byte-for-byte reproducible output.

The engine serves two distinct but deeply connected purposes:

1. **Standalone simulation platform** — enabling scenario designers, systems researchers, and policy analysts to define, run, compare, and inspect long-horizon "what if" experiments about governance, economics, and social stability.

2. **AI economic backend** — powering Parpour/Venture, an autonomous AI economic platform where AI agents run CivLab scenarios to validate, stress-test, and iteratively improve economic and governance policies before proposing them in real-world contexts.

CivLab is not a game engine. It is a scientific-grade simulation substrate. The simulation surface is domain-complete: every major lever of civilizational stability (energy, climate, institutions, demography, diplomacy, insurgency) is modeled with enough fidelity that meaningful policy experiments can produce unexpected and calibrated results.

### 1.2 Market Positioning

CivLab occupies a white space between:

- **Commercial strategy games** (Victoria 3, Stellaris) that simulate politics but are not programmable, not deterministic, and not API-accessible.
- **Agent-based modeling frameworks** (NetLogo, Mesa) that are programmable but lack deep civilizational domain models — researchers must build everything from scratch.
- **Academic simulation tools** (POLARIS, ASPATIAL) that are domain-specific, non-modular, and inaccessible to non-academics.

CivLab's positioning: **an open-source, embeddable, API-first civilization simulation engine that is rigorous enough for research and ergonomic enough for designers.**

```
                    Open Source / Embeddable
                              |
                              |  [CivLab]
            Broad Domain -----+------------- Narrow Domain
            Coverage          |               Coverage
                              |
                    Closed Source / Black Box
```

### 1.3 Core Value Propositions

| # | Value Proposition | Target | Differentiation |
|---|-------------------|--------|-----------------|
| 1 | **Byte-perfect determinism** | Researchers, AI agents | Seed → identical tick sequence guaranteed; D1-D7 ruleset enforced |
| 2 | **Domain completeness** | Policy analysts, designers | Energy, climate, institutions, demography, diplomacy, insurgency — all coupled |
| 3 | **Headless-first, embeddable** | Engineers, AI platforms | No GUI required; Rust crate + WASM target; integrate into any runtime |
| 4 | **Open-source core** | Community, academia | MIT/Apache-2.0 dual license; no vendor lock-in; forkable |
| 5 | **Modding platform** | Designers, researchers | WASM sandbox; four mod types; civlab-sdk; asset pipeline |
| 6 | **Parpour/Venture integration** | AI economic platforms | First-class API contract for AI agent scenario dispatch and result ingestion |

### 1.4 North Star Metric

> **Scenarios executed per day** across all deployment modes (local, cloud, Parpour/Venture) with 100% deterministic replay consistency.

This metric captures both adoption (volume of use) and core product health (determinism never regresses). A simulation platform that produces unreproducible results has zero scientific value. Growth in scenario volume with zero determinism regressions is the single best signal that CivLab is succeeding.

---

## 2. Problem Statement

### 2.1 Problems by Persona

#### Persona 1: Scenario Designer

The scenario designer builds simulation scenarios — defining initial conditions, governance structures, resource endowments, and policy regimes — to explore how civilizations evolve. They may be a game designer building a strategy title, a narrative designer constructing plausible historical counterfactuals, or an educator building teaching simulations.

**Core problems:**

- **Existing games (Victoria 3, Crusader Kings) are black boxes.** The designer can tweak surface parameters but cannot inspect the underlying model. When a society collapses, the cause is opaque. Designers cannot learn from the simulation.
- **Game engines are not simulation substrates.** Unity or Godot can render but do not provide energy/resource accounting, institutional modeling, or coupled social dynamics. Designers must build all domain logic from scratch.
- **Scenario authoring is code-heavy.** No domain-specific authoring language or structured UI for defining constitutions, resource endowments, and initial demographics exists in open tools.
- **Replay and comparison are absent.** Designers cannot branch a scenario at tick T, try two policy interventions, and compare the resulting divergence over 1000 ticks.

**Impact:** High iteration cost, low experimentation velocity, shallow scenario depth.

#### Persona 2: Policy Analyst / Systems Researcher

The policy analyst models real-world governance and economic systems to test hypotheses: "Does universal basic energy access reduce insurgency rates?" "How does climate shock timing interact with institutional resilience?" They may be academic researchers, think-tank analysts, or government modeling teams.

**Core problems:**

- **Agent-based frameworks (NetLogo, Mesa) require building everything from scratch.** A researcher must implement energy economics, climate feedback, institutional legitimacy, and social dynamics — a multi-year engineering effort — before running a single policy test.
- **Existing simulation tools are domain-siloed.** Climate models don't couple to governance. Economic models don't couple to demography. Real-world systems are deeply coupled; siloed models produce systematically misleading results.
- **Reproducibility is a crisis.** Published simulation results cannot be reproduced when tools are commercial, stochastic without controlled seeding, or dependent on deprecated environments.
- **Batch sweep infrastructure is DIY.** Running 10,000 parameter variations requires writing custom HPC scripts; there is no standard sweep API.

**Impact:** Slow research cycles, irreproducible findings, high infrastructure burden.

#### Persona 3: Research Operator (Parpour/Venture AI Agent)

The research operator is an autonomous AI agent (or the human orchestrating one) that uses CivLab as a simulation backend for policy testing. The agent proposes an economic or governance policy, dispatches a simulation scenario, evaluates the outcome metrics, and iterates. This is the Parpour/Venture integration persona.

**Core problems:**

- **No simulation engine exposes a clean programmatic API** for scenario definition, execution, and structured result retrieval. AI agents cannot interface with game UIs.
- **Stochastic simulations defeat AI learning loops.** If the same policy produces different outcomes each run (due to non-determinism), the AI agent cannot distinguish policy quality from random variance.
- **Result schemas are unstructured.** Game save files, NetLogo output CSVs, and academic tool outputs are not designed for machine consumption. AI agents need typed, versioned, schema-validated result payloads.
- **Long-horizon scenarios are computationally expensive.** Without performance guarantees (target: 100ms/tick), AI agent iteration loops are too slow for practical policy search.

**Impact:** AI policy agents cannot use existing simulation tools. CivLab is purpose-built to fill this gap.

### 2.2 Market Gap: Why Existing Tools Fail

| Tool | Strengths | Failure Mode for CivLab Use Cases |
|------|-----------|-----------------------------------|
| Dwarf Fortress | Deep simulation depth, citizen modeling | Closed source, not embeddable, non-deterministic, no API |
| Victoria 3 | Political economy, trade, demographics | Closed source, black box, not programmable, not headless |
| Factorio | Production chains, logistics, performance | No governance, no social dynamics, no demography |
| OpenTTD | Open source, moddable | Transport only; no political/social/economic governance |
| NetLogo / Mesa | Programmable, academic | No built-in domain models; build everything from scratch |
| AnyLogic | Hybrid simulation | Commercial, expensive, no civilization domain models |
| MASON / Repast | ABM frameworks | Research-grade, steep learning curve, no civilization models |

**The gap:** No tool simultaneously offers (a) deep civilizational domain models, (b) byte-perfect determinism, (c) headless embeddability, (d) open-source licensing, and (e) a clean API for programmatic scenario dispatch.

### 2.3 Opportunity Sizing

The addressable market for CivLab is segmented across three vectors:

**Open-source simulation community:** The intersection of strategy game developers, systems thinkers, and complexity researchers represents an estimated 50,000–200,000 technically proficient users globally. The success of Dwarf Fortress (2 million copies sold), OpenTTD (millions of downloads), and Factorio ($30M+ revenue) demonstrates strong market pull for deep, moddable simulation.

**Policy research and academic modeling:** University departments, government think-tanks, and international organizations (UN, World Bank) increasingly use computational simulation for policy analysis. Budget for simulation infrastructure at a single research institution can reach $100K–$500K/year.

**AI economic platforms (Parpour/Venture):** As AI agents are deployed for economic and policy analysis, demand for programmatic simulation backends is nascent but fast-growing. CivLab's Parpour integration targets this emerging segment directly, where the competitive set is essentially empty.

---

## 3. Product Vision and Strategy

### 3.1 Vision Statement

> CivLab is the open, deterministic substrate for civilizational simulation — a headless Rust engine that any researcher, designer, or AI agent can embed, extend, and trust to produce the same result from the same seed, every time, forever.

### 3.2 Strategic Pillars

#### Pillar 1: Determinism as Covenant

Determinism is not a feature — it is CivLab's foundational promise. Every product and engineering decision must preserve the D1-D7 determinism ruleset. Determinism erosion is an existential risk. The product will never ship a feature that compromises the seed-to-output guarantee, even at the cost of performance or expressiveness.

**Implications:**
- All randomness flows through ChaCha20Rng seeded from a user-supplied or system-derived seed.
- No wall-clock time, no float comparison, no hash map iteration order in simulation code.
- BLAKE3 state hash computed and exposed every tick for snapshot verification.
- CI pipeline enforces replay consistency as a hard gate.

#### Pillar 2: Domain Completeness Before Depth

The simulation must cover all major civilizational subsystems before deepening any single subsystem. A model that has a deep energy economy but no climate, no institutions, and no social dynamics produces fundamentally misleading results — it cannot capture real civilizational dynamics.

**Priority order:**
1. Energy economy (Joule Economy — the master constraint)
2. Climate (the long-horizon shock system)
3. Institutions (the governance and legitimacy layer)
4. Citizens/Demography (the social foundation)
5. Social/Insurgency (the legitimacy feedback)
6. War/Diplomacy (the external pressure)

Depth enhancements to any subsystem are deferred until all subsystems are present at MVP coverage.

#### Pillar 3: Headless-First, GUI as Optional Layer

CivLab's primary interface is its Rust API and WASM module. The Web RTS client (Pixi.js v8 + React 19) is a visualization layer, not the product. The Desktop client (Bevy 3D) is a future enhancement. All features must be exercisable without a GUI. GUI features that require GUI state unavailable to the headless API are not permitted.

**Implications:**
- Every simulation control available in the GUI must have a CLI/API equivalent.
- Scenarios are defined as structured data (TOML/JSON/MessagePack), not GUI-only workflows.
- Batch sweep, replay, and branch comparison are first-class headless operations.

#### Pillar 4: Open Ecosystem as Competitive Moat

CivLab's long-term defensibility comes not from the core engine alone but from the ecosystem built on top of it: community-created scenarios, WASM mods, research datasets, and integration adapters. The engine must be easy to embed, easy to mod, and easy to extend. The civlab-sdk and WASM registry are strategic investments in ecosystem lock-in through community, not through proprietary technology.

### 3.3 18-Month Roadmap Summary

| Phase | Timeline | Theme | Key Deliverables |
|-------|----------|-------|-----------------|
| Phase 0 | Months 0–2 | Core tick loop | Rust crate, ChaCha20Rng, BLAKE3 hash, D1-D7 harness, CI determinism gate |
| Phase 1 | Months 2–5 | Economy + Climate | Joule Economy (CIV-0100), Climate System (CIV-0102), metrics API |
| Phase 2 | Months 5–9 | Institutions + Citizens + Social | Institutions (CIV-0103), Demography (CIV-0104), Social/Insurgency (CIV-0106) |
| Phase 3 | Months 9–13 | War/Diplomacy + Mod Platform | War (CIV-0105), WASM mod sandbox (CIV-0700), civlab-sdk v0.1 |
| Phase 4 | Months 13–17 | Web client + Asset pipeline | Pixi.js Web RTS (CIV-0300), SDXL asset gen (CIV-0600), scenario authoring UI |
| Phase 5 | Months 17–24 | 3D + AI/NPC + Parpour GA | Bevy Desktop (CIV-0400), AI NPC (CIV-0601), Parpour/Venture GA integration |

### 3.4 Success Horizons

**At 12 months (Phase 3 complete):**
- All six simulation domains modeled at MVP coverage
- Headless API stable and versioned
- WASM mod sandbox operational
- Parpour/Venture beta integration running
- 10+ community-contributed scenarios in registry
- Zero determinism regressions in CI

**At 24 months (Phase 5 complete):**
- Web RTS client in public beta
- Bevy 3D desktop client in alpha
- Modding marketplace live with revenue share
- Cloud simulation credits platform in production
- 100+ community mods
- Parpour/Venture GA with SLA
- Academic publications citing CivLab reproducibility

---

## 4. Jobs To Be Done (JTBD)

The JTBD framework captures what users are trying to accomplish, at a level that transcends specific features. Jobs are expressed as: "When [situation], I want to [motivation], so I can [outcome]."

### 4.1 Scenario Designer JTBD

#### Functional Jobs

| Job ID | Job Statement | Priority |
|--------|---------------|----------|
| SD-F1 | When designing a new scenario, I want to define initial energy endowments, climate parameters, and institutional structures in a structured format, so I can start a simulation from a specific, reproducible initial state. | Critical |
| SD-F2 | When a simulation produces an unexpected collapse, I want to scrub back through the tick timeline and inspect the exact state at any tick, so I can identify the root cause of the failure. | Critical |
| SD-F3 | When testing a policy change, I want to branch the simulation at a specific tick and run two variants forward, so I can compare the counterfactual divergence over N ticks. | High |
| SD-F4 | When building a scenario library, I want to save, version, and share scenario definitions as portable artifacts, so collaborators can run the same scenario on their own machines. | High |
| SD-F5 | When the simulation produces interesting behavior, I want to see a visual timeline of key metrics (legitimacy, Joule stock, insurgency rate) so I can communicate the dynamic to stakeholders. | Medium |
| SD-F6 | When I want to extend the simulation beyond its built-in behaviors, I want to write a WASM mod (Policy, Economic, Event, or Scenario type) and load it into a running simulation. | Medium |

#### Social Jobs

| Job ID | Job Statement |
|--------|---------------|
| SD-S1 | Be recognized by the simulation design community as someone who builds rigorous, reproducible scenarios (not just "vibes-based" game balancing). |
| SD-S2 | Demonstrate to employers or collaborators that designed scenarios are based on a scientifically credible simulation substrate. |

#### Emotional Jobs

| Job ID | Job Statement |
|--------|---------------|
| SD-E1 | Feel confident that when I share a scenario, others will get exactly the same results I got. |
| SD-E2 | Feel in control of the simulation — understanding why things happen, not just watching them happen. |
| SD-E3 | Feel productive: rapid iteration without fighting tooling infrastructure. |

#### Pain Points and Gain Creators

| Pain Point | Severity | CivLab Gain Creator |
|------------|----------|---------------------|
| Black-box collapses with no traceable cause | High | Tick-level state inspection, BLAKE3 snapshot trail |
| No branching/counterfactual support | High | Branch API: fork at tick T, run N variants |
| Scenario sharing breaks due to non-determinism | High | Seed + initial-state artifact = portable, reproducible |
| Authoring requires writing raw code | Medium | TOML/JSON scenario definition + future authoring UI |
| Mod API is undocumented or unstable | Medium | civlab-sdk with versioned WASM API, typed mod types |

### 4.2 Policy Analyst / Systems Researcher JTBD

#### Functional Jobs

| Job ID | Job Statement | Priority |
|--------|---------------|----------|
| PA-F1 | When testing a policy hypothesis, I want to run 1,000+ parameter variations (sweep) from the CLI and collect structured result payloads, so I can do statistical analysis on outcome distributions. | Critical |
| PA-F2 | When writing a research paper, I want to publish a scenario artifact (seed + initial state + mod set) that any reader can run to reproduce my exact results, so my findings are independently verifiable. | Critical |
| PA-F3 | When studying climate-governance coupling, I want to see how climate shock timing interacts with institutional resilience across a parameter sweep, so I can identify non-linear thresholds. | High |
| PA-F4 | When analyzing instability events, I want the simulation to surface a structured causal trace (event chain: energy scarcity → legitimacy drop → insurgency threshold breach → state collapse), so I can validate or falsify theoretical claims. | High |
| PA-F5 | When comparing governance regimes, I want to run the same scenario under two constitutional configurations and see side-by-side metric divergence over 10,000 ticks. | High |
| PA-F6 | When I need to extend the model for a specific research question, I want to write a custom economic or event mod without forking the core engine. | Medium |

#### Social Jobs

| Job ID | Job Statement |
|--------|---------------|
| PA-S1 | Publish reproducible simulation results that survive peer review. |
| PA-S2 | Build a reputation for methodological rigor in computational social science. |
| PA-S3 | Collaborate with other researchers by sharing runnable, versioned simulation artifacts. |

#### Emotional Jobs

| Job ID | Job Statement |
|--------|---------------|
| PA-E1 | Trust that the simulation model is honest about its assumptions and limitations. |
| PA-E2 | Feel that the model is credible enough to cite in academic and policy contexts. |
| PA-E3 | Avoid the anxiety of discovering mid-paper that results cannot be reproduced. |

#### Pain Points and Gain Creators

| Pain Point | Severity | CivLab Gain Creator |
|------------|----------|---------------------|
| Building domain models from scratch (NetLogo/Mesa) | Critical | Pre-built coupled domain models; researcher focuses on policy levers |
| Irreproducible results block publication | Critical | Seed + BLAKE3 hash trail = cryptographic reproducibility |
| No batch sweep infrastructure | High | CLI sweep API: `civlab sweep --params params.toml --output results/` |
| Domain coupling is absent (siloed models) | High | All six domains modeled and coupled by design |
| Causal inspection is manual / impossible | High | Causal trace API: structured event chain for each instability |

### 4.3 Research Operator (Parpour/Venture AI Agent) JTBD

#### Functional Jobs

| Job ID | Job Statement | Priority |
|--------|---------------|----------|
| RO-F1 | When proposing a new economic policy, I want to dispatch a CivLab scenario via a typed API call, receive a structured result payload, and parse the outcome metrics programmatically, so I can score the policy without human intervention. | Critical |
| RO-F2 | When iterating on policy parameters, I want to run 100 scenario variants per hour with guaranteed &lt;100ms/tick latency, so my policy search loop is fast enough to be practical. | Critical |
| RO-F3 | When a policy produces unexpected outcomes, I want to retrieve the full tick-level state trace for causal analysis, so I can identify which policy parameter caused which outcome. | High |
| RO-F4 | When comparing two policy variants, I want to branch from a shared initial state, run both variants, and receive a structured diff of outcome metrics, so I can rank policies by objective function. | High |
| RO-F5 | When deploying in production Parpour/Venture, I want the CivLab API contract to be versioned and stable, so my integration does not break when CivLab is updated. | High |

#### Social Jobs (of the AI agent's operator)

| Job ID | Job Statement |
|--------|---------------|
| RO-S1 | Demonstrate to Parpour/Venture stakeholders that AI policy proposals are grounded in simulation evidence, not heuristics. |
| RO-S2 | Build trust in AI-driven policy recommendations by making the simulation backend auditable and reproducible. |

#### Emotional Jobs (of the AI agent's operator)

| Job ID | Job Statement |
|--------|---------------|
| RO-E1 | Feel confident that simulation results are not contaminated by non-determinism, enabling fair policy comparison. |
| RO-E2 | Feel that the AI policy loop is scientifically credible, not a black box. |

### 4.4 JTBD to Feature Mapping

| Job ID | Primary Feature | Secondary Feature |
|--------|-----------------|-------------------|
| SD-F1 | Scenario authoring (TOML/JSON schema) | Scenario library / versioning |
| SD-F2 | Replay inspector / tick scrubbing | BLAKE3 snapshot trail |
| SD-F3 | Branch API | Divergence comparison view |
| SD-F4 | Portable scenario artifacts | Scenario registry |
| SD-F5 | Metrics dashboard | Timeline visualization |
| SD-F6 | WASM mod sandbox | civlab-sdk |
| PA-F1 | CLI sweep API | Result export (Parquet/JSON) |
| PA-F2 | Reproducibility artifact export | BLAKE3 verification tool |
| PA-F3 | Cross-domain metric correlation view | Parameter sweep heatmap |
| PA-F4 | Causal trace API | Instability event log |
| PA-F5 | Side-by-side metric comparison | Regime diff view |
| PA-F6 | WASM mod: Economic / Event types | civlab-sdk documentation |
| RO-F1 | Programmatic scenario dispatch API | Structured result schema |
| RO-F2 | Headless batch runner | Performance SLO (&lt;100ms/tick) |
| RO-F3 | Tick-level state trace API | Causal trace export |
| RO-F4 | Branch API (headless) | Metric diff API |
| RO-F5 | Versioned API contract | Changelog + deprecation policy |

---

## 5. Product Surfaces and Feature Map

### 5.1 Product Surface Overview

```
+------------------------------------------------------------------+
|                        CivLab Platform                           |
+------------------------------------------------------------------+
|                                                                  |
|  +------------------+   +------------------+   +--------------+ |
|  | Scenario         |   | Simulation       |   | Replay       | |
|  | Authoring UI     |   | Runner           |   | Inspector    | |
|  | (Web RTS)        |   | (Headless/GUI)   |   | (Web/CLI)    | |
|  +------------------+   +------------------+   +--------------+ |
|           |                     |                     |          |
|  +------------------+   +------------------+   +--------------+ |
|  | Metrics          |   | Policy           |   | CLI / API    | |
|  | Dashboard        |   | Intervention     |   | Surface      | |
|  | (Web RTS)        |   | Controls         |   | (Headless)   | |
|  +------------------+   +------------------+   +--------------+ |
|           |                     |                     |          |
|  +----------------------------------------------------------+    |
|  |                    civlab-core (Rust)                    |    |
|  |  Tick Loop | ChaCha20Rng | BLAKE3 | Domain Systems       |    |
|  +----------------------------------------------------------+    |
|                              |                                   |
|  +----------------------------------------------------------+    |
|  |              WASM Mod Sandbox + civlab-sdk               |    |
|  |  Policy | Economic | Event | Scenario mod types          |    |
|  +----------------------------------------------------------+    |
|                                                                  |
+------------------------------------------------------------------+
         |                    |                    |
+--------+--------+ +---------+--------+ +---------+--------+
| Web RTS Client  | | Desktop (Bevy 3D)| | Parpour/Venture  |
| (Pixi.js + R19) | | [Future]         | | AI Agent API     |
+-----------------+ +------------------+ +------------------+
```

### 5.2 Scenario Authoring Interface

#### Description

The scenario authoring interface is the entry point for defining the initial conditions of a simulation. It allows designers and researchers to specify all starting parameters of a civilization: its energy endowments, climate configuration, institutional structure, citizen demographics, governance constitution, and diplomatic relations.

The interface exists in two forms:
1. **Structured data format (primary):** TOML/JSON scenario definition files, fully functional without any GUI. This is the authoritative representation.
2. **Web UI (secondary):** A visual authoring layer in the Web RTS client (Pixi.js v8 + React 19) that generates and validates the structured data format. The UI is a convenience — not the source of truth.

#### Key Features

| Feature | Description | Acceptance Criteria |
|---------|-------------|---------------------|
| Constitution editor | Define governance type, election cycle, enforcement power, judicial independence | Validates against governance schema; invalid constitutions rejected at load time with error message |
| Resource endowment configurator | Set initial Joule stocks, production capacity (kJ/tick), distribution infrastructure rating | All values typed as KiloJoules (i64); range validation; negative stocks rejected |
| Climate profile selector | Set base temperature, precipitation, volatility, and initial climate shock schedule | Climate config validates against climate schema; out-of-range parameters rejected |
| Citizen demographics editor | Set population size, age distribution, skill distribution, faction composition | Faction percentages must sum to 100%; population &gt; 1 |
| Diplomatic relations matrix | Set initial alliance, trade, and hostility values between simulated entities | Symmetric validation (if A is allied with B, B must be allied with A) |
| Seed configurator | Set simulation seed (u64) or generate random seed | Seed displayed prominently; copied to clipboard on demand |
| Scenario validation | Pre-flight check: validate all fields, surface errors with path and message | No scenario dispatched with validation errors; errors listed with TOML/JSON key paths |
| Scenario export | Export scenario as portable artifact (TOML + metadata JSON + mod manifest) | Exported artifact is self-contained; can be loaded on a different machine and produce identical results |
| Scenario library | Save, tag, search, and load scenarios from local or remote registry | Scenarios addressable by name+version; BLAKE3 hash of scenario artifact for integrity |

#### User Actions (Primary Flows)

1. **New scenario:** User creates a blank scenario from template, fills in fields, validates, saves, dispatches.
2. **Fork scenario:** User loads existing scenario, modifies one parameter, saves as new version, dispatches in parallel.
3. **Import scenario:** User receives a scenario artifact from a collaborator, imports it, verifies BLAKE3 hash, dispatches.
4. **Export for publication:** User runs scenario, captures result, exports scenario artifact + result bundle for reproducibility.

#### Performance Targets

- Scenario validation: < 50ms for any scenario definition
- Scenario load from file: < 100ms
- Scenario export: < 200ms including BLAKE3 computation

### 5.3 Simulation Runner

#### Description

The simulation runner is the core execution engine. It accepts a scenario definition, initializes the simulation state, and advances the state by one tick per invocation of the tick function. The runner exists in three modes:

1. **Headless mode:** CLI or API invocation; no rendering; maximum throughput.
2. **Interactive mode:** Web RTS or Desktop GUI; renders state at configurable FPS; allows pause, step, fast-forward, intervention.
3. **Batch/sweep mode:** CLI or API; runs N scenarios in parallel; collects structured results.

#### Key Features

| Feature | Description | Acceptance Criteria |
|---------|-------------|---------------------|
| Deterministic tick loop | Advance simulation by one tick; all randomness from ChaCha20Rng; no side effects | Same seed + scenario → identical tick sequence; verified by CI replay test |
| BLAKE3 state hash | Compute BLAKE3 hash of full simulation state after each tick | Hash changes IFF state changes; stored in tick log; used for snapshot verification |
| Pause / resume | Stop tick advancement; resume from exact state | State is identical before and after pause/resume cycle |
| Step mode | Advance exactly one tick at a time | Available in all runner modes; useful for debugging |
| Fast-forward | Advance N ticks as fast as possible (no frame cap) | Headless: no frame cap; GUI: configurable multiplier (1x, 5x, 20x, 100x, max) |
| Intervention injection | Apply a policy intervention at the current tick without branching | Intervention is applied atomically; state hash reflects intervention |
| Branch at tick T | Snapshot state at tick T; spawn two runner instances from that snapshot | Both instances produce identical tick T state; diverge from tick T+1 forward |
| Scenario abort | Terminate simulation on abort condition (e.g., population → 0, Joule stock → 0) | Abort condition logged with tick, cause, and last state hash |
| Headless batch runner | Run N scenarios (same or different seeds/params) in parallel | Thread pool sizing configurable; results collected to structured output directory |
| Parameter sweep | Run Cartesian product of parameter ranges; collect result metrics | Sweep config: TOML file specifying parameter ranges; output: Parquet or JSON Lines |

#### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Tick latency (single simulation) | &lt; 100ms per tick | Measured at p99 on reference hardware |
| Headless throughput | &gt; 600 ticks/minute per simulation instance | With all six domains active |
| Batch sweep throughput | &gt; 10,000 tick-scenarios/hour | On 8-core reference machine |
| Memory per simulation instance | &lt; 256MB | Full state in memory; no disk swapping |
| State hash computation | &lt; 5ms per tick | BLAKE3 is fast; must not dominate tick budget |

#### Tick Budget Allocation (Target, 100ms)

```
Tick Budget: 100ms
  ├── Joule Economy:       ~20ms
  ├── Climate System:      ~15ms
  ├── Institutions:        ~15ms
  ├── Citizens/Demography: ~20ms
  ├── Social/Insurgency:   ~10ms
  ├── War/Diplomacy:       ~10ms
  ├── BLAKE3 state hash:   ~5ms
  └── Overhead/serialize:  ~5ms
```

### 5.4 Replay Inspector and Timeline Viewer

#### Description

The replay inspector allows users to navigate the full tick history of a completed or paused simulation. It is the primary tool for post-hoc causal analysis: understanding why a civilization collapsed, why a legitimacy crisis emerged, or why an insurgency surged.

The inspector operates on a replay artifact: the complete sequence of state snapshots (or state diffs + full snapshots at interval) produced during a simulation run.

#### Key Features

| Feature | Description | Acceptance Criteria |
|---------|-------------|---------------------|
| Tick scrubbing | Navigate to any tick in the simulation history via slider or direct input | Seek to any tick in O(log N) time using snapshot index; state rendered within 200ms of seek |
| State inspection panel | View full simulation state at current tick: all domain metrics, entity states, event log | All domain state fields exposed; no hidden state |
| Event log | Chronological list of significant events (legitimacy threshold breach, insurgency outbreak, climate shock, war declaration, etc.) | Each event annotated with tick, type, causal chain summary, affected entities |
| Causal trace | For selected instability event, display structured causal chain: which domain transitions led to this event | Causal chain expressed as DAG: node = state transition, edge = causal dependency |
| Metric timeline | Time-series chart of any metric (Joule stock, legitimacy, insurgency rate, etc.) over the full simulation | Multiple metrics overlaid; cursor shows exact value at each tick; zoom to sub-range |
| Branch comparison | Load two replay artifacts from the same branch point; display side-by-side or overlay metric divergence | Tick alignment at branch point; divergence highlighted from tick T+1 |
| Annotation | Add user annotations to specific ticks or events for documentation or collaboration | Annotations stored in separate file; merged with replay artifact for sharing |
| Export | Export tick range as sub-replay or as CSV/Parquet for external analysis | Sub-replay is valid, self-contained replay artifact; CSV includes all metrics |
| BLAKE3 verification | Verify that replay artifact hashes match reference hashes for reproducibility | Verification result: pass/fail per tick; failures indicate determinism breach |

#### Replay Artifact Format

```
replay_artifact/
  metadata.json         # scenario seed, initial state hash, CivLab version, tick count
  snapshots/
    0000000000.snap     # full state snapshot at tick 0 (MessagePack)
    0001000000.snap     # full state snapshot at tick 1,000,000 (every N ticks)
  diffs/
    0000000001.diff     # state diff from tick 0→1 (MessagePack)
    ...
  events/
    events.json         # all significant events with tick, type, causal chain
  hashes/
    hashes.bin          # BLAKE3 hash per tick (binary, 32 bytes * N ticks)
```

### 5.5 Metrics Dashboard

#### Description

The metrics dashboard provides real-time and historical visualization of all tracked simulation metrics. It is the primary situational awareness surface for a running or completed simulation.

#### Tracked Metrics

**Joule Economy Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| joule_stock | Scalar | Total Joule reserves in system | kJ (i64) |
| joule_production_rate | Scalar | kJ produced per tick | kJ/tick |
| joule_consumption_rate | Scalar | kJ consumed per tick | kJ/tick |
| joule_distribution_efficiency | Ratio | Fraction of produced kJ reaching consumption nodes | 0.0–1.0 |
| joule_waste_rate | Scalar | kJ wasted per tick (not consumed, not stored) | kJ/tick |
| joule_surplus_rate | Scalar | kJ surplus per tick (produced - consumed - waste) | kJ/tick |
| joule_scarcity_index | Ratio | Population-weighted fraction experiencing energy deficit | 0.0–1.0 |
| per_capita_joule_access | Scalar | kJ per citizen per tick | kJ/citizen/tick |

**Governance and Institutional Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| institutional_legitimacy | Ratio | Population-weighted legitimacy score | 0.0–1.0 |
| tyranny_index | Ratio | Composite: enforcement overreach + political capture | 0.0–1.0 |
| institutional_resilience | Ratio | Capacity to absorb shocks without structural change | 0.0–1.0 |
| elite_capture_index | Ratio | Fraction of institutional decisions serving elite vs. population | 0.0–1.0 |
| policy_effectiveness | Ratio | Fraction of enacted policies producing intended outcome | 0.0–1.0 |
| election_cycle_health | Ordinal | Status of electoral processes (HEALTHY / STRESSED / SUSPENDED / COLLAPSED) | enum |

**Social and Demographic Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| population | Scalar | Total citizen count | persons |
| population_growth_rate | Scalar | Net population change per tick | persons/tick |
| gini_coefficient | Ratio | Resource inequality measure | 0.0–1.0 |
| social_cohesion | Ratio | Aggregate measure of inter-faction trust and cooperation | 0.0–1.0 |
| insurgency_rate | Ratio | Fraction of population in active insurgent activity | 0.0–1.0 |
| insurgency_severity | Ordinal | Composite severity (LATENT / ACTIVE / INSURGENCY / CIVIL_WAR) | enum |
| displacement_rate | Scalar | Citizens displaced per tick | persons/tick |

**Climate Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| mean_temperature | Scalar | Simulation-world mean temperature | degrees C (integer) |
| precipitation_index | Ratio | Normalized precipitation level | 0.0–1.0 |
| climate_shock_active | Boolean | Whether a climate shock event is currently active | bool |
| agricultural_yield_modifier | Ratio | Climate-driven modifier on food/resource production | 0.0–2.0 |
| climate_stress_index | Ratio | Long-run climate deviation from baseline | 0.0–1.0 |

**War and Diplomacy Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| conflict_active | Boolean | Whether armed conflict is currently occurring | bool |
| conflict_intensity | Ordinal | (NONE / SKIRMISH / CONFLICT / WAR / TOTAL_WAR) | enum |
| military_expenditure_ratio | Ratio | Fraction of Joule budget allocated to military | 0.0–1.0 |
| diplomatic_relations | Matrix | Pairwise relation score between simulated entities | -1.0–1.0 |
| territory_control | Map | Territory ownership by entity per tick | spatial |

#### Visualization Types

| Visualization | Metrics | Notes |
|---------------|---------|-------|
| Time-series line chart | All scalar/ratio metrics | Multi-metric overlay; log scale option |
| Stacked area chart | joule_production, joule_consumption, joule_waste, joule_surplus | Energy flow decomposition |
| Heat map (spatial) | territory_control, per-region resource access | Requires spatial grid model |
| Gauge (current tick) | institutional_legitimacy, tyranny_index, insurgency_rate, joule_scarcity_index | At-a-glance current state |
| Ordinal state badge | election_cycle_health, insurgency_severity, conflict_intensity | Color-coded: green/yellow/red/black |
| Scatter / correlation | Any two metrics over time | Useful for coupling analysis |
| Bar chart (distribution) | gini_coefficient, per_capita_joule_access by faction | Inequality visualization |
| Event log timeline | All events | Tick-indexed, filterable by type |

### 5.6 Policy Intervention Controls

#### Description

The policy intervention interface allows users to apply changes to a running simulation's governance parameters, resource allocations, or institutional structures — either as experimental interventions or as modeled policy actions.

Every intervention is a first-class simulation event: it is logged, causally attributable, and affects the BLAKE3 state hash. Interventions can be applied in interactive mode (via GUI) or in headless mode (via API or mod).

#### Intervention Types

| Intervention Type | Description | Authority Required | Branch |
|-------------------|-------------|-------------------|--------|
| Energy reallocation | Shift Joule distribution from one sector to another | Economic authority | Optional |
| Tax rate change | Modify the extraction rate from citizen surplus | Fiscal authority | Optional |
| Governance reform | Change institutional structure (election cycle, enforcement power, etc.) | Constitutional authority | Optional |
| Climate intervention | Apply geoengineering event (solar dimming, precipitation seeding) | N/A (researcher override) | Optional |
| Military mobilization | Allocate additional Joule budget to military; adjust conflict stance | Military authority | Optional |
| Diplomatic action | Send treaty proposal, impose sanctions, form alliance | Diplomatic authority | Optional |
| Emergency decree | Override normal governance channels for immediate action | Emergency authority | Recommended |
| Mod-defined intervention | Custom intervention type defined by a WASM Policy mod | Mod authority | Depends on mod |

#### Authority Model

The simulation models institutional authority: not all interventions are available at all times. Constitutional authority may be required for governance reforms; if that authority has been captured or delegated, the intervention may be blocked, delayed, or have unintended consequences.

In **researcher override mode** (enabled via API or CLI flag), all authority checks are bypassed. This is the default for headless research runs. In **realistic mode** (default for designed scenarios), authority checks are enforced by the institutions subsystem.

#### Branch Creation on Intervention

When applying an intervention in interactive mode, the user can optionally create a branch: the current state is snapshotted, the intervention is applied to a new runner instance, and the original instance continues unchanged. This allows comparison of "with intervention" vs. "without intervention" trajectories.

### 5.7 CLI and API Surface (Research Operator)

#### CLI Commands

```bash
# Run a single scenario
civlab run --scenario scenario.toml --seed 42 --ticks 10000 --output results/

# Run a parameter sweep
civlab sweep --params sweep.toml --output results/ --workers 8

# Replay a simulation
civlab replay --artifact results/run_42/ --from-tick 5000 --to-tick 6000

# Verify replay determinism
civlab verify --artifact results/run_42/ --hashes results/run_42/hashes.bin

# Branch a simulation
civlab branch --artifact results/run_42/ --at-tick 5000 \
  --intervention intervention_a.toml --output results/branch_a/

# Export metrics as CSV
civlab export --artifact results/run_42/ --metrics joule_stock,legitimacy --format csv

# Validate a scenario definition
civlab validate --scenario scenario.toml

# List available mods
civlab mods list

# Install a mod from registry
civlab mods install policy/universal-energy-access@0.3.2
```

#### HTTP API (for Parpour/Venture and programmatic access)

```
POST   /v1/scenarios                Create and dispatch a scenario
GET    /v1/scenarios/{id}           Get scenario status and metadata
GET    /v1/scenarios/{id}/results   Get structured result payload
POST   /v1/scenarios/{id}/branch    Branch at a specific tick
DELETE /v1/scenarios/{id}           Abort and clean up a scenario

GET    /v1/replays/{id}/ticks/{n}   Get full state at tick N
GET    /v1/replays/{id}/events      Get event log
GET    /v1/replays/{id}/causal/{n}  Get causal trace for event N
GET    /v1/replays/{id}/metrics     Get metric time series

POST   /v1/sweeps                   Submit a parameter sweep
GET    /v1/sweeps/{id}              Get sweep status
GET    /v1/sweeps/{id}/results      Get sweep result collection

GET    /v1/mods                     List available mods
POST   /v1/mods/{id}/install        Install a mod to a scenario
```

#### Result Payload Schema (JSON)

```json
{
  "scenario_id": "uuid",
  "seed": 42,
  "ticks_completed": 10000,
  "abort_reason": null,
  "final_state_hash": "blake3:...",
  "metrics": {
    "final": {
      "joule_stock": 1234567,
      "institutional_legitimacy": 0.72,
      "tyranny_index": 0.21,
      "insurgency_rate": 0.04,
      "population": 1500000,
      "conflict_active": false
    },
    "time_series": {
      "joule_stock": [/* array of i64 per tick */],
      "institutional_legitimacy": [/* array of f64 per tick */]
    }
  },
  "events": [
    {
      "tick": 4231,
      "type": "LEGITIMACY_CRISIS",
      "severity": "HIGH",
      "causal_chain": ["energy_scarcity_index > 0.6", "institutional_legitimacy < 0.4"]
    }
  ],
  "civlab_version": "0.2.0",
  "schema_version": "1.0.0"
}
```

### 5.8 Modding Platform

#### Overview

The CivLab modding platform allows developers to extend the simulation with custom behaviors without modifying the core engine. All mods run in a WASM sandbox with a capability-limited API surface. Mods are distributed through the CivLab registry (hosted) or as standalone WASM packages.

#### Four Mod Types

| Mod Type | Description | API Access | Example |
|----------|-------------|------------|---------|
| **Policy mod** | Defines a governance policy with effects on institutional and economic state | Economy, Institutions | Universal Basic Energy: every citizen receives minimum kJ/tick regardless of market |
| **Economic mod** | Adds or modifies production, distribution, or consumption rules | Economy | Renewable transition: replaces fossil Joule sources with renewable at defined rate |
| **Event mod** | Defines triggered events with conditions and state effects | All domains (read), Economy + Social (write) | Pandemic: reduces population and productivity under defined conditions |
| **Scenario mod** | Defines a full scenario template including initial state, constitution, and event schedule | Scenario authoring | Historical analog: pre-configured scenario approximating a historical civilization |

#### WASM Sandbox Constraints

- **Memory limit:** 64MB per mod instance
- **CPU budget:** max 10ms per tick contribution (enforced by host)
- **Capability model:** Mods declare required capabilities at load time; host validates and grants; no capability escalation at runtime
- **No I/O:** Mods cannot access filesystem, network, or system time
- **Determinism required:** Mods must be deterministic; any non-determinism is a validation failure at load time

#### civlab-sdk

The civlab-sdk is a Rust crate (and future bindings for other languages) that provides:

- Type definitions for all mod API types
- Macro helpers for mod registration
- Test harness for mod determinism and capability validation
- Documentation generator
- Local mod registry for development

```toml
# Cargo.toml for a Policy mod
[dependencies]
civlab-sdk = "0.1"

[lib]
crate-type = ["cdylib"]
```

#### Registry and Distribution

- **Registry URL:** registry.civlab.io
- **Mod manifest:** name, version, type, required capabilities, BLAKE3 hash, author
- **Installation:** `civlab mods install \<type\>/\<name\>@\<version\>`
- **Revenue share:** Paid mods in marketplace split revenue 70/30 (author/platform)

---

## 6. Simulation Domain Coverage

### 6.1 Domain Overview

CivLab models six coupled simulation domains. Each domain is a discrete subsystem with its own state, update logic, and event emission. Domains interact through a well-defined coupling interface — no domain directly mutates another domain's state. Cross-domain effects are mediated through the coupling layer.

```
+------------------+         +------------------+
|  Joule Economy   |<------->|  Climate System  |
+------------------+         +------------------+
        |  ^                         |  ^
        v  |                         v  |
+------------------+         +------------------+
|  Institutions    |<------->| Citizens/Demog.  |
+------------------+         +------------------+
        |  ^                         |  ^
        v  |                         v  |
+------------------+         +------------------+
| Social/Insurgency|<------->| War/Diplomacy    |
+------------------+         +------------------+
```

### 6.2 Joule Economy Subsystem

#### Purpose

The Joule Economy is the master constraint system. In CivLab, energy (measured in KiloJoules, represented as i64) is the primary resource. All production, distribution, consumption, and waste is denominated in kJ. Societies that cannot sustain energy production face cascading failures across all other domains.

#### Key Concepts

- **KiloJoule (kJ):** i64 newtype; the atomic unit of all economic transactions in CivLab.
- **Production:** Conversion of raw resources (modeled implicitly as capital stocks) into kJ per tick.
- **Distribution:** Movement of kJ from production nodes to consumption nodes; subject to infrastructure efficiency.
- **Consumption:** kJ consumed by citizens, institutions, military, and infrastructure per tick.
- **Waste:** kJ produced but neither consumed nor stored; represents inefficiency, corruption, or capacity limits.
- **Surplus:** kJ produced in excess of consumption and waste; accumulates in Joule stock.
- **Joule stock:** Reserve of kJ available for future ticks; represents civilizational energy savings.

#### Production → Distribution → Consumption → Waste Cycle

```
Resources → [Production] → kJ produced
                              |
                        [Distribution]
                        (efficiency %)
                         /           \
                   [Consumed]      [Lost/Waste]
                       |
                [Citizen needs]
                [Institutional ops]
                [Military budget]
                [Infrastructure]
```

#### Coupling Out (effects on other domains)

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Energy scarcity → legitimacy loss | Institutions | `scarcity_index > threshold` triggers legitimacy decay |
| Energy scarcity → insurgency pressure | Social/Insurgency | Unfed citizens join insurgent pool |
| Military Joule allocation | War/Diplomacy | Military capacity is a function of allocated kJ/tick |
| Infrastructure investment | Climate | Resilience to climate shocks requires Joule investment |
| Surplus → demographic growth | Citizens/Demography | Surplus energy supports population growth |

#### FR Coverage

- CIV-0100: Joule Economy MVP
- CIV-0101: Production chain extensions (planned Phase 2)

### 6.3 Climate System

#### Purpose

The climate system models long-horizon environmental dynamics: temperature drift, precipitation variability, and discrete climate shock events (droughts, floods, extreme heat). Climate interacts with agricultural production (affecting resource availability), infrastructure resilience (affecting distribution efficiency), and displacement (affecting demography).

#### Key Concepts

- **Climate baseline:** Initial temperature and precipitation parameters set in scenario definition.
- **Climate drift:** Slow monotonic or oscillatory change in mean climate over simulation ticks.
- **Climate shock:** Discrete, high-impact event (drought, flood, storm) with defined duration and severity.
- **Agricultural yield modifier:** Climate-driven multiplier on food and biomass production.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Drought → production loss | Joule Economy | Agricultural yield modifier reduces production kJ/tick |
| Flood → infrastructure damage | Joule Economy | Distribution efficiency decreases after flood event |
| Climate shock → displacement | Citizens/Demography | Displaced persons per tick increases during/after shock |
| Persistent climate stress → legitimacy pressure | Institutions | Long-run climate stress increases institutional demand for response |

#### FR Coverage

- CIV-0102: Climate System MVP

### 6.4 Institutions Subsystem

#### Purpose

The institutions subsystem models the governance layer: the structures through which a society makes collective decisions, enforces rules, and distributes authority. This includes executive, legislative, judicial, and electoral institutions.

#### Key Concepts

- **Constitution:** The foundational set of institutional rules (governance type, election cycle, rights, enforcement powers).
- **Legitimacy:** The population's acceptance of institutional authority. Legitimacy decays under scarcity, injustice, and ineffectiveness; it regenerates through effective policy and procedural justice.
- **Elite capture:** The degree to which institutional decisions serve narrow elite interests rather than the general population.
- **Enforcement power:** The institutional capacity to enforce rules; too low → rule breakdown; too high → tyranny.

#### Governance Types (MVP)

| Type | Description | Default Legitimacy | Default Tyranny |
|------|-------------|-------------------|-----------------|
| DEMOCRATIC | Elected government with institutional checks | 0.7 | 0.1 |
| AUTHORITARIAN | Centralized control; high enforcement, low checks | 0.4 | 0.6 |
| OLIGARCHIC | Small elite with nominal democratic structures | 0.5 | 0.4 |
| TECHNOCRATIC | Expert-managed with low popular accountability | 0.5 | 0.3 |
| ANARCHY | Absent central authority; emergent local governance | 0.3 | 0.0 |

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Policy effectiveness → Joule distribution | Joule Economy | Effective institutions improve distribution efficiency |
| Legitimacy collapse → insurgency threshold | Social/Insurgency | Legitimacy \< 0.3 triggers insurgency escalation risk |
| Elite capture → resource extraction | Joule Economy | Captured institutions extract surplus to elite, not public investment |
| Institutional collapse → diplomatic vulnerability | War/Diplomacy | Weak institutions invite external aggression |

#### FR Coverage

- CIV-0103: Institutions MVP

### 6.5 Citizens and Demography Subsystem

#### Purpose

The citizens/demography subsystem models the human population: its size, age structure, skill distribution, faction composition, and material conditions. Citizens are the primary source of legitimacy (or its withdrawal), the labor for production, and the participants in insurgency and war.

#### Key Concepts

- **Population:** Total citizen count (integer; cannot be negative).
- **Age structure:** Distribution of citizens across age cohorts; affects labor capacity, dependency ratio, and military conscription pool.
- **Factions:** Named sub-groups with shared interests, grievances, and loyalty profiles. Factions are the primary actors in social and political dynamics.
- **Grievance:** Per-faction accumulation of unmet needs (energy, security, political representation). High grievance increases insurgency risk.
- **Material conditions:** Per-citizen kJ access, housing, and security. Derived from Joule Economy outputs.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Population size → energy demand | Joule Economy | Consumption kJ/tick scales with population |
| Labor force → production capacity | Joule Economy | Production kJ/tick is a function of skilled labor supply |
| Faction grievance → insurgency pool | Social/Insurgency | High-grievance factions contribute to insurgent recruitment |
| Population → conscription pool | War/Diplomacy | Military manpower is a function of eligible population |
| Demographic pressure → institutional demand | Institutions | Population growth increases governance complexity and institutional load |

#### FR Coverage

- CIV-0104: Citizens/Demography MVP

### 6.6 Social and Insurgency Subsystem

#### Purpose

The social/insurgency subsystem models the transition from latent social tension to organized insurgency, and the dynamics of insurgency once active. It captures the legitimacy → grievance → insurgency escalation pathway and the feedback between insurgency, institutional response, and further legitimacy change.

#### Escalation Model

```
LATENT_TENSION → ACTIVE_DISCONTENT → INSURGENCY → CIVIL_WAR → STATE_COLLAPSE
      |                  |                |              |
  (grievance         (faction         (organized      (state
   accumulates)      organizing)       violence)      failure)
```

#### Key Dynamics

- **Grievance accumulation:** Energy scarcity, inequality, and legitimacy loss accumulate faction grievance per tick.
- **Insurgent recruitment:** When grievance exceeds faction threshold, citizens move from general population to insurgent pool.
- **Insurgent capacity:** Insurgent kJ access (from shadow economy or capture) determines conflict effectiveness.
- **Counterinsurgency:** Institutional enforcement actions reduce insurgent pool but risk legitimacy loss if applied excessively (tyranny feedback).
- **Negotiation:** Institutions can trade legitimacy concessions for insurgency de-escalation.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Insurgency → production disruption | Joule Economy | Active insurgency reduces infrastructure efficiency |
| Civil war → institutional stress | Institutions | Civil war damages institutional capacity |
| Insurgency → displacement | Citizens/Demography | Conflict drives population displacement |
| Insurgency → diplomatic signal | War/Diplomacy | Internal instability invites external opportunism |

#### FR Coverage

- CIV-0106: Social/Insurgency MVP

### 6.7 War and Diplomacy Subsystem

#### Purpose

The war/diplomacy subsystem models inter-entity relations: alliances, trade agreements, military posture, and armed conflict. In multi-entity scenarios, diplomacy determines the external environment within which each entity's internal dynamics unfold.

#### Key Concepts

- **Entity:** The primary actor in the simulation (state, faction, city-state, etc.). Multiple entities can exist in one simulation.
- **Diplomatic relation:** A pairwise score (-1.0 = total war, +1.0 = full alliance) between entities.
- **Military capacity:** A function of allocated Joule budget + conscripted population + institutional effectiveness.
- **Conflict escalation:** Diplomacy scores below threshold → skirmish → conflict → war → total war.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| War → Joule drain | Joule Economy | Military operations consume kJ/tick at high rate |
| War → production disruption | Joule Economy | Combat damages production infrastructure |
| War → casualties | Citizens/Demography | Combat deaths reduce population |
| War → legitimacy pressure | Institutions | Prolonged war strains institutional legitimacy |
| Peace dividend | Joule Economy | Demobilization frees military kJ budget for civilian use |

#### FR Coverage

- CIV-0105: War/Diplomacy MVP

### 6.8 Cross-Domain Coupling Map

The following table summarizes all first-order coupling effects between domains. "→" means "affects."

| From Domain | To Domain | Effect Summary |
|-------------|-----------|----------------|
| Joule Economy | Institutions | Scarcity → legitimacy decay |
| Joule Economy | Social/Insurgency | Scarcity → grievance accumulation |
| Joule Economy | Citizens/Demography | Surplus → growth; scarcity → mortality |
| Joule Economy | War/Diplomacy | Military budget is kJ allocation |
| Climate | Joule Economy | Shocks → production/distribution loss |
| Climate | Citizens/Demography | Shocks → displacement |
| Climate | Institutions | Stress → governance demand |
| Institutions | Joule Economy | Policy effectiveness → distribution efficiency |
| Institutions | Social/Insurgency | Legitimacy floor → insurgency threshold |
| Institutions | War/Diplomacy | Weakness → diplomatic vulnerability |
| Citizens/Demography | Joule Economy | Population → demand; labor → supply |
| Citizens/Demography | Social/Insurgency | Faction grievance → insurgent pool |
| Citizens/Demography | War/Diplomacy | Population → conscription pool |
| Social/Insurgency | Joule Economy | Active insurgency → infrastructure disruption |
| Social/Insurgency | Institutions | Civil war → institutional capacity damage |
| Social/Insurgency | Citizens/Demography | Conflict → displacement |
| Social/Insurgency | War/Diplomacy | Internal instability → external opportunism |
| War/Diplomacy | Joule Economy | War → kJ drain + production disruption |
| War/Diplomacy | Citizens/Demography | Casualties → population loss |
| War/Diplomacy | Institutions | Prolonged war → legitimacy stress |

---

## 7. Technical Product Constraints

### 7.1 Determinism as Non-Negotiable

Determinism is the foundational technical constraint. It is not a feature to be traded off against performance or expressiveness. The D1-D7 ruleset defines the complete contract:

| Rule ID | Rule | Rationale |
|---------|------|-----------|
| D1 | All randomness flows through ChaCha20Rng seeded from the scenario seed (u64). | Controlled randomness = reproducibility |
| D2 | No wall-clock time access in simulation code. | System time is non-deterministic |
| D3 | No floating-point comparison with `==` or `!=`. | Float comparison is platform-dependent |
| D4 | No HashMap iteration order dependencies. | HashMap iteration is unordered in Rust |
| D5 | No thread-local state in simulation code. | Thread-local state is execution-context-dependent |
| D6 | All domain update functions are pure: output depends only on input + RNG state. | Side effects break reproducibility |
| D7 | BLAKE3 state hash computed and stored after every tick. | Enables external verification of replay fidelity |

**Enforcement:** D1-D7 violations are caught by:
- Static analysis (custom Clippy lints for D2, D3, D4, D5)
- CI replay test: every PR runs the reference scenario twice with the same seed and asserts byte-identical output
- BLAKE3 hash comparison: hashes from two replay runs must match at every tick

**Consequence of violation:** A determinism violation is a P0 bug. Any release that ships a known determinism violation is blocked.

### 7.2 Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Tick latency (p99, single simulation) | &lt; 100ms | Benchmark suite (criterion) on reference hardware |
| Headless throughput | &gt; 600 ticks/min/instance | Benchmark suite |
| Batch sweep throughput | &gt; 10,000 tick-scenarios/hour | Integration benchmark |
| State hash computation | &lt; 5ms per tick | Profiled separately |
| Scenario load time | &lt; 100ms | End-to-end benchmark |
| Replay seek (to any tick) | &lt; 200ms | Replay benchmark |
| Memory per instance | &lt; 256MB | Memory profiler (Valgrind / heaptrack) |

**Reference hardware:** 8-core x86-64 Linux machine, 32GB RAM, NVMe SSD.

### 7.3 Portability

| Target | Status | Notes |
|--------|--------|-------|
| Linux x86-64 | Primary | CI + release target |
| macOS ARM64 (Apple Silicon) | Supported | CI target; developer primary |
| macOS x86-64 | Supported | CI target |
| Windows x86-64 | Supported | CI target; release binary |
| WASM32 (browser) | Supported | Web RTS client target; subset of API |
| WASM32-WASI | Planned | Server-side WASM deployment |
| Linux ARM64 | Planned | Cloud and embedded deployment |

**WASM constraints:** The WASM build excludes multi-threading (SharedArrayBuffer limitations in some environments) and filesystem access. Batch sweep and replay seek use in-memory representations in WASM builds.

### 7.4 Open-Source License

CivLab core engine is dual-licensed under **MIT / Apache-2.0** (user's choice). This is the standard Rust ecosystem dual license.

- **MIT:** Maximum permissiveness; compatible with commercial use.
- **Apache-2.0:** Adds patent protection; preferred by some enterprises.

**What is NOT open-source:**
- Cloud simulation credit platform (closed SaaS)
- Enterprise private cloud deployment tooling
- Parpour/Venture integration adapter (separate commercial license)
- SDXL asset generation pipeline (separate tooling)

### 7.5 Accessibility Requirements

| Requirement | Target | Notes |
|-------------|--------|-------|
| Web RTS UI color contrast | WCAG 2.1 AA | All metric gauges and state badges meet contrast ratio &gt; 4.5:1 |
| Keyboard navigation | Full keyboard coverage | All scenario authoring and replay inspector actions accessible via keyboard |
| Screen reader compatibility | ARIA labels on all interactive elements | Metrics dashboard uses chart.js with aria-label per data point |
| Colorblind mode | Deuteranopia + protanopia palettes | Metric charts offer colorblind-safe palette option |
| Font size | Min 14px body text in Web RTS | Scalable with browser zoom up to 200% without layout break |

---

## 8. Competitive Analysis

### 8.1 Comparison Matrix

| Dimension | CivLab | Dwarf Fortress | Victoria 3 | Factorio | OpenTTD | NetLogo/Mesa |
|-----------|--------|----------------|------------|----------|---------|--------------|
| Open source | Yes (MIT/Apache) | Partial (classic free, premium paid) | No | No | Yes (GPL) | Yes |
| Headless / embeddable | Yes | No | No | No | Partial | Yes |
| Deterministic replay | Yes (D1-D7) | No | No | Yes (partial) | No | Manual |
| Programmatic API | Yes (HTTP + CLI) | No | No | Mod API only | Partial | Script only |
| Energy economy | Joule Economy | Yes (deep) | Yes (trade) | Yes (deep) | No | No built-in |
| Climate system | Yes | Yes | Yes | No | No | No built-in |
| Governance/institutions | Yes | Partial | Yes (deep) | No | No | No built-in |
| Demography/citizens | Yes | Yes (deep) | Yes | No | No | No built-in |
| Social/insurgency | Yes | Partial | Yes (partial) | No | No | No built-in |
| War/diplomacy | Yes | Yes | Yes (deep) | No | No | No built-in |
| Mod platform | WASM + SDK | Custom (DFHack) | Paradox mod | Lua API | Squirrel API | Custom |
| Batch sweep / research | Yes | No | No | No | No | Yes |
| Causal trace / explainability | Yes | No | No | No | No | Manual |
| Performance SLO | 100ms/tick | Varies | Varies | High | High | Varies |
| AI agent integration | First-class | None | None | None | None | Manual |

### 8.2 Dwarf Fortress

**Strengths:**
- Unmatched simulation depth: fluid dynamics, geology, individual citizen psychology, artifact histories
- Decade-long community of dedicated players and modders
- No other simulation comes close in emergent narrative richness

**Weaknesses:**
- Not open-source in full (classic version is free but not open)
- Not embeddable: desktop application with complex rendering dependency
- Non-deterministic: no seed-to-output guarantee; replays are not guaranteed identical
- No programmatic API: cannot be controlled by external agents or scripts
- Steep learning curve; notoriously inaccessible to new users
- No causal trace: understanding WHY a fortress failed requires extensive manual inspection

**CivLab differentiation:** CivLab is not trying to match Dwarf Fortress's depth in any single subsystem. CivLab prioritizes breadth (all six civilizational domains), determinism, and embeddability. The target user for CivLab is a researcher or designer who needs a reproducible substrate, not a player seeking emergent narrative.

### 8.3 Victoria 3

**Strengths:**
- Sophisticated political economy: trade routes, pops, interest groups, laws
- Deep diplomatic and great-power dynamics
- High production value; accessible GUI

**Weaknesses:**
- Closed source; non-embeddable; no external API
- Non-deterministic; replays diverge
- Moddable but within Paradox's proprietary system; no WASM sandbox
- Political and trade model is rich but energy/Joule economy is absent
- Cannot be used as a research or AI backend

**CivLab differentiation:** CivLab provides a comparable political economy model (governance, legitimacy, elite capture) but adds the Joule Economy as the master constraint (Victoria 3 has no energy system), full determinism, and API-first headless operation.

### 8.4 Factorio

**Strengths:**
- Exceptionally polished production chain simulation
- High performance; scales to massive factory networks
- Excellent modding ecosystem (Lua API)
- Partial determinism in multiplayer

**Weaknesses:**
- No governance, institutions, or legitimacy
- No demography or social dynamics
- No climate system
- No diplomacy or armed conflict between civilizations
- Not a civilization simulator; it's a factory/logistics game

**CivLab differentiation:** Factorio's production chain model is an inspiration for CivLab's Joule Economy subsystem (production → distribution → consumption pipeline), but Factorio has zero coverage of the governance, social, climate, and conflict domains that are CivLab's core value.

### 8.5 OpenTTD

**Strengths:**
- Fully open-source (GPL); highly moddable
- Active community; long-term maintained
- Transport/logistics model is well-designed

**Weaknesses:**
- Transport and logistics only; no political/social/governance model
- No energy economy beyond transport fuel
- No demography, climate, or social dynamics
- No API for external agent control

**CivLab differentiation:** OpenTTD demonstrates that an open-source, community-driven simulation can build a large and loyal user base. CivLab aims to be OpenTTD for civilizational dynamics: an open, extensible substrate that the community can build on.

### 8.6 Summary: CivLab Differentiation

CivLab's unique position in the market is the combination of:

1. **Deterministic, byte-for-byte reproducible simulation** — no other tool in this space offers a formal D1-D7 determinism guarantee.
2. **All six civilizational domains, coupled** — no other open-source tool models energy + climate + governance + demography + social + war in a single coupled simulation.
3. **Headless, embeddable, API-first** — designed from day one for programmatic control by external agents (AI or human).
4. **Open-source core with commercial cloud layer** — community builds on the MIT/Apache-2.0 core; commercial value captured in cloud credits and enterprise deployment.

---

## 9. Business Model

### 9.1 Revenue Streams

| Stream | Type | Description | Target Customer |
|--------|------|-------------|-----------------|
| Open-source core | No revenue (cost center) | MIT/Apache-2.0 engine; community development | All users |
| Cloud simulation credits | Usage-based SaaS | Pay-per-tick on managed cloud infrastructure | Research operators, Parpour/Venture |
| Enterprise license | Annual SaaS + support | Private cloud deployment, SLA, dedicated infra, priority support | Enterprise research, government |
| Modding marketplace | Revenue share (30%) | Platform fee on paid mods sold through civlab registry | Mod developers |
| Parpour/Venture integration | API licensing + revenue share | Commercial API contract for Venture AI agent integration | Parpour |
| Training and consulting | Professional services | Scenario design, research methodology, custom mod development | Research institutions |

### 9.2 Pricing Model: Cloud Simulation Credits

Cloud simulation credits are the primary commercial revenue mechanism. Pricing is based on tick-compute units:

| Unit | Definition | Price (indicative) |
|------|------------|-------------------|
| 1 tick-compute unit (TCU) | 1 simulation tick on reference compute (8-core, 32GB RAM) | $0.0001 |
| Scenario bundle | 10,000 TCU | $1.00 |
| Research pack | 1,000,000 TCU + priority queue | $80/month |
| Enterprise tier | Unlimited TCU + private infra + SLA | Custom contract |

**Free tier:** 10,000 TCU/month free for registered users. Sufficient for exploration and small research runs.

### 9.3 Cost Model

| Cost Component | Estimate | Notes |
|----------------|----------|-------|
| Compute per tick | ~$0.00003 | On cloud VM; 100ms/tick target |
| Storage per scenario (10,000 ticks) | ~$0.01 | Replay artifact + metric time series |
| Storage per state snapshot | ~$0.001 | Compressed MessagePack full state |
| Gross margin target | ~70% | At scale; cloud infra costs dominate at small scale |

### 9.4 Open-Source Strategy

The OSS core is a strategic asset, not a cost. It:
- Drives community adoption and contribution
- Creates ecosystem (mods, scenarios, integrations) that increases switching cost
- Establishes CivLab as the credible, auditable substrate for research (reproducibility requires open source)
- Enables academic citations and research partnerships
- Reduces marketing cost: organic growth through GitHub, academic papers, and community

**OSS governance:** The core engine is maintained by the CivLab team with community contributions accepted via RFC process (see Section 14). The cloud platform, enterprise tooling, and Parpour integration are proprietary.

---

## 10. Success Metrics and KPIs

### 10.1 North Star Metric

**Scenarios executed per day** across all deployment modes, with 100% deterministic replay consistency.

- **Volume component:** Total scenarios dispatched (local + cloud + Parpour/Venture)
- **Quality gate:** Zero determinism regressions; any regression resets the quality gate

### 10.2 Product Health KPIs

| KPI | Definition | Target (12mo) | Target (24mo) | Measurement |
|-----|------------|---------------|---------------|-------------|
| Deterministic replay consistency | % of replay runs producing byte-identical output to original | 100% | 100% | CI test; automated nightly replay |
| Tick latency p99 | 99th percentile tick latency (ms) | &lt; 100ms | &lt; 80ms | Criterion benchmark |
| Scenarios executed/day | Total scenarios dispatched across all modes | 1,000/day | 10,000/day | Platform telemetry |
| Explainability score | % of instability events with a structured causal trace | 80% | 95% | Test coverage of causal trace API |
| Domain coverage | Fraction of planned domains at MVP coverage (6 total) | 6/6 | 6/6 (+ depth) | FR tracker |
| API adoption | % of scenarios dispatched via headless API vs. GUI | 60% API | 75% API | Platform telemetry |

### 10.3 Ecosystem KPIs

| KPI | Definition | Target (12mo) | Target (24mo) |
|-----|------------|---------------|---------------|
| Community scenarios in registry | Number of community-contributed scenario artifacts | 25 | 200 |
| Published mods | Number of mods in civlab registry | 10 | 100 |
| GitHub stars | civlab-core repository stars | 1,000 | 5,000 |
| Academic citations | Published papers citing CivLab | 3 | 20 |
| Monthly active researchers | Unique users running batch sweeps | 50 | 500 |

### 10.4 Developer Experience KPIs

| KPI | Definition | Target |
|-----|------------|--------|
| Time-to-first-run | Time from `cargo install civlab` to first scenario execution | &lt; 5 minutes |
| Time-to-first-sweep | Time from first run to first batch sweep | &lt; 15 minutes |
| Time-to-first-mod | Time from civlab-sdk installation to first working WASM mod | &lt; 60 minutes |
| Scenario validation error clarity | % of users who self-resolve validation errors without docs | 80% |
| CI build time | Total CI time per PR | &lt; 10 minutes |

### 10.5 Parpour/Venture Integration KPIs

| KPI | Definition | Target |
|-----|------------|--------|
| Venture API uptime | % uptime of CivLab API serving Venture agents | 99.9% |
| Venture scenario throughput | Scenarios/hour dispatched by Venture agents | &gt; 500/hour |
| Venture result latency | p99 latency from scenario dispatch to result retrieval | &lt; 30 seconds |
| Venture determinism rate | % of Venture-dispatched scenarios with verified deterministic replay | 100% |

---

## 11. Risks and Mitigations

### 11.1 Risk Register

| Risk ID | Risk | Probability | Impact | Severity |
|---------|------|-------------|--------|----------|
| R-01 | Determinism erosion: a subsystem or mod introduces non-determinism silently | Medium | Critical | Critical |
| R-02 | Performance regression: tick latency exceeds 100ms target after adding new domains | Medium | High | High |
| R-03 | Complexity ceiling: scenario authoring becomes too complex for non-experts | High | High | High |
| R-04 | Competition: well-funded commercial simulation tool copies CivLab's positioning | Low | High | Medium |
| R-05 | WASM mod security: malicious mod escapes sandbox | Low | Critical | High |
| R-06 | Ecosystem fragmentation: community forks produce incompatible scenario formats | Medium | Medium | Medium |
| R-07 | Parpour/Venture dependency: CivLab becomes too tightly coupled to Venture's specific needs | Medium | Medium | Medium |
| R-08 | Adoption plateau: open-source community does not grow beyond early adopters | Medium | Medium | Medium |

### 11.2 Risk Mitigations

#### R-01: Determinism Erosion

**Mitigation strategy:**
- D1-D7 ruleset is the authoritative contract; violations are P0 bugs.
- Custom Clippy lints enforce D2 (no system time), D3 (no float `==`), D4 (no HashMap iteration order).
- CI replay test runs on every PR: same seed + scenario → must produce byte-identical output. Hard gate; PR blocked if replay diverges.
- BLAKE3 hash per tick enables external verification; any hash mismatch is a determinism breach.
- Mod validation: WASM mods are tested for determinism at installation time using the civlab-sdk test harness. Non-deterministic mods are rejected.
- Scheduled fuzz testing: nightly CI run replays random scenarios with random seeds; any divergence triggers alert.

#### R-02: Performance Regression

**Mitigation strategy:**
- Criterion benchmark suite runs on every PR; performance regressions &gt; 5% relative to baseline trigger a review gate (not hard block, but requires explicit sign-off).
- Tick budget allocation document (Section 5.3) defines per-domain budget. Any domain exceeding its budget triggers a profiling requirement.
- Performance benchmarks are tracked in a time-series dashboard; trends are reviewed weekly.
- Domain implementations use SIMD and cache-friendly data layouts where applicable.
- bevy_ecs is being evaluated for potential ECS-based parallelization of independent domain updates.

#### R-03: Complexity Ceiling

**Mitigation strategy:**
- Scenario authoring schema is designed with progressive disclosure: minimal required fields; optional fields with sensible defaults.
- Schema validation provides clear, path-specific error messages. "Expected kJ value in range [0, i64::MAX] for field `energy.joule_stock`, got: -1000" — not "invalid config".
- Template library: pre-built scenario templates cover common starting conditions (medieval agrarian, industrial transition, post-scarcity, resource-constrained).
- Web authoring UI (Phase 4) provides guided workflow for common scenario patterns.
- Researcher documentation: quick-start guide targets &lt; 30-minute time-to-first-sweep for a policy analyst with no prior CivLab experience.
- Community scenario registry provides example scenarios that users can inspect, fork, and modify.

#### R-04: Competition Risk

**Mitigation strategy:**
- CivLab's moat is ecosystem, not technology alone. A competitor can build a similar engine, but cannot replicate 5 years of community scenarios, mods, and academic citations.
- Open-source licensing makes CivLab the default reference implementation. Even if a commercial tool exists, researchers require open-source for reproducibility.
- Parpour/Venture integration provides a durable commercial relationship that a new entrant cannot easily replicate.
- First-mover advantage in the AI agent simulation backend space is significant; the Parpour integration establishes CivLab as the reference implementation before competitors exist.

#### R-05: WASM Mod Security

**Mitigation strategy:**
- WASM sandbox runs in a separate process from the simulation engine. Process isolation contains any sandbox escape.
- Capability model is deny-by-default: mods must declare required capabilities at load time; undeclared capability access is a hard error.
- Memory limit (64MB) and CPU budget (10ms/tick) enforced by host; violations terminate the mod instance.
- Mod registry requires BLAKE3 hash verification of all installed mods; tampered mods rejected.
- Security audit of WASM sandbox implementation before civlab-sdk v1.0 release.

#### R-06: Ecosystem Fragmentation

**Mitigation strategy:**
- Scenario format is versioned (semver); schema migrations are provided by the civlab-core library.
- BLAKE3 hash of scenario artifact ties scenario to specific civlab-core version; incompatible versions are flagged at load time.
- RFC process (Section 14) requires community review for any breaking change to the scenario format.
- Scenario registry enforces format version; incompatible submissions are rejected with clear migration instructions.

#### R-07: Parpour/Venture Coupling Risk

**Mitigation strategy:**
- CivLab API is designed as a general-purpose simulation API, not a Venture-specific API. Venture is a first-class integration, not the only integration.
- Venture-specific API extensions (if needed) are implemented as a separate adapter layer, not in civlab-core.
- API versioning and changelog policy (Section 5.7) ensures backward compatibility for Venture's integration.
- Architecture review required for any CivLab change requested exclusively by Venture without general applicability.

#### R-08: Adoption Plateau

**Mitigation strategy:**
- Academic publishing: prioritize features that enable reproducible research (batch sweep, scenario export, causal trace). Academic citations are organic growth.
- Scenario design contest: annual community contest for most interesting/surprising scenario outcomes.
- Direct outreach to policy research institutions and complexity science departments.
- Parpour/Venture integration creates a showcase use case (AI-backed policy analysis) that generates press and community interest.

---

## 12. Roadmap and Milestones

### 12.1 Phase Table

| Phase | Timeline | Theme | Key Features | FR IDs | Success Criteria | Est. Complexity |
|-------|----------|-------|-------------|--------|-----------------|-----------------|
| Phase 0 | M0–M2 | Core tick loop | Rust crate, ChaCha20Rng seeding, BLAKE3 hash per tick, D1-D7 harness, CI replay gate, basic scenario TOML loader | CIV-0001–0010 | Replay test passes; tick loop runs at &gt; 10 ticks/ms; zero determinism violations in fuzz test | Medium |
| Phase 1 | M2–M5 | Economy + Climate | Joule Economy (production/distribution/consumption/waste cycle, KiloJoule type), Climate System (baseline, drift, shocks, yield modifier), metrics API for both domains | CIV-0100, CIV-0102 | All Joule Economy metrics tracked; climate shock events trigger and resolve correctly; batch sweep runs 100 variants | High |
| Phase 2 | M5–M9 | Institutions + Citizens + Social | Institutions (constitution, legitimacy, tyranny, elite capture), Citizens/Demography (population, age, factions, grievance), Social/Insurgency (escalation model, insurgent pool, counterinsurgency) | CIV-0103, CIV-0104, CIV-0106 | Legitimacy → insurgency pathway produces expected escalation; faction grievance model calibrated against reference scenarios | Very High |
| Phase 3 | M9–M13 | War/Diplomacy + Mod Platform | War/Diplomacy (entity relations, military capacity, conflict escalation), WASM mod sandbox (four mod types, capability model, memory/CPU limits), civlab-sdk v0.1 | CIV-0105, CIV-0700 | Multi-entity diplomatic simulation runs correctly; first community mod published and validated | Very High |
| Phase 4 | M13–M17 | Web client + Asset pipeline | Pixi.js v8 + React 19 Web RTS client, scenario authoring UI, metrics dashboard, replay inspector, SDXL asset generation pipeline | CIV-0300, CIV-0600 | Time-to-first-run &lt; 5min via Web UI; metrics dashboard displays all six domain metrics; replay seek latency &lt; 200ms | High |
| Phase 5 | M17–M24 | 3D + AI/NPC + Parpour GA | Bevy 3D Desktop client (CIV-0400), AI NPC integration (CIV-0601), Parpour/Venture GA integration, cloud simulation credits platform | CIV-0400, CIV-0601 | Venture AI agents run 500+ scenarios/hour; Bevy client renders 10,000-citizen simulation at &gt; 30fps; cloud credits platform in production | Very High |

### 12.2 Phase 0: Core Tick Loop (Months 0–2)

**Goal:** A minimal, correct, deterministic tick loop with no domain logic. The foundation that all subsequent phases build on.

**Deliverables:**
- `civlab-core` Rust crate published to crates.io (v0.0.1, pre-release)
- `ChaCha20Rng` integration: seeded from u64 scenario seed
- `BLAKE3` state hash computed after every tick
- D1-D7 rules encoded as custom Clippy lints (D2, D3, D4, D5)
- CI replay test: two runs with same seed → byte-identical BLAKE3 hashes at every tick
- Basic scenario TOML loader: seed, tick count, domain stubs
- Basic CLI: `civlab run --scenario scenario.toml --ticks 1000`
- Unit test coverage &gt; 90% for tick loop and hash logic

**Acceptance criteria:**
- CI replay test passes on Linux x86-64, macOS ARM64, macOS x86-64, Windows x86-64
- Tick loop runs at &gt; 10 ticks/ms with empty domain stubs (performance baseline)
- D1-D7 lints catch known violation examples in lint tests

### 12.3 Phase 1: Economy + Climate (Months 2–5)

**Goal:** The two foundational domain systems: the Joule Economy (master constraint) and the Climate System (long-horizon shock driver).

**Deliverables:**
- Joule Economy: KiloJoule i64 newtype, production/distribution/consumption/waste tick logic, Joule stock accumulation, scarcity index, per-capita access metric
- Climate System: baseline parameters, drift model, shock event scheduler, agricultural yield modifier
- Cross-domain coupling: climate shock → production loss; climate → distribution efficiency
- Metrics API: all Joule Economy and Climate metrics accessible via `civlab metrics get`
- Batch sweep CLI: `civlab sweep --params sweep.toml --workers 8 --output results/`
- Result export: JSON Lines and CSV output formats
- Integration tests: 10 reference scenarios with expected metric ranges; CI validates on every PR

**Acceptance criteria:**
- All Joule Economy metrics tracked at every tick with correct accounting (production = consumption + waste + stock delta)
- Climate shock events trigger at scheduled ticks, affect yield modifier correctly, and resolve after specified duration
- Batch sweep runs 100 variants in \< 60 seconds on reference hardware
- Zero determinism violations in 10,000-tick fuzz test with 100 random seeds

### 12.4 Phase 2: Institutions + Citizens + Social (Months 5–9)

**Goal:** The three human-systems domains: governance structures, demographic dynamics, and social conflict.

**Deliverables:**
- Institutions: governance type enum, constitution schema, legitimacy model, tyranny index, elite capture model, policy effectiveness
- Citizens/Demography: population integer, age structure cohorts, faction system, grievance accumulation, material conditions
- Social/Insurgency: escalation state machine (LATENT → CIVIL_WAR), insurgent pool, counterinsurgency mechanics, negotiation mechanic
- Cross-domain coupling: energy scarcity → legitimacy; legitimacy → insurgency; faction grievance → insurgent pool; insurgency → production disruption
- Causal trace API: structured event chain for instability events
- Reference scenario set: 5 designed scenarios demonstrating legitimacy crisis, insurgency outbreak, elite capture, and faction conflict
- Documentation: domain model documentation for all three subsystems

**Acceptance criteria:**
- Legitimacy → insurgency escalation pathway produces expected state machine transitions in reference scenarios
- Causal trace API returns structured chain for &gt; 80% of instability events
- Faction grievance model: all five reference scenarios produce expected faction behavior within 5% metric tolerance
- No regression in Phase 1 determinism or performance

### 12.5 Phase 3: War/Diplomacy + Mod Platform (Months 9–13)

**Goal:** External conflict dynamics and the modding platform that enables community extension.

**Deliverables:**
- War/Diplomacy: entity model (multi-entity scenario), diplomatic relation matrix, military capacity model, conflict escalation state machine, peace settlement mechanic
- WASM mod sandbox: wasmtime integration, capability model, memory/CPU limits, four mod types (Policy, Economic, Event, Scenario)
- civlab-sdk v0.1: Rust crate with type definitions, registration macros, test harness, documentation
- Mod registry: basic hosted registry at registry.civlab.io with hash verification
- Reference mods: one working example of each mod type
- Cross-domain coupling: war → Joule drain; war → casualties; war → legitimacy stress

**Acceptance criteria:**
- Multi-entity scenario with two civilizations in diplomatic conflict runs correctly with no determinism violations
- WASM sandbox correctly enforces memory limit (64MB), CPU budget (10ms/tick), and capability model
- First community-contributed mod validated and published to registry
- civlab-sdk example mod builds, validates, and runs without errors

### 12.6 Phase 4: Web Client + Asset Pipeline (Months 13–17)

**Goal:** The primary GUI surface: the Web RTS client built with Pixi.js v8 and React 19.

**Deliverables:**
- Pixi.js v8 + React 19 Web RTS client
- Scenario authoring UI: form-based scenario definition with validation
- Simulation runner UI: start/pause/step/fast-forward, tick counter, status panel
- Metrics dashboard: time-series charts for all domain metrics, event log, metric gauges
- Replay inspector: tick scrubber, state inspection panel, metric timeline, BLAKE3 verification
- Policy intervention controls: intervention type menu, authority display, branch creation
- SDXL asset generation pipeline: procedural terrain, unit, and building sprites
- Kira 0.12 audio integration: ambient and event audio

**Acceptance criteria:**
- Time-to-first-run &lt; 5 minutes from browser load for new user
- Metrics dashboard renders all six domain metrics for a running 10,000-citizen simulation at &gt; 30fps
- Replay seek to any tick in 10,000-tick simulation in &lt; 200ms
- Scenario authoring UI validates scenario and displays path-specific error messages
- WCAG 2.1 AA color contrast on all metric displays and state badges

### 12.7 Parpour/Venture Integration Milestone (Month 15 target)

**Goal:** Production-grade API integration between CivLab and Parpour/Venture AI agent platform.

**Deliverables:**
- Versioned HTTP API v1.0 (stable contract)
- Venture adapter: scenario dispatch, result retrieval, branch API
- API schema documentation published
- Integration test suite: 50 automated tests covering Venture agent workflow
- SLA definition: 99.9% uptime, &lt; 30-second scenario result latency
- Changelog and deprecation policy published

**Acceptance criteria:**
- Venture AI agents run &gt; 500 scenarios/hour sustained
- All dispatched scenarios verified deterministic (BLAKE3 hash match on replay)
- Zero API contract breaking changes without versioned migration path

---

## 13. Integration with Parpour/Venture

### 13.1 What Parpour/Venture Is

Parpour is an autonomous AI economic platform. Venture is its AI agent layer: agents that propose, test, and iterate on economic and governance policies. Venture agents use CivLab as their primary simulation backend: before recommending or enacting a policy in a real-world context, the agent runs the policy through a CivLab scenario to evaluate its outcomes across multiple dimensions.

This creates a tight integration requirement: CivLab must be reliable, fast, deterministic, and API-accessible enough to serve as the inner loop of an AI policy search.

### 13.2 Venture Agent Workflow

```
Venture Agent
     |
     v
[Policy Proposal]  →  Translate policy to CivLab scenario parameters
     |
     v
[Scenario Dispatch]  →  POST /v1/scenarios  (civlab HTTP API)
     |
     v
[Wait for result]  →  GET /v1/scenarios/{id}  (poll or webhook)
     |
     v
[Result retrieval]  →  GET /v1/scenarios/{id}/results  (structured JSON)
     |
     v
[Outcome evaluation]  →  Parse metrics; compute policy score
     |
     v
[Iterate]  →  Adjust parameters; dispatch next scenario variant
```

The agent runs this loop N times (N = 10–1,000 per policy search session), comparing outcome metrics across variants to identify the parameter configuration that optimizes the agent's objective function (e.g., maximize legitimacy at 10,000 ticks without entering civil war).

### 13.3 API Contract Between Venture and CivLab

The Venture-CivLab API contract is governed by the following principles:

| Principle | Description |
|-----------|-------------|
| **Versioned and stable** | API is semver-versioned; breaking changes require a new major version; prior version maintained for &gt; 6 months |
| **Typed and schema-validated** | All request/response payloads are schema-validated; schema published as OpenAPI 3.1 document |
| **Determinism guaranteed** | Every scenario dispatched via Venture API includes a seed; result payload includes BLAKE3 final state hash; Venture can verify replay determinism |
| **Idempotent dispatch** | Scenario dispatch is idempotent with client-provided idempotency key; duplicate dispatch with same key returns same result |
| **Structured errors** | All errors return structured JSON with error code, message, and remediation hint |
| **Async by default** | Scenarios are dispatched asynchronously; result is polled or pushed via webhook; no synchronous blocking on long-running simulations |

#### Core API Endpoints (Venture Integration)

```
POST /v1/scenarios
  Body: { scenario: ScenarioDefinition, seed: u64, ticks: u64, mods: [ModRef] }
  Returns: { scenario_id: uuid, status: "QUEUED" }

GET /v1/scenarios/{id}
  Returns: { scenario_id, status: "QUEUED"|"RUNNING"|"COMPLETE"|"ABORTED", progress: { tick, total_ticks } }

GET /v1/scenarios/{id}/results
  Returns: ScenarioResult (full result payload; see Section 5.7)

POST /v1/scenarios/{id}/branch
  Body: { at_tick: u64, intervention: InterventionDefinition }
  Returns: { branch_scenario_id: uuid }
```

### 13.4 Shared Artifact Determinism Requirements

When Venture stores a scenario result for audit, regulatory, or reproducibility purposes, the stored artifact must include sufficient information to reproduce the result independently:

| Artifact Component | Required | Purpose |
|-------------------|----------|---------|
| Scenario definition (TOML) | Yes | Defines initial conditions |
| Seed (u64) | Yes | Controls all randomness |
| CivLab version (semver) | Yes | Ties result to specific engine version |
| Mod manifest (name + version + BLAKE3 hash) | Yes if mods used | Reproducible mod set |
| BLAKE3 final state hash | Yes | Verification of replay fidelity |
| BLAKE3 hash at each tick (hashes.bin) | Optional | Full replay verification |
| Result JSON | Yes | Structured outcome metrics |

A Venture audit package contains all required components. Any third party with `civlab` installed can run `civlab verify --artifact \<package\>` to independently reproduce and verify the result.

### 13.5 Parpour Business Relationship

The CivLab-Parpour commercial relationship is structured as:

- **Integration API license:** Parpour pays a monthly API license fee for commercial use of the CivLab HTTP API (above free tier TCU allocation).
- **Revenue share on cloud credits:** Parpour/Venture consumes cloud simulation credits; CivLab charges at enterprise rate.
- **Co-development arrangement:** Feature requests from Venture that have general applicability to CivLab's broader user base are implemented in civlab-core (open source) and prioritized in the roadmap. Venture-specific adapter code is Parpour's responsibility.
- **Joint publication:** CivLab and Parpour co-author technical publications demonstrating AI-driven policy analysis on CivLab simulations.

---

## 14. Governance and Decision Framework

### 14.1 Product Priority Decision Authority

| Decision Type | Decision Authority | Process |
|--------------|-------------------|---------|
| Roadmap phase priorities | CivLab Product Lead | Annual planning; quarterly review; published in WORK_STREAM.md |
| Feature scope within phase | CivLab Engineering Lead | Sprint planning; FR tracker update |
| API breaking changes | CivLab Architecture Review | RFC required; &gt; 14-day community comment period |
| Scenario format changes | CivLab Architecture Review | RFC required; migration path required |
| D1-D7 ruleset amendments | CivLab Engineering Lead + Community RFC | Unanimous team sign-off + RFC process |
| Mod type additions | CivLab Product Lead | ADR required; civlab-sdk update |
| Open-source license changes | CivLab Legal + Community RFC | Board approval + community notice |
| Parpour API contract changes | CivLab Product Lead + Parpour | Joint review; semver versioning; deprecation policy |

### 14.2 Spec-First Requirement

All new features in CivLab must be specified before implementation begins. The spec-first requirement means:

1. A Functional Requirement (FR) entry is created in `FUNCTIONAL_REQUIREMENTS.md` with FR-SIM-NNN format.
2. An ADR entry is created for any architectural decision with ADR-NNN format.
3. Acceptance criteria are defined in the FR before any implementation PR is opened.
4. The FR is approved by the CivLab Product Lead before engineering begins.
5. Implementation PRs reference the FR ID in their description.
6. The FR is marked IMPLEMENTED when the feature passes all acceptance criteria tests.

This ensures that the roadmap, code, and tests are always traceable to a specification.

### 14.3 ADR Process

Architecture Decision Records (ADRs) document significant technical decisions, their context, alternatives considered, and rationale.

**When an ADR is required:**
- Choosing a new external dependency (crate, WASM runtime, database)
- Changing the tick loop architecture
- Adding a new domain coupling
- Modifying the scenario format in a potentially breaking way
- Choosing a custom implementation over an existing library (with justification)
- Any change to D1-D7 ruleset or BLAKE3 hash algorithm

**ADR format (ADR.md):**

```markdown
## ADR-NNN: [Short Title]
**Status:** PROPOSED | ACCEPTED | DEPRECATED | SUPERSEDED
**Date:** YYYY-MM-DD
**Deciders:** [names/roles]

### Context
[Why this decision is needed]

### Decision
[What was decided]

### Alternatives Considered
[What else was considered and why not chosen]

### Consequences
[Trade-offs; what becomes easier/harder]

### Validation
[How to verify the decision was correct]
```

### 14.4 Community RFC Process for Major Changes

For changes that affect the public API, scenario format, or D1-D7 ruleset, CivLab uses an RFC (Request for Comments) process:

**RFC Process:**
1. Author opens a GitHub Discussion in the `civlab-rfcs` repository with the RFC template.
2. RFC is open for community comment for a minimum of 14 days (or 30 days for breaking changes).
3. CivLab team synthesizes feedback and posts a disposition: ACCEPTED, REJECTED, or DEFERRED.
4. Accepted RFCs are converted to FR + ADR entries and added to the roadmap.
5. Implementation begins only after RFC acceptance.

**What requires an RFC:**
- Any change to the public CivLab HTTP API (v1+)
- Any change to the scenario TOML/JSON schema
- Any change to the civlab-sdk WASM mod API
- Any change to the D1-D7 determinism ruleset
- Any change to the replay artifact format
- Adding or removing a simulation domain at the top level

**What does NOT require an RFC:**
- Bug fixes that do not change API behavior
- Performance improvements with no API surface change
- Documentation improvements
- New default parameter values (backward compatible)
- Internal refactoring with no external behavior change

### 14.5 Quality Gates

All changes to civlab-core must pass the following gates before merge:

| Gate | Description | Failure Action |
|------|-------------|----------------|
| CI replay test | Two runs with same seed → byte-identical BLAKE3 hashes | Block merge |
| D1-D7 Clippy lints | No violations of D2, D3, D4, D5 rules | Block merge |
| Performance regression | Tick latency &gt; 5% above baseline | Require engineering lead sign-off |
| Test coverage | &gt; 90% unit test coverage for modified modules | Block merge |
| FR traceability | All new code references an FR ID | Block merge |
| API schema validation | All API changes update OpenAPI schema | Block merge |
| WASM build | WASM32 target builds without error | Block merge |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| KiloJoule (kJ) | The atomic unit of CivLab's energy economy; represented as i64 newtype |
| Tick | One discrete simulation step; all domain updates occur within a single tick |
| ChaCha20Rng | The deterministic random number generator used for all simulation randomness |
| BLAKE3 | The cryptographic hash function used to compute per-tick state hashes |
| D1-D7 | The seven determinism rules that govern all simulation code |
| Scenario | A complete description of a simulation's initial conditions (seed + parameters + mods) |
| Replay artifact | The complete recorded output of a simulation run (state snapshots + diffs + events + hashes) |
| Branch | A fork of a simulation at a specific tick, creating two independent forward trajectories |
| Sweep | A batch run of N scenario variants, typically varying one or more parameters |
| Causal trace | A structured DAG representing the event chain that led to an instability event |
| WASM mod | A WebAssembly module loaded into the CivLab simulation sandbox to extend behavior |
| civlab-sdk | The Rust crate providing types, macros, and test harness for WASM mod development |
| TCU (Tick-Compute Unit) | The pricing unit for cloud simulation credits; 1 TCU = 1 simulation tick on reference compute |
| RFC | Request for Comments; the community review process for major CivLab changes |
| ADR | Architecture Decision Record; documents significant technical decisions |
| FR | Functional Requirement; a SHALL statement specifying required system behavior |
| Venture | The AI agent layer of Parpour that uses CivLab as a policy simulation backend |
| Joule Economy | CivLab's energy-as-primary-resource economic subsystem |
| Legitimacy | The population-weighted measure of institutional acceptance; key governance metric |
| Tyranny index | Composite metric of enforcement overreach and political capture |
| Elite capture | The degree to which institutional decisions serve elite rather than general population interests |
| Insurgency | Organized armed resistance to institutional authority; modeled as a state machine |

---

## Appendix B: FR ID Reference

| FR Range | Domain |
|----------|--------|
| CIV-0001–CIV-0099 | Core tick loop, harness, and determinism |
| CIV-0100–CIV-0101 | Joule Economy |
| CIV-0102 | Climate System |
| CIV-0103 | Institutions |
| CIV-0104 | Citizens/Demography |
| CIV-0105 | War/Diplomacy |
| CIV-0106 | Social/Insurgency |
| CIV-0300 | Web RTS Client (Pixi.js + React) |
| CIV-0400 | Desktop Client (Bevy 3D) |
| CIV-0600 | Asset Generation (SDXL) |
| CIV-0601 | AI/NPC Integration |
| CIV-0700 | WASM Mod Platform + civlab-sdk |

---

*This document is the authoritative product model for CivLab's civ-sim platform. It is updated as part of the standard RFC and ADR process. Questions, corrections, and RFC proposals should be submitted via the civlab-rfcs GitHub repository.*
