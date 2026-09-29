# Container-Binding Triage — Sim / Domain Crates

**Date:** 2026-09-29
**Scope:** 164 `container-bindings.json` entries whose `file` starts with `crates/agents/`, `crates/economy/`, `crates/research/`, `crates/tactics/`, `crates/genetics/`, `crates/species/`, `crates/planet/`, `crates/legends/`, `crates/ai/`, `crates/civ-traffic/`, `crates/diplomacy/`, `crates/voxel/`, `crates/civ-emergence-metrics/`, `crates/build/`, `crates/server/`, or the enumerated `crates/engine/src/*.rs` files (Simulation / SimulationSnapshot / GOD_ACTION_AUDIT_CAP only in `engine.rs`).
**Method:** For every ID the real requirement sentence was read from its authoritative spec (`docs/specs/*.md`, `docs/specs/requirements/*.md`, or `agileplus-specs/*/spec.md`). The tagged item was then read in source, the owning crate grepped for a function performing the required behavior, and production consumers verified by re-export and cross-crate usage greps. No `cargo test` was run. No `.rs` file was modified.

## Verdict counts

| Verdict | Count |
|---|---|
| NOT-IMPLEMENTED | 81 |
| CONTAINER-ONLY (FALSE tag) | 42 |
| DATA-SHAPE-ONLY (tag legitimate) | 31 |
| IMPLEMENTED-BY-BEHAVIOR | 10 |
| **Total** | **164** |

**123 of 164 tags (75%) are FALSE** (CONTAINER-ONLY or NOT-IMPLEMENTED). Only 41 tags (25%) are legitimate.

---

## Systemic findings

**S1 — The `SESSION_HISTORY_CAP` block is the single largest defect.** `crates/server/src/session.rs:27` places 33 distinct `FR-SESSION-NNN` tags above `pub const SESSION_HISTORY_CAP: usize = 32`, whose own doc comment says *"Kept small: the only consumer that walks this is the audit log + tests."* The requirements are full behavioral SHALLs (turn tokens, hot-seat, observer rejection with error code `-32001`, `POST /api/v1/challenges`, BLAKE3-verified saves, UUIDv7 session ids, autosave slots). None of that exists anywhere in `session.rs` (272 lines, 15 fns, all connection-attribution bookkeeping). All 33 are NOT-IMPLEMENTED.

**S2 — The `WorldState` tag block is the confirmed-defect pattern at scale.** `crates/engine/src/engine.rs:381-409` stacks 34 requirement tags above `pub struct WorldState`, a flat scalar container (`tick`, `population`, `energy_budget_joules`, …). Every one of the 26 `FR-SOC-*` tags (INS/CIV/INT/COH/INTG) is a *dynamics* requirement whose spec test names its own required function. None of those functions exist: `compute_insurgency_risk_from_params`, `InsurgencyParams`, `advance_tick_capture_events`, `measure_net_compliance_effect`, `AmnestyCampaign`, `SocialSnapshot`, `SocialTickParams` return **zero matches** anywhere in `crates/`. `crates/social/src/insurgency.rs` has only a 3-field stress-threshold hysteresis toggle, which is the CIV-0106 `FR-SOCI-003/004` model, not the `FR-SOC-INS-*` risk model.

**S3 — The `fr_*.rs` test files are stubs and prove nothing.** `crates/engine/tests/fr_fr_soc_intg_001.rs` in full:
```rust
#[test]
fn verify_fr_soc_intg_001_basic() {
    let ws = civ_engine::WorldState::default();
    assert!(ws.tick == 0);
}
```
The file header even claims *"Upgraded from stub to real assertions."* — it was not. The same shape (`fr_fr_soc_coh_001.rs`, `fr_fr_soc_ins_001.rs`) appears across `crates/engine/tests/`. **These files are not evidence of any implementation and must not be counted as coverage.** The spec's own gate test names (`test_social_outputs_propagate_to_insurgency_coupling`, `test_risk_increases_under_max_drivers`, `test_cohesion_decays_under_max_coercion`) return **zero matches** in `crates/`.

**S4 — `market.rs` implements an order book, not the market model.** `crates/economy/src/market.rs` is 1149 lines / 65 fns, but every function is order-book machinery (`place_bid`, `place_ask`, `clear_all`, `ask_vwap`, `price_impact`, `microstructure`). Grepping the whole crate for `tâtonnement`/`tatonnement`, `projector`, `price_field`, `membership_weights`, `numeraire` returns **zero matches**. The four `FR-CIV-MARKET-002..005` requirements are about a damped tâtonnement price field, soft-membership type weights, and CDA trust-gated upgrades — none present. Having 65 functions is not evidence; the specific required functions are absent.

**S5 — Orphaned pub API with no production consumer.** Several tagged items have a real function but nothing outside `#[cfg(test)]` calls it: `project_zoom` (`lod.rs:96`, only `lib.rs:232` re-export + own tests), `check_build_capability` (`vehicle_types.rs:106`, **zero** cross-file references), `TickProfile` (`perf.rs:17`, zero references outside `perf.rs`). Per the brief these are marked DATA-SHAPE-ONLY with the orphan noted, not credited as implemented.

**S6 — Several `requirement` fields in the JSON are ID-rename table rows, not requirements.** E.g. `FR-CIV-METRICS-001-TIMESERIES` → *"the real hybrid-replay line is `PLAN.md:151`; non-hyphenated form is a phantom alias"*, and `FR-CIV-RESEARCH-00*-{EXPORT,SCENARIO,SNAPSHOT}` → *"(keep) | `PLAN.md:233-238`"*. For these the actual requirement was recovered from the owning spec, and all three `RESEARCH-00X-*` IDs are phantom aliases of non-hyphenated parents.

**S7 — Many IDs are not in `docs/specs/` at all.** `FR-CIV-FOG-001..005`, `FR-API-002..004`, `FR-CIV-EMERG-004/005` live only in `agileplus-specs/`. Earlier `docs/audits/fr-matrix-2026-06-10.md` records `FR-CIV-FOG-002` and `FR-CIV-EMERG-004` as `SPEC-ONLY`; this audit confirms that status. Any tooling that only scans `docs/specs/` will mis-handle these.

---

## Full verdict table

