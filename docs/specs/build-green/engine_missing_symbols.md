# Engine build-green: missing FR symbols (SDD/TDD autograder manifest)

Each row is a symbol the engine references but that was never implemented.
Implement minimal-but-correct (preserve determinism), with a #[cfg(test)] acceptance
test asserting the FR criterion. The autograder loop = `cargo check -p civ-engine` (0 errors)
+ `cargo test -p civ-engine` (acceptance tests green).

| Symbol | Site | FR | Minimal impl + acceptance test |
|---|---|---|---|
| `civ_needs::should_reproduce` | engine.rs:21 | FR-CIV-LIFE-003 | fn(agent needs/age) -> bool; test: a satisfied adult returns true, a starving/juvenile false |
| `civ_economy::LaborCapacityAllocator` | engine.rs:16 | FR-CIV-LIFE-020 | struct allocating labor by lifecycle weight; test: working-age get more capacity than young/old |
| `DoctrineLibrary` | save.rs:12 | FR-CIV-RELIGION | registry of doctrines (Vec/map); test: insert+lookup roundtrips |
| `awakening_belief_gain` | emergence.rs:36 | FR-CIV-PSYCHE-911 | fn(context)->belief delta; test: stimulus raises belief, none keeps flat |
| `genetics::sentience` | engine.rs:719 | FR-CIV-SPECIES | fn over genome -> sentience score; test: monotonic in the relevant gene |
| `DemographicsSnapshot` | building_layouts.rs:14 | FR-CIV-LIFE | struct {pop, age buckets}; test: counts sum to total |
| `ReplayLog` | save.rs:13 | FR-CIV-REPLAY | append-only event log; test: append preserves order |
| `ModGuestStateSave` | save_bundle.rs:12 | FR-MOD | serializable mod-guest state; test: serde roundtrip |
| `MilitaryUnit` | save.rs:65 | FR-CIV-TACTICS | unit struct {hp, pos, morale}; test: construct + serde roundtrip |
| `impact` (disasters.rs:287) | disasters.rs | FR-CIV-DISASTER | local var/binding missing — compute impact from severity; test: higher severity = higher impact |
| `add_cohesion/add_trust/faction_count/...` | lib.rs:94 | FR-CIV-GOV-030 | re-exports of cohesion fns; implement or fix the pub-use to point at real fns |
| `last_tick_unrest*` | lib.rs:103 | FR-CIV-UNREST-001 | accessor fns for unrest snapshot; test: returns last computed value |
| `grid_to_norm` | lib.rs:57 | FR-WORLDGEN | grid->normalized coord fn; test: corner maps to 0/1 bounds |
