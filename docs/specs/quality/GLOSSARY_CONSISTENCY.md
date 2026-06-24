# GLOSSARY CONSISTENCY AUDIT

Scope: `docs/THEOREM_CHAIN.md`, `docs/TECHNICAL_SPEC.md`, `docs/GLOSSARY_SYMBOL_TABLE.md`

Method: cross-check every named term and symbol appearing in the theorem chain and technical spec against the glossary symbol table, then flag:
- undefined terms,
- duplicate definitions,
- term/implementation drift where the code or spec usage does not match the glossary definition.

## Summary

- The glossary is much narrower than the technical spec.
- `tick` is the only clearly shared term with matching meaning.
- No duplicated glossary entries were found in the supplied glossary file.
- No explicit code-vs-glossary drift was found for the glossary terms that actually appear in the technical spec.
- Most technical-spec identifiers are undefined in the glossary and should either be added to the glossary or removed from glossary expectations.

## Defined And Consistent

| Term | Glossary definition | Usage in source docs | Status |
|---|---|---|---|
| `tick` | atomic time-step for state transition | The spec uses `tick` as the simulation update unit and a 60 Hz cadence (`docs/TECHNICAL_SPEC.md:39`, `docs/TECHNICAL_SPEC.md:66`, `docs/TECHNICAL_SPEC.md:284`, `docs/TECHNICAL_SPEC.md:326`, `docs/TECHNICAL_SPEC.md:332`) | Consistent |
| `S_t` | simulation state at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `u_t` | control vector emitted by policy evaluator at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `M_t` | metrics snapshot at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `E_t` | net energy stock/availability at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `L_t` | legitimacy metric at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `C_t` | capture metric at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `D_t` | climate damage estimate at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |
| `R_t` | resilience/adaptation capacity at tick `t` | Appears only in glossary; no conflicting usage found in the other two docs | Consistent |

## Undefined In Glossary

### Technical spec terms

These terms appear in `docs/TECHNICAL_SPEC.md` but are not defined in `docs/GLOSSARY_SYMBOL_TABLE.md`:

- `Fixed` (`docs/TECHNICAL_SPEC.md:55`, `docs/TECHNICAL_SPEC.md:96`, `docs/TECHNICAL_SPEC.md:101`, `docs/TECHNICAL_SPEC.md:107`, `docs/TECHNICAL_SPEC.md:334`, `docs/TECHNICAL_SPEC.md:337`)
- `Simulation` (`docs/TECHNICAL_SPEC.md:326`)
- `WorldState` (`docs/TECHNICAL_SPEC.md:332`)
- `world` / `World` as ECS/API types (`docs/TECHNICAL_SPEC.md:326`)
- `Citizen`, `Position`, `Building`, `Resources`, `Production`, `MilitaryUnit`, `Faction` component names (`docs/TECHNICAL_SPEC.md:147` to `docs/TECHNICAL_SPEC.md:155`)
- `JobType`, `FactionId`, `TechId`, `BuildingType` used in query/API examples and goal definitions (`docs/TECHNICAL_SPEC.md:173`, `docs/TECHNICAL_SPEC.md:245`, `docs/TECHNICAL_SPEC.md:254`, `docs/TECHNICAL_SPEC.md:263`, `docs/TECHNICAL_SPEC.md:270`, `docs/TECHNICAL_SPEC.md:271`, `docs/TECHNICAL_SPEC.md:319`)
- `WorldState.population`, `WorldState.energy_budget_joules`, `WorldState.rng_seed`, `WorldState.factions`, `WorldState.faction_treasury` field names (`docs/TECHNICAL_SPEC.md:332` to `docs/TECHNICAL_SPEC.md:337`)
- `tick rate`, `snapshot`, `restore`, `serialization`, `network sync`, `delta sync`, `query`, `component flags`, `goal planner`, `behavior trees`, `pathfinding`, and similar subsystem nouns used throughout the spec

### Theorem-chain terms

The theorem chain introduces theorem titles that are not defined in the glossary:

- `Sanctions leakage threshold theorem` (`docs/THEOREM_CHAIN.md:4`)
- `Authoritarian enforcement backfire theorem` (`docs/THEOREM_CHAIN.md:5`)
- `Coalition sanctions stability theorem` (`docs/THEOREM_CHAIN.md:6`)
- `Constitutional constraint necessity theorem` (`docs/THEOREM_CHAIN.md:7`)
- `Minimal constraint set theorem` (`docs/THEOREM_CHAIN.md:8`)

## Duplicates

No duplicate glossary symbol definitions were found in `docs/GLOSSARY_SYMBOL_TABLE.md`.

## Drift Checks

### No drift detected

- `tick` is defined as an atomic time-step in the glossary and used as the simulation time-step in the technical spec.
- `S_t`, `u_t`, `M_t`, `E_t`, `L_t`, `C_t`, `D_t`, and `R_t` appear only once in the glossary and do not conflict with the other two docs.

### Potential future drift to watch

- `Fixed` has a concrete code meaning in the spec (`i64` scaled by `10^6`), but there is no glossary entry yet. If a glossary entry is added later, it should explicitly match the current spec wording.
- The theorem names are policy/research labels only; if they later become formal symbols, they should be given glossary entries instead of remaining free-form prose.

## Recommendation

1. Add glossary entries for the core simulation/code nouns that are already part of the spec vocabulary, especially `Fixed`, `Simulation`, `WorldState`, `Citizen`, `Faction`, `BuildingType`, and `JobType`.
2. Decide whether theorem titles are meant to remain prose labels or become first-class glossary symbols.
3. Keep `tick` as-is unless the simulation timing model changes away from a discrete atomic step.
