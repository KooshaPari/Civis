# CIV-0106: Settlement cohesion and social fabric (PROPOSED)

Status: PROPOSED. Documents the behavior the engine implements today. Pending owner review. Where this document and the code disagree, the code is current behavior and this document is the question.

## FR-CIV-COHESION-001

Each tick the cohesion phase aggregates kinship, trust and institutions into one snapshot per settlement, with a tiered "fabric" classification.

### Per-actor score

For each actor in a settlement:

    actor_score = edge_count * 10 + trust - max(hardship, 0) + temple_bonus + garrison_bonus

- edge_count: kinship edges the actor holds.
- trust: the actor's summed trust value.
- hardship: subtracted only when positive.
- temple_bonus / garrison_bonus: institution bonuses for that actor (the phase code is in `crates/engine/src/engine/social_settlement_phases.rs`).

### Settlement fabric (proposed semantics: MEAN)

The settlement score is the integer MEAN of its actors' scores (sum divided by actor count, minimum divisor 1). It is not the sum. A settlement does not become "tighter" by having more people; it becomes tighter when its typical member is well bonded.

### Tiers (`FabricTier::from_score`)

| Score | Tier      |
| ----- | --------- |
| >= 80 | Tight     |
| >= 40 | Loosened  |
| >= 10 | Strained  |
| < 10  | Fractured |

### Worked example (matches the traceability test)

Two actors in one settlement. Actor 1 has one kinship edge, trust 40 and a temple bonus of 30: 1\*10 + 40 + 30 = 80. Actor 2 is isolated: 0. Mean = (80 + 0) / 2 = 40, tier Loosened. Under a SUM rule the same settlement would score 80 and be Tight.

### Other snapshot fields

- kin_count: total kinship edges in the settlement.
- trust_sum: sum of trust across actors (this one is a sum, by name).
- faction_count: number of actors counted.
- fragmentations: fragmentation events this tick.

### Open question for the owner

Confirm MEAN. If the intent is SUM, change the aggregation in the snapshot builder and update the test `fr_civ_cohesion_001_phase_aggregates_kinship_trust_into_settlement_fabric` (expected tier Tight).
