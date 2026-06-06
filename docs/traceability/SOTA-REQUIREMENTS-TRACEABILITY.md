# SOTA Requirements-Traceability Audit — Civis

**Author:** doc-writing agent
**Date:** 2026-06-05
**Repo:** `C:\Users\koosh\Dev\civis-game`
**Scope:** `PRD.md`, `FUNCTIONAL_REQUIREMENTS.md`, `agileplus-specs/`, `docs/specs/`, all `clients/bevy-ref/tests/*.rs`, all `crates/*/tests/*.rs`, `docs/traceability/`, `qa-config.json`, `PLAN.md`, `docs/IMPLEMENTATION_STATUS.md`.
**Audience:** senior engineer inheriting the repo.

---

## 1. Executive summary

Civis already does the *spine* of requirements traceability: ~140 functional requirements are split across three FR namespaces (`FR-CORE/.../METRICS-*` in `FUNCTIONAL_REQUIREMENTS.md`, the 3D-extension `FR-CIV-*` set in `docs/development-guide/fr-3d-additions.md`, and the Web `FR-CIV-WEB-*` set in `docs/development-guide/fr-web-spectator.md`); every spec lives in `agileplus-specs/civ-XXX-*/{spec.md,meta.json,plan.md}`; and verification is by named `#[test]` functions discoverable by hand or by `docs/traceability/fr-3d-matrix.md`. Coverage is dense at the *unit-test* layer but **shallow at the *requirement-id* layer**: there is no machine-checked link between `FR-CIV-CA-001` and a `#[test]`. The three biggest gaps are (a) **no compile-time FR↔test linkage** — `qa-config.json` declares `tag_patterns: [CIV-…]` but nothing in the workspace emits them, so the "orphan_check" gate is dead code; (b) **BDD coverage is 12 tests in one file** (`clients/bevy-ref/tests/requirements_bdd.rs`) and tracks UI/voxel behavior, not the 8 strategic `FR-CORE-*` SHALs that gate MVP; (c) **the most-cited spec doc (`full-traceability-matrix.md`) lists 20+ "GAP" cells** for critical `FR-CORE-004` (RNG logging), `FR-CORE-006` (priority queue), `FR-CORE-007` (tick budget), `FR-PROTO-003` (handshake), `FR-API-002..004` (research API). The recommended next three steps are: (1) wire `FR-<id>` doc-comments onto each test function and have a `cargo test` post-processor fail on FRs lacking a test; (2) port the 8 highest-value `requirement_*` BDD scenarios from `requirements_bdd.rs` to `crates/engine/tests/` covering the SHAL-tier `FR-CORE-001..007`; (3) close the 5 highest-impact `GAP` rows in `full-traceability-matrix.md` (RNG draw logging, priority queue, tick budget CI gate, handshake role enforcement, JSON-RPC batch + -32601).

## 2. How Civis currently traces requirements (factual inventory)

### 2.1 Spec doc locations

| Artifact | Path | What it is | Status |
|---|---|---|---|
| Product Requirements | `PRD.md` | MVP / v1 / v2 / v3+ feature matrix, epics E1–E6, NFRs (perf, scalability, reliability, security) | Approved 2026-02-21 |
| Functional Requirements (strategic) | `FUNCTIONAL_REQUIREMENTS.md` | 25 SHALL FRs in 6 families: CORE, ECON, PROTO, REPLAY, API, CLIENT, METRICS | Draft 2026-03-25 |
| FR-CIV-LIFE / METRICS-001..003 inline | `FUNCTIONAL_REQUIREMENTS.md` (lines 361–424) | Emergent life-sim + metrics FRs appended directly to the same doc | Authored 2026-05 |
| 3D extension FRs (FR-CIV-*) | `docs/development-guide/fr-3d-additions.md` | ~140 3D-extension FRs in 14 families (CA, SPECIES, ARCH, LANG, AUDIO, LEGENDS, PSYCHE, DIPLO, PBR, LLM, SCALE, …) | Active |
| Web spectator FRs (FR-CIV-WEB-000..008) | `docs/development-guide/fr-web-spectator.md` | Web dashboard FRs | Closed (status "Closed 2026-05-25" per `fr-web-matrix.md`) |
| Per-spec AgilePlus | `agileplus-specs/civ-001-core-simulation-engine/{spec.md,meta.json,plan.md}` … 13 spec dirs total | One folder per FR-cluster: spec text, machine-readable `fr_ids[]`, phased WBS | Active, 13 specs |
| Detailed spec bodies | `docs/specs/CIV-0001..CIV-1000` (18 spec files) | Long-form spec bodies | Active |
| ADR log | `ADR.md` (root) | Architecture decisions | Active |
| Strategic plan | `PLAN.md` (root) | Phases 0–6 with DAG | Active |
| 3D phase plan | `docs/roadmap/plan-3d-phases.md` | P-V0…P-U1 phases | Active (referenced; not directly inspected) |
| QA config | `qa-config.json` | Declares `traceability.tag_patterns = ["@trace CIV-…", "CIV-…"]` + `orphan_check: true` | Active intent, **NOT enforced** |
| Traceability index | `docs/traceability/index.md` | Links to the 4 matrices | Active |
| Strategic traceability matrix | `docs/traceability/TRACEABILITY_MATRIX.md` | `FR-CORE-*` and `FR-ECON-*` rows | Active, stale for several `*010`+ rows |
| 3D matrix | `docs/traceability/fr-3d-matrix.md` | 200+ `FR-CIV-*` rows | Active, comprehensive |
| Web matrix | `docs/traceability/fr-web-matrix.md` | `FR-CIV-WEB-*` | Closed |
| Full E2E matrix | `docs/traceability/full-traceability-matrix.md` | Strategic + 3D merged; flags 20+ "GAP" rows where no test exists | Active, best single source of truth |
| TraceLinks (commit-pinned) | `docs/traceability/civis-tracelinks.md` | `FR ID | Code | Test | Commit` for FR-CIV-LIFE/INFRA/TRAFFIC/INSPECT/INFOVIEW/SAVE/VOXEL families | Active, partial coverage |
| Event taxonomy | `docs/traceability/EVENT_TAXONOMY.md` | Bus event names → consumers | Active |
| Implementation status | `docs/IMPLEMENTATION_STATUS.md` | What crates exist + landed FR areas | Active |