### crates/agents + crates/ai

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-CULT-001 | DATA-SHAPE-ONLY | `culture.rs:40` `CultureProfile::new` + `culture.rs:137` `drift_populations` | `agileplus-specs/civ-009-culture-diffusion/spec.md` | Requirement is "the Culture entity must exist with this shape"; struct exists and is driven by `drift_populations`/`mutate_traits`, so the tag is legitimate. |
| FR-CIV-3D-002 | CONTAINER-ONLY | none | `agileplus-specs/civ-011-bevy-primary-client/spec.md` | "LOD **Budget Enforcement**" is a behavior; `LodTier` (`lib.rs:273`) is a 3-variant enum. Grep for `lod_budget`/`LodBudget`/`budget_check` in `crates/agents/src/` returns zero — no budget check exists. |
| FR-CIV-PSYCHE-003 | IMPLEMENTED-BY-BEHAVIOR | `psyche.rs:174` `nudge_temperament` | `docs/specs/requirements/FR-CIV-PSYCHE.md` | `nudge_temperament` computes `plasticity = (1.0 - maturity * 0.8).clamp(0,1)` and `lr = 0.002 * plasticity`, exactly the spec's gate; called in production at `engine/src/emergence.rs:749`. Tag is on `Temperament` but the behavior is real and wired. |
| FR-CIV-PSYCHE-002 | IMPLEMENTED-BY-BEHAVIOR | `psyche.rs` `psyche_from_dna` | `docs/specs/requirements/FR-CIV-PSYCHE.md` | "Temperament emerges from DNA" is satisfied by `psyche_from_dna`, re-exported through `civ_agents` and imported at `engine/src/emergence.rs:13`. |
| FR-CIV-PSYCHE-006 | CONTAINER-ONLY | none | `docs/specs/requirements/FR-CIV-PSYCHE.md` | "Cold agents **collapse to cluster-level aggregates** (mean mood, belief centroid, tie density)" is a behavior; `Psyche` (`psyche.rs:68`) is a struct. `update_beliefs` exists but no cold-tier aggregate-collapse path does. |
| FR-CIV-PSYCHE-005 | CONTAINER-ONLY | none | `docs/specs/requirements/FR-CIV-PSYCHE.md` | "Cost O(MAX_TIES) bounded; decay **amortised** (touch on access / periodic sweep)" is a decay-schedule behavior; `PsychGenomeProfile` (`psyche.rs:84`) only holds slot vectors. Grep for `decay`/`sweep` in `psyche.rs` returns zero. |
| FR-CIV-AI-015 | NOT-IMPLEMENTED | none | `agileplus-specs` (balance-analyst row) | "Heuristic anomaly detection → SLM triage" has no implementation; `AiConfig` (`config.rs:7`) is a plain config struct and `crates/ai` has no `anomal`/`balance`/`triage` code. |
| FR-CIV-AI-013 | NOT-IMPLEMENTED | none | `agileplus-specs` (culture-drift row) | "Embeddings → cosine drift → speciation threshold" has no implementation; `EmbedRequest` (`lib.rs:185`) is a request DTO. Grep for `cosine` in `crates/ai/src/` returns zero. |

