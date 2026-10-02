# Civis mature contract recovery — draft v0.1

Program `gaming-pair-20260929`; date2026-09-29. Source **b3cd62a7394878cc64d024fbfcfd398b8bb88bf1**. **Incomplete recovery draft, not an accepted replacement specification.** No requirement quota. The source/authority denominator is OPEN; candidate anchors here are excluded from completion grading until accepted.

## Product identity and corrected scope

Civis is an emergence-first living-world/civilization simulation and interactive godgame, with CivLab research/platform lineage. Recovered user intent includes origin/protolife-to-civilization, societal/economic experimentation and observable player/agent intervention. The charter seeks life, species/sentience, psyche, culture/language, markets and polities emerging from modeled substrate/rules rather than a prescribed list of social outcomes.

Sources: current charter, PRD, ADRs, implementation and retrieved February19-21/August25-29/September16 conversations. Conversation retrieval is discovery evidence, not a full transcript. **The specific May29 charter correction and May30 Accepted-as-recorded ADR drop global deterministic replay.** The May31 v1 boundary excludes multiplayer/co-op/spectator and includes actor rigging/animation. Old multi-client PRD or later generic deterministic prose does not silently override those explicit corrections. Original authorization and any subsequent explicit reversal still require recovery.

Do not conflate a playable simulation with a scientifically validated model. A believable world, correctly executed code, calibrated model and entertaining product are different claims with different evidence. Physics-like rules alone do not prove reality-like social predictions.

## Mature horizon, retained from the beginning

The evidenced horizon includes a physical/material/planetary/genomic substrate; origin/world setup; life/species/cognitive progression; individual needs/psyche/memory; emergent communication/culture/legends; diverse economics/institutions/polities and conflict; agent/user-built infrastructure; direct user intervention and inspection; spatial/multi-resolution world evolution; faithful persistence; extension/model/content interfaces; legible native human experience and machine interfaces; operations, diagnosis and maintained compatibility. Each remains subject to detailed obligation recovery. Specific scientific promises, multiplayer stage/need, all renderer backends, online services and model-validation standards are NOT newly mandated by this draft.

## Ontology and orthogonal projections

| Projection | Entities and relations | Key semantic distinction |
|---|---|---|
| Model/rules | model version -> substrate/rule -> parameters/units -> process/schedule -> observed pattern | Authored rule vs derived measurement vs narrative interpretation; metadata label is not emergence |
| World/state | world identity, regions/chunks, material/energy stocks, agents/genomes, pending events, relationships, structures | Authoritative persisted state vs reconstructible projection vs explicitly ephemeral state |
| Social/behavioral | individuals -> overlapping contacts/kinship/groups -> exchanges/beliefs/coordination -> observed institutional patterns | Fixed faction enum must not silently replace emergent/overlapping membership in the charter |
| User experience | new/origin world -> observe -> inspect -> intervene -> understand consequence -> save/return | UI affordance, commanded mutation and observed outcome must refer to the same world |
| Runtime modes | standalone, headless/server and attached client projections | Transport/client multiplicity is not multiplayer and does not imply the same simulation instance |
| Extension/content | model/plugin manifest, capability grants, host API version, guest state, assets/rigs/audio | Signing authenticates origin, not safety or correctness; deterministic VM admission is not globally required |
| Scale | spatial resolution, simulation fidelity, active working set, promote/demote transfers | Rendering LOD differs from simulation LOD; conservation/identity/coupling cross boundaries |
| Verification | accepted obligation/quality target -> behavior/model oracle -> evaluation -> candidate/config/world -> artifacts | Scientific, functional, visual, usability and provenance evidence are separate types |
| Development | worker attempts linked to durable effort, independent product state | Killing an agent cannot erase world truth, accepted scope or assessment history |

The mature contract is a graph with multiple projections. Do not force uniform child counts or multiply generic quality concerns into artificial functional rows.

## Stage projections, not core rewrites