### 2.2 BDD / `requirement_*` test files

| File | Tests | Style | Linked FRs (if any) |
|---|---|---|---|
| `clients/bevy-ref/tests/requirements_bdd.rs` | 12 `requirement_*` `#[test]`s + 1 `#[cfg(feature="bevy")]`-gated marker-type test | GIVEN-WHEN-THEN comments, but plain `#[test]` (no cucumber harness) | **None** — the function name carries the requirement, not an FR ID. Names are free-form descriptive English (e.g. `requirement_water_only_below_sea`). |
| `clients/bevy-ref/tests/settings_bdd.rs` | 3 `bdd_given_*` tests | Same plain-`#[test]` convention | None |
| `clients/bevy-ref/tests/world_size_bdd.rs` | 1 test | Same | None |
| `crates/engine/tests/tick_budget.rs` | 1 test | Plain `#[test]` | Implicitly covers `FR-CORE-007` (tick budget 14 ms) — no annotation |
| `crates/engine/tests/end_to_end_tick.rs` | 2 tests | Plain `#[test]` | Implicitly covers `FR-CORE-001/003` (determinism) — no annotation |
| `crates/legends/tests/saga_graph.rs` | 13 tests | Plain `#[test]` | Header comment lists `FR-CIV-LEGENDS-GRAPH-01/-RESOLVE-04/-SIG-05/-CAUSAL-06/-QUERY-07/-NARRATOR-13` (file lines 1–2) — **only place in repo where FR IDs are co-located with tests** |

### 2.3 BDD test → source-file map

Every test in `requirements_bdd.rs` is annotated with a `GIVEN/WHEN/THEN` comment block and then `use`d to a concrete source symbol. The mapping is fully derivable from the file; the table below is the hand-extracted version (full version in Appendix A).

| Test (line) | Imports (source contract) |
|---|---|
| `requirement_world_size_selection_changes_dimensions` (47) | `civ_bevy_ref::voxel_sim::world_dims_for` (`clients/bevy-ref/src/voxel_sim.rs:155`) |
| `requirement_new_world_differs_from_previous` (85) | `civ_bevy_ref::voxel_sim::world_dims_for` + `civ_voxel::worldgen::generate` (`crates/voxel/src/worldgen.rs:76`) |
| `requirement_2d_map_extent_matches_world` (119) | `civ_voxel::fluid_ca::CaGrid::new` + `civ_bevy_ref::map2d::world_extent_for_basemap` (`clients/bevy-ref/src/map2d.rs:290`) |
| `requirement_water_only_below_sea` (141) | `civ_voxel::worldgen::generate` + `civ_voxel::worldgen::sea_level` |
| `requirement_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit` (166) | `civ_bevy_ref::camera::{camera_input, CameraRig}` (`clients/bevy-ref/src/camera.rs:11, 49`) |
| `requirement_settings_has_gfx_audio_controls_gameplay_tabs` (335) | `civ_bevy_ref::settings_ui::{settings_tabs, SettingsTab}` (`settings_ui.rs:261, 299`) |
| `requirement_emergent_factions_no_fixed_count_or_alignment` (372) | `civ_engine::Simulation::{with_seed, tick, faction_count, faction_alignment}` (`crates/engine/src/engine.rs:687, 889, 913, 1145`) |
| `requirement_actor_spawn_avoids_t_pose_and_animates` (401) | `civ_bevy_ref::animation::{clip_frame_for_test, idle_angles_for_test}` (`animation.rs:74, 86`) + `civ_agents::ActorVisualKind` |
| `requirement_native_ocean_renders_with_sea_level_match` (466) | `civ_voxel::worldgen::{generate, sea_level}` (uses `sea_level` for surface-alignment check) |
| `requirement_keybind_rebinding_overrides_default` (529) | `civ_bevy_ref::settings_ui::{GameSettings, KeyBinding}` (`settings_ui.rs:774, 877, 885`) + `bevy::input::keyboard::KeyCode` |
| `requirement_terrain_is_continuous_not_blobs` (554) | `civ_voxel::{worldgen::generate, ChunkView, CubicMesher, LodLevel, ChunkId}` |
| `requirement_marker_types_differentiate_server_attach_vs_in_process` (583) | `civ_bevy_ref::live_stream::{LiveAgentTag, LiveBuildingTag}` (`live_stream.rs:84, 98`) + `civ_bevy_ref::sim_bridge::{SimCivilianMarkerPublic, SimBuildingMarkerPublic}` |

