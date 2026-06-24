# Civis Rust workspace test coverage inventory

Summary: verified `#[test]`/`#[tokio::test]` count across discovered crate targets is **717** (subtotal for listed crates, excluding `clients/bevy-ref` because command tooling failed before that pass).

| Crate/Client | #Tests | Covered | Big Gaps | FR/NFR |
| --- | ---: | --- | --- | --- |
| `crates/engine` | 116 | Engine domain modules with in-repo tests (serialization, simulation orchestration, scheduling helpers). | worldgen-relief and high-level runtime integration surfaces appear weakly covered relative to complexity. | no FR mapped |
| `crates/voxel` | 45 | voxel terrain/mesh/data-path helper modules. | **Surface Nets/smooth mesher path is a known high-risk untested area.** | no FR mapped |
| `crates/civis-cli` | 4 | CLI argument/command parsing and helper utilities. | CLI orchestration and failure-mode matrix coverage is shallow. | no FR mapped |
| `crates/legends` | 17 | Legends subsystem unit behavior and transform helpers. | Cross-system persistence/replay coupling behavior is not clearly covered. | no FR mapped |
| `crates/agents` | 51 | Agent lifecycle, planning and policy glue. | AI decision/actionability boundaries vs event-driven runtime behavior likely undertested. | no FR mapped |
| `crates/needs` | 15 | Need and balance primitives. | Resource coupling and pathological depletion/recovery scenarios likely missing. | no FR mapped |
| `crates/diffusion` | 16 | Diffusion kernels/helpers. | large-step stability and non-happy-path boundaries likely sparse. | no FR mapped |
| `crates/laws` | 6 | Rule/law application helpers and simple contracts. | Rule conflict and override precedence scenarios likely untested. | no FR mapped |
| `crates/research` | 16 | Research state/progression helpers. | progression lock-state and rollback/undo behavior likely untested. | no FR mapped |
| `crates/ai` | 7 | AI behavior primitives and utility functions. | Decision traces, fallbacks, and deterministic tie-breaking behavior are high risk. | no FR mapped |
| `crates/tactics` | 93 | Tactics/strategy logic and action-resolution helpers. | Combat/edge-case simulation breadth likely limited by combinatorial growth. | no FR mapped |
| `crates/planet` | 12 | Planet/system generation helpers. | **Worldgen relief/heightmap generation remains a known high-risk untested area.** | no FR mapped |
| `crates/protocol-3d` | 14 | Serialization/contracts for protocol frames. | versioning/compatibility failure tests are likely incomplete. | no FR mapped |
| `crates/mod-host` | 50 | Mod-host runtime and validation helpers. | runtime-to-wasm boundary and rejection-mode behavior needs additional property tests. | no FR mapped |
| `crates/civlab-sdk` | 3 | SDK helper/API surface sanity checks. | integration with engine-side ABI and error surfacing is likely weak. | no FR mapped |
| `crates/genetics` | 9 | Genetics generation/helpers. | crossover/mutation corner cases likely sparse. | no FR mapped |
| `crates/economy` | 29 | Budget/transaction/accounting pathways in unit scope. | inflation/rounding and saturation cases underrepresented. | no FR mapped |
| `crates/species` | 12 | Species traits and composition helpers. | ecology interaction and mutation constraints likely undertested. | no FR mapped |
| `crates/save-db` | 5 | persistence save/load utility tests. | migration/backward-compatibility and corrupted snapshot handling. | no FR mapped |
| `crates/civ-traffic` | 14 | movement/pathing utilities and queueing helpers. | congested multi-agent concurrency behavior likely untested. | no FR mapped |
| `crates/infra` | 20 | infra-level utilities and adapters. | infra error recovery and timeout semantics appear thin. | no FR mapped |
| `crates/watch` | 36 | watch/web telemetry and state-change helpers. | websocket reconnect and stale-state invalidation edge cases. | no FR mapped |
| `crates/build` | 16 | build graph/configuration helpers. | non-happy-path config/feature combinations and downgrade paths. | no FR mapped |
| `crates/server` | 111 | server state, message flow, and transport-adjacent logic. | snapshot/event envelope compatibility and auth-gating paths likely high risk. | no FR mapped |
| `clients/bevy-ref` | N/A (not collected) | N/A | **sim_bridge**, **autoshot/standalone.rs** screenshot flow were not reachable from this pass. | no FR mapped |

Top untested risks:
1. worldgen relief / heightmap generation: add regression tests covering heightmap input extrema and deterministic generation invariants across seeded runs.
2. smooth mesher (Surface Nets, `crates/voxel` + `voxel_smooth_mesher`): add topology/normal/edge-case mesh integrity tests and degenerate cell fixtures.
3. brush mutation / terraform (`material_brush`, `terraform_brush`): add property-based tests for brush composition, overlap precedence, and out-of-bounds mutation constraints.
4. cellular automata (fluid CA / CA step): add invariants and conservation tests over random seeds and bounded domains.
5. sim_bridge (`clients/bevy-ref`): add integration tests around bridge handoff, reconnect, and lifecycle event ordering.
6. autoshot / headless screenshot path (`standalone.rs`): add smoke tests asserting successful capture/encode/flush with fake frame payloads and failure-mode fallback.
7. FR/NFR audit alignment: add explicit FR-mapped tests for any high-turnaround requirements in `FUNCTIONAL_REQUIREMENTS.md` (not completed in this pass).

Requirements traceability gaps:
- `FUNCTIONAL_REQUIREMENTS.md` FR/NFR IDs could not be safely mapped because command tooling failed before source cross-references were harvested.
- `#` none can be declared "covered by tests" with confidence from this pass; treat **all mapped FR/NFR as currently unverified** until a full scan reruns.