| Stage | Usable encapsulated form | Required journey shape | Transition rule |
|---|---|---|---|
| CVP | A bounded single-player world with a real observable intervention and faithful save/return; includes an origin/substrate scenario where accepted scope requires it | C-J01 create/observe/intervene/inspect plus C-J02 save/recover; one coherent world identity | Narrow model/region breadth may be explicit, but persistence and world identity must be mature-compatible |
| MVP | Sustained coherent simulation with intelligible coupled behavior and normal player management | CVP plus C-J03 follow/inspect emergent outcomes and C-J04 invalid input/extension recovery within declared scope | Add model/capability breadth without replacing world/state identities or fabricating scripted social outcomes |
| Beta / v1 | Supported single-player experience, declared content/model/asset quality and compatibility | All v1-selected journeys including onboarding, animation, settings, save migration and diagnostics | Multiplayer/co-op/spectator excluded by current v1 charter; extra clients are not prerequisites by default |
| GA | Qualified supported scope, trustworthy release/artifact identity and demonstrated user outcomes | Complete GA journey and applicable critical quality set | Stable release requires explicit approval; simulation realism claims remain separately qualified |
| Mature | All accepted model/intervention/extension/scale/experience horizons represented and supported | Full accepted journey graph, with validated scope-specific model/quality claims | Staged additions/enrichment and adapter widening; unresolved long-horizon ideas remain proposals |

No stage is certified reached. A narrow vertical slice must not erase origin-to-civilization intent; equally, a speculative million-agent benchmark must not prevent honestly labeling a bounded useful form. Record debt explicitly rather than burying it in an aspirational mature percentage.

## Distinct obligation anchors — candidate register

### REC-CIVIS-STATE-RESTORE — restore actual accepted world state, not just a loadable file

Statement: a supported save restores its declared authoritative world/model/configuration state, required components and pending realized operations consistently; missing or conflicting required components are not silently treated as successful restoration. Parent: persistence/world continuity. Rationale: snapshot semantics survive the explicit removal of global deterministic replay. Sources: charter/ADR; save_bundle.rs; source ledger C-S04/05/11/17. Authority: Accepted-as-recorded design plus source-derived recovery candidate. Role: core spine. Stages: all persistent stages. Journeys: C-J02/04. Dependencies: authoritative-state manifest, save/model versions, migration and failure policy.

Positive: independently prepared non-default state across required components, saved and restored, agrees on contract-defined present-state observations and identities. Negative: missing current-format metadata/sidecar/world state, foreign spec/model/tick, cross-world component swap or rejected migration cannot produce a false restoration pass. Supported legacy fixtures must remain usable under explicit classification/migration. Surfaces: CivSaveBundle save_dir/load_dir/save_archive/load_archive and actual callers; detailed state owners and callers open. Oracle: exhaustive persisted-state mapping, pre/post semantic comparison, deliberate missing/conflicting fixtures and preserved source bytes. Growth: additive/versioned state and migrations; not more manually mirrored fields without ownership proof.

### REC-CIVIS-SAVE-COMMIT — interrupted writes preserve a recoverable committed world

Statement: interruption while saving/replacing a supported slot has an explicit commit boundary so the previous accepted save or complete new save is recoverable; mixed partial data must not masquerade as either. Parent: durable persistence. Sources: faithful snapshot design and source-observed sequential/direct writes. Role: core; stages: CVP onward; journey C-J02/04. Dependencies: storage/rename/transaction behavior on supported platform, component integrity policy.

Positive: normal save creates a fully restorable declared snapshot. Negative: fault after each file/output step and during overwrite exposes no silently accepted mixed save; previous committed source is preserved or an explicit defined recovery path exists. Surfaces: save_bundle entrypoints and slot/database/caller layers not fully inspected. Oracle: write-fault injection on real filesystem plus independent loader and artifact identity. Growth: new storage backends must preserve the same contract; arbitrary serialization success is not sufficient.

### REC-CIVIS-WORLD-IDENTITY — user/machine actions, display and evidence refer to one intended world

Statement: the accepted action and resulting observation/display/save refer to the requested world and runtime mode; stale handles and a different standalone/server instance cannot qualify the effect. Parent: command/observation interfaces. Sources: separate runtime modes in README/history and current evidence mandate. Role: core; stages: all interactive stages; journeys C-J01/02/03/04. Dependencies: stable world/model identity, command correlation, permissions and projection ownership.

