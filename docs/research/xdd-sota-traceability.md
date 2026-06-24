# xDD / SOTA Traceability for Civis

**Status:** actionable recommendation  
**Scope:** requirements → docs → tests → code → PR traceability for Civis, with Rust-first implementation and TS/React support.  
**Decision:** extend Civis' existing AgilePlus/spec corpus with a small local traceability harness; do **not** build a new platform first.

## Executive recommendation

Civis already has the hard parts of xDD in place: `docs/specs/requirements/`, BDD-style Rust tests in `clients/bevy-ref/tests/requirements_bdd.rs`, FR/NFR references in docs, CI-quality gates, and a strong agent-delivery workflow. The fastest stable path is:

1. Treat every feature as a trace tuple: **Requirement ID → spec section → test name → code path → commit/PR**.
2. Add a lightweight local trace index generator that scans markdown + Rust tests + git history.
3. Keep Rust tests as the source of executable truth; use docs only for intent and acceptance criteria.
4. Add web/client e2e only where Rust cannot validate player-visible behavior.

Do not replace AgilePlus or Tracera yet. Wrap them if useful; Civis needs a deterministic, local, OSS/free, repo-native trace loop.

## Current Civis trace state

| Layer | Current asset | Status | Next action |
|---|---|---|---|
| Requirements | `docs/specs/requirements/*.md`, `docs/specs/backlog.md` | Good corpus | Normalize IDs and acceptance checks |
| BDD tests | `clients/bevy-ref/tests/requirements_bdd.rs`, `settings_bdd.rs`, `world_size_bdd.rs` | Improving; 6/12 requirements active | Continue converting ignored stubs into public APIs |
| Rust implementation | `crates/*`, `clients/bevy-ref/src/*` | Broad, modular | Add FR doc-comments where APIs are public |
| Verification | `cargo check`, `cargo test`, targeted feature checks | Good | Add trace report to quality target |
| PR/commit trail | commit messages already cite feature work | Partial | Add optional FR trailers in commits/PR bodies |

## Rust xDD stack

| Need | Recommended tool | Why |
|---|---|---|
| Unit and integration tests | Built-in `cargo test` | Deterministic and already in use |
| BDD scenarios | Existing Rust test files + naming convention (`requirement_*`, `bdd_given_*`) | Lower friction than adding a Gherkin runtime |
| Gherkin if needed | `cucumber` / cucumber-rs | Use only for stakeholder-facing feature files |
| Parameterized tests | `rstest` | Good for requirement matrices and scenario tables |
| Property tests | `proptest` | Fits deterministic simulation invariants |
| Fuzzing | `cargo-fuzz` | Best for parsers, protocol frames, save/replay inputs |
| Snapshot tests | `insta` | Good for JSON protocol frames, generated docs, UI state dumps |
| Compile-fail tests | `trybuild` | Useful for SDK/plugin API invariants |
| Bench/perf gates | `criterion` | Use for voxel meshing, streaming, simulation ticks |

Guidance: keep BDD as ordinary Rust first. Add Gherkin only when the scenario prose itself needs to be an artifact.

## TS/React xDD stack

| Need | Recommended tool | Why |
|---|---|---|
| Unit/component | `vitest` | Fast, local, modern Vite fit |
| DOM/component | Testing Library | Tests behavior, not implementation details |
| E2E | Playwright | Screenshot/video artifacts, robust browser automation |
| Visual regression | Playwright screenshots or `@storybook/test-runner` if Storybook is adopted | Keep browser checks localized to UI |
| Contract tests | Generated protocol fixtures from `crates/protocol-3d` | Avoid duplicated client truth |

Use `bun` for package management per repo policy.

## Screenshot / render e2e

For Bevy and wgpu-rendered features, use a layered strategy:

1. Pure Rust invariants first: world extents, marker ownership, water placement, mesh continuity.
2. Headless/scene-dump tests second: entity counts, materials, labels, camera state.
3. GPU screenshots only for features where pixels are the requirement: ocean shading, T-pose absence, bloom/GI, map composition.

Do not block ordinary CI on unavailable GPU paths. Provide software mock assertions for physics invariants and run GPU captures in the native dev lane.

## Proposed local trace harness

Add a small `xtask` or `crates/civis-cli trace` command later. Minimal behavior:

1. Scan `docs/specs/**/*.md` for IDs like `FR-CIV-*`, `NFR-CIV-*`, `GODTOOL-*`, `PSYCHE-*`, `ROAD-*`.
2. Scan `clients/**/tests/**/*.rs` and `crates/**/tests/**/*.rs` for `requirement_`, `bdd_`, and ID mentions.
3. Scan source doc-comments for requirement IDs.
4. Emit `docs/reports/traceability-matrix.md` with:
   - requirement ID
   - spec file
   - tests
   - implementation files
   - latest commit touching each file
   - status: `covered`, `stubbed`, `missing-test`, `missing-code`.

This stays local, OSS/free, deterministic, and agent-friendly.

## Build vs extend decision

**Extend Civis.** Build only the thin scanner/harness that Civis lacks.

Reasons:

- AgilePlus/spec docs already encode the product intent.
- BDD Rust tests are already converting ignored requirements into executable APIs.
- Adding a heavy external ALM/requirements platform would slow agent iteration and create sync drift.
- A local trace report can be generated in CI and reviewed in PRs.

## Immediate WBS / DAG

| Phase | Task | Depends on | Output |
|---|---|---|---|
| 1 | Normalize active BDD test names and requirement IDs | none | `requirement_*` / `bdd_given_*` convention |
| 1 | Convert ignored BDD stubs to public test APIs | none | More green requirements in `requirements_bdd.rs` |
| 2 | Add trace scanner command | Phase 1 | `cargo run -p civis-cli -- trace` or `xtask trace` |
| 2 | Emit trace matrix report | scanner | `docs/reports/traceability-matrix.md` |
| 3 | Gate PRs on no newly orphaned requirement IDs | matrix | CI check |
| 3 | Add Playwright/scene screenshots for pixel-only requirements | render harness | screenshot artifacts |

## Cross-project reuse opportunities

- **Trace scanner:** candidate shared Phenotype utility after Civis proves the shape.
- **BDD naming convention:** reusable in other Phenotype Rust workspaces.
- **Protocol fixtures:** reusable across Bevy/Godot/Unreal/web clients.
- **Agent prompt templates:** `.worktrees-prompts/` should stay ignored locally, but stable prompts can move into `docs/reference/` once proven.

## Next worker prompts

1. Convert `requirement_settings_has_gfx_audio_controls_gameplay_tabs` from ignored to active by exposing `settings_tabs()`.
2. Convert `requirement_keybind_rebinding_overrides_default` by exposing `GameSettings::rebind` and RON round-trip test.
3. Convert `requirement_native_ocean_renders_with_sea_level_match` with a software invariant test for sea-level and sky-piercing water columns.
4. Add `civis-cli trace` to generate a markdown matrix.
5. Add CI job: run trace and fail on newly introduced requirement IDs without tests.