### crates/economy

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-ECON-003 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | Numeraire selection is the required behavior; `JouleAllocator` (`allocation.rs:55`) is a unit struct whose `allocate` is a linear scale, and `numeraire` returns zero matches in the crate. |
| FR-CIV-MARKET-006 | CONTAINER-ONLY | `allocation.rs:191` `allocate_with` (dispatch only) | `agileplus-specs/civ-002-economy-joule-system/spec.md` | "Polity coercion overlap flips the locale's regime to `Planned`" needs a coercion→regime function; `allocate_with` merely dispatches on the enum. No coercion coupling exists. |
| FR-CIV-MARKET-007 | DATA-SHAPE-ONLY | `currency_trust.rs:264` `step_currency_trust` | `agileplus-specs/civ-002-economy-joule-system/spec.md` | Requirement is the currency-trust state shape + monotone trust update; `CurrencyTrust` is the type and `step_currency_trust` performs the volume/hyperinflation math. Tag legitimate. |
| FR-CIV-MARKET-008 | DATA-SHAPE-ONLY | `institution.rs:220` `post` + `:323` `verify_conservation` | `agileplus-specs/civ-002-economy-joule-system/spec.md` | Credit/debt is a ledger-shape requirement; `LedgerSide` is the type and double-entry `post` + `verify_conservation` implement it. Tag legitimate. |
| FR-CIV-ECON-001-MARKET | CONTAINER-ONLY | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | The `requirement` field is an ID-hyphen-collapse *decision-table row* ("RENAME candidates / Future"), not a requirement; `SCHEMA_VERSION` is `"0.1.0-stub"` and implements nothing. |
| FR-CIV-MARKET-001 | CONTAINER-ONLY | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | Same traceability-table row (a `Traceability:` header line), not a requirement; the tagged `SCHEMA_VERSION` string constant is a version marker only. |
| FR-CIV-ECON-004 | IMPLEMENTED-BY-BEHAVIOR | `institution.rs:391` `collect_taxes` + `:503` `step_institutions` | `agileplus-specs/civ-002-economy-joule-system/spec.md` | "Policy-driven fiscal control" is realized by basis-point rate collection with a per-institution cap, posting balanced transfers. Tag is on `LedgerEntry` (misplaced) but the behavior genuinely exists and is exercised. |
| FR-CIV-MARKET-002 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | "Two projector functions produce the price field" — `projector` and `price_field` return zero matches in `crates/economy/src/`. `market.rs` builds order books, not a projected price field. |
| FR-CIV-MARKET-003 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | "Locales hold a **soft membership over types (weights)**, not a hard switch" — no weights vector exists; `MarketState` is a hard container. `membership_weights` returns zero matches. |
| FR-CIV-MARKET-004 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | "Every priced locale runs **damped tâtonnement** as the baseline price-discovery dynamic" — `tâtonnement`/`tatonnement` return zero matches in the entire crate. The required dynamic does not exist. |
| FR-CIV-MARKET-005 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-002-economy-joule-system/spec.md` | "Where `trust` and trade volume are high AND the locale is near-camera/active, the locale upgrades [to CDA]" — no trust-gated upgrade path; the only `CDA` hit is a doc comment in `allocator.rs:1`. |

### crates/tactics + crates/voxel

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-WAR-030 | DATA-SHAPE-ONLY | `doctrine_fitness.rs:28` `score_doctrine_fitness` | `agileplus-specs/civ-006-deep-combat/spec.md` §4.2 | Requirement is the engagement-stats record feeding doctrine fitness; `FactionEngagementStats::net_pressure` + `score_doctrine_fitness` implement it, and `military_phases.rs:95` calls it in production. |
| FR-CIV-FOG-001 | IMPLEMENTED-BY-BEHAVIOR | `fog_of_war.rs:66` `FogOfWar::update` + `:128` `is_visible` | `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47` | Visibility is genuinely a function of `(unit position, vision radius, terrain LOS)`: Chebyshev-bounded scan, Euclidean radius test, then `line_of_sight_grid`. Wired via `war_bridge.rs:96` `build_fog_for_units`. |
| FR-CIV-FOG-002 | IMPLEMENTED-BY-BEHAVIOR | `war_bridge.rs:149-151` fog gate in `resolve_combat` | `agileplus-specs/civ-015-.../spec.md:51` | "SHALL NOT queue `DamageEvent`s where the attacker has no friendly unit with LOS" is enforced: `if !fog.is_visible(shooter.faction_id, cell) { continue }` before damage is queued. |
| FR-CIV-FOG-003 | DATA-SHAPE-ONLY | `war_bridge.rs:96` `build_fog_for_units`; `scenario.rs` `fog_vision_radius` | `agileplus-specs/civ-015-.../spec.md:54` | Radius is plumbed scenario→config→fog, but the tag sits on the `FogOfWar` container, not the plumbing; the field exists and is consumed. Caveat: the spec's "default baseline scenario SHALL set it to 4 hex" is **not** met — the baseline is `Some(8)` (`engine_tests.rs`), and `build_fog_for_units` is called from the tactics crate only, with the default clamp at 16. |
| FR-CIV-FOG-004 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-015-.../spec.md:57` | "Web dashboard SHALL provide a tactics panel with unit selection, fog overlay, jump-to-engagement" — `web/dashboard/src` has no tactics panel; only `updateTacticsOverlay` in `scene3d.tsx:2018` toggling three.js fog. No unit selection, no jump-to-engagement. `fr-matrix-2026-06-10.md` marks it `SPEC-ONLY`. |
| FR-CIV-FOG-005 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-015-.../spec.md:61` | "Server-side visibility filtering before transmission; observer never receives hidden state" — no observer-mode filter exists. `session.rs` `subscribed_frame_kinds` is an *audit mirror*, and `get_snapshot_for_session` (`engine.rs:3296`) returns the **full unfiltered** snapshot with the filtering stub explicitly deferred in its doc comment. |
| FR-CIV-WAR-013 | DATA-SHAPE-ONLY | `lib.rs:144` `apply_damage` | `agileplus-specs/civ-006-deep-combat/spec.md` §2.6 | "Bridge to tactical" is a data-shape requirement; `DamageEvent` is the type and `apply_damage` converts it into voxel destruction. Tag legitimate. |
| FR-CIV-WAR-022 | DATA-SHAPE-ONLY | `lib.rs:144` `apply_damage` | `agileplus-specs/civ-006-deep-combat/spec.md` §3.4 | "Voxel destruction feedback" is satisfied by `apply_damage` carving a sphere (`within_sphere`, `material_is_solid`) and `DamageEvent::estimated_casualties`. Tag legitimate. |
| FR-CIV-WAR-030 | DATA-SHAPE-ONLY | `lib.rs:200` `evolve_doctrine` | `agileplus-specs/civ-006-deep-combat/spec.md` §4.2 | Doctrine record + genetic-algorithm evolution; `Doctrine` is the type, `evolve_doctrine` mutates composition by fitness, called in production at `military_phases.rs`. |
| FR-CIV-WAR-021 | IMPLEMENTED-BY-BEHAVIOR | `morale.rs:231` `apply_casualties`, `:247` `apply_encirclement`, `:301` `tick_morale` | `agileplus-specs/civ-006-deep-combat/spec.md` §3.3 | "Reads from psyche/agent state" is realized: `MoraleState` derives stance from casualty and encirclement inputs and emits `MoraleEvent`s. Caveat: the morale module is **not yet called from `crates/engine/src/engine/`** — production wiring is pending. |
| FR-CIV-WAR-021 | DATA-SHAPE-ONLY | `morale.rs:211` `stance` | `agileplus-specs/civ-006-deep-combat/spec.md` §3.3 | Same requirement, second tag; `MoraleState` is the state holder and the required behavior lives in its `impl` block. Tag legitimate for the container. |
| FR-CIV-WAR-011 | DATA-SHAPE-ONLY | `movement.rs:44` `operational_movement_pulse` | `agileplus-specs/civ-006-deep-combat/spec.md` §2.4 | Maneuver config + pulse; `OperationalMovementConfig` is the shape and `operational_movement_pulse`/`tick_operational_movement` implement it, imported by `engine/src/engine.rs:50`. |
| FR-CIV-RTS-001 | CONTAINER-ONLY | none | `agileplus-specs/civ-012-godot-secondary-client/spec.md` | "`Q` — Move command (click target to confirm)" is a UI input-binding + confirmation behavior; `GridMove` (`movement.rs:30`) is a 3-field move intent. No keybinding or click-to-confirm handler exists in `crates/`. |
| FR-CIV-WAR-013 | DATA-SHAPE-ONLY | `war_bridge.rs` `tick_war_bridge` | `agileplus-specs/civ-006-deep-combat/spec.md` §2.6 | `CombatEngagement` is the bridge record shape; the bridge drains it into `DamageEvent`s on cadence. Tag legitimate. |
| FR-CIV-RENDER-002 | DATA-SHAPE-ONLY | `voxel/src/material.rs` `MaterialDef` | `docs/specs/CIV-0101-two-zoom-lod-v1.md` | Material definition is inherently a data shape; the type exists and `civ_voxel` consumes it (`MaterialId` is threaded through every voxel call site). |
| FR-CIV-RENDER-001 | DATA-SHAPE-ONLY | `voxel/src/window/mod.rs:127` `WindowPolicy` | `docs/specs/CIV-0101-two-zoom-lod-v1.md` | Window sizing/policy policy is a config shape; the type exists in the voxel crate. |

### crates/server

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-SERVER-001 | DATA-SHAPE-ONLY | `session.rs:43` `SharedSession` | `docs/specs/CIV-0200-client-protocol.md` | WebSocket session identity is a shape requirement; the struct exists, is re-exported, and is documented as the audit key. Tag legitimate. |
| FR-CIV-SERVER-002 | DATA-SHAPE-ONLY | `session.rs:150` `SessionSnapshot` | `docs/specs/CIV-0200-client-protocol.md` | Session snapshot wire shape; type exists and is exported. Tag legitimate. |
| FR-SESSION-001 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md` | `pve` session type with one human + AI nations: no session-type field or AI-nation config exists. `session.rs` has no `pve`/`hot_seat` concept. |
| FR-SESSION-002 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§seed` | Per-AI-nation `ChaCha20Rng` sub-stream derived from session seed: no RNG field or sub-stream derivation in `session.rs`. |
| FR-SESSION-003 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§authority` | Human permanent input authority for the session: `SharedSession.role` is an *operator* role, unrelated to nation ownership. |
| FR-SESSION-004 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§queue` | AI nations submitting through the same `NationAction` queue: no `NationAction` type exists anywhere in `crates/`. |
| FR-SESSION-005 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§validate` | Rejection of non-`NationAction` AI submissions: depends on a `NationAction` type that does not exist. |
| FR-SESSION-006 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§hotseat` | `hot_seat` multi-human shared WebSocket: no session-type field exists. |
| FR-SESSION-007 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§turn` | Turn-token enforcement with error `-32001`: no turn token in `session.rs`; no `-32001` anywhere in `crates/server/`. |
| FR-SESSION-008 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§turn` | `session.turn.end` RPC advancing/validating rotation: no such JSON-RPC method (the method enum has no turn methods). |
| FR-SESSION-009 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§turn` | Turn timeout auto-advance at `expires_at_tick`: no `turn_timeout_ticks` or `expires_at_tick` field exists. |
| FR-SESSION-010 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§sim` | Simultaneous-turn action collection and deterministic resolution: no implementation. |
| FR-SESSION-011 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§observer` | Observer connections receiving broadcasts without injection ability: no observer flag in `SharedSession` (only `role: Option<String>`). |
| FR-SESSION-012 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§observer` | Rejecting observer RPCs with a specific error code: no observer concept or error code present. |
| FR-SESSION-013 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§omniscient` | Omniscient observer mode with `tick_stride`: no mode field exists. |
| FR-SESSION-014 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§visibility` | "SHALL apply server-side visibility filtering before [transmission]" — contradicted by source: `engine.rs:3296 get_snapshot_for_session` returns the full snapshot and its doc comment defers filters to "follow-up lanes". |
| FR-SESSION-015 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§replay` | Replay observer for ENDED sessions with seek: no session-status field or seek handler. |
| FR-SESSION-016 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§challenge` | `POST /api/v1/challenges` with `challenge_id` and queue position: no HTTP route and no `ChallengeBundle` type in the crate. |
| FR-SESSION-017 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§challenge` | Fully headless challenge sessions at max tick rate: no headless/challenge mode exists. |
| FR-SESSION-018 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§score` | Baseline score from an AI-only session: no scoring code. |
| FR-SESSION-019 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§score` | Weighted normalized metric deltas in fixed-point: no score computation. |
| FR-SESSION-020 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§challenge` | `GET /api/v1/challenges/{id}/replay` storing `.civreplay`: no route, no challenge storage. |
| FR-SESSION-021 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§pause` | `session.pause` halting the tick loop at a tick boundary: no `session.pause` method in `JsonRpcMethod` and no pause field in `SharedSession`. |
| FR-SESSION-022 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§resume` | `session.resume` restoring tick loop and continuing the BLAKE3 chain: not implemented. |
| FR-SESSION-023 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§speed` | `session.set_speed` accepting `1..=100` with boundary application: no `ticks_per_second` field or handler. |
| FR-SESSION-024 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§ff` | `session.fast_forward` broadcast suppression + final snapshot: not implemented. |
| FR-SESSION-025 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§events` | `session.paused.v1` / `resumed.v1` / `speed_changed.v1` events: no such event types exist. |
| FR-SESSION-026 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§save` | `session.save` serializing SimState to a named slot with a BLAKE3 hash: no save RPC in `session.rs` (save logic lives in `server/src/saves.rs`, unreachable from a session method). |
| FR-SESSION-027 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§load` | `session.load` verifying the BLAKE3 hash before restore: no load RPC in `session.rs`. |
| FR-SESSION-028 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§autosave` | Autosave to the `"autosave"` slot every `autosave_interval_ticks`: no autosave field or timer. |
| FR-SESSION-029 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§branch` | Loading from an ENDED session creating a branched session with a new `session_id`: no session status field or branch logic. |
| FR-SESSION-030 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§uuid` | UUIDv7 assigned at `session.create`: `SharedSession::new` mints a **UUID v4** per its own doc comment, and there is no `session.create` RPC. |
| FR-SESSION-031 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§validate` | Full `SessionConfig` validation at create: no `SessionConfig` type exists. |
| FR-SESSION-032 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§persist` | Persisting session state to a `sessions` table before responding: **no database layer exists** in `crates/server` and no `sessions` table. |
| FR-SESSION-033 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0900-...:§restart` | Reloading incomplete sessions on engine restart: no persistence layer to reload from. |

