# Civis PR1569 next integration gate — opt-in bundle bridge

Date 2026-09-30. Applies only to experiment/semantic-save-manifest-20260930.

Do not replace default CivSaveBundle save/load yet.

## Opt-in bridge contract

A candidate `save_dir_semantic_vNext` / `load_dir_semantic_vNext` may be added only behind an explicit experiment API/flag.

Save ordering:
1. snapshot/capture accepted canonical domains;
2. write existing v5-compatible bundle components into a staged generation;
3. write semantic-state.json with generation/world/tick binding;
4. write integrity/component manifest covering semantic-state;
5. validate staged generation;
6. atomically publish generation pointer/slot according to platform-specific policy;
7. retain prior accepted generation until commit succeeds.

Load ordering:
1. classify outer format before trusting removable metadata;
2. validate integrity and required semantic component;
3. resolve scenario/profile + exact active mod set/artifacts;
4. validate mod compatibility;
5. import guest memory only after compatible mod host exists;
6. apply semantic state;
7. reconstruct projections/caches;
8. return explicit degraded/refused result for unresolved requirements.

## Required experiments

- current v5 -> vNext adapter migration preserves original v5 bytes;
- missing semantic-state fails closed for vNext;
- future schema fails closed;
- metadata deletion cannot downgrade vNext/current;
- mixed semantic component from another world/generation rejected;
- missing/incompatible mod refuses/degrades before guest-memory import;
- interrupted staged write before semantic component leaves prior generation active;
- interruption after staged validation but before pointer switch leaves prior generation active;
- pointer switch interruption semantics tested on target filesystems;
- policy/research/control-policy/market/tutorial/religion/caravan red controls turn green through the opt-in bridge;
- old default CivSaveBundle path remains available as comparison until migration acceptance.

The125 Simulation /48 WorldState denominator remains open; this bridge is not complete state coverage.
