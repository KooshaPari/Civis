# Container-Binding Triage — Engine / Core / Social / Protocol

**Audit date:** 2026-09-29
**Scope:** the 46 entries in `docs/audits/container-bindings.json` whose `id` starts with `FR-SOC-`, `FR-PROT-`, `NFR-CIV-PERF-002`, `FR-CIV-ARCH-`, or `FR-CIV-CORE-`.
**Method:** read the spec requirement text for every ID, then locate the artifact that actually performs the behavior. A test that asserts `struct::default()` parses, or a test whose assertion contradicts the requirement, is **not** an implementation.

## The defect, in one picture

`crates/engine/src/engine.rs:381-409` stacks **29 separate FR/NFR tags** into one comment block immediately above `pub struct WorldState`:

```
381  // FR-CIV-PERF-RT-003, FR-SOC-INS-001 .. FR-SOC-INS-007
382      // FR-CIV-ARCH-006
383      // FR-CIV-CORE-002
384      // FR-CIV-CORE-004
385      // FR-CIV-CORE-019
386      // FR-SOC-CIV-001
...
403  // FR-SOC-INTG-001   <- through
409  // FR-SOC-INTG-007
410  #[derive(Debug, Clone, Serialize, Deserialize)]
411  pub struct WorldState {
412      pub tick: u64,
413      pub population: u64,
414      pub energy_budget_joules: Fixed,
415      pub rng_seed: u64,
```

`WorldState` is a passive aggregate of ~120 public fields. It has no `impl` block performing any of the tagged behaviors. A second block at `engine.rs:713-719` tags `pub struct Simulation` with 7 more behavior requirements. Neither struct is a legal artifact for the requirements they claim.

### Crate wiring (the decisive structural fact)

| Crate | In workspace | `pub use` in lib.rs | Depended on by another crate | Production consumer path |
|---|---|---|---|---|
| `civ-engine` | yes (`Cargo.toml:44` area) | n/a (the crate itself) | **yes — 7 dependents** | **yes** — `civ-server`, `civ-watch`, `clients/bevy-ref`, `clients/godot-ref`, `civ-agents`, `civ-planet`, `civ-emergence-oracle`, `fuzz` |
| `civ-social` | yes (`Cargo.toml:44`) | yes, all modules (`lib.rs:8-18`) | **no** — zero `civ_social` references repo-wide | **none** |
| `civ-server` | yes | yes, extensive (`lib.rs:12-53`) | yes (bevy-ref client, CLI) | **yes** |

**Correction (2026-09-29).** An earlier draft of this report recorded `civ-engine` as an orphan with no production consumer path. That is wrong. `crates/server/Cargo.toml:10` and `:50`, `crates/watch/Cargo.toml:18`, and `clients/bevy-ref/Cargo.toml:25` all depend on `civ-engine`, and `clients/godot-ref/rust/Cargo.toml`, `crates/agents`, `crates/planet`, `crates/emergence-oracle`, and `fuzz` depend on it too — seven dependents, the most of any crate in the workspace. The tags on `WorldState` and `Simulation` are therefore still false, and the per-tag verdicts below are unaffected: `WorldState` remains a passive ~120-field aggregate with no `impl` block performing any tagged behavior, and a container is not a legal artifact for those requirements whether or not its crate is consumed. What the error did undermine is the escalation argument in the following paragraph, which claimed the tags "cannot even be wired up later" because nothing imports them. That reasoning is retracted: the engine is very much wired up, and the honest statement is narrower — the symbols are reachable, but the specific behaviors are performed by other symbols or not at all, and no amount of wiring makes `WorldState`'s field list discharge them.

`civ-social` is a genuine orphan. That part of the original claim stands, and it matters for the `FR-SOC-*` rows: `InsurgencyTracker` is not merely a weak implementation, it is unreachable from any production path.

Of the crates carrying the bogus `WorldState` / `Simulation` / `CommandQueue` tags, only `civ-social` is an orphan, and for that crate the "cannot be wired up later" point is fair. `civ-engine` is not, so for its tags the defect has to be stated on the merits alone: a passive field aggregate is not a legal artifact for a behavior requirement, and no downstream consumer changes that. By contrast the four `FR-PROT-*` requirements are genuinely implemented in `civ-server`, which *is* on a production path, and their tag on `WorldState` is simply in the wrong file.