### crates/engine — `WorldState` tag block (`engine.rs:381-409` → `struct WorldState` at `:411`)

All 34 tags sit on a flat scalar container. For each, the spec's own required function was searched repo-wide; none exist for the `FR-SOC-*` family.

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-SOC-INS-001 | NOT-IMPLEMENTED | none (nearest: `social/src/insurgency.rs:48` `InsurgencyTracker::tick`) | `docs/specs/CIV-0106-...-v1.md:1692` | "Risk increases under max coercion + max stress" needs `compute_insurgency_risk_from_params` + `InsurgencyParams`; **both return zero matches repo-wide**. `InsurgencyTracker` is a stress-threshold boolean, not a risk model. |
| FR-SOC-INS-002 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1708` | "Coercion inflection — above inflection, marginal return is negative" needs `measure_net_compliance_effect`; zero matches repo-wide. |
| FR-SOC-INS-003 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1719` | "Cell formation fires when mobilization crosses threshold" needs `advance_tick_capture_events` + `SocialSnapshot::with_mobilization`; zero matches repo-wide. |
| FR-SOC-INS-004 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1733` | "Amnesty campaign reduces mobilization and risk" needs `AmnestyCampaign`; zero matches repo-wide. |
| FR-SOC-INS-005 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1752` | "Non-linear risk jump near mobilization threshold" — part of the absent risk function; no non-linear mobilization model exists. |
| FR-SOC-INS-006 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4432` | "Insurgency cell **lifecycle transitions**" needs an `InsurgencyCell` state machine; no cell type exists anywhere. |
| FR-SOC-INS-007 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4453` | "COIN detection probability is monotonically [..]" — no counterinsurgency detection probability model exists. |
| FR-SOC-CIV-001 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4326` | "Civic compartmental R₀ bounded correctly" needs a civic R₀ function; grep for `r0`/`R0`/`reproduction` in `crates/` returns nothing. |
| FR-SOC-CIV-002 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4355` | "Civic compartment population conserved (E+A+R=1.0)" needs a civic compartment step fn; `civic_compartment` returns zero matches. |
| FR-SOC-INT-001 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1770` | "Each intervention toggles output in declared direction" needs an intervention registry + apply fn; `intervention` returns zero matches in `crates/`. |
| FR-SOC-INT-002 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1778` | "Information integrity program reduces ideology diffusion rate" — part of the absent intervention system. |
| FR-SOC-INT-003 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1786` | "Intervention effects are event-emitting" — same absent system; no intervention event types. |
| FR-SOC-INT-004 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1797` | "Expired interventions have no effect" (expiry/time-to-live) — same absent system. |
| FR-SOC-COH-001 | NOT-IMPLEMENTED | none (nearest: `social_types.rs:274` `CohesionEvent`) | `docs/specs/CIV-0106-...-v1.md:1533` | "Cohesion **decays under max coercion**" needs a coercion-driven decay fn; no `cohesion_decay` function exists. `CohesionEvent`/`FabricTier::from_score` classify a score but never apply coercion. Test stub: `engine/tests/fr_fr_soc_coh_001.rs`. |
| FR-SOC-COH-002 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1546` | "Cohesion reinforced by welfare floor increase" — no welfare→cohesion coupling exists. |
| FR-SOC-COH-003 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1559` | "Polarization feedback accelerates cohesion decay" — no polarization→cohesion feedback term. |
| FR-SOC-COH-004 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1571` | "Spatial diffusion propagates across adjacent regions" — no region-adjacency cohesion diffusion. |
| FR-SOC-INTG-001 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1812` | **The confirmed defect.** "Social module outputs couple correctly to insurgency in CIV-0105" is a cross-module coupling behavior; tagged on a plain `pub struct WorldState`. No coupling function exists. Test stub: `engine/tests/fr_fr_soc_intg_001.rs` asserts only `ws.tick == 0`. |
| FR-SOC-INTG-002 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1822` | "Coercion index from CIV-0105 raises insurgency risk" — no diplomacy→insurgency edge. `FactionRelations` carries diplomacy signals, never insurgency. |
| FR-SOC-INTG-003 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:1830` | "Citizen lifecycle dissenting stage raises recruit susceptibility" — no dissenting-stage-to-recruitment coupling. |
| FR-SOC-INTG-004 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4478` | "Faction coalition stability feeds back into insurgency propensity" — no coalition-stability metric, let alone feedback. |
| FR-SOC-INTG-005 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4495` | "Epidemic reduces joule output" (health→economy) — no epidemic-to-joule coupling. `health.rs` exists in `civ-social` but nothing reads it in the economy path. |
| FR-SOC-INTG-006 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4517` | "Radicalization attractor stability" — no attractor dynamics; `ideology.rs` tracks norms without a radicalization attractor. |
| FR-SOC-INTG-007 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0106-...-v1.md:4537` | "Civic recovery path — sustained welfare + legitimacy improvement" — no civic recovery state machine. |
| FR-CIV-CORE-002 | CONTAINER-ONLY | none on WorldState; determinism is tested elsewhere | `docs/specs/CIV-0001-core-simulation-loop.md:872` | "Same state + control → identical state" is a tick-loop behavior; the tag sits on a data struct. Replay determinism is separately exercised by `replay.rs:734` `replay` — the tag points at the wrong symbol. |
| FR-CIV-CORE-003 | CONTAINER-ONLY | `Simulation.rng: SimRng` (seeded) + stochastic phase | `docs/specs/CIV-0001-...:877` | "Stochastic events use ChaCha8Rng seeded with seed" is a behavior; `Simulation` is a container, though the seeded `rng` field is real. |
| FR-CIV-CORE-004 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0001-...:882` | "Single tick completes in < 16 ms wall time" needs a tick-timing assertion; the only real budget code is `perf.rs`, which is orphaned (zero consumers). |
| FR-CIV-CORE-019 | CONTAINER-ONLY | `World` (hecs) field on Simulation | `docs/specs/CIV-0001-...:957` | "Entities modeled as dense arrays / SparseSet; no allocations per iteration" is a layout + performance property; `WorldState` is not the ECS model (`Simulation.world` is). |
| FR-CIV-ARCH-006 | CONTAINER-ONLY | none | `docs/audits/_id_inventory_v3.json` (no spec text) | `requirement` field is empty and no spec defines it; tag is unjustified either way. |
| FR-CIV-ARCH-NOSVG-001 | NOT-IMPLEMENTED | none | `docs/audits/fr-matrix` row (CI script) | "No Runtime SVG Parsing" is enforced by `check_bundle_no_svg_runtime.sh` in CI, not by `WorldState`. No runtime SVG-parsing code exists, but neither does the tagged struct implement a check. |
| FR-PROT-001 | CONTAINER-ONLY | none | `docs/specs/CIV-0200-client-protocol.md` | Protocol requirement tagged on a state struct; no protocol logic in `WorldState`. |
| FR-PROT-002 | CONTAINER-ONLY | none | `docs/specs/CIV-0200-client-protocol.md` | Same. |
| FR-PROT-003 | CONTAINER-ONLY | none | `docs/specs/CIV-0200-client-protocol.md` | Same. |
| FR-PROT-005 | CONTAINER-ONLY | none | `docs/specs/CIV-0200-client-protocol.md` | Same. |
| FR-CIV-PERF-RT-003 | CONTAINER-ONLY | none | `docs/audits/fr-matrix` row (`test_sprite_pool.spec.ts`) | "Sprite Pool Pre-Warm" is a client render-warmup behavior; no sprite pool exists in the Rust engine and `WorldState` does not implement it. |
| NFR-CIV-PERF-002 | CONTAINER-ONLY | none | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md` | An NFR (performance budget) is a measured property, not a struct; `requirement` field is empty. |
| FR-CIV-GODTOOL-921 | NOT-IMPLEMENTED | none | `docs/specs/requirements/FR-CIV-GODTOOL.md` | "God-tool actions SHALL support undo and a blueprint copy/paste of a region" is a behavior; `GOD_ACTION_AUDIT_CAP` (`engine.rs:1136`) is a bare numeric cap. Grep for `fn undo` in the engine returns **zero matches** — undo is absent. |
| FR-CIV-CORE-009 | CONTAINER-ONLY | `server/src/jsonrpc.rs` `JsonRpcMethod` dispatch | `docs/specs/CIV-0001-...:907` | "Implement JSON-RPC 2.0 methods: handshake, command, snapshot, subscribe" is real but lives in `civ-server`, not in `SimulationSnapshot` (`engine.rs:3424`). The tag points at the wrong symbol. |