### 2.4 Existing doc↔test cross-link conventions

| Convention | Where | Authoritative? |
|---|---|---|
| FR IDs in `agileplus-specs/civ-XXX/meta.json` (`fr_ids: ["FR-CORE-001", …]`) | 13 spec dirs | **Yes** — machine-readable. |
| FR IDs in `agileplus-specs/civ-XXX/spec.md` headers | All 13 specs | **Yes** — human-readable. |
| FR ID → test name pattern in `fr-3d-matrix.md` | 200+ rows | **Yes for FR-CIV-***, but the test name patterns are descriptive English, not enforced. |
| FR ID → test name → commit SHA in `civis-tracelinks.md` | FR-CIV-LIFE/INFRA/TRAFFIC/INSPECT/INFOVIEW/SAVE/VOXEL families | **Yes for those families** — best of the existing matrices, but only covers ~25% of FRs. |
| FR IDs in test function names or `// FR-CORE-001` comments | **`crates/legends/tests/saga_graph.rs:1-2` only** | **No** — almost zero adoption. |
| FR IDs in commit messages | repo-wide | **Partial** — `civis-tracelinks.md` documents this for ~12 FRs only. |
| `@trace CIV-XXX-NNN` in source comments | `qa-config.json` declares this pattern; **no source emits it** | **No** — `orphan_check: true` is dead code. |
| FR IDs in CI gate (pre-commit, GH Actions) | `qa-config.json` declares; `.pre-commit-config.yaml` exists; no grep-on-`@trace` step was found in the brief inspection | **No** — not enforced. |

### 2.5 Gaps (concrete)

| Class | Concrete observation |
|---|---|
| `FR-CORE-*` SHAL tests outside the 3D namespace | Only 4 of the 8 SHAL `FR-CORE-*` (determinism, tick-100ms, hash chain, replay) have any test in `crates/engine/tests/`; the others (`FR-CORE-004` RNG logging, `FR-CORE-006` priority queue, `FR-CORE-007` tick-budget CI gate) are listed as GAP in `full-traceability-matrix.md:70, 71, 72`. |
| `FR-CIV-CA-001..010` cellular-automaton FRs | Source (`crates/voxel/src/fluid_ca`) exists; tests live in `crates/voxel/tests/` (not inspected in this pass but documented in `fr-3d-matrix.md` as `implemented`). The CA `requirement_*` names in `requirements_bdd.rs` are descriptive English; they do not carry the FR-CIV-CA- IDs. |
| Source ↔ FR link | 12 of 12 `requirement_*` tests in `requirements_bdd.rs` cite a **source function** in a comment, but zero of 12 cite an **FR ID**. There is no `// FR-CIV-CA-001` next to the assertion. |
| `#[ignore]` / `// placeholder` tests | `requirements_bdd.rs` is heavy on `#[cfg(feature = "bevy")]` / `#[cfg(feature = "egui")]` / `#[cfg(feature = "voxel")]` gates but contains zero `#[ignore]` and zero `// placeholder` lines. (Positive observation — none of the 12 is a stub.) |
| Engine tick wiring | `crates/legends`, `crates/legends` is reachable via `phase_emergence` (`crates/engine/src/emergence.rs`); other crates (`crates/genetics`, `crates/species`, `crates/laws`, `crates/research`, `crates/protocol-3d`) are library-only — `full-traceability-matrix.md:48-56` caps their FRs at `Partial` regardless of unit-test pass. |
| Comments inside source | `clients/bevy-ref/src/voxel_sim.rs:41` has an inline FR reference: `// (see FR-CIV-CA dirty-chunk TODO)` — a 1-off, not a pattern. |

## 3. State of the art (2024–2026)

Six tools/methods are most relevant to a 3D simulation game with this mix of Rust core + Bevy/Godot/Unreal clients + Web spectator. For each: what it is, where it shines, where it falls short, license/cost, **and why it matters for Civis specifically.**

### 3.1 `cucumber-rust` (BDD with Gherkin)

