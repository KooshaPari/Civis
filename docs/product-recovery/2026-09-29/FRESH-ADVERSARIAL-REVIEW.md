# Independent adversarial review — Civis non-execution contract

Date: 2026-10-01.
Method: assume the recovered contract is wrong and N alternatives are valid.

## Challenges

### A1 — Civis is primarily a persistence engine
Falsified by recovered mega-simulation/god-game intent. Persistence is continuity spine.

### A2 — Civis is a research ABM framework
Too narrow. Mesa-class tools are better commodity research frameworks. Civis survives as integrated playable causal world + intervention + emergence; research is a projection.

### A3 — Civis is a scripted civilization game
Conflicts with emergence-first charter. Authored rules/initial conditions are allowed; hardcoded outcome labels cannot alone satisfy emergence claims.

### A4 — Full deterministic replay should define correctness
Explicit May correction/ADR drops global bit-identical replay. Subsystems may opt in; faithful accepted state remains required.

### A5 — Serialize the whole Simulation and be done
Rejected: duplicates/mirrors/caches/external bindings/pending operations need semantic ownership and migration. Blind dumps hide contradictions and make evolution brittle.

### A6 — Recompute everything from seed
Rejected by dropped global determinism and mutable intervention/world state.

### A7 — Save only visible WorldState
Rejected by reproduced losses and P0 Simulation-owned causal state.

### A8 — Every field must persist
Also wrong. Contract supports derived/cache/ephemeral/audit/mirror dispositions with explicit reconstruction/rationale.

### A9 — RNG stream must persist globally
Not automatically. Need only the accepted semantics of each subsystem; random future may diverge unless a subsystem promises replay.

### A10 — Emergence can be graded by detecting a culture/religion/polity label
Rejected by causal/ablation invariants.

### A11 — Deep physics automatically improves realism
Rejected: model validity, gameplay value and performance are separate claims requiring evidence.

### A12 — One fidelity level avoids LOD complexity
Possible alternative architecture, but it must satisfy scale/workload goals. Contract does not force LOD if empirical need disappears; if LOD exists, transfer invariants apply.

### A13 — SQLite should be canonical save truth
Possible, but current contract requires an explicit authority decision rather than assuming filesystem or DB. Either can satisfy it if reconciliation is coherent.

### A14 — Wasmtime guarantees plugin safety
Rejected: sandboxing is substrate; capability/resource/host API/state compatibility remain product policy.

### A15 — Multiplayer should be mature mandatory because old PRD said multi-client
Current v1 explicit exclusion supersedes it for v1. Mature multiplayer horizon remains an authority decision, not silently deleted forever.

### A16 — Save at any instruction boundary
Not necessarily. Contract permits defined supported save boundaries; queues such as pending_damage must be drained/persisted/reconstructed accordingly.

## Orphan check

All major mature dimensions map to P1-P15. The 125/48 state inventory maps to the terminal-disposition ontology; unresolved rows remain blockers, not orphans. Model/emergence, UX, extension, LOD and operations each have journeys/oracles.

## Fresh-review conclusion

The non-execution semantic contract explains the recovered competing interpretations without forcing one implementation where alternatives remain valid. It survives this review.

Remaining falsification is empirical/authority-based:
- causal model produces no useful emergence;
- scale architecture fails;
- state denominator reveals an unmodeled owner class;
- user selects a different scientific/platform/multiplayer authority;
- narrower alternative stack produces equal outcomes at lower burden.

Result: NON-EXEC SEMANTIC REVIEW PASS, empirical/authority blockers retained.