### crates/engine — `Simulation` tag block (`engine.rs:713-719` → `struct Simulation` at `:721`)

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-CORE-006 | DATA-SHAPE-ONLY | (verified by absence) | `docs/specs/CIV-0001-...:892` | "No `SystemTime::now()` in simulation crate" is a negative/structural property. Verified: `SystemTime::now` appears in `server`, `watch`, `mod-host`, `voxel::stream` (test helper) and `engine/tests`, but **not** in `crates/engine/src/` simulation paths. Tag is acceptable as a structural marker. |
| FR-CIV-CORE-007 | IMPLEMENTED-BY-BEHAVIOR | `engine.rs:3207` `Simulation::snapshot`; `save.rs:221` `snapshot_sim` | `docs/specs/CIV-0001-...:897` | "State can be serialized to a JSON snapshot without loss" is performed by `snapshot()` and the `save.rs` snapshot builders, with a real round-trip test (`save_and_load_round_trip_snapshot_state`). Tag on `Simulation` is misplaced but the behavior exists. |
| FR-CIV-CORE-011 | DATA-SHAPE-ONLY | `replay.rs:734` `replay`; `integrity.rs` | `docs/specs/CIV-0001-...:917` | "Replay a `.civreplay` and verify determinism by state-hash match" is structurally provided by `replay()` + the hash-chain verifier, with 9 real replay-determinism tests. Tag acceptable. |
| FR-CIV-CORE-013 | DATA-SHAPE-ONLY | `engine/src/engine/policy_econ_phases.rs:60` `phase_policy` + phase fns | `docs/specs/CIV-0001-...:927` | "Ticks execute phases in order (Command → Policy → Transition → Stochastic → Metrics → Broadcast)" is structurally satisfied by the phase methods; ordering is asserted by `phase_policy_runs_before_phase_economy`. Tag acceptable as a structural marker. |
| FR-CIV-CORE-014 | IMPLEMENTED-BY-BEHAVIOR | `replay.rs:149` `ReplayLog` + `record_*` | `docs/specs/CIV-0001-...:932` | "Every state-mutating action emits an event to log" is performed by `ReplayLog` recording calls across the tick; `replay_log_round_trips_through_save_load` asserts count > 0. |
| FR-CIV-CORE-017 | CONTAINER-ONLY | none | `docs/specs/CIV-0001-...:947` | "Clients can request **partial snapshots** (filter by entity type, region)" — `get_snapshot_for_session` accepts `subscribed_frame_kinds` but **ignores** it, returning the full snapshot; its doc comment defers filters to "follow-up lanes". No filtering function exists. |