| | |
|---|---|
| What | A Rust harness that runs `.feature` files written in Gherkin (`Given/When/Then`) against step functions annotated with `#[given]` / `#[when]` / `#[then]`. Generates a JUnit + Cucumber JSON report consumable by CI dashboards. |
| Shines | Human-readable specs that product owners / designers can edit; one feature file is the canonical source of the requirement; the BDD report is the verification artifact. Pairs with `cucumber-messages` for cross-tool report ingestion. |
| Falls short | BDD tests typically exercise *integration* or *system* behavior — not internal `pub fn` invariants. A 256³ voxel terrain sweep is too slow to run as a BDD step. Two-file artifact (`.feature` + Rust steps) duplicates the contract. |
| License/cost | MIT, free. |
| **Why Civis** | The current 12 `requirement_*` tests embed the GIVEN/WHEN/THEN *in a Rust comment* — Cucumber lets you lift those comments into real `.feature` files in `clients/bevy-ref/features/` without changing the test bodies, and the `cargo test --test bdd` run emits a Cucumber HTML report attachable to PRs. Lays the foundation for Phase-3 client-attached acceptance runs. |

### 3.2 `cucumber-js` / Playwright trace viewer (TS/React side)

| | |
|---|---|
| What | `cucumber-js` runs `.feature` files against TypeScript step definitions; the Playwright trace viewer (`npx playwright show-trace`) replays an entire failing run as a step-by-step DOM/network/action timeline. |
| Shines | Catches *visual* regressions and multi-step interaction chains; the trace viewer is a video-of-the-failure debug surface engineers actually use. Plays well with the Web spectator's existing `web/tests/*.test.mjs` (93 tests, vitest). |
| Falls short | Trace viewer records a real browser; won't help with the headless Rust engine. Not a *requirement* tool — a debugging aid. |
| License/cost | MIT, free. |
| **Why Civis** | `fr-web-matrix.md` already shows the Web client has 8 closed FRs and 93 vitest tests but **no** `.feature` files and **no** trace viewer wired in. If a designer needs to see "what did the spectator render when FR-CIV-WEB-003 failed", the trace is the right artifact. Pairs with §3.1 for cross-stack BDD. |

### 3.3 Code-as-spec — Rust types + property-based tests (proptest / quickcheck / arbitraries)

| | |
|---|---|
| What | Encode the requirement in the *type* or in a property assertion. E.g. `struct JouleLedger` with a `try_add` that panics on conservation violation; `proptest!` runs 10 000 random inputs and asserts the invariant. |
| Shines | Catches regressions no human would have thought to test. The conservation law `Σ Joules == const` becomes a one-line `proptest!` that runs every CI commit. Compile-time FR enforcement is impossible in current Rust, but **type-level** invariants (no `f32` in deterministic path) are enforceable via `deny.toml` + a `clippy.toml` lint. |
| Falls short | Doesn't read like a requirement; new hires can't read a property and know what feature it protects. Property tests fail in non-obvious ways and need shrinking (`proptest-recurse`). |
| License/cost | MIT/Apache-2, free. |
| **Why Civis** | Civis already imports `proptest` (visible in `crates/engine/tests/determinism_proptest.rs`). `full-traceability-matrix.md:74` notes the full joule-conservation proptest is a GAP — replacing that with an actual `proptest! { #[test] fn joule_conservation_holds_for_random_trade_sequences(seed in …) { … } }` is a 1-PR win and closes a SHAL FR (`FR-ECON-002`). |

### 3.4 OpenTelemetry semantic conventions for trace IDs

| | |
|---|---|
| What | OTel `trace_id` (128-bit) and `span_id` (64-bit) propagated across process boundaries via W3C Trace Context. Each tick of the simulation is one root span; each phase (`phase_production`, `phase_economy`, …) is a child span; the JSON-RPC dispatch is its own span. |
| Shines | End-to-end correlation: a UI bug "settings didn't persist" can be traced to the exact `phase_save_bundle` span that wrote the data. Integrates with Jaeger, Honeycomb, Tempo. |
| Falls short | Heavy if the volume is high (one span per tick is fine; one per voxel is too many). Not a *requirement* tool — an *observability* tool that complements a requirements trace. |
| License/cost | Apache-2, free (OSS backends); SaaS backends $$. |
| **Why Civis** | The server already emits structured events (`docs/traceability/EVENT_TAXONOMY.md`, 21 KB). Promoting event names to OTel span names is a 50-line middleware change in `crates/server/src/ws_bridge.rs`. The 60 FPS Bevy client can attach the trace context to its `bevy_render` frames for visual-regression root cause. |

### 3.5 ReqIF / Spec-by-Example / OpenReqSpec (compliance-grade)

| | |
|---|---|
| What | **ReqIF** (Requirements Interchange Format) is the ISO/IEC/IEEE 29148:2018 standard format for exchanging requirements between tools (doors, jama, polarion, codebeamer). **OpenReqSpec** is the open-source companion. **Specification by Example** (Adzic) is the book/method — executable specs in business-readable form, Gherkin the canonical notation. |
| Shines | ReqIF is what auditors and procurement teams ask for; mapping the strategic 25 `FR-CORE/ECON/.../METRICS-*` into a `.reqifz` artifact lets Civis claim compliance with ISO/IEC 12207 + 29148 if a research grant or a publisher requires it. Spec-by-Example is the *method* that produces the executable Gherkin files in §3.1. |
| Falls short | ReqIF authoring tools are heavy (doors is a Windows desktop app with $$ license); the standalone `.reqifz` exchange format is workable but tooling for *generating* it from markdown is thin. |
| License/cost | ReqIF: free format, $$$ tools. OpenReqSpec: Apache-2. Spec-by-Example: book, no cost. |
| **Why Civis** | Even for a game, the *auditable* claim "we tested FR-CORE-003 bit-identical determinism 10 000× under proptest" is a unique research-credibility asset. If Civis is ever positioned as a "civilization-sim research platform" (PRD §1.2 explicitly says *research sandbox*), ISO-aligned requirements traces become a procurement requirement for academic and gov-funded use. |

