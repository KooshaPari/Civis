# Emergent-Systems Traceability Ledger — feat/sim-emergence-batch

Maps the emergence-batch gameplay systems and their downward-causation couplings
to requirement IDs, implementing code, and verifying tests. Goal: keep this batch
out of the `CODE-ONLY-no-spec` bucket (634 IDs as of `fr-matrix-2026-06-13`) by
asserting spec + code + test for each row → `COVERED`.

Charter: hardcode only physical/environmental/genomic law; life, society, economy,
belief, diplomacy EMERGE from state with bidirectional coupling (downward
causation), never scripted silos. See `project_civis_emergence_charter`,
`project_civis_emergence_design_layer`.

Spec roots: `docs/specs/CIV-0100-economy-v1.md`, `CIV-0107-joule-economy-system-v1.md`,
emergence charter (FR-CIV-0100 §3 emergence).

## Systems (tick-loop phases)

| # | System | FR | Code (crates/engine/src) | Test(s) |
|---|--------|----|--------------------------|---------|
| 1 | Research accrual | FR-CIV-0100 §research | `engine.rs:phase_research` (1582) | `phase_research_accrues_from_population` (3003), `phase_research_quiescent_without_population` (3020) |
| 2 | Belief/faith accrual | FR-CIV-0100 §belief | `engine.rs:phase_belief` (1593) | `phase_belief_accrues_from_population` (3065) |
| 3 | Emergent market pricing | FR-CIV-0100 §3d | `engine.rs:phase_economy` (2135); `economy/src/market.rs:apply_pressure` | `phase_economy_steps_market_prices` (2601), `apply_pressure_*` (market.rs) |
| 4 | Emergent diplomacy | FR-CIV-0100 §3 | `engine.rs:phase_diplomacy` (~2060) | `diplomacy_threshold_*` (2658–2677) |
| 5 | Divine powers / faith spend | FR-CIV-0100 §belief | `engine.rs:try_invoke_divine_power` (1122); `disasters.rs:invoke_divine_disaster` (49) | `try_invoke_divine_power_gates_on_belief` (3082), `invoke_divine_disaster_*` (276, 286) |
| 6 | Disasters (wildfire/quake) | FR-CIV-0100 §disasters | `engine.rs:phase_disasters`; `disasters.rs:trigger_disaster` | disasters.rs `#[cfg(test)] mod tests` |

## Couplings (downward causation — the emergence-defining links)

| Coupling | Direction | FR | Code anchor | Test(s) |
|----------|-----------|----|-------------|---------|
| disasters → belief | fear breeds faith | FR-CIV-0100 §3 emergence | `disasters.rs:trigger_disaster` (+`add_belief(50)`) | `invoke_divine_disaster_*` |
| belief → divine-disaster | faith enables miracles (loop) | FR-CIV-0100 §belief | `disasters.rs:invoke_divine_disaster` (49) | `invoke_divine_disaster_requires_faith` (286) |
| research → economy | tech raises carrying capacity → demand/supply | FR-CIV-0100 §3d | `engine.rs:carrying_capacity` (1105) via `research_tier` | `phase_economy_*`, capacity tests (~2974) |
| faction wealth → market demand | prosperity lifts demand | FR-CIV-0100 §3d | `engine.rs:phase_economy` (2135) | `ws_smoke.rs` snapshot + economy tests |
| economy scarcity → population | costly food damps births | FR-CIV-0100 §3 emergence | `engine.rs:food_scarcity_birth_factor` (2326), `phase_citizen_lifecycle` | `food_scarcity_birth_factor_*` (2617–2643) |
| belief → diplomacy | shared faith breeds peace | FR-CIV-0100 §3 | `engine.rs:diplomacy_conflict_threshold` (2342), `phase_diplomacy` | `diplomacy_threshold_*` (2658–2677) |

## Loop closure (no parallel silos)

belief now both **accrues** (population, disasters) and **acts** (divine power, diplomacy);
research **accrues** (population) and **acts** (carrying capacity → economy → population);
economy **acts back** on population (scarcity → births). These bidirectional links are the
compositionality test from `project_civis_emergence_design_layer` — state feeds forward and
backward through shared resources, not one-way API calls.

## Open traceability gaps (next lanes)

- Add explicit `FR-CIV-0100-§N` IDs to the spec doc so the matrix generator links these
  rows as COVERED rather than CODE-ONLY-no-spec.
- Wire these test names into `docs/audits/_id_inventory_v3.json` on the next matrix refresh.
- Candidate next couplings (refill DAG): unrest ∝ scarcity, trade-volume ∝ price-gap,
  research → disaster-mitigation.
