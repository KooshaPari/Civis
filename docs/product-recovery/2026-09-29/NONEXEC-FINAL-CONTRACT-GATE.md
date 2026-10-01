# Final non-execution contract and gate matrix — Civis

Date: 2026-10-01. Program gaming-pair-20260929.
Scope: all specification/documentation/test-design/research/trace layers that can be responsibly closed without completing production implementation or mounted execution.

## Mature product contract

Civis is an emergence-first civilization/god-game mega-simulation integrating a playable world with inspectable causal mechanisms, intervention, warfare/economy/society, deep substrate where useful, and extension/research interfaces. Persistence is spine infrastructure, not product identity.

## Product pillars

C-P1 World substrate: space/material/environment/energy/resources and world identity.
C-P2 Life/agents: organisms/people, needs, lifecycle, genetics/phenotype, cognition/psyche/memory.
C-P3 Social emergence: contacts/kinship/trust/groups/culture/language/religion/legends.
C-P4 Economy/logistics: stocks, production, markets, trade, scarcity, households/settlements.
C-P5 Institutions/politics/diplomacy: organization, power, policy, relations, governance.
C-P6 Warfare/security: forces/doctrine/conflict/damage and world consequences.
C-P7 Technology/progression: research/era/capability evolution from arbitrary starts toward advanced civilization.
C-P8 Construction/urbanism: buildings, settlement/city evolution, infrastructure.
C-P9 Player/god interaction: observe/inspect/intervene/build/control and understand consequences.
C-P10 Scale/LOD: spatial/simulation fidelity transitions preserving accepted identity/conservation.
C-P11 Presentation/UX: rendering, animation, audio, legibility, onboarding/settings.
C-P12 Persistence/continuity: authoritative causal state, migration, recovery.
C-P13 Extension/model platform: mods/plugins/models/assets/capabilities with sandbox/version/state semantics.
C-P14 Research/observability: model documentation, experiments, metrics, causal/emergence explanation.
C-P15 Operations/evidence: exact world/model/config/candidate/run identity, releases/support.

## State ontology

Every state owner gets one terminal disposition:
DURABLE_CANONICAL; DURABLE_EXTERNAL_BINDING; DERIVED_DETERMINISTIC; CACHE_REBUILDABLE; EPHEMERAL_ACCEPTED; AUDIT_ONLY; MIRROR; UNSUPPORTED/REMOVED.

Non-terminal labels are not coverage.

Rules:
- future-state drivers default to unresolved until persist/rebuild/boundary proof;
- MIRROR names canonical owner and explicit restore/rebuild;
- DERIVED names exact inputs/function/version;
- external binding stores exact artifact/config identity and resolves before first dependent phase;
- pending operations define supported save boundary;
- causal emergence substrate cannot be omitted merely because a derived label is saved.

Known P0 findings include current_tick mirror restore gap, next_civilian_id allocator decision, pending_damage boundary semantics, partially-derived economy_state, and mutable settlement food/housing/crime inputs.

## Journeys

C-J01 Genesis/play: choose/create accepted starting world -> simulate -> inspect meaningful state -> intervene -> observe causal consequence.
C-J02 Continuity: nontrivial world -> save -> destroy process -> load -> same accepted causal state -> continue and preserve consequence trajectory semantics (not necessarily identical random future).
C-J03 Emergence explanation: lower-level conditions/processes -> observed higher-level phenomenon -> inspect contributing causes; removing causal mechanism must alter oracle outcome.
C-J04 Invalid/corrupt recovery: missing/mixed/future/incompatible component -> refuse/degrade/migrate explicitly -> never false green.
C-J05 Mod/model extension: resolve exact plugin/model artifacts/capabilities -> instantiate -> restore compatible guest state -> reject incompatible ownership before import.
C-J06 Scale transition: promote/demote region/agent fidelity -> conserve accepted identity/stocks/relationships and avoid duplicate/lost events.
C-J07 Economy/settlement continuity: stocks/market/trade/policy survive restart and next tick behaves consistently.
C-J08 Social continuity: kinship/trust/culture/language/religion/institutions survive/reconstruct and influence subsequent behavior.
C-J09 Warfare continuity: conflict/forces/pending realized operations save at defined boundary and resume without duplication/loss.
C-J10 Research/progression continuity: research/era/technology state and queues survive and continue.
C-J11 Local UX: user can create/play/intervene/save/load through mounted native client.
C-J12 Server/attached UX: client action targets same authoritative server world; save/load is server capability, not disabled because local SimState is absent.
C-J13 Experiment: exact model/config/world -> run -> metrics/artifacts -> reproducible description/evaluation, without claiming global bit-identical replay.
C-J14 Migration: accepted old save -> non-destructive migration -> new generation validates -> original remains recoverable until publication.