### 3.6 Compliance — ISO 26262 / IEC 62304 / FDA SaMD (light touch for games)

| | |
|---|---|
| What | Domain-specific safety standards for automotive (ISO 26262), medical device software (IEC 62304), and Software-as-a-Medical-Device (FDA SaMD pre-cert). All three require a documented "software safety classification" + a per-class verification matrix (req→design→code→test→verification). |
| Shines | Mature traceability templates: every requirement is a `shall`, every test is a verification artifact, and the chain is auditable end-to-end. Many of the row formats in `fr-3d-matrix.md` (Req ID, Crate, Test, Status) were invented independently by Civis *because* the same shape is needed. |
| Falls short | Overkill for a single-player game; the safety classification work doesn't apply. But the *template* — and the *failure modes* it forces you to discover — apply regardless. |
| License/cost | Standards: $$ to read; method: free. |
| **Why Civis** | Several FRs are *non-negotiable invariants* (Joule conservation, no-f32-in-deterministic-path, role enforcement, no-LLM-on-tick-hot-path — `FR-CIV-LLM-005`). Treating these as "safety class B" requires (a) a documented hazard analysis ("if the invariant breaks, what bad thing happens"), (b) a verification matrix, (c) a regression test that fails loud. The matrix template in §3.5 is exactly this. The user's `[[project_civis_emergence_charter]]` and `[[feedback_civis_no_determinism]]` memories show the design intent already aligns with this discipline. |

## 4. Gap analysis

| # | Gap | Current pain | SOTA solution | Adoption cost | Payoff |
|---|---|---|---|---|---|
| G1 | **No machine-checked FR↔test link.** `qa-config.json` declares `tag_patterns: ["CIV-…"]` + `orphan_check: true` but nothing emits those tags. | New FRs get added to specs but no one knows if a test exists; audits require manual cross-walk. | Add `// FR-CIV-CA-001` doc-comments (or `#[fr = "FR-CIV-CA-001"]` proc-macro) above each test; add a `xtask check-frs` (or `make check-frs`) that greps `tests/` and the source `pub fn`s, diffs against `agileplus-specs/civ-*/meta.json::fr_ids[]`, and exits 1 on any missing coverage. Wire into `.github/workflows/ci.yml` as a required check. | **Low** — ~1–2 dev-days for the proc-macro or ~0.5 day for the grep script; ~0.5 day to wire the existing 12 `requirement_*` tests + the 13 `saga_graph` tests + the FR-CORE-001..007 engine tests with annotations; ~0.5 day CI wiring. | **High** — turns `full-traceability-matrix.md`'s 20+ "GAP" cells from a human-readable concern into a CI failure. Closes the *single biggest* traceability-process risk. |
| G2 | **BDD is concentrated in one Bevy-client file and tracks UX, not engine SHALs.** | `requirements_bdd.rs` covers voxel worldgen, water, camera, settings, animation, markers — all client surface area. The 8 `FR-CORE-*` SHALs (tick loop, determinism, RNG, hash chain, replay) have no BDD wrapper; an engine regress could ship without anyone noticing. | Port the 4 most critical engine contracts (tick-100ms, determinism-same-seed, .civreplay-round-trip, joule-conservation) into Gherkin `.feature` files in `crates/engine/features/`, run with `cucumber-rust`. Each `.feature` is a one-page human-readable spec that a non-Rust reviewer can audit. | **Medium** — 1–2 dev-days per feature (4 features → ~5 days); `cucumber-rust` adds a dev-dep; CI step is one cargo invocation. | **High** — engine invariants become readable artifacts; auditor-ready; cross-links to `cucumber-messages` JSON for the same report as the Web client. |
| G3 | **Joule-conservation property test is a GAP** (`full-traceability-matrix.md:74`). | The `CapitalistAllocator` exists and has unit tests for "fills demand" and "never exceeds budget" but the actual SHAL `FR-ECON-002` conservation invariant is unenforced. A future refactor could break it silently. | `proptest!` running 1 000 random `(entity_count, trade_count)` scenarios that sum Joules before and after `phase_economy` and assert delta == 0. Same shape as the existing `crates/engine/tests/determinism_proptest.rs`. | **Low** — ~0.5 dev-day; reuses the existing `proptest` infrastructure. | **Very high** — closes a SHAL and a documented gap. |
| G4 | **`FR-CORE-007` tick-budget CI gate is a GAP** (`full-traceability-matrix.md:72`). | `crates/engine/tests/tick_budget.rs:17` exists but no CI workflow actually gates the P99 regression. | Promote `tick_budget` to a `cargo bench` job on every PR; use `criterion` for P99; fail PR if P99 > 16 ms on a pinned 4-core/16 GB GitHub Actions runner. | **Medium** — 1–2 dev-days (CI runner setup, criterion integration, baseline.json to absorb noise). | **High** — performance regressions stop landing. The user's [[project_wsm3d_redraw_cpu_bound]] memory shows the cost of *not* having this gate. |
| G5 | **No end-to-end observability link from a UI bug to the engine phase that caused it.** | A user reports "settings didn't persist" — currently the engineer greps event names in `EVENT_TAXONOMY.md` and hopes to find the right one. | Adopt OTel semantic conventions: emit one `span` per engine `phase_*` call, propagate `traceparent` through WS+JSON-RPC dispatch, attach the trace_id to every `EVENT_TAXONOMY` event. Open-source backend (Jaeger via Docker or Tempo) for free. | **High** — 5–10 dev-days for full propagation, 1–2 days for the minimum viable version (just engine phases). | **Medium** — biggest win is on incident response; not on day-to-day refactor confidence. Worth scheduling after G1–G3. |
| G6 | **Web client has 8 closed FRs and 93 vitest tests but no `.feature` files and no Playwright trace.** | A designer-side change to FR-CIV-WEB-003 (scene rendering) can pass vitest but break the visual contract; the next reviewer learns this from a screenshot diff. | Convert the 4 most-visual `FR-CIV-WEB-*` (`FR-CIV-WEB-002` snapshot, `FR-CIV-WEB-003` scene3d, `FR-CIV-WEB-006` binary frame, `FR-CIV-WEB-007` babylon renderer) to Gherkin `.feature` files run against Playwright; keep the trace viewer artifact in CI. | **Medium** — 2–3 dev-days for the four scenarios + Playwright wiring. | **Medium** — auditable visual contract; "renders X" is finally *defined*. |