### crates/engine — remaining files

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-INFOVIEW-914 | DATA-SHAPE-ONLY | `building_tiers.rs:337` `available_upgrades` | `docs/audits/fr-matrix` row (B2 Needs Pressure) | Info-view row describing an overlay; the tagged `BuildingTierEngine` is the underlying tier model with real `upgrade`/`downgrade`/`tick`. Tag acceptable as a data-shape pointer. |
| FR-CIV-TERRAIN-005 | CONTAINER-ONLY | `climate.rs:55` `register_coastal_water_column` (writes the marker) | `docs/specs/CIV-0102-climate-followup-v1.md` | "Water placement tools SHALL respect a single source of [truth]" is a tool-behavior; `WATER_MARKER_MATERIAL` (`climate.rs:13`) is a `const MaterialId` alias. The single source exists and is used, but the const itself is not the implementation. |
| FR-CIV-TERRAIN-002 | CONTAINER-ONLY | none | `docs/specs/CIV-0101-two-zoom-lod-v1.md` | "civ-voxel chunk seams SHALL be free of visible [artifacts]" is a rendering property; `CoastalColumn` (`climate.rs:21`) is a 3-field water-level record. No seam-hiding logic exists. |
| FR-CLIENT-003 | DATA-SHAPE-ONLY | `jsonrpc.rs:435` role-gated `sim.command` | `docs/specs/CIV-0200-client-protocol.md` | "Role authorization" is realized by `require_operator` in `jsonrpc.rs` with 4 real tests; `CommandKind` is the shape. Tag acceptable. |
| FR-CIV-CORE-008 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0001-...:902` | "Commands from multiple clients applied in deterministic order (**priority queue**)" — `CommandQueue` (`command_queue.rs:31`) is a plain FIFO `Vec` with `push`/`pop`/`drain`; there is no priority field, no sort, and no `client_priority`. |
| FR-CIV-CORE-016 | NOT-IMPLEMENTED | none | `docs/specs/CIV-0001-...:942` | "Commands prioritized by `(client_priority, tick_received)`" — no priority concept in `Command` or `CommandQueue` (only `client_id`, `seq`, `kind`, `tick_issued`). |
| FR-CIV-NOTIFY-921 | CONTAINER-ONLY | none | `docs/specs/requirements/FR-CIV-NOTIFY.md` | "Rebindable hotkey map" is an input-manager behavior; it is tagged on `CommandQueue`, a data structure with no keybinding logic. |
| FR-CIV-0104-010 | CONTAINER-ONLY | `constraints.rs:260` `recovery_window` (a field) | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md` | "Recovery Window Tracking" is a tracker behavior; the tagged item is the `ConstraintState` struct. A `recovery_window: u64` field exists with value 50, but nothing advances or evaluates it, so the tag is a container claim. |
| FR-CIV-0104-007 | CONTAINER-ONLY | none | `docs/specs/CIV-0104-...` | "Baseline stable under full constraint set" needs a baseline-stability assertion; grep for `baseline` in `constraints.rs` returns only doc comments. No baseline computation exists. |
| FR-CIV-0104-003 | DATA-SHAPE-ONLY | `constraints.rs:593-595` sets `ablation_mode` | `docs/specs/CIV-0104-...` | "ABLATION_MODE flag propagates" is realized: `ConstraintState::ablation_mode` is set from any `Halt` result and copied into the report at `:619`. Tag acceptable. |
| FR-CIV-POLITY-004 | DATA-SHAPE-ONLY | `diplomacy.rs:69` `score_to_kind` | `agileplus-specs/civ-007-diplomacy-laws-government/spec.md` | "A polity's displayed shape is a pure function of its internal edge structure" is realized by `FactionRelations::score_to_kind` mapping a relation score to a `RelationKind`. Tag acceptable. |
| FR-CIV-POLITY-001 | DATA-SHAPE-ONLY | `diplomacy.rs:83` `apply_signal` + `:519` `tick_faction_relation_drift` | `agileplus-specs/civ-007-...` | "Depth (deeper relations)" is realized by a relation matrix updated from `DiplomacySignal`s with per-tick drift. `FactionRelations` is the shape. Tag acceptable. |
| FR-CIV-POLITY-003 | DATA-SHAPE-ONLY | `diplomacy.rs:186` `run_macro_diplomacy_event` | `agileplus-specs/civ-007-...` | "Coercion derives, never declared" — diplomacy state is derived from `apply_signal` + drift each tick rather than assigned, and `DiplomacyEvent` records the derived transitions. Tag acceptable. |
| NFR-CIV-DET-003 | DATA-SHAPE-ONLY | `fixed_math.rs` `Fixed` impls | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md` | Determinism NFR is structurally served by the fixed-point type; `requirement` field is empty and the NFR text is a performance/determinism property, so the tag is a structural marker. |
| FR-CIV-CORE-012 | DATA-SHAPE-ONLY | `fixed_math.rs:4` `Fixed` (+ its arithmetic impls) | `docs/specs/CIV-0001-...:922` | "No floating-point in money, resources, or energy; all i64" is precisely a data-shape/type requirement and `Fixed` is the i64-backed type. **Tag fully legitimate.** |
| FR-CIV-CORE-015 | IMPLEMENTED-BY-BEHAVIOR | `hash_chain.rs:34` `tick_hash` + `:44` `HashChainState` | `docs/specs/CIV-0001-...:937` | "Every event includes the hash of state that produced it" is performed: BLAKE3 per-tick digests chained into `HashChainState`, with tamper-detection tests. Tag legitimate. |
| FR-CIV-INFOVIEW-916 | CONTAINER-ONLY | none | `docs/audits/fr-matrix` row (A3 Temperature) | "Temperature overlay" needs a compute fn producing overlay values; `InfoOverlay` (`info_views.rs:95`) is a metadata record (id, name, group, render_kind, legend stops). No overlay value computation exists in this file. |
| FR-CIV-INFOVIEW-917 | CONTAINER-ONLY | none | `docs/audits/fr-matrix` row (A8 Resource Deposits) | Same — a catalog entry, not an implemented overlay. |
| FR-CIV-INFOVIEW-918 | CONTAINER-ONLY | none | `docs/audits/fr-matrix` row (E1 Roads) | Same — the spec row itself notes "`TrafficGraph` exists; first Gizmo render-kind exemplar", i.e. the overlay is aspirational. |
| FR-CIV-INFOVIEW-919 | CONTAINER-ONLY | none | `docs/audits/fr-matrix` row (C4 Wealth) | Same — "NEAR" priority in the spec's own table; no compute fn. |
| FR-CIV-INFOVIEW-921 | CONTAINER-ONLY | none | `docs/audits/fr-matrix` row (B6 Migration Flow) | Same — "NEAR"; no migration-flow overlay compute fn exists. |
| FR-CIV-TERRAIN-004 | DATA-SHAPE-ONLY | `lod.rs:96` `project_zoom` | `docs/specs/CIV-0101-two-zoom-lod-v1.md` | "Map2D zoom levels SHALL round-trip without voxel-data" — `ZoomLevel` is the shape and `project_zoom` performs the round-trip without touching voxel state. **Caveat: `project_zoom` has no production consumer** (only `lib.rs:232` re-export and its own tests). |
| FR-CIV-METRICS-001-TIMESERIES | CONTAINER-ONLY | none | `docs/audits/fr-matrix` (rename table) | The `requirement` field is an ID-rename row explicitly calling this a **phantom alias** of `FR-CIV-METRICS-001` ("non-hyphenated form is a phantom alias"). `MetricsFixed` (`metrics.rs:40`) is a wrapper type. Phantom ID on a container. |
| FR-CIV-PERF-006 | NOT-IMPLEMENTED | none | `docs/audits/fr-matrix` row (10k Citizens) | "Full snapshot at 10k citizens" needs a 10k-citizen snapshot test in the tick path; `TickProfile` (`perf.rs:17`) is a struct of counters with **zero references outside `perf.rs`** — nothing ever constructs it in production. |
| FR-CIV-PERF-004 | NOT-IMPLEMENTED | none | `docs/audits/fr-matrix` row (WS Command Latency) | "WebSocket command latency" measurement — same orphaned `TickProfile`; no latency instrumentation is wired. |
| FR-CIV-POLITY-007 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-007-...` | "A polity dissolves when its internal mean coord falls below the anarchic floor for a sustained window" — grep for `anarch`/`dissolve` returns nothing; `StratBand` (`social_types.rs:104`) is a 4-variant stratification enum with no collapse logic. |
| FR-CIV-NOTIFY-901 | CONTAINER-ONLY | none | `docs/specs/requirements/FR-CIV-NOTIFY.md` | "Alert rules in RON (happiness < X, …) — measured, not scripted" requires RON rule loading; `UnrestLevel` (`social_types.rs:337`) is a 4-variant enum with a hardcoded `from_score` ladder. **Thresholds are hardcoded in Rust, not RON — the requirement's explicit intent is violated.** |
| FR-CIV-NOTIFY-920 | CONTAINER-ONLY | `tutorial.rs:50` `advance_from_sim` | `docs/specs/requirements/FR-CIV-NOTIFY.md` | "Tutorial milestones" is a behavior; `TutorialMilestone` (`tutorial.rs:14`) is an enum. `advance_from_sim` does implement advancement, but the tag is on the enum, not that function. |
| FR-CIV-VEHICLE-002 | DATA-SHAPE-ONLY | `vehicle_types.rs:106` `check_build_capability` | `docs/specs/CIV-0101-two-zoom-lod-v1.md` | "Removing a required material makes new builds of that kind [unavailable]" — `VehicleArchetype` is the shape and `check_build_capability` returns `CapabilityGate::Fail { missing }` for each missing material. **Caveat: `check_build_capability` has zero references outside its own definition** — no build path calls it, so the gate is not actually enforced. |