## Verdict summary

| Verdict | Count |
|---|---|
| IMPLEMENTED-BY-BEHAVIOR (elsewhere; tag on `WorldState` is misplaced but a real impl exists) | 12 |
| DATA-SHAPE-ONLY (struct is the correct artifact) | 2 |
| CONTAINER-ONLY (**FALSE tag**) | 18 |
| NOT-IMPLEMENTED (**FALSE tag**) | 14 |

## FR-SOC — social module (23 tags)

The `civ-social` crate has five modules: `events`, `health`, `ideology`, `insurgency`, `stress`. There is **no** coercion, cohesion, civic-compartment, polarization, spatial-diffusion, intervention, or cross-module-coupling code anywhere in it or in the engine. `InsurgencyTracker::tick` (`crates/social/src/insurgency.rs:48`) is a two-branch boolean hysteresis toggle over one integer — it computes no risk function, no coercion response, no mobilization, no cell lifecycle, no amnesty, no COIN detection.

| ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-SOC-INS-001 | NOT-IMPLEMENTED | none | CIV-0106:1998 | "Risk increases under max drivers" is a risk *function* over coercion+stress; `WorldState` has no such field and `InsurgencyTracker::tick` only toggles a bool at a fixed threshold. |
| FR-SOC-INS-002 | NOT-IMPLEMENTED | none | CIV-0106:1999 | "Coercion inflection point" needs a continuous coercion input; no coercion term exists in any crate. |
| FR-SOC-INS-003 | CONTAINER-ONLY | `InsurgencyTracker::tick` — `crates/social/src/insurgency.rs:48` | CIV-0106:2000 | A threshold start test does exist, but it fires on aggregate stress only; the tag on `WorldState` is a container, and cell formation is not modeled. |
| FR-SOC-INS-004 | NOT-IMPLEMENTED | none | CIV-0106:2001 | Amnesty campaign is not implemented anywhere; no amnesty action, input, or effect exists. |
| FR-SOC-INS-005 | NOT-IMPLEMENTED | none | CIV-0106:2002 | Non-linear risk jump near a mobilization threshold requires a mobilization scalar and a risk curve; neither exists. |
| FR-SOC-INS-006 | CONTAINER-ONLY | `InsurgencyTracker::tick` — `crates/social/src/insurgency.rs:55-67` | CIV-0106:4575 | Start/end transitions exist as a 2-state toggle (nascent/active), which is a subset of the declared lifecycle rules; tagged struct is still a container. |
| FR-SOC-INS-007 | NOT-IMPLEMENTED | none | CIV-0106:4576 | COIN detection probability is not implemented; no detection probability function or counterinsurgency agent exists. |
| FR-SOC-CIV-001 | NOT-IMPLEMENTED | none | CIV-0106:4570 | Civic compartmental R₀ / criticality is not computed anywhere; no R₀ symbol, no civic compartment type. |
| FR-SOC-CIV-002 | NOT-IMPLEMENTED | none | CIV-0106:4571 | E+A+R compartment conservation has no representation; `WorldState` holds no such compartments. |
| FR-SOC-COH-001 | CONTAINER-ONLY | `Simulation::phase_cohesion` — `crates/engine/src/engine/social_settlement_phases.rs:273` | CIV-0106:1986 | A real cohesion phase exists, but its `fabric_score` (line 307) is kinship+trust−hardship+institutions with **no coercion term**; decay-under-coercion is not implemented. |
| FR-SOC-COH-002 | CONTAINER-ONLY | `Simulation::phase_cohesion` — `social_settlement_phases.rs:307` | CIV-0106:1987 | `fabric_score` has no welfare-floor term, so welfare does not reinforce cohesion. |
| FR-SOC-COH-003 | NOT-IMPLEMENTED | none | CIV-0106:1988 | No polarization variable exists in any crate; feedback acceleration cannot be implemented. |
| FR-SOC-COH-004 | NOT-IMPLEMENTED | none | CIV-0106:1989 | `phase_cohesion` iterates settlements independently and never diffuses to neighbors; no spatial adjacency term. |
| FR-SOC-INT-001 | NOT-IMPLEMENTED | none | CIV-0106:2003 | Interventions do not exist as a type; no welfare-floor or coverage toggle. |
| FR-SOC-INT-002 | NOT-IMPLEMENTED | none | CIV-0106:2004 | Info-integrity intervention and ideology diffusion rate are absent; grep for `diffusion` in `crates/social/src` returns 0 matches. |
| FR-SOC-INT-003 | NOT-IMPLEMENTED | none | CIV-0106:2005 | "Interventions emit events" presumes interventions; none exist to emit. |
| FR-SOC-INT-004 | NOT-IMPLEMENTED | none | CIV-0106:2006 | Expiry semantics require an intervention lifetime; no such type exists. |
| FR-SOC-INTG-001 | CONTAINER-ONLY | none (the named coupling is absent) | CIV-0106:2007 | **The confirmed defect.** Spec requires social outputs to propagate into CIV-0105 insurgency; `civ-social` has zero dependents, so the coupling cannot run. Tag on `WorldState` is a false claim. |
| FR-SOC-INTG-002 | NOT-IMPLEMENTED | none | CIV-0106:2008 | CIV-0105 coercion index → insurgency risk edge does not exist. |
| FR-SOC-INTG-003 | NOT-IMPLEMENTED | none | CIV-0106:2009 | Dissenting-stage → susceptibility edge does not exist. |
| FR-SOC-INTG-004 | NOT-IMPLEMENTED | none | CIV-0106:4577 | Coalition stability → insurgency propensity edge does not exist. |
| FR-SOC-INTG-005 | NOT-IMPLEMENTED | none | CIV-0106:4578 | Epidemic → labor/joule output edge does not exist; no cross-module health-labor link. |
| FR-SOC-INTG-006 | NOT-IMPLEMENTED | none | CIV-0106:4579 | Radicalization attractor dynamics are not implemented; `IdeologyScore` is a static score, not an ODE. |
| FR-SOC-INTG-007 | NOT-IMPLEMENTED | none | CIV-0106:4580 | Civic recovery path under sustained welfare+legitimacy is not implemented. |

