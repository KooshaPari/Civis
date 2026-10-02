# Civis pass 5 — existing-test oracle audit

Date 2026-09-29. Concurrent source `54d5758970249c8d1f24688ea45920b530e77299`. No native execution by this recovery worker.

## Existing tests confirm artifact integrity but expose semantic gaps

The active save_bundle tests are substantial for byte integrity: component enumeration, per-file BLAKE3/length, path containment, unlisted components, root mismatch, archive corruption and mutation checks. These should be retained and credited to their exact artifact-integrity criteria.

They are not state-completeness tests.

### C-F17 — guest-memory tests explicitly decouple memory from loaded mod identity

Two active tests:
- `civsave_folder_round_trips_mod_guest_state`
- `civsave_archive_round_trips_mod_guest_state`

construct a fresh `Simulation::with_seed`, directly call `mod_host_mut().restore_guest_memory("test-mod", ...)` / `"archive-mod"`, save, load, and assert the same guest bytes exist.

They do **not load a corresponding mod manifest/artifact first**. Thus the test intentionally proves that arbitrary guest-memory namespaces survive independently of the loaded mod registry.

That is valid as a blob round-trip unit test, but it cannot qualify "mod state restored" at product level. It actually supplies a minimal counterexample: a loaded save can contain guest memory for a mod that is not loaded.

Required semantic oracle:
1. install/load exact mod artifact/version;
2. create guest memory through or attributable to that mod;
3. save;
4. load with same mod set → memory and behavior restored;
5. load with mod absent → explicit compatibility/recovery result;
6. load with incompatible/new version → migration or explicit refusal/degraded state;
7. no orphan memory is silently presented as fully restored active mod state.

## C-F18 — economy policy omission has no discovered save test

Repository search found policy behavior/interface tests but no save/load test for `economy_policy`. Active save_bundle write/read code contains no economy-policy component, and replay has no SetPolicy event. This strengthens C-F14.

The correct test is not an internal assignment only: use the mounted `sim.set_policy` server/MCP path where possible, save through the mounted save path, reload, then observe policy and next economy result. If policy is intentionally scenario/profile configuration rather than world state, load must deterministically rebind the accepted scenario/profile and expose that identity. Current silent constructor default cannot be presumed correct.

## C-F19 — research cache is user-visible outcome state but no persistence mapping found

ResearchCache is surfaced in SimulationSnapshot and conditions use `researched.len()` for an Age of Enlightenment victory outcome. Search found behavior/snapshot tests but no save mapping/test. Replay ResearchOutcome is applied by a method that currently discards snapshot_hash/accepted after updating tick.

Thus a save can potentially lose progress that directly affects player-visible tech state/outcome. This is a high-priority native omission fixture:
- set researched + in-progress state through actual research mechanics or controlled test hook;
- assert snapshot/outcome before save;
- save/load;
- assert same snapshot/outcome and next research behavior.

## Test-authority rule

Do not remove existing narrow tests because they fail to prove broader semantics. Reclassify:
- byte-integrity tests → artifact-integrity evidence;
- guest-memory roundtrip → blob serialization evidence;
- selected-field roundtrips → named field evidence;
- whole WorldState equality → only the five fields in its PartialEq unless individually asserted.

A test earns exactly the claim its assertions and mounted path establish.

## Executable experiment patches

Before implementation fixes, add focused tests on a dedicated experimental branch/worktree or as non-product recovery fixtures:
- `save_roundtrip_preserves_runtime_economy_policy_or_declared_rebinding`;
- `save_roundtrip_preserves_research_outcome_state`;
- `save_load_detects_orphan_mod_guest_memory`;
- metadata removal from a freshly written v5 bundle;
- autosave forced tick advance between identity capture and write.

Expected failures are evidence, not regressions to hide. Do not weaken the oracle to make current code green.