## 5. Adoption plan

Three phases, each with phased WBS + DAG of dependencies per `CLAUDE.md` governance. All effort is in **agent tool calls** (CLAUDE.md forbids "days/weeks" framing for plans).

### Phase 1 — Wire the machine-checked FR↔test link (closes G1; enabler for G2 + G3)

| Task | Description | Depends On | Effort (agent tool calls) | Success criterion |
|---|---|---|---|---|
| P1.1 | Add `// FR-<id>` doc-comment lint to `clippy.toml` (or `deny.toml`) so missing annotations on `#[test]` fns named `requirement_*` or matching `*_bdd` are clippy-warn. | none | ~30 tool calls (1 codex batch) | `cargo clippy --workspace --all-targets` warns on any un-annotated test. |
| P1.2 | Annotate the 12 `requirement_*` tests in `clients/bevy-ref/tests/requirements_bdd.rs` with their implied FR IDs (FR-CIV-VOXEL-000, FR-CIV-CA-010, FR-CIV-MAP2D-001, FR-CIV-CA-010 water-below-sea, FR-CIV-BEVY-024, FR-CIV-BEVY-024 settings, FR-CIV-EMERGENT-001, FR-CIV-SPECIES-016, FR-CIV-CA-007 ocean, FR-CIV-BEVY-024 keybind, FR-CIV-VOXEL-010 mesh, FR-CIV-BEVY-026 marker-types). | P1.1 | ~20 tool calls | All 12 tests carry an FR comment; clippy clean. |
| P1.3 | Add `make check-frs` (`@trace`/FR-id grep + diff against `agileplus-specs/civ-*/meta.json::fr_ids[]`); wire as required check in `.github/workflows/ci.yml`. | P1.1, P1.2 | ~40 tool calls | PR with an FR not covered by any test fails CI with the specific FR ID. |
| P1.4 | Annotate the 13 `crates/legends/tests/saga_graph.rs` tests with the FR-CIV-LEGENDS-* IDs already listed in the file header (lines 1-2). | P1.1 | ~10 tool calls | Header comment can be removed; tests self-cite. |
| P1.5 | Annotate the 4 `crates/engine/tests/{tick_budget,end_to_end_tick,determinism_proptest,invariants_proptest}` tests with FR-CORE-001/003/004/007. | P1.1 | ~10 tool calls | All engine SHALs have at least one annotated test. |

**Total: ~110 tool calls, ~3–5 min wall clock with parallel codex workers.** Blocks: nothing; P1.3 unlocks the G1 closure check.

### Phase 2 — BDD Gherkin for the 4 highest-value engine invariants (closes G2 + G3)