## FR-PROT — protocol (4 tags)

All four requirements are real, are specified in `docs/traceability/TRACEABILITY_MATRIX.md:178-183` (source CIV-0200), and **are** implemented in `civ-server` — a crate that is on a production path. The defect is purely that the tags were pasted onto `WorldState` in a different crate. Note `crates/protocol/` referenced by the stale `docs/fragmented/` matrix does not exist; the real home is `crates/server/`.

| ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-PROT-001 | IMPLEMENTED-BY-BEHAVIOR | `dispatch_request` / `parse_request` / `JsonRpcRequest` — `crates/server/src/jsonrpc.rs`, re-exported `lib.rs:36-43` | TRACEABILITY_MATRIX.md:178 | JSON-RPC 2.0 over the ws bridge is implemented in `civ-server`; the `WorldState` tag is misplaced. |
| FR-PROT-002 | IMPLEMENTED-BY-BEHAVIOR | `run_ws_bridge` / `spawn_ws_bridge` — `crates/server/src/ws_bridge.rs`, re-exported `lib.rs:50-53` | TRACEABILITY_MATRIX.md:179 | Notification broadcasting exists in the ws bridge; the matrix's cited test symbol `event_envelope_valid` does not exist under that name, so the named evidence is stale even though the behavior is real. |
| FR-PROT-003 | IMPLEMENTED-BY-BEHAVIOR | `SnapshotFields` / `parse_request` — `crates/server/src/jsonrpc.rs`, re-exported `lib.rs:40-42` | TRACEABILITY_MATRIX.md:180 | Request/response field structs supply the envelope fields; cited symbol `envelope_fields_present` does not exist. |
| FR-PROT-005 | IMPLEMENTED-BY-BEHAVIOR | `BearerToken::parse` — `crates/server/src/authn.rs:51`, re-exported `lib.rs:30` | TRACEABILITY_MATRIX.md:182 | Bearer parsing with rejection of missing/unsupported authorization is implemented and unit-tested; cited symbol `unauthenticated_rejected` does not exist. |

## FR-CIV-ARCH / NFR (6 tags)

| ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-ARCH-006 | CONTAINER-ONLY | `World` (hecs) held at `engine.rs:723` | not found in any spec | No spec defines FR-CIV-ARCH-006 (`defined_by_spec: false`); the ECS world is a separate `hecs::World` field, not `WorldState`. |
| FR-CIV-ARCH-NOSVG-001 | DATA-SHAPE-ONLY | `crates/asset-pipeline` (`resvg`, `svg_export` bin) | CIV-0600:3218 | This is a build-time/CI bundle assertion (`check_bundle_no_svg_runtime.sh`) about the asset pipeline, with no bearing on `WorldState`; the tag is meaningless here. |
| NFR-CIV-PERF-002 | NOT-IMPLEMENTED | none | non-functional-requirements.md (NFR-CIV-PERF-002) | Requires 60 FPS on Apple M1 Metal at 2560×1440 with 500 entities, verified by bench `client_render_500_entities_metal`; that bench does not exist anywhere in the repo. A `WorldState` field cannot evidence a render-loop frame time. |
| FR-CIV-PERF-RT-003 (line 381, in same block) | NOT-IMPLEMENTED | none | — | Listed for completeness; tagged on the same `WorldState` line, no runtime-perf artifact found. |

## FR-CIV-CORE — simulation loop (13 tags)

Spec source for all: `docs/specs/CIV-0001-core-simulation-loop.md:865-960`, where each is stated as a behavior with a `**Test:**` clause.

| ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-CORE-002 | IMPLEMENTED-BY-BEHAVIOR | `Simulation::save_state_mirror` — `engine.rs:2216`; `Simulation::hash_chain_root` — `engine.rs:2915`; `check_integrity_after_replay_load` — `crates/engine/src/integrity.rs:128` | CIV-0001:872 | "Same state + control → identical state" is enforced by state-hash comparison in the integrity path; `WorldState` is the state that gets mirrored, not the thing that proves determinism. |
| FR-CIV-CORE-003 | DATA-SHAPE-ONLY | `SimRng = ChaCha8Rng` — `engine.rs:228`; `Simulation::with_seed` — `engine.rs:1330`; `rng_mut` — `engine.rs:2113` | CIV-0001:877 | The requirement is "stochastic events use ChaCha8Rng seeded with the seed parameter" — that is genuinely a statement about the RNG *type and seeding*, so a type/field is the right artifact. The tag is defensible but belongs on the `SimRng` alias, not `Simulation`. |
| FR-CIV-CORE-004 | NOT-IMPLEMENTED | none | CIV-0001:882 | "Single tick completes in < 16 ms wall time" requires a timing measurement; grep for `tick_compute_time` across `crates/engine` returns 0 matches. No perf gate backs this. |
| FR-CIV-CORE-006 | IMPLEMENTED-BY-BEHAVIOR | absence enforced by lint; only test-code `SystemTime::now` at `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:25` | CIV-0001:892 | "No `SystemTime::now()` in simulation crate" holds — the sole occurrence is in a test. This is a *negative* constraint, so a struct tag is not the right artifact, but the behavior is genuinely upheld. |
| FR-CIV-CORE-007 | IMPLEMENTED-BY-BEHAVIOR | `#[derive(Serialize, Deserialize)]` on `WorldState` `engine.rs:410`; `SimulationSnapshot` `engine.rs:3427`; `save_bundle::load_dir` — `crates/engine/src/save_bundle.rs:419` | CIV-0001:897 | Round-trip lossless snapshot serialization is real and exercised by the save-bundle loader; here the struct genuinely is part of the mechanism, so the tag is materially legitimate. |
| FR-CIV-CORE-008 | CONTAINER-ONLY | `CommandQueue::push`/`pop` — `crates/engine/src/command_queue.rs:58,72` | CIV-0001:902 | The spec demands a **priority queue** ordering by client priority; the impl is a plain `VecDeque` FIFO with no priority field. The test at `engine/tests/fr_fr_civ_core_008.rs:14` only asserts FIFO. `CommandQueue` is used **only in tests** — no production path. |
| FR-CIV-CORE-009 | IMPLEMENTED-BY-BEHAVIOR | `dispatch_request` — `crates/server/src/jsonrpc.rs`; `JsonRpcMethod` — re-exported `lib.rs:41` | CIV-0001:907 | handshake/command/snapshot/subscribe are implemented in `civ-server`; the tag on `SimulationSnapshot` is misplaced (that struct carries no JSON-RPC behavior). |
| FR-CIV-CORE-011 | IMPLEMENTED-BY-BEHAVIOR | `Simulation::load_replay_from_file` — `engine.rs:2925`; `check_integrity_after_replay_load` — `integrity.rs:128` | CIV-0001:917 | Replay-then-compare-state-hash is implemented and asserted at `integrity.rs:139`; `Simulation` is the replay target, not the verifier. |
| FR-CIV-CORE-012 | DATA-SHAPE-ONLY | `pub struct Fixed(pub(crate) i64)` — `crates/engine/src/fixed_math.rs:14`; enforced by `clippy::float_arithmetic` (doc `fixed_math.rs:8-11`) | CIV-0001:922 | "No floating-point in money, resources, or energy; all i64" is a statement about the numeric *representation*, so the `Fixed` type is exactly the right artifact. **This is the one clearly legitimate container tag in the batch.** |
| FR-CIV-CORE-013 | IMPLEMENTED-BY-BEHAVIOR | `Simulation::tick` — `engine.rs:2295` (phase calls 2328-2384) | CIV-0001:927 | A real ordered phase schedule exists, though it does **not** match the spec's declared order (Command → Policy → Transition → Stochastic → Metrics → Broadcast); the tag on the struct is a container, the behavior lives in `tick()`. |
| FR-CIV-CORE-014 | CONTAINER-ONLY | per-tick buffers e.g. `last_tick_voxel_events` `engine.rs:794`, `chronicle` `engine.rs:548` | CIV-0001:932 | Events are emitted through many per-tick buffers, but there is no single unified event log; `Simulation` merely holds the containers. |
| FR-CIV-CORE-015 | CONTAINER-ONLY | `HashChainState::advance` — `crates/engine/src/hash_chain.rs:58`; `chain_root_from_ticks` — `hash_chain.rs:67` | CIV-0001:937 | Hash-chain *machinery* is real, but the requirement is "every event includes hash of state that produced it" — per-event hash stamping is not implemented; the struct tag is a container. |
| FR-CIV-CORE-016 | NOT-IMPLEMENTED | none | CIV-0001:942 | **Priority tiers do not exist.** `client_priority` appears exactly once repo-wide, in the test's own doc comment (`engine/tests/fr_fr_civ_core_016.rs:6`). `Command` has no priority field, and the test `earlier_tick_processed_first` (line 14) asserts plain FIFO — the opposite of the required priority ordering. |
| FR-CIV-CORE-017 | IMPLEMENTED-BY-BEHAVIOR | `SubscriptionFilter::filter_frames` — `crates/server/src/subscription_filter.rs:167`; `apply_subscribe_params` — line 112 | CIV-0001:947 | Partial/filtered subscription is genuinely implemented in `civ-server`; the tag on `SimulationSnapshot` is misplaced. |
| FR-CIV-CORE-019 | CONTAINER-ONLY | `pub world: World` (hecs) — `engine.rs:723` | CIV-0001:957 | Entities live in the separate hecs `World` field, not in `WorldState`; no `SparseSet` usage found, so the "dense arrays" assertion is unverified. |

