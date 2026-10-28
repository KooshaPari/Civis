# Civis architecture decision record — authoritative save state

Date 2026-09-30. Status: PROVISIONAL, experiment-backed.

## Evidence forcing the decision

Recovery run 36628556544 on candidate eb74d072 compiled and executed three semantic persistence tests:
- economy_policy reset to DEFAULT_ECONOMY_POLICY;
- research_cache.researched reset to empty;
- mod guest memory survived while corresponding loaded mod identity remained absent.

Current v5 integrity manifest verifies serialized component bytes, but not whether all accepted durable state was serialized.

## Alternatives

### A — add policy/research/mod fields directly to WorldState

Pros: smallest code change; existing world_state.json/version machinery.
Cons: WorldState is already a mirror/projection of many Simulation owners; six fields are serde-skipped; duplicate ownership grows; mod artifacts and ECS/voxel state do not fit cleanly.

Disposition: REJECT as blanket mature strategy; individual fields may move there only if WorldState becomes their explicit canonical owner.

### B — keep componentized bundle and add an authoritative state manifest

Each durable state domain has a named component/version/owner. Bundle metadata declares scenario/profile identity, mod-set identity, and component schema versions. Save constructs components from canonical owners; load validates compatibility, reconstructs canonical owners, then derives projections.

Pros: fits existing v5 component/integrity design; isolates migrations; supports optional/required components and external bindings; makes completeness auditable.
Cons: requires explicit denominator and migrations; cross-component transaction/consistency still required.

Disposition: PREFERRED.

### C — event-source all simulation state and reconstruct exclusively from replay

Pros: audit/history; conceptual single source.
Cons: current charter explicitly dropped global deterministic replay; replay events intentionally omit climate/mod/RNG and ResearchOutcome does not rebuild research cache; enormous event/schema burden; floating/random simulation need not reproduce identical future.

Disposition: REJECT as authoritative persistence model. Replay remains audit/history and optional subsystem evidence.

### D — serialize the entire Simulation object graph

Pros: superficially exhaustive.
Cons: trait objects, runtime handles, caches, ECS/world internals, mod host/runtime resources, migration brittleness and ephemeral state make this unsafe and semantically opaque.

Disposition: REJECT.

## Preferred bundle model

Outer save identity:
- save_format_version;
- product/model/schema version;
- world_id;
- tick;
- scenario/profile identity + configuration digest;
- required component manifest;
- active mod-set identities/artifact digests/API compatibility;
- component versions/digests;
- migration lineage.

Components should be semantic domains, not arbitrary struct dumps:
- world core;
- environment;
- economy policy + durable market/economic state;
- research/technology;
- institutions/settlements;
- ECS/entity durable subset;
- voxel/material substrate;
- mod set + guest state;
- queued durable operations where applicable;
- replay/audit history.

Each domain explicitly classifies ephemeral caches and reconstruction rules.

## External patterns

TUF's snapshot metadata is useful prior art for binding a consistent set of versioned metadata so clients do not combine files from different repository states; Civis can borrow the consistency principle without adopting TUF as its save format. citeturn0search2

Kubernetes resourceVersion/generation patterns illustrate the value of rejecting updates against stale versions and distinguishing desired/current observed state. This is conceptual prior art for save/world/component identity, not a claim that Kubernetes semantics map directly onto a simulation. citeturn0search1turn0search6

## Migration / compatibility policy to prototype

- Existing v5 save loads through explicit v5 adapter.
- Missing newly durable components are reconstructed only where a documented invariant exists; otherwise load reports degraded/incompatible state.
- Economy policy: either persist per-world mutable value, or bind to scenario/profile and reapply exact configuration. Because mounted APIs mutate it at runtime, the default proposal is persist the effective per-world policy plus source/provenance.
- Research cache: persist as durable technology component because it affects snapshots, progression and victory.
- Mods: persist exact active set + versions/artifact identities/capabilities; restore guest memory only after compatibility resolution. Missing mod produces explicit degraded/refused load according to manifest policy.
- Original save bytes remain intact on failed migration.

## Prototype before broad migration

Create a new experimental component manifest/version that only adds the three reproduced domains:
1. economy policy;
2. research state;
3. active mod-set identity + guest-state compatibility.

Re-run the three red oracles, then add:
- missing component;
- incompatible mod version;
- changed scenario/profile;
- mixed component from another world/tick;
- migration from current v5;
- interrupted write before/after new manifest publication.

Do not migrate the rest of Simulation until this slice proves the state-manifest architecture.