Positive: UI or machine action produces a visible and state-observed effect in the selected world and survives a supported save/return. Negative: route to another process/world, rejected mutation, disconnected client or stale snapshot cannot appear as successful control. Surfaces: actual server/watch/CLI/MCP/client dispatch and state ownership still open. Oracle: independent action/state correlation, current raw runtime evidence and matching world/candidate/config IDs. Growth: extra clients/adapters do not create alternate truths or imply multiplayer.

### REC-CIVIS-EMERGENCE-EXPLANATION — claimed emergent patterns are grounded in actual modeled causes

Statement: for every accepted emergent-behavior claim, identify the authored primitives/rules, observed pattern, causal/measurement mechanism and scope limitations; do not satisfy the claim by merely assigning a fixed outcome label. Parent: emergent model semantics. Sources: governing charter; old fixed faction/tech descriptions are contradictions to resolve. Role: differentiation candidate and core product meaning. Stages: applicable to every exposed claimed-emergent behavior, increasing breadth by stage. Journey C-J03; dependencies: domain definitions and accepted pattern-level criteria.

Positive: declared perturbation/ablation changes a meaningful measured behavior consistently with the accepted model claim over a justified evaluation design. Negative: removing the claimed causal subsystem while retaining a `faction`, `sentience` or `technology` label is detected; observational correlation is not mislabeled causal/scientifically predictive evidence. Surfaces: substrate, agents, social/economic phases, UI interpretations and event feeds; mostly not inspected. Oracle: model/rule dossier, domain-specific negative controls, appropriate statistical/causal analysis and independent reviewer. No arbitrary sample size/effect threshold chosen now; those require task-specific justification. Growth: replace/explain a model under versioned semantics without claiming all society follows from a fixed scripted ladder.

### REC-CIVIS-RESOLUTION-TRANSFER — changing simulation resolution preserves accepted cross-scale semantics

Statement: supported region promotion/demotion/streaming preserves declared entity identity, owned events and conserved quantities, with explicit approximation bounds where appropriate. Parent: scale/world substrate. Sources: charter streaming/LOD ambitions; unverified 'disk-bound' claim is not a design fact. Role: mature scale backbone; earliest bounded stages may keep one fidelity while retaining compatible identities. Journeys: C-J01/02/03 at stages exposing LOD. Dependencies: state ownership and accepted model-specific conservation/approximation policy.

Positive: region crosses fidelity boundaries without duplicate/lost entities or unaccounted stocks; effects match a declared reference within accepted bounds. Negative: duplicate ownership, missing pending event or arbitrary mass/energy creation on promotion is caught. Surfaces: voxel/streaming/engine scheduler and projection clients uninspected. Oracle: controlled promote/demote workloads versus reference simulation, state invariant checks and named hardware/configuration performance measurements. Growth: widen map/working set; do not rewrite identity/state or imply visual LOD alone solves simulation cost.

## Quality overlays and unresolved authority

Native responsiveness/frame-time, simulation throughput, memory/storage, save durability, sandbox security, accessibility, legibility, scientific validity, licensing and support are separate overlays on exact subjects/configurations. Existing PRD numbers are not accepted qualified budgets until authority/workload is recovered. Seeded recognizable worldgen is a convenience, not reinstated global bit-identity. Deterministic replay tests may remain for explicitly opted-in subsystems without redefining product scope.

Transition debts to investigate: replay-driven reconstruction mixed with snapshot restore; duplicated WorldState/Simulation/sidecar ownership; fixed social enums vs emergent intent; client-local mutations vs server truth; static labels vs actual model coupling; extra renderer scaffolds; handrolled numerical/asset systems when mature libraries suffice; LOD transfer without conservation evidence.

## Alternatives and freeze policy

WorldBox, Mesa-based model tooling and pinned Bevy ecosystem are realistic goal-specific alternatives. Their combination is not falsely described as a single integrated world. See registry research commit `7c7c724ebcfd0fa353da6b090c36e66ddb474b03`, `docs/sessions/20260929-gaming-pair/Civis/PASS-01-ARCHAEOLOGY-SOTA.md` for qualifications and open bootstrap/academic gates. No custom engine, multiplayer, generalized ABM platform or added backend is approved by this draft. Existence, architecture optimality and mature completeness remain unproven.
