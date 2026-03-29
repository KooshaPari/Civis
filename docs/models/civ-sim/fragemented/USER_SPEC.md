# CivLab Civ-Sim User Specification

**Document Status:** Draft v0.1
**Last Updated:** 2026-02-21
**Owner:** CivLab Product Engineering
**Audience:** UX Designers, Frontend Engineers, Backend Engineers, QA

---

## Table of Contents

1. [Overview and User Philosophy](#1-overview-and-user-philosophy)
2. [User Personas](#2-user-personas)
3. [Core User Flows](#3-core-user-flows)
4. [UX Requirements (FR Format)](#4-ux-requirements-fr-format)
5. [Information Architecture](#5-information-architecture)
6. [Interaction Patterns](#6-interaction-patterns)
7. [CLI Interface](#7-cli-interface)
8. [API Interface](#8-api-interface)
9. [Accessibility and Localization](#9-accessibility-and-localization)
10. [Acceptance Criteria](#10-acceptance-criteria)

---

## 1. Overview and User Philosophy

### 1.1 What CivLab Is

CivLab is a headless deterministic civilization simulation engine. Its purpose is to serve as a rigorous analytical instrument — not an entertainment product. Users reason about civilizational dynamics: energy economics, institutional stability, climate feedback, social cohesion, conflict, and diplomacy. Every output the system produces must be traceable, reproducible, and explainable.

The engine simulates at 100ms per tick, uses ChaCha20Rng for all stochastic sampling, and generates a BLAKE3 hash of full simulation state on every tick. Determinism is not optional — it is the foundation on which user trust is built. A user who cannot replay a run and get the same result cannot trust any result.

### 1.2 Design Principles

**Principle 1: Provenance First**
Every number, chart, and table presented to a user must carry an unambiguous trail back to the scenario configuration, seed, and tick range that produced it. The system must never present outputs that cannot be reconstructed.

**Principle 2: Determinism Is a User Feature**
Users must be able to share a run ID and have any other user reproduce the exact simulation state. Run IDs encode the seed and config hash. Replay from any tick must be a first-class operation, not a debugging affordance.

**Principle 3: Fail Loudly**
If a simulation encounters an unstable state, a determinism violation, or a configuration error, it surfaces this immediately with specific detail. The system does not attempt graceful degradation. It halts and explains.

**Principle 4: Minimize Cognitive Overhead for Branching**
Counterfactual reasoning — "what if we had applied intervention X at tick 3000?" — is a primary analytical operation. The UI must make branching and comparing branches as direct as possible. No accidental destruction of branches. No hidden history.

**Principle 5: CLI and API Are First-Class**
Research Operators run hundreds of parameter sweeps. The CLI and JSON-RPC API must be as capable and ergonomic as the web UI. Nothing available in the web UI should be unavailable via CLI or API.

**Principle 6: Explicit Assumptions**
Scenario configurations encode explicit assumptions. The UI must surface these assumptions throughout the workflow — when creating, when viewing outputs, and when exporting. Outputs without visible assumptions are analytically worthless.

**Principle 7: Modular Extension Without Trust Violations**
Mods execute in a wasmtime 26.x WASM sandbox. Mod authors have a defined SDK surface. The system must make it clear to users when a mod is active in a run, and what that mod's claimed behavior is.

### 1.3 Who This System Serves

CivLab serves three primary user populations and one secondary population:

| Population | Primary Need | Primary Surface |
|---|---|---|
| Scenario Designer | Author and version simulation scenarios | Web UI (scenario editor) |
| Policy Analyst | Compare policy regimes; produce reports | Web UI (comparison panels, export) |
| Research Operator | Batch sweeps, statistical aggregation, automation | CLI + REST/WebSocket API |
| Modder / Extension Developer | Author WASM mods; test against engine | CLI + civlab-sdk crate + local test harness |

These populations have overlapping but distinct tool preferences, workflow patterns, and output requirements. The system must serve all four without forcing any of them into workflows optimized for another group.

### 1.4 Non-Goals

This specification does not cover:

- Entertainment gameplay features (no victory conditions, no player agency in the game-design sense)
- Real-time multiplayer simulation
- Cloud infrastructure provisioning
- Commercial licensing workflows
- Mobile client implementation details (future)
- Bevy 3D desktop client implementation details (future)

---

## 2. User Personas

### 2.1 Persona A: Scenario Designer

#### 2.1.1 Background

**Name (representative):** Maren
**Role:** Computational social scientist or policy simulation architect
**Technical level:** High. Comfortable with TOML/JSON config authoring, version control concepts, and command-line tools. May not be a software engineer but understands structured data.
**Context:** Works at a think tank, university research center, or government modeling office. Produces scenarios that other analysts then run and study.

#### 2.1.2 Goals

1. Author scenario configurations that encode explicit, peer-reviewable assumptions about initial civilization state, policy parameters, and environmental conditions.
2. Iterate rapidly on scenario variants (e.g., high-energy vs. low-energy starting conditions) without duplicating work.
3. Version and archive scenarios so that published analyses can reference a specific, immutable scenario version.
4. Validate scenario configurations before publishing them to ensure they are well-formed, deterministic-safe, and within engine parameter bounds.
5. Document the rationale for specific parameter choices directly within the scenario artifact.

#### 2.1.3 Pain Points (Without CivLab)

- Scenarios exist as ad hoc TOML files with no enforced schema, leading to silent misconfiguration.
- No versioning: a scenario file changed after analysis invalidates prior results with no audit trail.
- Validation is a manual trial-and-error process against the engine.
- Sharing scenarios requires sharing raw config files with no guarantee of reproducibility.
- Rationale for parameter choices lives in separate documents that drift out of sync with configs.

#### 2.1.4 Key Tasks

| Task | Frequency | Criticality |
|---|---|---|
| Create new scenario from template | 2-5x/week | High |
| Edit scenario parameters | Daily | High |
| Validate scenario config | Every edit session | Critical |
| Save scenario version (immutable snapshot) | Weekly or at milestone | Critical |
| Fork scenario to create variant | 1-3x/week | High |
| Add inline rationale/annotation to parameters | Per editing session | Medium |
| Browse scenario version history | 1-2x/week | Medium |
| Publish scenario to shared workspace | 1-2x/month | High |
| Diff two scenario versions | As needed | Medium |

#### 2.1.5 Workflow Patterns

**Pattern 1: Template-to-Scenario**
Maren starts from a curated engine template (e.g., "Pre-Industrial Island State") and modifies parameters section by section. She uses the schema-aware editor to catch out-of-range values immediately. She saves draft frequently and commits a named version when satisfied.

**Pattern 2: Variant Branching**
Maren takes a validated scenario and forks it to create a variant that differs in one policy cluster (e.g., trade liberalization on vs. off). She wants the diff between fork and parent to be explicit and reviewable.

**Pattern 3: Collaborative Review**
Maren shares a scenario version link with a colleague. The colleague must be able to load the exact configuration with no ambiguity.

#### 2.1.6 UI Surface Requirements

- Schema-aware TOML/JSON editor with inline validation and error messages referencing the parameter name and valid range.
- Scenario version history panel with diff view between any two versions.
- Inline annotation fields per parameter group (free-text, markdown-rendered).
- Template library browser with parameter preview and one-click fork.
- Scenario publish/share workflow producing a stable reference URL encoding the scenario hash.
- Fork origin display on all scenario views (shows parent version).

#### 2.1.7 Success Criteria

- Maren can create a valid, versioned scenario from a template in under 10 minutes.
- Any saved scenario version is immutable and permanently replayable.
- Validation errors are actionable (specific parameter name, current value, valid range, suggested fix).
- Diff between two scenario versions is readable without external tooling.
- Scenario reference URL is stable and can be cited in a published document.

---

### 2.2 Persona B: Policy Analyst

#### 2.2.1 Background

**Name (representative):** Tariq
**Role:** Policy analyst, economist, or political scientist
**Technical level:** Medium. Proficient with spreadsheets, data visualization tools, and can read JSON. Not a programmer.
**Context:** Works at a policy institute, legislative research office, or international organization. Uses CivLab to evaluate civilizational outcomes under different policy regimes and produce decision-support reports.

#### 2.2.2 Goals

1. Run pre-authored scenarios (produced by Scenario Designers) and inspect outcomes.
2. Compare two or more policy regimes side-by-side across multiple metrics.
3. Understand the causal chain behind a specific outcome (why did legitimacy collapse at tick 4200?).
4. Produce professional-grade export packages containing charts, data tables, assumption disclosures, and narrative annotations.
5. Annotate specific moments in a simulation timeline with interpretive commentary.
6. Share findings with non-technical stakeholders in formats that do not require CivLab access.

#### 2.2.3 Pain Points (Without CivLab)

- Comparing policy regimes requires manually aligning outputs from different simulation runs in a spreadsheet.
- Charts have no provenance — cannot verify what scenario or parameters produced them.
- Timeline inspection is not possible; only endpoint summaries are available.
- Export artifacts are not self-contained; they require the reader to have access to raw data files.
- Assumptions are not disclosed in exported reports.

#### 2.2.4 Key Tasks

| Task | Frequency | Criticality |
|---|---|---|
| Browse available scenarios | Daily | High |
| Run scenario (trigger engine) | Daily | Critical |
| Inspect simulation timeline | Daily | High |
| Navigate hex map at two LOD levels | Daily | Medium |
| Add annotation to timeline event | 3-5x/week | Medium |
| Open comparison panel (2+ runs) | 3-5x/week | High |
| Select metrics for comparison chart | Per comparison session | High |
| Export report package | 1-2x/week | Critical |
| Share run reference with colleague | 1-2x/week | High |
| Investigate metric anomaly (drill down) | 2-3x/week | High |

#### 2.2.5 Workflow Patterns

**Pattern 1: Baseline vs. Intervention**
Tariq runs the baseline scenario to completion, then runs the same scenario with an intervention applied at a specific tick. He opens the comparison panel, selects the metrics he cares about (tyranny, legitimacy, trade balance), and reviews the divergence curve.

**Pattern 2: Policy Regime Matrix**
Tariq compares four runs simultaneously: low tax + open trade, low tax + closed trade, high tax + open trade, high tax + closed trade. He produces a summary table showing endpoint metrics for each regime and exports it as a report package.

**Pattern 3: Causal Chain Investigation**
A metric spikes unexpectedly. Tariq scrubs the timeline to the spike point, inspects the event log at that tick, and follows the causal chain backward through prior ticks.

#### 2.2.6 Multi-Run Comparison Requirements

- Side-by-side metric charts with aligned time axes.
- Metric selector: choose any combination of the eight core metrics (waste, surplus, tyranny, legitimacy, resilience, Joule stock, trade balance, population).
- Delta chart mode: show difference between runs rather than absolute values.
- Divergence point detection: automatic annotation of the first tick where two runs diverge beyond a configurable threshold.
- Regime label assignment: user can name each run for display purposes (e.g., "Open Trade", "Closed Trade").

#### 2.2.7 Export Requirements

- Export formats: PDF report (human-readable), JSON bundle (machine-readable), CSV (tabular metrics), Parquet (analytical datasets).
- Report bundle must include: scenario config hash, run ID, seed, parameter assumptions, all charts as vector SVG, all data tables as embedded CSV, user annotations, generation timestamp, engine version.
- All exported charts must embed their provenance in SVG metadata.
- PDF report must include a cover page with assumption disclosure section.
- Export package must be cryptographically signed (BLAKE3 hash of bundle contents) for integrity verification.

#### 2.2.8 Annotation Requirements

- Annotations attach to a specific tick range and optionally to a specific metric.
- Annotations are free-text with Markdown rendering.
- Annotations appear in the timeline view as markers and in the exported report.
- Annotations are saved per-run, not per-scenario. A scenario run can have multiple annotation sets (one per analyst).
- Annotations must be exported with the report and attributed to the author.

#### 2.2.9 Success Criteria

- Tariq can produce a side-by-side comparison of two runs within 5 minutes of run completion.
- Exported report is self-contained: a reader with no CivLab access can verify what assumptions produced the charts.
- Timeline scrubbing to a specific tick is responsive (under 200ms from drag to render).
- Metric anomalies surface in the event log with causal attribution.
- Report PDF renders without external fonts or assets.

---

### 2.3 Persona C: Research Operator

#### 2.3.1 Background

**Name (representative):** Priya
**Role:** Quantitative researcher, computational economist, or simulation engineer
**Technical level:** Very high. Comfortable with shell scripting, Python/R data analysis, distributed compute, and structured output formats.
**Context:** Runs large-scale parameter sweeps to characterize simulation sensitivity, build confidence intervals over stochastic draws, and produce machine-readable datasets for downstream analysis pipelines.

#### 2.3.2 Goals

1. Execute parameter sweeps over a scenario config with a defined parameter grid.
2. Run multiple seeds per parameter combination to produce distributions over stochastic outcomes.
3. Aggregate outputs into machine-readable datasets (CSV, JSON, Parquet) without manual intervention.
4. Detect and flag parameter combinations that produce unstable or degenerate simulation states.
5. Integrate CivLab into automated pipelines (CI/CD, Jupyter notebooks, Python analysis scripts).
6. Control resource usage: max concurrent runs, memory limits, disk quotas.

#### 2.3.3 Pain Points (Without CivLab)

- No native sweep command; sweeps require custom shell scripts with manual job management.
- No structured output format; must parse engine logs manually.
- No automatic stability detection; degenerate runs silently corrupt aggregate statistics.
- No confidence interval tooling; statistical analysis happens entirely outside the engine.
- Parallelism is limited by manual process management.

#### 2.3.4 CLI-First Expectations

Priya expects:

- A single `civlab sweep` command that accepts a sweep manifest file.
- Progress reporting on stdout in a machine-parseable format (JSON lines or structured text).
- Run artifacts written to a configurable output directory with a predictable naming convention.
- Exit codes that distinguish success, partial failure (some runs failed), and total failure.
- A `civlab aggregate` command that ingests a sweep output directory and produces summary statistics.
- Man pages or `--help` output that is complete and accurate.

#### 2.3.5 Headless and API Usage Patterns

- Priya invokes CivLab from Python scripts via subprocess or the REST API.
- She expects JSON output from all CLI commands when `--format json` is specified.
- She submits batch run requests via the REST API and polls for completion.
- She streams tick-level data from running simulations via the WebSocket JSON-RPC API.
- She uses the API to inject interventions programmatically during a live run.

#### 2.3.6 Output Format Requirements

| Format | Use Case | Required Fields |
|---|---|---|
| JSON Lines | Per-tick metric stream | tick, seed, run_id, metric_name, value |
| CSV | Endpoint summary per run | run_id, seed, param_key, param_value, metric_name, endpoint_value |
| Parquet | Full time-series dataset | All CSV fields + tick column + all metric columns |
| JSON bundle | Full run artifact | run_id, scenario_hash, config, seed, tick_hashes[], metrics_by_tick{} |

#### 2.3.7 Multi-Run Aggregation and Statistical Analysis

- The `civlab aggregate` command computes: mean, median, p5, p25, p75, p95, std dev, and IQR per metric per parameter combination.
- Degenerate run detection: runs where any core metric reaches `NaN`, `Inf`, or a predefined instability threshold are flagged and excluded from aggregates with a warning.
- Confidence interval output: 95% CI reported alongside each aggregate.
- Sensitivity analysis output: Spearman rank correlation between each swept parameter and each endpoint metric.
- All aggregate outputs available as CSV, JSON, and Parquet.

#### 2.3.8 Success Criteria

- A sweep of 1000 runs executes without requiring any manual intervention.
- Degenerate runs are automatically excluded and logged; aggregate statistics are correct over the remaining runs.
- All outputs land in a predictable directory structure that can be committed to a data repository.
- Integration into a Python analysis script requires no more than 10 lines of subprocess/API calls.
- `civlab --help` is sufficient to complete a sweep without reading external documentation.

---

### 2.4 Persona D: Modder / Extension Developer

#### 2.4.1 Background

**Name (representative):** Ola
**Role:** Software engineer, researcher, or advanced user building domain-specific extensions
**Technical level:** Very high. Comfortable with Rust or AssemblyScript targeting WASM, the civlab-sdk crate, and the engine's hook system.
**Context:** Building a custom Policy mod, Economic model extension, Event generator, or Scenario template that extends engine behavior within the WASM sandbox.

#### 2.4.2 Goals

1. Author WASM mods using the civlab-sdk crate with a clear, stable API surface.
2. Test mods locally against the engine before publishing to the registry.
3. Understand the sandbox constraints (what the mod can and cannot do).
4. Distribute mods via the CivLab mod registry with a signed artifact.
5. Inspect which mods are active in a given run and what they affect.

#### 2.4.3 WASM Sandbox Constraints (visible to user)

| Constraint | Value |
|---|---|
| WASM runtime | wasmtime 26.x |
| Mod types | Policy, Economic, Event, Scenario |
| Max WASM memory | 256 MB per mod instance |
| Max execution time per hook | 10ms |
| Host imports allowed | civlab_sdk::host::* (enumerated in SDK docs) |
| Forbidden | File I/O, network, random (must use engine-provided RNG) |
| Determinism requirement | Mods must be pure functions of their inputs |

#### 2.4.4 SDK Workflow

1. Install civlab-sdk crate (`cargo add civlab-sdk`).
2. Implement the appropriate mod trait (e.g., `PolicyMod`, `EconomicMod`).
3. Build to WASM target (`cargo build --target wasm32-wasip2`).
4. Run local test harness (`civlab mod test ./my_mod.wasm --scenario examples/island_state.toml`).
5. Inspect test output: hook call counts, metric effects, determinism verification.
6. Package and sign (`civlab mod package ./my_mod.wasm --sign`).
7. Publish to registry (`civlab mod publish ./my_mod.civmod`).

#### 2.4.5 UI Surface Requirements

- Mod browser panel in web UI: browse registry, view mod metadata, enable/disable mods per run.
- Active mod list visible in run header (any run with mods active displays a mod badge).
- Mod hook call log in developer mode: shows which hooks fired, execution time, and claimed effects.
- Mod integrity verification status displayed per-mod (signature valid / signature invalid / unsigned).

#### 2.4.6 Success Criteria

- A competent Rust developer can author, test, and publish a minimal Policy mod in under one day.
- Mods that violate sandbox constraints fail at load time with a specific error message.
- Any run using mods is labeled in all exports and reports.
- Mod-active runs are still fully replayable, provided the same mod WASM artifact is available.

---

## 3. Core User Flows

### 3.1 Flow 1: Create Scenario → Validate → Save Version

#### 3.1.1 Description

The Scenario Designer authors a new scenario configuration, validates it against the engine schema and parameter bounds, and saves an immutable versioned snapshot.

#### 3.1.2 Preconditions

- User is authenticated and has write access to a workspace.
- Engine schema is loaded and available for validation.

#### 3.1.3 Step-by-Step

**Step 1: Initiate scenario creation**
User navigates to the Scenarios panel and selects "New Scenario". System presents two paths: (a) Start from blank, (b) Browse template library.

**Step 2: Select template (if applicable)**
User browses the template library. Templates are categorized by starting civilization type (Island State, Continental Empire, Archipelago Federation, etc.). Each template shows a parameter preview panel with key metrics. User selects a template and clicks "Fork as New Scenario".

**Step 3: Open scenario editor**
System opens the schema-aware scenario editor. The editor displays the TOML configuration with section panels: Initial State, Policy Parameters, Climate Parameters, Institutional Parameters, RNG Seed.

**Step 4: Edit parameters**
User modifies parameters. The editor validates each field against the schema on change (not on submit). Validation errors appear inline below the field with the constraint violated and the valid range. Out-of-range values are highlighted in amber. Schema-invalid values are highlighted in red.

**Step 5: Add annotations**
User opens the Annotations panel. Annotations can be added per parameter section. The user enters free-text rationale for parameter choices. Annotations are stored alongside the config.

**Step 6: Run validation**
User clicks "Validate". The system sends the full config to the engine validation endpoint. Validation checks: (a) schema conformance, (b) parameter range bounds, (c) cross-parameter constraint satisfaction, (d) determinism precondition checks (no floating-point dependencies on external state). Validation result is displayed as a structured report.

**Step 7: Address validation errors**
If validation fails, the error report lists each failed check with the parameter path, current value, and the violated constraint. User returns to editor to fix errors. Repeat from Step 4.

**Step 8: Save draft**
User saves the current state as a draft. Drafts are mutable. The system assigns a draft ID and displays a draft indicator in the scenario header.

**Step 9: Commit version**
When satisfied, user clicks "Commit Version". System requires a version label (e.g., "v1.0-baseline") and an optional commit message. System computes BLAKE3 hash of the config, stores an immutable snapshot, and assigns a version ID of the form `{scenario_id}@{config_hash_prefix}`. The committed version cannot be modified.

**Step 10: Confirm version saved**
System displays the version ID, config hash, and timestamp. A stable reference URL is generated. The version appears in the scenario version history panel.

#### 3.1.4 Error States and Recovery

| Error | System Response | User Action |
|---|---|---|
| Schema validation failure | Inline error on field, validation report | Fix parameter, re-validate |
| Parameter out of range | Amber highlight, range tooltip | Adjust to valid range |
| Cross-parameter constraint failure | Validation report with constraint description | Adjust dependent parameters |
| Version label collision | Error: "Version label already exists" | Choose a different label |
| Network error during commit | Error banner, draft preserved | Retry commit |

#### 3.1.5 ASCII Flow Diagram

```
User                      Web UI                     Engine Validation
 |                           |                               |
 |-- "New Scenario" -------> |                               |
 |                           |-- Show template browser ----> |
 |-- Select template ------> |                               |
 |                           |-- Fork config, open editor -> |
 |-- Edit parameters ------> |                               |
 |                           |-- Inline validate (schema) -> |
 |                           |<-- Validation result -------- |
 |<-- Inline errors -------- |                               |
 |-- Fix parameters -------> |                               |
 |-- Add annotations ------> |                               |
 |-- "Validate" -----------> |                               |
 |                           |-- Full validation request --> |
 |                           |<-- Validation report --------- |
 |<-- Validation report ----- |                               |
 |                           |                               |
 |   [If errors: fix and     |                               |
 |    repeat from Edit]      |                               |
 |                           |                               |
 |-- "Commit Version" ------> |                               |
 |   (label + message)       |-- Compute config BLAKE3 ----> |
 |                           |-- Store immutable snapshot -> |
 |                           |-- Assign version ID --------> |
 |<-- Version confirmed ------ |                               |
 |   (version_id, hash, URL) |                               |
```

---

### 3.2 Flow 2: Run Simulation → Inspect Timeline → Compare Against Baseline

#### 3.2.1 Description

The Policy Analyst runs a scenario version, inspects the resulting simulation timeline at two levels of detail (strategic hex view and city-level operational view), and compares it against a baseline run side-by-side.

#### 3.2.2 Preconditions

- At least one committed scenario version exists.
- A baseline run for the scenario exists (or the user designates one after running).

#### 3.2.3 Step-by-Step

**Step 1: Select scenario version**
User navigates to the Scenario panel, selects a committed scenario version, and clicks "Run". System presents the run configuration dialog: seed input (auto-generated or user-specified), optional mod selection, run label.

**Step 2: Submit run**
User confirms the run configuration and submits. System creates a run record with a run ID of the form `{scenario_hash}-{seed}-{timestamp}`. Engine begins executing. Run status indicator appears in the run list: Queued → Running → Complete.

**Step 3: Monitor run progress**
While running, the Progress panel shows: current tick, estimated completion, tick rate (ticks/second), current metric snapshots updated every 50 ticks. User can close the panel; the run continues in the background.

**Step 4: Open completed run**
User clicks on a completed run. System loads the timeline view. Default state: strategic hex map centered on the simulation world, timeline scrubber positioned at tick 0, metric panel showing all eight core metrics as time-series sparklines.

**Step 5: Navigate hex map (strategic view)**
User pans the hex map with click-drag. Zoom with scroll wheel or pinch. At strategic zoom, hex tiles show aggregated data: color-coded by dominant metric (e.g., legitimacy gradient from green to red). Hovering a hex tile shows a tooltip with aggregated metrics for that tile's region.

**Step 6: Descend to operational view**
User double-clicks a hex tile or uses the "Zoom to City" button. System transitions to operational (city-level) LOD. City view shows: individual citizen agents (aggregated by cohort), building footprints, resource flows, institution status indicators. A breadcrumb shows the current LOD level and location.

**Step 7: Scrub timeline**
User drags the timeline scrubber to a specific tick. System updates all views (hex map, metric charts, event log) to reflect state at that tick. Scrubbing is frame-synced: display updates within 200ms of scrubber position change.

**Step 8: Inspect event log**
The event log panel shows all events fired at the current tick and surrounding window (&plusmn;5 ticks). Events include: policy changes, resource threshold crossings, institution stability changes, conflict onset, diplomatic events. Each event has a causal attribution field showing what triggered it.

**Step 9: Annotate a moment**
User right-clicks on the timeline at a specific tick and selects "Add Annotation". A text field appears. User enters commentary. Annotation marker appears on the timeline.

**Step 10: Open comparison panel**
User clicks "Compare" in the toolbar. The comparison panel opens. User selects the baseline run from the run list. System loads both runs into the comparison view.

**Step 11: Configure comparison**
User selects which metrics to compare (checkbox list). User selects display mode: Absolute (both time series on same chart) or Delta (difference between runs on a single chart). Divergence point detection runs automatically and marks the first tick of significant divergence.

**Step 12: Interpret comparison**
Both runs are displayed with aligned time axes. The baseline run is shown in a muted color; the comparison run in a prominent color. Regime labels assigned by user appear in the legend. The divergence point annotation displays the metric that diverged first and the magnitude.

#### 3.2.4 ASCII Flow Diagram

```
User                      Web UI                     Engine / API
 |                           |                               |
 |-- Select scenario ver. -> |                               |
 |-- Configure run --------> |                               |
 |   (seed, mods, label)     |-- Submit run request -------> |
 |                           |<-- run_id assigned ---------- |
 |                           |                               |
 |<-- Run status: Queued --- |   [Engine executing]          |
 |<-- Run status: Running -- |<-- Progress stream (WS) ----- |
 |<-- Run status: Complete - |<-- Completion event (WS) ---- |
 |                           |                               |
 |-- Open completed run ----> |                               |
 |                           |-- Load timeline data -------> |
 |<-- Timeline view loaded - |                               |
 |                           |                               |
 |-- Pan/zoom hex map ------> |                               |
 |                           |-- LOD switch (if needed) ---> |
 |<-- Updated map state ----- |                               |
 |                           |                               |
 |-- Scrub timeline --------> |                               |
 |                           |-- Fetch tick state ----------> |
 |<-- All views updated ----- |                               |
 |                           |                               |
 |-- "Compare" ------------- > |                               |
 |-- Select baseline run ---> |                               |
 |                           |-- Load baseline run data ---> |
 |<-- Comparison panel ------- |                               |
 |                           |                               |
 |-- Select metrics --------> |                               |
 |-- Select display mode ---> |                               |
 |<-- Comparison charts ------ |                               |
 |<-- Divergence annotation - |                               |
```

---

### 3.3 Flow 3: Trigger Intervention → Replay Branch → Evaluate Metric Deltas

#### 3.3.1 Description

The Policy Analyst (or Scenario Designer) pauses a simulation at a specific tick, applies a counterfactual intervention, runs a new branch from that tick forward, and evaluates how outcomes diverge from the original run.

#### 3.3.2 Preconditions

- An original run exists and is complete.
- The user has write access to the workspace.

#### 3.3.3 Step-by-Step

**Step 1: Navigate to intervention point**
User opens a completed run in the timeline view. User scrubs to the tick where they want to apply the intervention (e.g., tick 3000).

**Step 2: Open intervention panel**
User right-clicks on the timeline at tick 3000 and selects "Create Intervention Branch". Alternatively, user clicks the "Branch" button in the toolbar and enters a tick number. System opens the Intervention Panel.

**Step 3: Define intervention**
The Intervention Panel presents a structured form for the supported intervention types:
- Policy parameter override: select parameter, enter new value (validated against schema bounds).
- Institution shock: select institution, apply a legitimacy delta.
- Resource injection/removal: select resource pool, enter delta value.
- Climate event injection: select event type, duration, and severity.
- Diplomatic state change: select actor pair, change relation state.

User configures the intervention. The panel shows a preview of what changes will be applied at the branch tick.

**Step 4: Name the branch**
User enters a branch label (e.g., "Trade Liberalization at Tick 3000"). System appends a branch ID derived from the parent run ID and the intervention hash.

**Step 5: Submit branch run**
User clicks "Run Branch". System forks the engine state at tick 3000 (using the tick state hash to reconstruct state exactly), applies the intervention, and runs forward with the same RNG seed continuation.

**Step 6: Monitor branch run**
Branch run appears in the run list under the parent run, visually indented to show the branch relationship. Progress indicator shows the branch run status.

**Step 7: Open branch comparison**
Once complete, the system automatically opens the branch comparison view. The original run is shown as the "trunk" and the branch as the "fork". The divergence point (tick 3000) is marked on the timeline.

**Step 8: Evaluate metric deltas**
Delta chart mode is active by default: each metric chart shows (branch_value - trunk_value) over time. Positive delta is colored green; negative delta red. The summary panel shows the metric deltas at the final tick and at the user-annotated evaluation point.

**Step 9: Annotate findings**
User adds annotations to the branch explaining the intervention rationale and interpreting the delta patterns.

**Step 10: Export branch comparison**
User exports the branch comparison as a report package. The report includes both runs, the intervention specification, and the delta charts.

#### 3.3.4 Branching Model Details

- Branch point: any tick T where a tick state hash exists.
- Engine reconstructs state at tick T by replaying from seed to T (deterministic).
- Intervention is applied as a state patch at T+1.
- Branch run uses the same ChaCha20Rng stream advanced to the post-T position, then diverges based on the patched state.
- Branch ID: `{parent_run_id}@branch-T{tick}-{intervention_hash_prefix}`.
- Branches can themselves be branched (tree structure, not just fork).
- Maximum branch depth: 10 (engine limit, surfaced as a UI constraint).

#### 3.3.5 ASCII Flow Diagram

```
User                      Web UI                     Engine / API
 |                           |                               |
 |-- Open completed run ----> |                               |
 |-- Scrub to tick T -------> |                               |
 |-- "Create Intervention  -> |                               |
 |   Branch"                 |                               |
 |                           |                               |
 |-- Define intervention ---> |                               |
 |   (type, params, values)  |-- Validate intervention ----> |
 |                           |<-- Validation result --------- |
 |<-- Preview patch --------- |                               |
 |                           |                               |
 |-- Name branch -----------> |                               |
 |-- "Run Branch" ----------> |                               |
 |                           |-- Fork state at tick T ------> |
 |                           |   (reconstruct via replay)    |
 |                           |-- Apply intervention patch --> |
 |                           |-- Run branch forward -------> |
 |<-- Branch progress -------- |<-- Progress stream (WS) ---- |
 |<-- Branch complete -------- |<-- Completion event (WS) --- |
 |                           |                               |
 |                           |-- Load both runs -----------> |
 |<-- Branch comparison view- |                               |
 |   (trunk + fork aligned)  |                               |
 |                           |                               |
 |-- Review delta charts ---> |                               |
 |-- Add annotations -------> |                               |
 |-- Export report ----------> |                               |
 |                           |-- Generate signed bundle ---> |
 |<-- Download report -------- |                               |
```

---

### 3.4 Flow 4: Export Report Package

#### 3.4.1 Description

The Policy Analyst produces a self-contained, signed export package containing all assumptions, outputs, charts, and annotations from one or more runs.

#### 3.4.2 Step-by-Step

**Step 1: Initiate export**
User opens a run or comparison view and clicks "Export". The Export dialog opens.

**Step 2: Configure export**
Export dialog options:
- Export format: PDF Report, JSON Bundle, CSV Tables, Parquet Dataset, or All (ZIP containing all formats).
- Content scope: Current run only, Selected runs, All runs in comparison panel.
- Include charts: Yes/No (PDF and JSON only).
- Include raw tick data: Yes/No (adds tick-level metric time series).
- Include annotations: Yes/No.
- Include assumption disclosure: always included, cannot be disabled.
- Signing: always applied (BLAKE3 hash of bundle, displayed as artifact fingerprint).

**Step 3: Generate export**
User clicks "Generate". System assembles the bundle. Progress indicator shows assembly stages: Collecting run data, Rendering charts, Compiling tables, Signing artifact.

**Step 4: Download**
Download link appears. User downloads the bundle. The bundle filename encodes: `civlab-export-{run_id_prefix}-{timestamp}.{ext}`.

**Step 5: Verify integrity (optional)**
User can verify the bundle's BLAKE3 hash using `civlab verify {bundle_file}`. Output: PASS or FAIL with the expected and actual hashes.

#### 3.4.3 Report Bundle Format (JSON)

```json
{
  "civlab_export_version": "1.0",
  "generated_at": "2026-02-21T14:30:00Z",
  "engine_version": "0.9.0",
  "artifact_fingerprint": "<BLAKE3 hex>",
  "runs": [
    {
      "run_id": "abc123def456-seed42-20260221",
      "scenario_hash": "<BLAKE3 hex>",
      "scenario_label": "Pre-Industrial Island State v1.2",
      "seed": 42,
      "tick_count": 10000,
      "tick_hashes": ["<BLAKE3>", "..."],
      "config": { /* full scenario TOML as JSON */ },
      "assumptions": { /* annotated parameter rationale */ },
      "mods_active": [],
      "metrics_summary": { /* endpoint values per metric */ },
      "metrics_timeseries": { /* tick-indexed per metric */ },
      "annotations": [
        {
          "author": "Tariq",
          "tick_range": [4200, 4250],
          "metric": "legitimacy",
          "text": "Legitimacy collapse triggered by tax shock at tick 4195"
        }
      ]
    }
  ],
  "charts": [
    {
      "chart_id": "legitimacy-timeseries",
      "run_ids": ["abc123def456-seed42-20260221"],
      "metric": "legitimacy",
      "format": "svg",
      "data_uri": "data:image/svg+xml;base64,..."
    }
  ]
}
```

---

### 3.5 Flow 5: Parameter Sweep (Research Operator)

#### 3.5.1 Description

The Research Operator executes a large-scale parameter sweep over a scenario configuration, collects results, and produces an aggregated statistical dataset.

#### 3.5.2 Sweep Manifest Format

```toml
# sweep_manifest.toml
[sweep]
scenario = "scenarios/island_state_v1.2.toml"
seeds = [1, 2, 3, 4, 5]           # 5 seeds per parameter combination
max_concurrent = 8                  # parallel run limit
output_dir = "./sweep_output"
output_formats = ["csv", "parquet", "json"]
stability_check = true              # exclude degenerate runs

[[sweep.parameters]]
name = "policy.tax_rate"
type = "linspace"
min = 0.05
max = 0.40
steps = 8

[[sweep.parameters]]
name = "policy.trade_openness"
type = "values"
values = [0.0, 0.5, 1.0]

[sweep.metrics]
endpoint_ticks = [5000, 10000]
time_series = true
```

#### 3.5.3 CLI Execution

```bash
# Run the sweep
civlab sweep --manifest sweep_manifest.toml

# With progress output
civlab sweep --manifest sweep_manifest.toml --progress

# Dry run (validate manifest, count runs)
civlab sweep --manifest sweep_manifest.toml --dry-run

# Resume interrupted sweep
civlab sweep --manifest sweep_manifest.toml --resume ./sweep_output
```

#### 3.5.4 Output Directory Structure

```
sweep_output/
  manifest.toml                   # Copy of sweep manifest
  sweep_id.txt                    # Unique sweep ID
  runs/
    {run_id}/
      config.toml                 # Effective config for this run
      metrics_timeseries.parquet  # Tick-level metrics
      metrics_endpoint.json       # Endpoint summary
      tick_hashes.bin             # BLAKE3 hashes per tick
      status.json                 # Run status and metadata
  aggregate/
    summary.csv                   # Mean/CI per parameter combo
    sensitivity.csv               # Spearman correlations
    degenerate_runs.json          # Flagged unstable runs
    full_dataset.parquet          # All runs, all metrics, all ticks
```

#### 3.5.5 Aggregation Command

```bash
# Aggregate completed sweep
civlab aggregate --input ./sweep_output --output ./analysis

# Specific aggregation options
civlab aggregate \
  --input ./sweep_output \
  --output ./analysis \
  --metrics legitimacy,tyranny,resilience \
  --ci 0.95 \
  --exclude-degenerate \
  --format parquet,csv
```

#### 3.5.6 ASCII Flow Diagram

```
Research Operator           civlab CLI                  Engine Pool
 |                               |                           |
 |-- Write sweep_manifest.toml   |                           |
 |-- civlab sweep --manifest --> |                           |
 |                               |-- Validate manifest ----> |
 |                               |-- Compute run matrix ---> |
 |                               |   (params x seeds)        |
 |<-- Dry run count: 120 runs -- |                           |
 |-- (confirm, run for real) --> |                           |
 |                               |-- Spawn workers (N=8) --> |
 |                               |   Per worker:             |
 |                               |   - Submit run            |
 |                               |   - Poll completion       |
 |                               |   - Write artifacts       |
 |<-- Progress: [========  ] --> |<-- Run completions ------- |
 |   (runs done / total)         |                           |
 |                               |-- [All runs complete]     |
 |<-- Sweep complete ----------- |                           |
 |   (summary: N ok, M skipped)  |                           |
 |                               |                           |
 |-- civlab aggregate ----------> |                           |
 |   --input ./sweep_output      |-- Read run artifacts      |
 |                               |-- Compute statistics      |
 |                               |-- Detect degenerate runs  |
 |                               |-- Write aggregate outputs |
 |<-- Aggregation complete ------- |                           |
 |   (summary.csv, full.parquet) |                           |
```

---

### 3.6 Flow 6: Mod Authoring → Test → Publish

#### 3.6.1 Description

The Modder authors a WASM mod using the civlab-sdk crate, tests it locally, and publishes it to the mod registry.

#### 3.6.2 Step-by-Step

**Step 1: Initialize mod project**

```bash
civlab mod new --type policy --name my_tax_policy
# Creates:
#   my_tax_policy/
#     Cargo.toml        (civlab-sdk dep pre-configured)
#     src/lib.rs        (PolicyMod trait skeleton)
#     tests/            (test harness scaffold)
#     civmod.toml       (mod manifest)
```

**Step 2: Implement the mod trait**

Mod author implements the required trait in `src/lib.rs`. The SDK provides:
- `PolicyMod::apply(&self, state: &SimState, tick: u64) -> PolicyEffect`
- `EconomicMod::compute_production(&self, state: &SimState) -> ResourceDelta`
- `EventMod::should_fire(&self, state: &SimState, rng: &mut EngineRng) -> Option\<Event\>`

**Step 3: Build**

```bash
cargo build --target wasm32-wasip2 --release
# Output: target/wasm32-wasip2/release/my_tax_policy.wasm
```

**Step 4: Run local test harness**

```bash
civlab mod test \
  ./target/wasm32-wasip2/release/my_tax_policy.wasm \
  --scenario examples/island_state_v1.2.toml \
  --ticks 1000 \
  --verify-determinism \
  --verbose
```

Test harness output:
- Hook call counts per tick
- Metric effect summary
- Determinism verification (run twice, compare hashes)
- Sandbox constraint violations (if any)
- Execution time per hook call (flagged if > 10ms limit)

**Step 5: Package and sign**

```bash
civlab mod package \
  ./target/wasm32-wasip2/release/my_tax_policy.wasm \
  --manifest civmod.toml \
  --sign  # Uses local signing key from civlab keychain
# Output: my_tax_policy-0.1.0.civmod
```

**Step 6: Publish**

```bash
civlab mod publish ./my_tax_policy-0.1.0.civmod
# Submits to registry, awaits signature verification
# Returns: registry URL for mod
```

---

## 4. UX Requirements (FR Format)

### 4.1 Provenance Requirements

**FR-UX-001: Every chart must display its provenance.**
Every chart rendered in the web UI must display a provenance badge showing: run ID (truncated to 8 chars), scenario label and version, seed, and tick range. Clicking the provenance badge opens the full provenance panel with complete metadata. Provenance must be present in all exported chart SVGs as embedded metadata.

**FR-UX-002: Exported artifacts must include full assumption disclosure.**
Every export package (PDF, JSON, CSV, Parquet) must include the full scenario configuration used to produce it. This cannot be disabled by the user. The assumption disclosure section must appear on page 1 of the PDF report.

**FR-UX-003: Run IDs must be stable, unique, and human-readable.**
Run IDs must be deterministic from the inputs: `{scenario_hash_8char}-s{seed}-{ISO8601_compact}`. Format example: `a3f8b21c-s42-20260221T143000Z`. The same scenario + seed + timestamp always produces the same run ID. Run IDs must be displayed wherever a run is referenced.

**FR-UX-004: Replay references must be self-contained.**
A replay reference must encode everything needed to reproduce the run: scenario hash, seed, engine version. Format: `civlab://replay/{scenario_hash}/{seed}/{engine_version}`. The web UI must render replay references as copyable links. The CLI must accept replay references as arguments to `civlab replay`.

**FR-UX-005: Tick state hashes must be visible in timeline view.**
The timeline view must provide access to the BLAKE3 hash of each tick's state. Users must be able to copy a tick hash from the timeline. The tick hash must be included in exported artifacts.

### 4.2 Run ID and Replay Reference Display

**FR-UX-006: Run list must display run ID, scenario label, seed, status, and tick count.**
The run list panel must show these fields for each run in a scannable table format. Status is one of: Queued, Running, Complete, Failed, Cancelled. Completed runs show tick count and total wall clock time.

**FR-UX-007: Run header must be persistent during timeline inspection.**
When inspecting a run, a persistent header bar must display: run ID, scenario label, version, seed, and total tick count. The header must remain visible while scrolling the timeline or panning the hex map.

**FR-UX-008: Comparison panel must display regime labels prominently.**
When the user assigns regime labels to runs in a comparison, those labels must appear in chart legends, axis annotations, and the comparison summary panel. Labels must persist in exported reports.

### 4.3 Warning State System

**FR-UX-009: Low-confidence states must be surfaced with a visible warning indicator.**
If any metric enters a low-confidence state (defined as: the metric value is within the top or bottom 5% of the parameter's historical distribution across all completed runs, or the metric has exceeded a predefined instability threshold), the metric chart must display an amber warning icon and a tooltip explaining the cause. The timeline must show a low-confidence band overlay on the affected tick range.

**FR-UX-010: Determinism violations must trigger an immediate error state.**
If the engine detects a determinism violation (tick hash mismatch on replay), the affected run must be marked with a red "Determinism Violation" badge. All charts and tables from that run must display a prominent warning that data integrity is uncertain. The user must acknowledge the warning before proceeding. The violation must be logged with the specific tick, expected hash, and actual hash.

**FR-UX-011: Unstable simulation states must halt the run and surface an error.**
If the engine halts a run due to an unstable state (NaN, Inf, or degenerate metric value), the run status must show "Failed - Instability" with a detailed error panel showing: the tick of failure, the metric that triggered failure, the last known stable state hash, and a suggested diagnosis. The run must not be marked as Complete.

**FR-UX-012: Mod-active runs must display a persistent mod badge.**
Any run where mods are active must display a "Mods Active" badge in the run header. The badge must list the active mods with their registry names and versions. Clicking the badge opens the mod details panel. Mod-active status must appear in all exports.

**FR-UX-013: Configuration schema violations must block scenario commit.**
If the scenario config contains any schema violation, the "Commit Version" action must be disabled. The UI must display a count of blocking violations with a link to the validation report. Committing a schema-invalid scenario is not possible by any code path.

### 4.4 Cognitive Load Requirements for Branching and Rollback

**FR-UX-014: Branch origin must be persistently visible during branch run inspection.**
When inspecting a branch run, the timeline must display a vertical marker at the branch tick labeled with the branch label. The trunk run must be shown as a faded overlay in metric charts. The branch origin (parent run ID, branch tick) must be displayed in the branch run header.

**FR-UX-015: Branch tree must be navigable from a visual run tree panel.**
A run tree panel must show the parent-child relationships between runs as an expandable tree. Trunk runs are shown at the top level; branch runs are indented under their parent. Clicking any run in the tree opens that run's timeline view.

**FR-UX-016: Rollback to a prior branch point must be one action.**
The user must be able to create a new branch from any tick in any completed run without navigating away from the current view. The action "Branch from here" must be available via right-click on the timeline and via the toolbar.

**FR-UX-017: Destructive actions must require explicit confirmation with consequences stated.**
Any action that would delete or overwrite simulation data must present a confirmation dialog that states specifically what will be deleted and cannot be undone. The dialog must require the user to type a confirmation string (the run ID or scenario name) before the action proceeds.

### 4.5 Accessibility Requirements

**FR-UX-018: All interactive elements must meet WCAG 2.1 AA contrast requirements.**
Text and interactive elements must have a minimum contrast ratio of 4.5:1 against their background. Large text (18pt+ or 14pt bold+) must have a minimum contrast ratio of 3:1. Color must not be the only means of conveying information.

**FR-UX-019: All interactive elements must be keyboard-accessible.**
Every action available via mouse must be accessible via keyboard. Tab order must be logical and follow reading order. Focus indicators must be visible (2px solid outline, minimum). Custom keyboard shortcuts must be documented in a keyboard shortcut panel accessible via `?`.

**FR-UX-020: Screen reader support for simulation state panels.**
All metric panels, event log entries, and timeline annotations must have appropriate ARIA labels and roles. Dynamic content updates (run status changes, metric updates) must use `aria-live` regions with `polite` politeness. The hex map must expose a text-based alternative view (tabular grid of region metrics) accessible via screen reader.

**FR-UX-021: Color-blind safe palette required for all metric visualizations.**
All charts and hex map coloring must use a color-blind safe palette. Default palette must be distinguishable under deuteranopia, protanopia, and tritanopia. A palette selector must allow users to choose from: Default (color-blind safe), High Contrast, Monochrome. Palette preference must persist across sessions.

**FR-UX-022: Font sizes must be user-adjustable.**
Base font size must be adjustable from 12px to 24px in the application settings. All layout must accommodate the full range without horizontal scrollbars or overlapping elements at viewport widths >= 1024px.

### 4.6 Performance Requirements

**FR-UX-023: Timeline scrubber response must be under 200ms.**
From the moment the user releases the scrubber handle to the moment all visible views (hex map, metric charts, event log) reflect the selected tick, no more than 200ms must elapse. This must hold for runs up to 100,000 ticks.

**FR-UX-024: Hex map pan and zoom must achieve 60fps.**
Pan and zoom interactions on the hex map must render at 60 frames per second on target hardware (defined as: laptop with integrated GPU, 1080p display). Frame drops below 45fps must trigger an automatic LOD reduction.

**FR-UX-025: Run list must load in under 500ms for up to 500 runs.**
The run list panel must be populated within 500ms of navigation to the runs view, for workspaces containing up to 500 runs. Pagination kicks in above 500 runs.

**FR-UX-026: Export generation must not block the UI.**
Export bundle generation must run asynchronously. The UI must remain interactive during export. A progress indicator must show export stages. The user must be able to cancel an in-progress export.

**FR-UX-027: Comparison panel must handle up to 8 simultaneous runs.**
The comparison panel must support up to 8 runs simultaneously without exceeding 500ms render time for metric chart updates. Above 4 runs, the layout switches from side-by-side to overlay mode.

---

## 5. Information Architecture

### 5.1 Data Hierarchy

```
Workspace
  └── Scenario Collection
        └── Scenario
              ├── Scenario Versions (immutable snapshots)
              │     └── Version (scenario_hash, label, timestamp, config)
              └── Runs
                    ├── Run (run_id, seed, status, tick_count)
                    │     ├── Tick States (hash per tick)
                    │     ├── Metric Time Series (per metric, all ticks)
                    │     ├── Event Log (events per tick)
                    │     ├── Annotations (per tick range, per analyst)
                    │     └── Branch Runs (tree, max depth 10)
                    └── Sweep (sweep_id, manifest, aggregate outputs)
                          └── Sweep Runs (run_id per param combo x seed)
```

### 5.2 Screen / View Map

```
App Root
  ├── Workspace Dashboard
  │     ├── Scenario Collection Browser
  │     ├── Recent Runs Panel
  │     └── Workspace Settings
  ├── Scenarios
  │     ├── Scenario List
  │     ├── Scenario Detail
  │     │     ├── Version History
  │     │     ├── Version Diff
  │     │     └── Run List (for this scenario)
  │     └── Scenario Editor
  │           ├── Parameter Editor (schema-aware)
  │           ├── Annotation Panel
  │           ├── Validation Report
  │           └── Template Browser
  ├── Runs
  │     ├── Run List (workspace-level)
  │     ├── Run Detail
  │     │     ├── Timeline View
  │     │     │     ├── Hex Map (strategic LOD)
  │     │     │     ├── City View (operational LOD)
  │     │     │     ├── Timeline Scrubber
  │     │     │     ├── Metric Sparklines Panel
  │     │     │     └── Event Log Panel
  │     │     ├── Run Tree Panel
  │     │     └── Annotation Manager
  │     └── Comparison Panel
  │           ├── Run Selector
  │           ├── Metric Selector
  │           ├── Comparison Charts
  │           └── Divergence Annotation
  ├── Sweeps
  │     ├── Sweep List
  │     ├── Sweep Detail
  │     │     ├── Sweep Progress
  │     │     ├── Aggregate Summary
  │     │     └── Sensitivity Analysis View
  │     └── Sweep Export
  ├── Mods
  │     ├── Mod Registry Browser
  │     ├── Active Mods Panel
  │     └── Mod Detail
  ├── Export
  │     └── Export Configuration Dialog
  └── Settings
        ├── Workspace Settings
        ├── Accessibility (palette, font size)
        ├── Keyboard Shortcuts
        └── API Keys
```

### 5.3 Navigation Structure

**Primary navigation:** Left sidebar with icons + labels for: Workspace, Scenarios, Runs, Sweeps, Mods, Settings.

**Secondary navigation:** Breadcrumb trail within each section (e.g., Scenarios > Island State v1.2 > Run abc123).

**Contextual navigation:** Right-click context menus on timeline, hex map, and run list items for quick-access actions.

**Run tree navigation:** The run tree panel provides hierarchy navigation within a scenario's runs, showing parent-branch relationships.

**Keyboard navigation map:**

| Shortcut | Action |
|---|---|
| `G S` | Go to Scenarios |
| `G R` | Go to Runs |
| `G W` | Go to Sweeps |
| `G M` | Go to Mods |
| `?` | Open keyboard shortcuts panel |
| `Ctrl+K` | Open command palette |
| `[` | Scrub timeline backward 100 ticks |
| `]` | Scrub timeline forward 100 ticks |
| `Shift+[` | Scrub timeline backward 1000 ticks |
| `Shift+]` | Scrub timeline forward 1000 ticks |
| `C` | Open comparison panel |
| `B` | Create branch from current timeline position |
| `E` | Open export dialog |
| `A` | Add annotation at current timeline position |

---

## 6. Interaction Patterns

### 6.1 Timeline Scrubber Interaction Model

**Component:** Horizontal scrubber bar spanning the full width of the timeline view.

**Visual elements:**
- Track: full-width bar representing the run duration (tick 0 to max tick).
- Handle: draggable circular indicator showing current tick position.
- Tick labels: major tick markers at regular intervals (auto-scaled to run length).
- Annotation markers: vertical tick marks on the track at annotated ticks.
- Low-confidence bands: amber overlay regions on the track.
- Branch point marker: vertical line at branch tick (branch runs only).
- Event density heatmap: subtle color intensity on the track showing event frequency per tick range.

**Interactions:**
- Click anywhere on the track: jump to that tick position.
- Click and drag the handle: scrub continuously; views update at 60fps during drag.
- Release handle: final tick position committed; all views update to final state.
- Keyboard `[` / `]`: step backward/forward 100 ticks.
- Keyboard `Shift+[` / `Shift+]`: step backward/forward 1000 ticks.
- Keyboard `Home` / `End`: jump to tick 0 or max tick.
- Right-click on track: context menu with "Add Annotation", "Create Branch", "Copy Tick Hash".
- Scroll wheel over scrubber: fine adjustment &plusmn;1 tick per scroll click.

**Performance contract:** All view updates triggered by scrubber interaction must complete within 200ms. If data for a tick is not yet in the client cache, a skeleton loader appears immediately and is replaced when data loads. The scrubber handle moves immediately on drag; views may lag up to 200ms.

### 6.2 Hex Map Pan / Zoom Gestures

**Component:** Pixi.js v8 canvas rendering hex tile grid.

**Strategic LOD (default):**
- Hex tiles represent geographic regions (e.g., 50km per hex).
- Each tile shows: color coding by metric (legitimacy gradient by default), resource icon if a significant resource is present, institution status icon if an institution is active.
- Hovering a tile: tooltip with all eight core metric aggregates for that region at current tick.
- Clicking a tile: selects the region; metric charts filter to that region; event log filters to events in that region.
- Double-clicking a tile: transitions to operational LOD for that region.

**Operational LOD (city-level):**
- Shows city layout with building footprints, citizen cohort indicators, resource flow arrows.
- Breadcrumb displays: `Strategic View > [Region Name] > Operational View`.
- Clicking "Zoom Out" in breadcrumb or pressing `Escape`: returns to strategic LOD.

**Gesture model:**
- Pan: left-click drag.
- Zoom: scroll wheel (smooth, not stepped). Pinch-to-zoom on touchpad.
- Reset zoom: double-click on empty canvas area, or press `Home` while map is focused.
- LOD transition: automatic when zoom level crosses the LOD threshold. Manual via double-click on tile.

**Performance contract:** Pan and zoom at 60fps. LOD switch completes within 300ms including data fetch. Maximum map size supported: 1024x1024 hex grid (tiled rendering; only visible tiles rendered).

### 6.3 Intervention Panel Interaction

**Trigger:** Right-click on timeline at target tick > "Create Intervention Branch", or toolbar "Branch" button.

**Panel layout:**
- Header: "New Branch from Tick {T}"
- Branch label field: text input, required, max 64 chars.
- Intervention type selector: dropdown (Policy Override, Institution Shock, Resource Delta, Climate Event, Diplomatic Change).
- Intervention form: dynamically rendered based on type selection.
- Preview panel: shows the exact state delta that will be applied at tick T+1 as a JSON diff.
- Validation indicator: green checkmark or amber warning based on intervention validity.
- "Run Branch" button: disabled until intervention is valid and branch label is non-empty.
- "Cancel" button: closes panel without creating branch.

**Intervention form fields by type:**

| Type | Fields |
|---|---|
| Policy Override | Parameter selector (searchable), Current value (read-only), New value (validated input) |
| Institution Shock | Institution selector, Legitimacy delta (-1.0 to +1.0), Duration (ticks) |
| Resource Delta | Resource pool selector, Delta value, Unit (read-only) |
| Climate Event | Event type selector, Severity (0.0-1.0), Duration (ticks) |
| Diplomatic Change | Actor A selector, Actor B selector, Relation state selector |

### 6.4 Comparison Panel Layout

**Default layout (2 runs):**
- Left panel: Run A timeline and metric charts.
- Right panel: Run B timeline and metric charts.
- Both panels share a synchronized timeline scrubber.
- A center divider with a swap button allows swapping run positions.

**Overlay mode (3-8 runs):**
- Single chart area with all runs overlaid.
- Legend shows run ID truncated + regime label for each run.
- Line style differentiation: solid, dashed, dotted, dash-dot (cycles for runs 5-8).
- Color assignment: color-blind safe palette, one color per run.

**Display mode toggle:**
- "Absolute" mode: shows raw metric values. Y-axis scaled to accommodate all runs.
- "Delta" mode: shows (run_value - baseline_value) per tick. Baseline run is selectable.
- "Normalized" mode: shows each run normalized to its own tick-0 value (percentage change).

**Metric selector:** Checkbox grid with all eight core metrics plus any mod-contributed metrics. Selection persists within the session.

**Divergence annotation:** Automatically computed as the first tick where any selected metric diverges beyond 5% between any two runs (configurable threshold). Displayed as a vertical dashed line across all charts with a label showing the diverging metric and magnitude.

### 6.5 Notification and Alert System

**Alert levels:**

| Level | Color | Persistence | User Action Required |
|---|---|---|---|
| Info | Blue | Auto-dismiss 5s | None |
| Warning | Amber | Persistent until dismissed | Dismiss |
| Error | Red | Persistent until acknowledged | Acknowledge |
| Critical | Red + border | Blocks interaction | Acknowledge + confirm |

**Notification types and their level:**

| Event | Level |
|---|---|
| Run completed | Info |
| Run failed (instability) | Error |
| Determinism violation detected | Critical |
| Export complete | Info |
| Validation error count | Warning |
| Mod integrity verification failed | Error |
| Workspace approaching storage limit | Warning |
| Sweep completed with degenerate runs | Warning |

**Notification panel:** Accessible via bell icon in top nav. Shows last 50 notifications with timestamp, level, and description. Persistent notifications remain until the underlying issue is resolved (not just dismissed).

---

## 7. CLI Interface

### 7.1 Command Structure

```
civlab <command> [subcommand] [flags] [args]
```

Top-level commands:

| Command | Description |
|---|---|
| `civlab run` | Execute a single simulation run |
| `civlab sweep` | Execute a parameter sweep |
| `civlab replay` | Replay a run from a run ID or replay reference |
| `civlab export` | Export run data or report bundles |
| `civlab aggregate` | Aggregate sweep outputs into statistics |
| `civlab verify` | Verify integrity of an export bundle |
| `civlab mod` | Mod management subcommands |
| `civlab scenario` | Scenario management subcommands |
| `civlab config` | CivLab CLI configuration |
| `civlab version` | Print version information |

### 7.2 `civlab run`

```
USAGE:
    civlab run [FLAGS] <scenario>

ARGS:
    <scenario>    Path to scenario TOML/JSON file, or scenario ID

FLAGS:
    -s, --seed <seed>              RNG seed (default: randomly generated)
    -t, --ticks <ticks>            Number of ticks to simulate (default: scenario default)
    -o, --output <dir>             Output directory (default: ./civlab_runs/{run_id})
    -f, --format <format>          Output format: json, csv, parquet, all (default: json)
        --mods <mod1,mod2>         Comma-separated mod IDs to activate
        --label <label>            Human-readable run label
        --no-timeseries            Suppress tick-level time series output (endpoint only)
        --stream                   Stream tick data to stdout as JSON lines
        --max-ticks <n>            Hard stop at n ticks regardless of scenario config
    -q, --quiet                    Suppress progress output
    -v, --verbose                  Verbose logging (repeat for more: -vv, -vvv)
        --format <format>          Output format for CLI messages: text, json (default: text)
    -h, --help                     Print help

EXAMPLES:
    # Run with auto-generated seed, default output
    civlab run scenarios/island_state_v1.2.toml

    # Run with specific seed and label
    civlab run scenarios/island_state_v1.2.toml --seed 42 --label "Baseline Run"

    # Stream tick data to stdout
    civlab run scenarios/island_state_v1.2.toml --seed 42 --stream | jq '.metrics.legitimacy'

    # Run with mods, output as Parquet
    civlab run scenarios/island_state_v1.2.toml --mods progressive_tax_v1 --format parquet

EXIT CODES:
    0    Run completed successfully
    1    Run failed due to simulation instability
    2    Configuration error (invalid scenario file)
    3    Engine error (unexpected engine failure)
    4    Mod load error
    5    Output write error
```

### 7.3 `civlab sweep`

```
USAGE:
    civlab sweep [FLAGS] --manifest <manifest>

FLAGS:
    -m, --manifest <manifest>      Path to sweep manifest TOML file (required)
    -o, --output <dir>             Output directory (default: ./civlab_sweeps/{sweep_id})
        --resume <dir>             Resume interrupted sweep from output directory
        --dry-run                  Validate manifest and print run count without executing
        --max-concurrent <n>       Max parallel runs (default: CPU cores / 2)
        --format <format>          Output format: json, csv, parquet, all (default: parquet,csv)
        --no-timeseries            Suppress tick-level outputs; endpoint metrics only
    -p, --progress                 Show progress bar and ETA
    -q, --quiet                    Suppress all output except errors
        --format <format>          CLI message format: text, json (default: text)
    -h, --help                     Print help

EXAMPLES:
    # Run sweep from manifest
    civlab sweep --manifest sweep_manifest.toml --progress

    # Dry run to validate and count
    civlab sweep --manifest sweep_manifest.toml --dry-run
    # Output: Sweep manifest valid. 120 runs (8 param combos x 5 seeds x 3 configs).

    # Resume interrupted sweep
    civlab sweep --manifest sweep_manifest.toml --resume ./civlab_sweeps/sweep_abc123

    # JSON output for pipeline integration
    civlab sweep --manifest sweep_manifest.toml --format json --quiet

EXIT CODES:
    0    All runs completed successfully
    1    Partial failure (some runs failed; aggregate computed over successful runs)
    2    Manifest validation error
    3    All runs failed
    4    Resume directory not found or corrupt
    5    Output write error
```

### 7.4 `civlab replay`

```
USAGE:
    civlab replay [FLAGS] <run-ref>

ARGS:
    <run-ref>    Run ID, replay reference URI (civlab://replay/...), or path to run artifact dir

FLAGS:
    -o, --output <dir>             Output directory
    -t, --from-tick <tick>         Replay starting from this tick (requires full artifact)
        --verify                   Verify tick hashes during replay and report any mismatch
    -f, --format <format>          Output format: json, csv, parquet (default: json)
        --stream                   Stream replayed tick data to stdout as JSON lines
    -q, --quiet                    Suppress progress output
    -h, --help                     Print help

EXAMPLES:
    # Replay a run by ID (requires engine to have original scenario and seed)
    civlab replay a3f8b21c-s42-20260221T143000Z

    # Replay from a replay reference URI
    civlab replay "civlab://replay/a3f8b21c/42/0.9.0"

    # Replay and verify all tick hashes
    civlab replay a3f8b21c-s42-20260221T143000Z --verify

    # Replay from tick 3000 forward
    civlab replay ./civlab_runs/a3f8b21c-s42-20260221T143000Z --from-tick 3000

EXIT CODES:
    0    Replay completed; all hashes verified (if --verify)
    1    Determinism violation detected (hash mismatch)
    2    Run reference not found
    3    Engine version mismatch
    4    Incomplete artifact (cannot replay)
```

### 7.5 `civlab export`

```
USAGE:
    civlab export [FLAGS] <run-id-or-dir>

ARGS:
    <run-id-or-dir>    Run ID or path to run artifact directory

FLAGS:
    -o, --output <path>            Output path for export bundle
    -f, --format <format>          Export format: pdf, json, csv, parquet, all (default: json)
        --include-timeseries       Include full tick-level time series (default: endpoint only)
        --include-charts           Generate and include SVG charts (default: false for CLI)
        --metrics <list>           Comma-separated metric names to include (default: all)
    -h, --help                     Print help

EXAMPLES:
    # Export run as JSON bundle
    civlab export a3f8b21c-s42-20260221T143000Z --format json

    # Export as all formats
    civlab export ./civlab_runs/a3f8b21c-s42-20260221T143000Z --format all

    # Export specific metrics as CSV with time series
    civlab export a3f8b21c-s42-20260221T143000Z \
      --format csv \
      --metrics legitimacy,tyranny,resilience \
      --include-timeseries

EXIT CODES:
    0    Export completed and written to output path
    1    Run not found
    2    Format not supported
    3    Output write error
```

### 7.6 `civlab aggregate`

```
USAGE:
    civlab aggregate [FLAGS] --input <sweep-dir>

FLAGS:
    -i, --input <sweep-dir>        Path to sweep output directory (required)
    -o, --output <dir>             Output directory for aggregate files (default: {sweep-dir}/aggregate)
    -f, --format <format>          Output format: csv, json, parquet, all (default: csv,parquet)
        --metrics <list>           Comma-separated metric names to aggregate (default: all)
        --ci <level>               Confidence interval level: 0.90, 0.95, 0.99 (default: 0.95)
        --exclude-degenerate       Exclude flagged degenerate runs from aggregates
        --endpoint-ticks <list>    Comma-separated tick values for endpoint metrics (default: final tick)
        --sensitivity              Compute Spearman rank correlations (parameter sensitivity analysis)
    -h, --help                     Print help

EXAMPLES:
    # Basic aggregation
    civlab aggregate --input ./civlab_sweeps/sweep_abc123

    # Aggregation with sensitivity analysis, 95% CI, exclude degenerate
    civlab aggregate \
      --input ./civlab_sweeps/sweep_abc123 \
      --output ./analysis \
      --ci 0.95 \
      --exclude-degenerate \
      --sensitivity \
      --format parquet,csv

EXIT CODES:
    0    Aggregation completed successfully
    1    Input directory not found or invalid sweep structure
    2    No successful runs to aggregate (all degenerate or failed)
    3    Output write error
```

### 7.7 `civlab verify`

```
USAGE:
    civlab verify [FLAGS] <bundle-path>

ARGS:
    <bundle-path>    Path to export bundle file

FLAGS:
    -h, --help    Print help

OUTPUT:
    PASS  artifact_fingerprint: <BLAKE3 hex>
    FAIL  artifact_fingerprint: <BLAKE3 hex> (expected), <BLAKE3 hex> (actual)

EXIT CODES:
    0    Bundle integrity verified (PASS)
    1    Bundle integrity failed (FAIL)
    2    Bundle file not found or cannot be read
    3    Bundle format not recognized
```

### 7.8 `civlab mod` Subcommands

```
civlab mod new --type <type> --name <name>
    # Initialize a new mod project scaffold
    # Types: policy, economic, event, scenario

civlab mod test <wasm-path> [FLAGS]
    # Test a WASM mod against a scenario
    FLAGS:
        --scenario <path>          Scenario file to test against (required)
        --ticks <n>                Number of ticks to test (default: 1000)
        --verify-determinism       Run twice, compare hashes (verifies mod purity)
        --verbose                  Show per-hook call log

civlab mod package <wasm-path> [FLAGS]
    # Package and sign a mod artifact
    FLAGS:
        --manifest <civmod.toml>   Mod manifest file (default: ./civmod.toml)
        --sign                     Sign with local signing key

civlab mod publish <civmod-path>
    # Publish mod to the registry

civlab mod list
    # List installed/available mods

civlab mod inspect <mod-id>
    # Show mod metadata, hook declarations, and sandbox requirements
```

---

## 8. API Interface

### 8.1 REST Endpoints for Scenario Management

**Base URL:** `http://localhost:7777/api/v1` (local engine server)

All requests and responses use `Content-Type: application/json`.

**Authentication:** Bearer token in `Authorization` header. Token obtained from `civlab config auth`.

#### Scenarios

```
GET    /scenarios                    List all scenarios in workspace
POST   /scenarios                    Create new scenario (body: scenario config JSON)
GET    /scenarios/{id}               Get scenario metadata
GET    /scenarios/{id}/versions      List all versions of a scenario
POST   /scenarios/{id}/versions      Commit a new scenario version (body: config + label)
GET    /scenarios/{id}/versions/{v}  Get specific version (config + metadata)
POST   /scenarios/validate           Validate a scenario config (body: config JSON)
```

**POST /scenarios/validate response:**

```json
{
  "valid": false,
  "errors": [
    {
      "path": "policy.tax_rate",
      "message": "Value 0.95 exceeds maximum allowed value 0.80",
      "current_value": 0.95,
      "constraint": "max: 0.80"
    }
  ],
  "warnings": []
}
```

#### Runs

```
GET    /runs                         List all runs in workspace
POST   /runs                         Submit a new run (body: {scenario_id, version, seed, label, mods})
GET    /runs/{run_id}                Get run metadata and status
DELETE /runs/{run_id}                Delete run (requires confirmation token)
GET    /runs/{run_id}/metrics        Get endpoint metric summary
GET    /runs/{run_id}/metrics/tick/{t}  Get metric values at specific tick
GET    /runs/{run_id}/tick-hashes    Get all tick hashes (paginated)
GET    /runs/{run_id}/events         Get event log (paginated, filterable by tick range)
POST   /runs/{run_id}/annotations    Add annotation to run
GET    /runs/{run_id}/annotations    List annotations for run
POST   /runs/{run_id}/branch         Create branch run from tick T (body: {tick, intervention, label})
GET    /runs/{run_id}/branches       List all branch runs of a run
```

**POST /runs request body:**

```json
{
  "scenario_id": "scen_abc123",
  "version": "v1.2",
  "seed": 42,
  "label": "Baseline Run",
  "mods": [],
  "ticks": 10000
}
```

**GET /runs/{run_id} response:**

```json
{
  "run_id": "a3f8b21c-s42-20260221T143000Z",
  "scenario_id": "scen_abc123",
  "scenario_hash": "a3f8b21c...",
  "scenario_label": "Island State v1.2",
  "seed": 42,
  "label": "Baseline Run",
  "status": "complete",
  "tick_count": 10000,
  "created_at": "2026-02-21T14:30:00Z",
  "completed_at": "2026-02-21T14:32:45Z",
  "wall_clock_seconds": 165,
  "mods_active": [],
  "parent_run_id": null,
  "branch_tick": null
}
```

#### Sweeps

```
POST   /sweeps                       Submit a sweep job (body: sweep manifest JSON)
GET    /sweeps/{sweep_id}            Get sweep status and progress
GET    /sweeps/{sweep_id}/runs       List all runs in a sweep
GET    /sweeps/{sweep_id}/aggregate  Get aggregate statistics (if computed)
POST   /sweeps/{sweep_id}/aggregate  Trigger aggregation computation
DELETE /sweeps/{sweep_id}            Cancel and delete sweep
```

#### Export

```
POST   /export                       Request export bundle generation
    Body: { run_ids: [...], format: "json|pdf|csv|parquet|all", options: {...} }
    Response: { export_id: "...", status: "queued" }

GET    /export/{export_id}           Get export status
GET    /export/{export_id}/download  Download completed export bundle
```

### 8.2 WebSocket JSON-RPC for Live Run Data

**Endpoint:** `ws://localhost:7777/ws`

**Protocol:** JSON-RPC 2.0 over WebSocket.

#### Subscribe to Run Progress

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "subscribe_run",
  "params": {
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "tick_interval": 50
  }
}

// Response (immediate)
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "subscription_id": "sub_xyz",
    "run_id": "a3f8b21c-s42-20260221T143000Z"
  }
}

// Streaming events (push, no id)
{
  "jsonrpc": "2.0",
  "method": "run_tick",
  "params": {
    "subscription_id": "sub_xyz",
    "tick": 100,
    "tick_hash": "...",
    "metrics": {
      "waste": 0.12,
      "surplus": 0.45,
      "tyranny": 0.08,
      "legitimacy": 0.82,
      "resilience": 0.67,
      "joule_stock": 142500,
      "trade_balance": 3200,
      "population": 48000
    }
  }
}

// Completion event
{
  "jsonrpc": "2.0",
  "method": "run_complete",
  "params": {
    "subscription_id": "sub_xyz",
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "final_tick": 10000,
    "final_tick_hash": "..."
  }
}
```

#### Get Tick State

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "get_tick_state",
  "params": {
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "tick": 4200
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tick": 4200,
    "tick_hash": "...",
    "metrics": { ... },
    "events": [ ... ],
    "hex_state": { ... }
  }
}
```

#### Inject Intervention (live run)

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "inject_intervention",
  "params": {
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "at_tick": 5000,
    "intervention": {
      "type": "policy_override",
      "parameter": "policy.tax_rate",
      "value": 0.25
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "branch_run_id": "a3f8b21c-s42-20260221T143000Z@branch-T5000-b7c9d2",
    "status": "queued"
  }
}
```

### 8.3 Error Response Format

All API errors use a consistent structure:

```json
{
  "error": {
    "code": "RUN_NOT_FOUND",
    "message": "Run a3f8b21c-s42-20260221T143000Z does not exist in this workspace.",
    "details": {
      "run_id": "a3f8b21c-s42-20260221T143000Z"
    }
  }
}
```

**Standard error codes:**

| Code | HTTP Status | Description |
|---|---|---|
| `SCENARIO_NOT_FOUND` | 404 | Scenario ID does not exist |
| `VERSION_NOT_FOUND` | 404 | Scenario version does not exist |
| `RUN_NOT_FOUND` | 404 | Run ID does not exist |
| `VALIDATION_FAILED` | 422 | Scenario config failed validation |
| `SEED_REQUIRED` | 400 | Seed must be provided for this operation |
| `DETERMINISM_VIOLATION` | 409 | Replay produced mismatched tick hash |
| `ENGINE_INSTABILITY` | 500 | Engine halted due to unstable simulation state |
| `MOD_LOAD_FAILED` | 422 | WASM mod failed to load or sandbox verification failed |
| `BRANCH_DEPTH_EXCEEDED` | 422 | Branch tree exceeds maximum depth of 10 |
| `UNAUTHORIZED` | 401 | Invalid or missing Bearer token |
| `FORBIDDEN` | 403 | Token lacks required workspace permission |

---

## 9. Accessibility and Localization

### 9.1 WCAG 2.1 AA Compliance Requirements

CivLab web UI must achieve WCAG 2.1 Level AA compliance at launch. The following items are required:

**Perceivable:**
- All non-text content (charts, hex map, icons) must have text alternatives (alt text, ARIA labels, or adjacent text).
- Color is never the only means of conveying information. All color-coded information must have a secondary indicator (pattern, icon, or text label).
- All audio content must have captions or transcripts (not applicable at launch; no audio content planned).
- Text can be resized up to 200% without loss of functionality.
- No content flashes more than 3 times per second (no animations trigger photosensitive seizures).

**Operable:**
- All functionality available from a keyboard.
- No keyboard traps.
- Skip navigation link available at top of page (bypasses nav to reach main content).
- All interactive elements have visible focus indicators.
- Page titles are descriptive and unique per view.
- Link and button text is descriptive (no "Click here", no "More").

**Understandable:**
- Page language is set (`lang="en"` or appropriate locale).
- Unusual abbreviations (e.g., "FR-UX-001", "Joule stock") are expandable via definition/glossary.
- Error messages identify the field in error and describe how to fix it.
- Labels are associated with their form controls.

**Robust:**
- HTML is valid and well-formed.
- ARIA attributes used correctly.
- Custom components (timeline scrubber, hex map) expose accessible roles, states, and properties.

### 9.2 Screen Reader Support for Simulation State

The hex map and timeline scrubber are rich visual components that require special accessibility treatment.

**Hex Map Accessible Alternative:**
- A "Table View" toggle renders the hex map as a data table: rows = regions, columns = metrics.
- Table updates when the timeline scrubber position changes.
- Screen reader announcement: "Simulation state at tick {T}. {N} regions. Data table follows."

**Timeline Scrubber Accessible Alternative:**
- The scrubber is implemented as `role="slider"` with `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-label="Simulation timeline, tick {T} of {max_tick}"`.
- Keyboard focus on the scrubber: `aria-live="polite"` region announces the current tick and the top changed metric on each tick change.

**Metric Charts Accessible Alternative:**
- Each chart has a "Data Table" toggle rendering the time series as a scrollable table.
- Chart SVGs include a `\<title\>` element with a human-readable description.
- Example: `\<title\>Legitimacy metric over 10000 ticks: starts at 0.82, peaks at 0.91 at tick 2300, declines to 0.34 at tick 4200</title>`.

**Event Log:**
- Each event log entry is a list item (`\<li\>`) with full text description.
- Causal attribution is included in the text: "Legitimacy threshold breach at tick 4195, caused by tax_rate increase at tick 4000."

**Run Status Changes:**
- Run status changes (Queued → Running → Complete) are announced via `aria-live="polite"` region.
- Run failures are announced via `aria-live="assertive"` region.

### 9.3 Color-Blind Safe Palette

The default palette must be distinguishable under deuteranopia (red-green), protanopia (red-green), and tritanopia (blue-yellow) color blindness.

**Default Metric Color Assignments:**

| Metric | Hex | Deuteranopia-safe |
|---|---|---|
| Legitimacy | `#1A85FF` (blue) | Yes |
| Tyranny | `#E66100` (orange) | Yes |
| Resilience | `#5DB0FA` (light blue) | Yes |
| Waste | `#994F00` (brown) | Yes |
| Surplus | `#40B0A6` (teal) | Yes |
| Joule Stock | `#FFC20A` (yellow) | Yes |
| Trade Balance | `#DC3977` (rose) | Yes |
| Population | `#785EF0` (purple) | Yes |

Source palette: IBM Color Blind Safe palette (2022). All colors verified against Coblis color blindness simulator.

**Hex Map Gradient:**
- Default gradient: white (low) to `#1A85FF` (high) — avoids red/green.
- Alternative gradient options in Accessibility settings: Viridis, Cividis, Greys.

**Chart Lines:**
- Up to 8 runs in comparison: line style is differentiated by both color AND line dash pattern (solid, dashed, dotted, dash-dot, long-dash, dash-dot-dot, short-dash, two-dash).

### 9.4 i18n Readiness

At launch, CivLab ships in English only. The codebase must be i18n-ready for future localization.

**Requirements:**
- All user-visible strings must be externalized to a strings file (`en.json`). No hardcoded user-facing strings in component code.
- Date formatting must use the `Intl.DateTimeFormat` API with locale awareness.
- Number formatting must use the `Intl.NumberFormat` API with locale awareness.
- Currency and unit symbols must be configurable, not hardcoded.
- RTL layout support must be considered in component design (no fixed left/right assumptions that break under `dir="rtl"`). Full RTL support is not required at launch but must not be architecturally precluded.
- Locale must be configurable in application settings.

**Strings file structure:**

```json
{
  "nav.scenarios": "Scenarios",
  "nav.runs": "Runs",
  "nav.sweeps": "Sweeps",
  "run.status.queued": "Queued",
  "run.status.running": "Running",
  "run.status.complete": "Complete",
  "run.status.failed": "Failed",
  "run.status.cancelled": "Cancelled",
  "metric.legitimacy": "Legitimacy",
  "metric.tyranny": "Tyranny",
  "metric.resilience": "Resilience",
  "metric.waste": "Waste",
  "metric.surplus": "Surplus",
  "metric.joule_stock": "Joule Stock",
  "metric.trade_balance": "Trade Balance",
  "metric.population": "Population"
}
```

---

## 10. Acceptance Criteria

### 10.1 Persona A: Scenario Designer Acceptance Tests

These tests can be performed by Maren (or a QA proxy) without access to engine internals.

**AT-SD-001: Create scenario from template**
- Navigate to Scenarios > New Scenario > Browse Templates.
- Select "Pre-Industrial Island State" template.
- Click "Fork as New Scenario".
- Verify: Editor opens with template parameters populated.
- Verify: Template origin shown in scenario header ("Forked from: Pre-Industrial Island State").
- Pass criterion: Complete in under 5 minutes.

**AT-SD-002: Inline validation error**
- In the scenario editor, set `policy.tax_rate` to `1.5` (above valid maximum of 0.80).
- Verify: Field is immediately highlighted amber.
- Verify: Error message appears below field: "tax_rate must be between 0.0 and 0.80. Current value: 1.50."
- Verify: "Commit Version" button is disabled.
- Pass criterion: Error appears within 500ms of field change without page reload.

**AT-SD-003: Full validation report**
- With a scenario containing at least one cross-parameter constraint violation, click "Validate".
- Verify: Validation report lists all violations with parameter path, current value, and constraint description.
- Verify: Report distinguishes errors (blocking) from warnings (non-blocking).
- Pass criterion: Validation report rendered within 2 seconds.

**AT-SD-004: Commit immutable version**
- With a valid scenario, click "Commit Version".
- Enter label "v1.0-test" and commit message "Test commit".
- Verify: System displays version ID, config hash (BLAKE3 hex), and stable reference URL.
- Attempt to edit any parameter of the committed version.
- Verify: Parameters are read-only; no edit controls present.
- Pass criterion: Version ID is stable and the reference URL resolves to the same config.

**AT-SD-005: Scenario version diff**
- Create two versions of the same scenario that differ in one parameter.
- Navigate to Version History and select both versions.
- Click "Diff".
- Verify: Diff view highlights the changed parameter with old and new values.
- Verify: Unchanged parameters are not shown in the diff by default (collapsed).
- Pass criterion: Diff renders within 1 second.

**AT-SD-006: Scenario reference URL stability**
- Commit a scenario version. Copy the reference URL.
- In a new browser session (or incognito), navigate to the reference URL.
- Verify: The same scenario version loads with the same config.
- Pass criterion: Reference URL is stable and human-shareable.

### 10.2 Persona B: Policy Analyst Acceptance Tests

**AT-PA-001: Run a scenario**
- Navigate to a committed scenario version.
- Click "Run". Accept default seed. Click "Submit".
- Verify: Run appears in run list with status "Queued" then "Running" then "Complete".
- Verify: Run ID is displayed in the run list entry.
- Pass criterion: Run status reflects accurate engine state within 2 seconds of change.

**AT-PA-002: Timeline scrubbing responsiveness**
- Open a completed run with at least 5000 ticks.
- Drag the timeline scrubber rapidly across the full range.
- Verify: All views (hex map, metric charts, event log) update to reflect the new tick position.
- Verify: Update completes within 200ms of releasing the scrubber.
- Pass criterion: No view shows stale tick data after scrubber release.

**AT-PA-003: Hex map LOD transition**
- In strategic LOD, double-click a hex tile.
- Verify: Transition to operational (city-level) LOD within 300ms.
- Verify: Breadcrumb shows "Strategic View > [Region Name] > Operational View".
- Click breadcrumb "Strategic View" link.
- Verify: Returns to strategic LOD with the same camera position.
- Pass criterion: LOD transition is smooth and reversible.

**AT-PA-004: Side-by-side comparison**
- Run the same scenario with two different seeds (or two different interventions).
- Click "Compare" and select both runs.
- Assign regime labels to each run.
- Select three metrics for comparison.
- Verify: Side-by-side charts render for each selected metric.
- Verify: Regime labels appear in chart legends.
- Verify: Divergence point annotation appears if runs diverge beyond threshold.
- Pass criterion: Comparison panel populated within 1 second of run selection.

**AT-PA-005: Export report package**
- From a comparison view with annotations, click "Export" > "All".
- Verify: Export dialog shows all format options.
- Verify: Assumption disclosure cannot be unchecked.
- Download the ZIP bundle.
- Verify: ZIP contains PDF, JSON, CSV, and Parquet files.
- Verify: JSON bundle contains `artifact_fingerprint` field (BLAKE3 hash).
- Run `civlab verify \< bundle.json>` on the JSON bundle.
- Verify: Output is "PASS".
- Pass criterion: Export complete without UI blocking; bundle passes integrity check.

**AT-PA-006: Provenance badge on chart**
- In the timeline view, hover over the provenance badge on any metric chart.
- Verify: Badge displays run ID (8 chars), scenario label, seed, and tick range.
- Click the badge.
- Verify: Full provenance panel opens with complete metadata.
- Pass criterion: Provenance information matches the run's actual parameters.

### 10.3 Persona C: Research Operator Acceptance Tests

**AT-RO-001: Sweep dry run**
- Write a sweep manifest with 3 parameter combinations and 5 seeds (15 total runs).
- Run `civlab sweep --manifest manifest.toml --dry-run`.
- Verify: Output states "15 runs" without executing any simulation.
- Verify: Exit code 0.
- Pass criterion: Completes in under 5 seconds.

**AT-RO-002: Sweep execution and output structure**
- Execute the same sweep without `--dry-run`.
- After completion, verify the output directory contains:
  - `manifest.toml`, `sweep_id.txt`
  - `runs/{run_id}/` directories for all 15 runs
  - `aggregate/summary.csv`, `aggregate/full_dataset.parquet`
- Verify: Exit code 0 for all-success, exit code 1 if any runs failed.
- Pass criterion: Output structure matches specification exactly.

**AT-RO-003: Degenerate run detection**
- Construct a sweep manifest with at least one parameter combination known to produce instability (e.g., extreme parameter values).
- Run sweep.
- Verify: Degenerate runs appear in `aggregate/degenerate_runs.json` with instability reason.
- Verify: Aggregate statistics computed over remaining valid runs only.
- Verify: Warning in stdout: "N runs excluded as degenerate. Aggregates computed over M runs."
- Pass criterion: Degenerate runs do not corrupt aggregate statistics.

**AT-RO-004: JSON streaming output**
- Run `civlab run scenarios/island_state.toml --seed 1 --stream --ticks 100`.
- Verify: Stdout is a sequence of valid JSON lines, one per tick.
- Pipe to `jq '.metrics.legitimacy'` and verify numeric values are emitted.
- Verify: Exit code 0.
- Pass criterion: Every tick produces exactly one JSON line; all lines are valid JSON.

**AT-RO-005: REST API run submission and polling**
- Submit a run via `POST /api/v1/runs` with a valid scenario ID and seed.
- Verify: Response contains `run_id` and `status: "queued"`.
- Poll `GET /api/v1/runs/{run_id}` until status is `complete` or `failed`.
- Verify: On completion, `GET /api/v1/runs/{run_id}/metrics` returns all eight core metric values.
- Pass criterion: Full run cycle completable via REST API without web UI.

**AT-RO-006: WebSocket tick streaming**
- Connect to WebSocket endpoint and send `subscribe_run` request for a running simulation.
- Verify: `run_tick` events are received at the configured `tick_interval`.
- Verify: `run_complete` event is received when the run finishes.
- Verify: All `run_tick` events contain all eight core metrics.
- Pass criterion: No ticks missed; events arrive in tick order.

**AT-RO-007: Sweep resume**
- Start a sweep. Interrupt it after 50% completion (kill the process).
- Rerun with `--resume ./sweep_output`.
- Verify: Only the remaining runs execute (already-complete run artifacts are not re-run).
- Verify: Final aggregate includes all runs (both pre-interrupt and post-resume).
- Pass criterion: Resume produces identical aggregate to uninterrupted run.

### 10.4 Persona D: Modder Acceptance Tests

**AT-MD-001: Mod scaffold generation**
- Run `civlab mod new --type policy --name test_policy`.
- Verify: Directory `test_policy/` created with `Cargo.toml`, `src/lib.rs`, `civmod.toml`, `tests/` scaffold.
- Verify: `Cargo.toml` contains `civlab-sdk` dependency at the correct version.
- Pass criterion: `cargo build --target wasm32-wasip2` succeeds on the scaffold without modification.

**AT-MD-002: Mod test harness determinism check**
- Run `civlab mod test ./my_mod.wasm --scenario examples/island_state.toml --ticks 500 --verify-determinism`.
- Verify: Test runs the simulation twice with the same mod.
- Verify: Output states "Determinism check: PASS" if all tick hashes match between runs.
- Verify: Output states "Determinism check: FAIL — tick {T} hash mismatch" if hashes diverge.
- Pass criterion: A pure mod passes; a non-deterministic mod fails.

**AT-MD-003: Sandbox constraint violation**
- Compile a mod that attempts file I/O (violates sandbox constraints).
- Run `civlab mod test` against this mod.
- Verify: Output states "Sandbox violation: file_io attempted at hook PolicyMod::apply".
- Verify: Exit code is non-zero.
- Pass criterion: Violation detected at load time or first hook call; run does not proceed.

**AT-MD-004: Active mod badge in web UI**
- Run a scenario with a mod activated.
- In the web UI, open the completed run.
- Verify: "Mods Active" badge is visible in the run header.
- Click the badge.
- Verify: Panel shows mod name, version, registry URL, and signature status.
- Pass criterion: Mod badge is always present for mod-active runs, on every view.

### 10.5 Usability Benchmarks

These benchmarks define the minimum acceptable usability bar, measurable in usability testing with representative users.

| Benchmark | Target | Measurement Method |
|---|---|---|
| Time to create first scenario from template | < 10 minutes (median) | Task time in usability session |
| Time to produce side-by-side comparison | < 5 minutes from run completion (median) | Task time in usability session |
| Time to produce export report | < 3 minutes from comparison view (median) | Task time in usability session |
| Error rate on scenario commit | < 10% of attempts blocked by preventable errors | Count blocked commits in usability session |
| Timeline scrubber discoverability | > 90% of users find scrubber without instruction | Observation in usability session |
| CLI sweep command success without docs | > 70% of Research Operators complete a sweep using only `--help` | Task success rate in usability session |
| Provenance badge comprehension | > 80% of Policy Analysts correctly describe what produced a chart | Comprehension question in usability session |

---

*End of CivLab Civ-Sim User Specification v0.1*
*Document maintained by CivLab Product Engineering*
*Next review: 2026-03-21*
