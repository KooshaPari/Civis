# Civis pass 4 — active save component map and omission controls

Date 2026-09-29. Concurrent implementation inspected: `54d5758970249c8d1f24688ea45920b530e77299`. Original recovery snapshot remains separately frozen. Source evidence only.

## Active bundle components

The mounted `CivSaveBundle::save_dir` writes:
- metadata.json — format/spec/tick label;
- mod_state.json — exported **guest scratch memory**;
- world_state.json — clone of WorldState after `save_state_mirror_to`;
- environment.json — planet, moon, climate, weather grid, coastal columns;
- cluster_stocks.json;
- institutions.json — settlements, institutions, emitted-level set;
- replay.civreplay;
- integrity.json — hashes all serialized files except metadata/integrity.

Load begins with `Simulation::load_replay_from_file`, then restores guest memory, replaces WorldState, mirrors a named subset back to Simulation, and restores environment/stocks/institutions.

## Replay is not a general state reconstruction mechanism

`ReplayLog::replay` actively applies BuildingSpawn, VoxelWrite, Damage, Diplomacy, Combat, ResearchOutcome and Tick. It treats Climate, ModLoaded, ModUnloaded, SessionSaved, ModPermissionViolation, EmergenceMetrics and RngDraw as no-op/informational during reconstruction.

Therefore a recorded `ModLoaded` event does **not** repopulate ModHost. Recorded RNG draws do **not** advance/restore RNG state. Climate replay events do not restore climate. Climate is instead handled by environment.json; mod host identity requires another mechanism if it is intended durable.

This matters because `load_replay_from_file` starts with `Simulation::with_seed(log.seed)`. State not reconstructed by active replay and not overwritten by a bundle component retains constructor/default state.

## C-F13 — guest memory is persisted without an observed durable loaded-mod-set restore

save_dir serializes `sim.export_mod_guest_state()` to mod_state.json. load_dir calls `sim.restore_mod_guest_state(&save)`.

But replay ModLoaded/ModUnloaded are no-ops and the inspected bundle has no separate mod-manifest/lock/set component. A new Simulation begins with `ModHost::new()`. Guest memory alone cannot establish that the corresponding mod version/code/capabilities are loaded.

This creates a first-class save/mod compatibility obligation:
- persist the exact active mod set and artifact/manifest identities, or explicitly require the scenario/profile to reconstruct it before guest-state import;
- reject/diagnose guest memory whose mod identity/version is absent or incompatible;
- define migrations;
- bind evidence to the same mod artifacts.

Native fixture: load a mod, mutate guest memory, save, construct a process/profile where the mod is absent or a different version, load, and inspect mod browser + guest state + behavior. A load must not claim faithful restoration merely because guest-state JSON parsed.

## C-F14 — runtime economy policy has no observed active persistence path

`Simulation::economy_policy` is mutable through server `sim.set_policy` and god-tool difficulty controls and is consumed each economy phase. It is not a WorldState field, not present in the inspected save sidecars, and no ReplayEvent records SetPolicy. `load_replay_from_file` constructs DEFAULT_ECONOMY_POLICY.

Unless another uninspected mounted layer reapplies policy after load, a save after runtime policy mutation will restore default policy, changing future behavior.

This is now a concrete source-derived omission candidate, not merely an inventory priority.

Native fixture: set non-default scarcity/base consumption through the real interface; save; load; inspect policy and next economy step. Decide product semantics: durable per-world policy vs profile/scenario configuration intentionally reapplied. Either is valid if explicit; silent default is not.

## C-F15 — market/research/other Simulation-owned future state require the same proof

`market_state` is observable in SimulationSnapshot and mutates each economy phase. `research_cache` is also surfaced in snapshots. Neither is directly in WorldState. The inspected bundle components do not name them. Replay ResearchOutcome currently calls `apply_replay_research`, whose implementation only consumes arguments and does not reconstruct research state.

Therefore research-cache completeness is a strong omission candidate. MarketState may be derivable from tick under the current scripted step, but design documents are actively moving markets toward endogenous trade-derived state; treating it as safely reconstructible would create transition debt. Its persistence disposition must be explicit and versioned.

Do not declare every Simulation field durable. Per-tick event buffers can be ephemeral by contract. The key test is whether loss changes the accepted present observation, future behavior, or user-visible continuity.

## C-F16 — metadata/integrity authority still has a downgrade boundary

The manifest intentionally excludes metadata.json and compares metadata field-wise elsewhere, but metadata's format_version decides whether manifest verification executes. A v5 bundle with metadata removed is interpreted as v1 before the v5 manifest is consulted.

Required classifier: distinguish genuine legacy-without-metadata from damaged modern bundle. Candidate solutions include a non-downgradable outer container marker, archive format marker, legacy directory classifier, or always inspecting an existing integrity manifest before trusting metadata absence. Do not reject all metadata-less saves without first defining supported legacy.

## State-manifest priority rows

| State | Runtime owner | Active save evidence | Replay reconstruction | Current classification |
|---|---|---|---|---|
| WorldState serializable fields | WorldState + mirrors | world_state.json | partial replay then overwrite | DURABLE, mapping incomplete |
| six serde-skip WorldState extensions | WorldState | excluded from world_state JSON | no general reconstruction found | UNKNOWN / likely omission unless intentionally ephemeral |
| environment | Simulation | environment.json | Climate event no-op | DURABLE |
| cluster stocks | Simulation | cluster_stocks.json | none observed | DURABLE |
| institutions/settlements/emitted levels | Simulation | institutions.json + mirrored WorldState overlap | none observed | DURABLE with duplicate authority |
| mod guest memory | ModHost | mod_state.json | none | DURABLE payload, identity dependency unresolved |
| loaded mod set/artifacts | ModHost | no component observed | ModLoaded/Unloaded no-op | BLOCKING UNKNOWN/OMISSION candidate |
| economy_policy | Simulation | no component observed | no event | BLOCKING semantics decision |
| market_state | Simulation | no component observed | no event | UNKNOWN/derivation debt |
| research_cache | Simulation | no component observed | ResearchOutcome does not restore it | BLOCKING omission candidate |
| RNG internal state | Simulation | seed in replay | RngDraw no-op | semantics changed by no-global-determinism; classify based on future continuity, not bit replay |
| per-tick buffers | Simulation | mixed/no direct components | mixed | likely EPHEMERAL, each must justify |
| replay log/hash | Simulation | replay.civreplay | itself | DURABLE audit/history |
| metadata DB outside bundle | server/watch | separate SaveDb | n/a | INDEX/PROJECTION authority unresolved |

## Next

Continue field extraction and mutation-site classification; prioritize fields exposed in snapshot or consumed by future phases. Then native omission tests. No requirement-count expansion.