| Task | Description | Depends On | Effort | Success criterion |
|---|---|---|---|---|
| P2.1 | Add `cucumber` (Rust crate) to `crates/engine/Cargo.toml` dev-deps; scaffold `crates/engine/features/`. | P1.3 | ~20 tool calls | `cargo test --test bdd` runs an empty feature file. |
| P2.2 | Write `crates/engine/features/tick_loop.feature` covering `FR-CORE-001` (fixed 100 ms tick, monotonic counter, 10 000 ticks in < 2 min). | P2.1 | ~30 tool calls | Feature file is reviewable in plain English; cargo test passes. |
| P2.3 | Write `crates/engine/features/determinism.feature` covering `FR-CORE-003` (bit-identical replay, same seed → same state hash). | P2.1 | ~30 tool calls | Same. |
| P2.4 | Write `crates/engine/features/replay_round_trip.feature` covering `FR-REPLAY-001/002` (.civreplay header + log + checksum; load-and-verify). | P2.1 | ~30 tool calls | Same. |
| P2.5 | Write `crates/engine/features/joule_conservation.feature` and add the corresponding `proptest! { … }` closure for `FR-ECON-002`. Closes G3. | P2.1 | ~40 tool calls | Property test asserts Σ Joules is invariant for 1 000 random seeds. |
| P2.6 | Add `cargo test --test bdd` to `make validate`; cucumber-messages JSON published as a CI artifact. | P2.2-P2.5 | ~10 tool calls | PR shows the BDD report link. |

**Total: ~160 tool calls, ~5–7 min wall clock.** Blocks: nothing; G2 + G3 closed.

### Phase 3 — Performance gate + observability (closes G4 + G5; informs G6)

| Task | Description | Depends On | Effort | Success criterion |
|---|---|---|---|---|
| P3.1 | Integrate `criterion` for `tick_budget` benchmark; pin a 4-core/16 GB GH Actions runner as the reference host. Closes G4. | P1.3 | ~50 tool calls | P99 tick budget enforced as a CI gate; baseline.json committed. |
| P3.2 | Adopt OTel semantic conventions; emit one `span` per `phase_*` call; propagate `traceparent` through `crates/server` WS+JSON-RPC. (Minimum viable: just the engine phases, no UI integration.) Closes G5 (partial). | P1.3 | ~80 tool calls | `jaeger` UI shows one trace per `sim.snapshot` call with engine-phase children. |
| P3.3 | Convert the 4 visual `FR-CIV-WEB-*` (002/003/006/007) to Playwright + Gherkin. Closes G6. | P1.3 | ~120 tool calls | CI attaches the Playwright trace viewer zip for every PR. |
| P3.4 | (Optional) Generate a ReqIF export of all `FR-*` IDs + status from `agileplus-specs/civ-*/meta.json` for procurement-grade audit. Informs §3.5. | P1.3 | ~30 tool calls | `make reqif-export` produces a `.reqifz` artifact committed to the release. |

**Total: ~280 tool calls, ~10–15 min wall clock.** Blocks: nothing; fully parallelizable with Phase 1 + Phase 2 (no cross-phase DAG edges).

### 5.5 DAG of phase dependencies

```
P1.1 (clippy lint)
 ├─ P1.2 (annotate bevy-ref tests) ─┐
 ├─ P1.4 (annotate saga_graph)      ├─ P1.3 (make check-frs)
 └─ P1.5 (annotate engine tests)   ─┘            │
                                                 ├─ P2.1 (cucumber scaffold)
                                                 │   ├─ P2.2 (tick_loop.feature)
                                                 │   ├─ P2.3 (determinism.feature)
                                                 │   ├─ P2.4 (replay_round_trip.feature)
                                                 │   ├─ P2.5 (joule_conservation.feature) ─ closes G3
                                                 │   └─ P2.6 (BDD CI wiring)
                                                 ├─ P3.1 (criterion tick budget) ─ closes G4
                                                 ├─ P3.2 (OTel engine phases) ─ partial G5
                                                 ├─ P3.3 (Playwright Gherkin) ─ closes G6
                                                 └─ P3.4 (ReqIF export) ─ §3.5 readiness
```

## 6. Appendix A — Full BDD scenario → source-file map

Every `#[test]` in `clients/bevy-ref/tests/*.rs`. "FR ID (best-fit)" is the most defensible current FR to map the test to, based on the test's GIVEN/WHEN/THEN comment, the symbols it imports, and the candidate rows in `fr-3d-matrix.md` and `FUNCTIONAL_REQUIREMENTS.md`. Mappings marked **(none yet)** indicate an FR ID that should be authored (P1.2 in the adoption plan covers this).