## Highest-confidence FALSE tags

These are not judgment calls. The requirement text is explicit, and the tagged artifact cannot satisfy it:

1. **FR-CIV-CORE-016** — no priority field, no priority code, and the sole test asserts FIFO, directly contradicting the spec.
2. **FR-SOC-INTG-001..007** (7 IDs) — the entire `civ-social` crate has **zero** dependents and lacks coercion, cohesion, civic, polarization, diffusion, and intervention code; the cross-module couplings the spec names cannot run.
3. **FR-PROT-001/002/003/005** — real requirements implemented in `civ-server`, tagged on `WorldState` in a crate that is not even in the dependency path. Also, the three cited test symbols (`event_envelope_valid`, `envelope_fields_present`, `unauthenticated_rejected`) do not exist under those names.
4. **NFR-CIV-PERF-002** — a 60 FPS / 16.67 ms M1 Metal render NFR tagged onto a server-side data struct; the required bench does not exist.
5. **FR-CIV-CORE-004** — a wall-clock budget with no timing harness (`tick_compute_time` → 0 matches).
6. **FR-SOC-CIV-001/002, FR-SOC-COH-003/004, FR-SOC-INT-001..004** — the required variables (R₀, compartments, polarization, spatial adjacency, interventions) do not exist in any crate.