### crates/build, research, genetics, planet, civ-emergence-metrics, civ-traffic, diplomacy

| ID | Verdict | Real implementing symbol | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-API-002 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-013-research-api/spec.md:26` | "Python scenario runner — `civlab.run_scenario(path, ticks=50)` … `pip install civlab`" is a Python package requirement. `crates/build` is pure Rust; `SCHEMA_VERSION` is `"0.1.0-stub"`. No Python package exists. |
| FR-API-003 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-013-.../spec.md:27` | "Policy parameter override … invalid param names raise `ValueError`" — a Python-side behavior; nothing in `crates/build` implements overrides or `ValueError`. |
| FR-API-004 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-013-.../spec.md` (Phase 4) | "Data export" is a pipeline phase; no export function in `crates/build` (which is building/tileset placement code). |
| FR-CIV-CLIENT-GODOT-001 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-012-godot-secondary-client/spec.md` (Phase 1) | "WebSocket connection and handshake" — `CultureEraWealthVector` (`build/lib.rs:119`) is a 3-field data DTO for procedural generation. No client handshake code in `crates/build`. |
| FR-CIV-CLIENT-GODOT-002 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-012-.../spec.md` (Phase 2) | "3D scene rendering" — same DTO tag; no rendering in `crates/build`. |
| FR-CIV-BIO-001 | CONTAINER-ONLY | `build/lib.rs:311` `insert_parcel` + `:326` `set_provenance` | `agileplus-specs/civ-008-genetics-species/spec.md` (Phase 1) | "Species registry and **YAML schema**" — `BuildingGraph` (`build/lib.rs:286`) is a building-graph structure, not a species registry. It does round-trip RON, but the tagged type does not implement a species registry. |
| FR-CIV-EMERGENCE-003 | CONTAINER-ONLY | `dashboard.rs:68` `EmergenceDashboard::compute` | `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:47` | "Metrics SHALL be exposed on `sim.snapshot.emergence` and the `emergence_metrics.v1` replay-bus event emitted once per N ticks" — `jsonrpc.rs` does build a `sim.snapshot.emergence` block, so the RPC half is met; the `emergence_metrics.v1` replay-bus event is absent. Tag on the struct is a container claim. |
| FR-CIV-EMERGENCE-012 | CONTAINER-ONLY | none | (no spec text; `requirement` empty) | No spec defines this ID; tagged on the dashboard struct. Unjustified tag. |
| FR-CIV-EMERGENCE-013 | CONTAINER-ONLY | none | (no spec text; `requirement` empty) | No spec defines this ID; tagged on the dashboard struct. Unjustified tag. |
| FR-CIV-EMERG-004 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-019-.../spec.md:51` | "Web dashboard SHALL provide an `EmergencePanel` component with a per-metric sparkline (last 120 ticks) and a threshold-color chip" — **`web/dashboard/src` has no `EmergencePanel` file**; the panel set is agents/diplomacy/economy/religion/mods/perf/stats/tech_tree. `fr-matrix-2026-06-10.md` marks it `SPEC-ONLY`. A `sparkline.tsx` primitive exists but is not wired to emergence metrics. |
| FR-CIV-EMERG-005 | NOT-IMPLEMENTED | none | `agileplus-specs/civ-019-.../spec.md:55` | "Bevy primary client SHALL provide a `live_emergence_overlay` HUD toggle (E) … glassmorphism chip group" — `clients/bevy-ref` contains no `live_emergence_overlay` (grep across `clients/` returns zero). |
| FR-CIV-EMERGENCE-011 | CONTAINER-ONLY | none | (no spec text; `requirement` empty) | `EmergenceSampleSnapshot` (`sample_snapshot.rs:10`) is a data carrier; no spec text exists for this ID. |
| FR-CIV-ROAD-902 | DATA-SHAPE-ONLY | `civ-traffic/src/lib.rs:256,288` provenance stamping | `agileplus-specs` road spec | "All structures + roads SHALL carry shared data tags regardless of author" is realized: `record_traffic` stamps `InfraProvenance::Emergent` and `place_segment` stamps `UserPlaced` on the **same** `TrafficGraph`, asserted by `user_and_emergent_share_one_graph`. `InfraProvenance` is the shape. Tag legitimate. |
| FR-DIPL-007 | CONTAINER-ONLY | none | (no spec text; `requirement` empty) | `ShadowNetworkState` (`shadow_networks.rs:192`) is a state record; the `requirement` field is empty and no spec sentence was located. Unjustified tag. |
| FR-CIV-GODTOOL-911 | DATA-SHAPE-ONLY | `genetics/src/lib.rs:97` `mutate`, `:108` `recombine`, `:126` `fitness`, `:162` `should_speciate` | `docs/specs/requirements/FR-CIV-GODTOOL.md` | "Life/spawn tools SHALL seed DNA-bearing organisms; outcome emerges, never scripted" — the outcome functions are genuinely emergent (random mutate/recombine over a `Dna` byte vector, fitness against an environment). `DnaClass` is the data-driven schema. Tag legitimate. |
| FR-CIV-3D-011 | DATA-SHAPE-ONLY | `planet/src/geology.rs:108` `classify_biome` (+ 6 climate-band helpers) | `docs/specs/CIV-0101-two-zoom-lod-v1.md` (Biome Coverage) | "Biome coverage" is satisfied by a full `BiomeKind` enum driven by a real elevation/temperature/moisture classifier, with monotonicity and completeness tests. Tag legitimate. |
| FR-CIV-3D-015 | CONTAINER-ONLY | none | `docs/specs/CIV-0101-...` (Texture Atlas Completeness) | "Texture atlas completeness" requires an atlas table mapping every biome to texture slots; grep for `atlas` in `geology.rs` returns **zero matches**. `BiomeKind` alone does not establish atlas completeness. |
| FR-CIV-RESEARCH-003-EXPORT | CONTAINER-ONLY | `research/src/lib.rs:616` hybrid-replay line | `docs/audits/fr-matrix` rename table | The `requirement` field is an ID-rename row ("the real hybrid-replay line is `crates/research/src/lib.rs:616`"), not a requirement. Phantom alias on a `TechCard` struct. |
| FR-CIV-RESEARCH-001-SCENARIO | CONTAINER-ONLY | `research/src/lib.rs` `LlmEvent::cache_key` | `docs/audits/fr-matrix` rename table | The `requirement` field is a rename row explicitly saying "the real LLM cache + card acceptance is `FR-CIV-RESEARCH-001`". Phantom alias on `TechCard`. |
| FR-CIV-RESEARCH-002-SNAPSHOT | CONTAINER-ONLY | `research/src/lib.rs:179` `replay_advance_llm_event` | `docs/audits/fr-matrix` rename table | The `requirement` field is a rename row ("the real canonical-replay line is `crates/research/src/lib.rs:601`"). Phantom alias on `ValidationOutcome`. |
| FR-CIV-TECH-007 | IMPLEMENTED-BY-BEHAVIOR | `research/src/lib.rs:179` `replay_advance_llm_event` | `agileplus-specs` tech spec (AC-P1) | "Canonical saves never emit `LlmEvent`s; invention draws from canonical + diffused candidates only" is enforced: `ReplayMode::Canonical => Refused(ReplayRefusal::CanonicalLlmEvent)`, asserted by `canonical_replay_refuses_llm`. Real behavior. Tag on `ReplayMode` is defensible. |