| # | File:line | Test name | Source contract (imports) | FR ID (best-fit, P1.2 work) | Status (gated / passing) |
|---|---|---|---|---|---|
| 1 | `requirements_bdd.rs:47` | `requirement_world_size_selection_changes_dimensions` | `civ_bevy_ref::voxel_sim::world_dims_for` | **FR-CIV-VOXEL-000** (voxel substrate — size presets); see `fr-3d-matrix.md:18-22` | gated `#[cfg(feature="voxel")]`; passing in that config |
| 2 | `requirements_bdd.rs:85` | `requirement_new_world_differs_from_previous` | `civ_voxel::worldgen::generate` + `civ_bevy_ref::voxel_sim::world_dims_for` | **FR-CIV-VOXEL-001** (regenerate) | gated `voxel`; passing |
| 3 | `requirements_bdd.rs:119` | `requirement_2d_map_extent_matches_world` | `civ_voxel::fluid_ca::CaGrid::new` + `civ_bevy_ref::map2d::world_extent_for_basemap` | **FR-CIV-MAP2D-001** *(author — closest is the 3D-matrix `MAP2D` rows, none exist yet)* | gated `egui`; passing |
| 4 | `requirements_bdd.rs:141` | `requirement_water_only_below_sea` | `civ_voxel::worldgen::{generate, sea_level}` | **FR-CIV-CA-007** (sea-level pass) | un-gated (uses worldgen only); passing |
| 5 | `requirements_bdd.rs:166` | `requirement_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit` | `civ_bevy_ref::camera::{camera_input, CameraRig}` + bevy `ButtonInput`, `Time`, `Messages<{MouseMotion,MouseWheel}>` | **FR-CIV-BEVY-024** (P-W1 item 49, controls) | gated `bevy`; passing |
| 6 | `requirements_bdd.rs:335` | `requirement_settings_has_gfx_audio_controls_gameplay_tabs` | `civ_bevy_ref::settings_ui::{settings_tabs, SettingsTab}` | **FR-CIV-BEVY-024** (settings UI) | gated `egui`; passing |
| 7 | `requirements_bdd.rs:372` | `requirement_emergent_factions_no_fixed_count_or_alignment` | `civ_engine::Simulation::{with_seed, tick, faction_count, faction_alignment}` + `civ_agents::Alignment` | **FR-CIV-EMERGENT-001** (emergent factions) | gated `bevy`; passing |
| 8 | `requirements_bdd.rs:401` | `requirement_actor_spawn_avoids_t_pose_and_animates` | `civ_bevy_ref::animation::{clip_frame_for_test, idle_angles_for_test}` + `civ_agents::ActorVisualKind` | **FR-CIV-SPECIES-016** (≥100 morphed agents @ 60 FPS) | gated `bevy`; passing |
| 9 | `requirements_bdd.rs:466` | `requirement_native_ocean_renders_with_sea_level_match` | `civ_voxel::worldgen::{generate, sea_level}` (then comments that bevy_water visual match is in interactive dev) | **FR-CIV-CA-007** + **FR-CIV-CA-009** | un-gated; passing |
| 10 | `requirements_bdd.rs:529` | `requirement_keybind_rebinding_overrides_default` | `civ_bevy_ref::settings_ui::{GameSettings, KeyBinding}` + `bevy::input::keyboard::KeyCode` + `ron::{ser,from_str}` | **FR-CIV-BEVY-024** (controls + persistence) | gated `egui`; passing |
| 11 | `requirements_bdd.rs:554` | `requirement_terrain_is_continuous_not_blobs` | `civ_voxel::worldgen::generate` + `civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel}` | **FR-CIV-VOXEL-010** (Mesher trait watertight) | un-gated; passing |
| 12 | `requirements_bdd.rs:583` | `requirement_marker_types_differentiate_server_attach_vs_in_process` | `civ_bevy_ref::live_stream::{LiveAgentTag, LiveBuildingTag}` + `civ_bevy_ref::sim_bridge::{SimCivilianMarkerPublic, SimBuildingMarkerPublic}` | **FR-CIV-BEVY-026** (live attach markers) | gated `bevy`; passing |
| 13 | `settings_bdd.rs:10` | `bdd_given_ultra_preset_when_applied_then_all_rich_fields_are_max` | `civ_bevy_ref::settings_ui::{GraphicsSettings, QualityPreset, ShadowQuality, AntiAliasing, TextureQuality}` | **FR-CIV-BEVY-024** (graphics settings) | gated `bevy+egui`; passing |
| 14 | `settings_bdd.rs:29` | `bdd_given_manual_shadow_change_when_applied_then_quality_becomes_custom` | same | **FR-CIV-BEVY-024** | same; passing |
| 15 | `settings_bdd.rs:40` | `key_lookup_is_authoritative` | `civ_bevy_ref::settings_ui::{GameSettings, KeyBinding}` + `bevy::input::{KeyCode, MouseButton}` | **FR-CIV-BEVY-024** | same; passing |
| 16 | `world_size_bdd.rs:6` | `world_size_selection_maps_to_increasing_dimensions` | `civ_bevy_ref::voxel_sim::world_dims_for` | **FR-CIV-VOXEL-000** | gated `voxel`; passing |

**Total: 16 BDD tests, 0 ignored, 0 placeholders, 0 unrelated to the FR catalogue.** 5 of the 16 are gated by a single feature flag (`bevy` or `egui` or `voxel`); without those features they don't run, which is fine because the gated code is also not compiled. The single finding is **#6, #10, #13, #14, #15 all map to the same FR (FR-CIV-BEVY-024)** — that FR is doing a lot of work and should be split before the formal audit; recommend a follow-up doc-cleanup task.

---

*End of report. For changes, open a PR against this file and re-run `make check-frs` (P1.3).*