Stages:
CVP closes J01/J02/J04 on a bounded single-player world.
MVP adds sustained coupled behavior, J03 and normal management.
Beta/v1 closes selected P1-P15 breadth under explicit single-player scope, migration/onboarding/animation/settings.
GA requires external usability and declared support.
Mature spans accepted origin-to-advanced-civilization horizon and scale/extension/research projections.

## Model/emergence invariants

C-I01 Authored primitive/rule is distinct from measured emergent phenomenon.
C-I02 Label/event existence alone cannot prove emergence.
C-I03 Ablating a claimed causal mechanism must affect the corresponding emergence oracle where the thesis says it is causal.
C-I04 Scientific validity, gameplay value, functional correctness and performance are separate evidence dimensions.
C-I05 Model assumptions/units/schedules/initialization/input are documented in an ODD-compatible model projection.
C-I06 Random future identity is not required globally; faithful accepted state is.
C-I07 Simulation LOD differs from render LOD.
C-I08 Cross-LOD transfer has conservation/identity/event semantics.
C-I09 Intervention is bound to exact authoritative world and produces traceable state transition.
C-I10 External model/plugin state cannot be restored without exact compatible artifact/capability identity.

## Persistence invariants

C-SI01 outer format classification cannot be downgraded by deleting removable metadata.
C-SI02 required component absence is not green.
C-SI03 component world/generation/model identity must agree.
C-SI04 staged candidate does not replace CURRENT until validated publication.
C-SI05 publication intent is not publication evidence.
C-SI06 prior accepted generation remains recoverable across precommit failure.
C-SI07 filesystem vs SQLite index authority/reconciliation is explicit.
C-SI08 guest-memory owner IDs are checked against resolved active mods before import.
C-SI09 mirrors such as current_tick resynchronize before dependent post-load phase.
C-SI10 migration never silently invents missing causal state; default/reconstruct/degrade requires documented invariant.

## Oracle catalogue

Every durable causal domain:
non-default setup -> consequence baseline -> save -> process destruction -> load -> state check -> at least one relevant post-load action/tick -> consequence check -> missing/corrupt negative.

Emergence:
positive causal scenario + ablation/counterfactual + confound controls + measured outcome. No hardcoded label oracle.

LOD:
round-trip promote/demote, conservation, identity, pending-event, boundary-crossing, repeated oscillation.

Extensions:
exact artifact/version/API/capabilities, missing/wrong artifact, guest-state compatibility, resource/sandbox failure.

UX:
mounted route/action/state/render evidence bound to same world/run.

Performance:
named workload/hardware/config; no extrapolation from unrelated benchmark.

## Architecture decisions frozen at non-exec layer

1. Snapshot/semantic persistence, not global deterministic replay, is the main continuity contract.
2. Componentized semantic ownership + explicit outer generation identity.
3. Wasmtime/component substrate is preferred over inventing a plugin VM where applicable.
4. ODD-compatible model documentation is a research projection, not the product tree.
5. World/model/plugin identity is first-class.
6. Emergence requires causal/ablation oracles.
7. Multi-LOD boundaries require explicit transfer semantics.
8. vNext persistence remains opt-in until migration/fault/mounted gates close.
9. filesystem publication capability is target-specific; generic rename is not universal crash proof.
10. research and game claims remain independently qualified.

## Trace skeleton

USER INTENT mega-sim/emergence/god-game
-> pillars P1-P15
-> state/model/experience/extension obligations
-> design/ADRs
-> implementation owners
-> oracles/tests
-> exact candidate/world/model/config evidence
-> mounted runtime/user outcome.

Authority is carried on every edge. Generated FR catalogs are quarantined unless independently recovered as accepted obligations.

## Remaining non-execution authority decisions

- final authority of some historical FUNCTIONAL_REQUIREMENTS/per-ID design catalogs;
- exact scientific/model-validation claims Civis intends to make publicly;
- final supported platform/client matrix beyond current v1 exclusions;
- final multiplayer horizon after v1;
- stable support/compatibility promises.

## Execution/implementation blockers preventing overall100%

- complete terminal disposition of all125 Simulation/48 WorldState rows requires some implementation-owner experiments;
- filesystem durability on target Linux/Windows and FS-vs-SQLite reconciliation;
- mounted save/process-restart/load journeys;
- complete causal/emergence ablations;
- LOD boundary experiments;
- playable origin-to-society vertical;
- performance on named scale workloads;
- external user/research pilot.

Therefore: non-execution contract/oracle/trace structure is closed subject to named authority decisions, but overall original completion gate remains below100 because required experimental/mounted evidence is intentionally not fabricated.