---

## FALSE tags requiring removal or retargeting

These 123 tags assert a behavior (or nothing at all) at a bare container. They should be deleted, or moved onto the implementing symbol once one exists.

**`crates/server/src/session.rs:27` — 33 tags on `const SESSION_HISTORY_CAP` (all NOT-IMPLEMENTED):**
`FR-SESSION-001` … `FR-SESSION-033` (contiguous, no gaps).

**`crates/engine/src/engine.rs:381-409` — 26 tags on `struct WorldState` (all NOT-IMPLEMENTED):**
`FR-SOC-INS-001` … `FR-SOC-INS-007`, `FR-SOC-CIV-001`, `FR-SOC-CIV-002`, `FR-SOC-INT-001` … `FR-SOC-INT-004`, `FR-SOC-COH-001` … `FR-SOC-COH-004`, `FR-SOC-INTG-001` … `FR-SOC-INTG-007`.

**`crates/engine/src/engine.rs:381-409` — 8 more on `struct WorldState` (CONTAINER-ONLY / NOT-IMPLEMENTED):**
`FR-CIV-CORE-002`, `FR-CIV-CORE-004`, `FR-CIV-CORE-019`, `FR-CIV-ARCH-006`, `FR-CIV-ARCH-NOSVG-001`, `FR-CIV-PERF-RT-003`, `NFR-CIV-PERF-002`, `FR-PROT-001`, `FR-PROT-002`, `FR-PROT-003`, `FR-PROT-005`.

**`crates/engine/src/engine.rs:713-719` — 6 tags on `struct Simulation`:**
`FR-CIV-CORE-003`, `FR-CIV-CORE-009`, `FR-CIV-CORE-017`, plus `FR-CIV-GODTOOL-921` on `GOD_ACTION_AUDIT_CAP`.

**`crates/build/src/lib.rs:50,116,117` — 5 tags:**
`FR-API-002`, `FR-API-003`, `FR-API-004` (on `SCHEMA_VERSION`), `FR-CIV-CLIENT-GODOT-001`, `FR-CIV-CLIENT-GODOT-002` (on `CultureEraWealthVector`).

**`crates/economy/` — 8 tags:**
`FR-CIV-ECON-003`, `FR-CIV-ECON-001-MARKET`, `FR-CIV-MARKET-001`, `FR-CIV-MARKET-002`, `FR-CIV-MARKET-003`, `FR-CIV-MARKET-004`, `FR-CIV-MARKET-005` (all NOT-IMPLEMENTED), `FR-CIV-MARKET-006` (CONTAINER-ONLY).

**`crates/tactics/` + `crates/voxel/` — 4 tags:**
`FR-CIV-FOG-004`, `FR-CIV-FOG-005` (NOT-IMPLEMENTED), `FR-CIV-RTS-001` (CONTAINER-ONLY), `FR-CIV-3D-002` on `LodTier` (CONTAINER-ONLY).

**`crates/agents/` + `crates/ai/` — 5 tags:**
`FR-CIV-3D-002`, `FR-CIV-PSYCHE-005`, `FR-CIV-PSYCHE-006` (CONTAINER-ONLY), `FR-CIV-AI-013`, `FR-CIV-AI-015` (NOT-IMPLEMENTED).

**`crates/engine/src/` remaining — 21 tags:**
`FR-CIV-INFOVIEW-916` … `FR-CIV-INFOVIEW-919`, `FR-CIV-INFOVIEW-921` (5 CONTAINER-ONLY), `FR-CIV-CORE-008`, `FR-CIV-CORE-016` (NOT-IMPLEMENTED), `FR-CIV-0104-007`, `FR-CIV-0104-010`, `FR-CIV-NOTIFY-921`, `FR-CIV-TERRAIN-002`, `FR-CIV-TERRAIN-005` (CONTAINER-ONLY), `FR-CIV-NOTIFY-901` (CONTAINER-ONLY, violates the RON requirement), `FR-CIV-NOTIFY-920` (CONTAINER-ONLY), `FR-CIV-POLITY-007`, `FR-CIV-PERF-004`, `FR-CIV-PERF-006` (NOT-IMPLEMENTED), `FR-CIV-3D-015` (CONTAINER-ONLY), `FR-CIV-METRICS-001-TIMESERIES` (CONTAINER-ONLY, phantom alias), `FR-CIV-BIO-001`, `FR-CIV-EMERGENCE-003`, `FR-CIV-EMERGENCE-011`, `FR-CIV-EMERGENCE-012`, `FR-CIV-EMERGENCE-013`, `FR-DIPL-007` (CONTAINER-ONLY), `FR-CIV-EMERG-004`, `FR-CIV-EMERG-005` (NOT-IMPLEMENTED), `FR-CIV-RESEARCH-001-SCENARIO`, `FR-CIV-RESEARCH-002-SNAPSHOT`, `FR-CIV-RESEARCH-003-EXPORT` (phantom aliases).

## Recommended follow-up (out of scope, read-only audit)

1. **Stop generating tags from ID provenance rather than from implemented code.** The dominant failure mode is a tool asserting a requirement at the nearest-named struct. Tags should be emitted only where a function body demonstrably performs the requirement.
2. **Delete or quarantine the `fr_*.rs` stub tests.** They inflate apparent coverage while asserting only `Default::default()` parses. Any coverage metric counting them is wrong.
3. **Reclassify the 3 `FR-CIV-RESEARCH-00X-*` and 1 `FR-CIV-METRICS-001-TIMESERIES` phantom aliases** in the ID inventory so they stop producing bindings.
4. **Point the tooling at `agileplus-specs/`.** 9 in-scope IDs live only there; `docs/specs/`-only scanning mis-grades them.
5. **Genuine implementation gaps worth real work**, in priority order: the CIV-0106 social dynamics model (26 requirements, zero implementation), the PvE/session layer (33 requirements, zero implementation), the economy market model (4 requirements, zero implementation), and the web/Bevy emergence + tactics panels (4 requirements, zero implementation).
