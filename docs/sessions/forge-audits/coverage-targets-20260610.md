# Coverage target audit: SDK / Bridge untested public classes

Source basis:
- Coverage report: `src/Tests/coverage.cobertura.xml` (all SDK/Bridge classes in this report are at 0% line coverage)
- Test-file scan: `src/Tests/**/*.cs` (no corresponding test file found for the classes below)

Prioritization rule used:
- Since the available coverage report is uniformly 0%, classes were prioritized by larger public surface area / method count, then filtered to public classes with no corresponding test file in `src/Tests`.

| Priority | Class path | Public method count | Dependency style | Test guidance |
|---|---|---:|---|---|
| 1 | `src/SDK/Registry/RegistryEntry.cs` (`DINOForge.SDK.Registry.RegistryEntry<T>`) | 1 constructor + 5 public properties | Pure-logic / data wrapper; no interface deps | Add a focused unit test for constructor behavior (`Priority` calculation, property assignment, default `loadOrder`). |
| 2 | `src/SDK/Models/SkillDefinition.cs` (`DINOForge.SDK.Models.SkillDefinition`) | 1 public method (`Validate`) | Pure-logic validation; no interface deps | Add validation tests for missing `Id`, null/empty `Effects`, and invalid child `SkillEffect` values. |
| 3 | `src/SDK/Models/SquadDefinition.cs` (`DINOForge.SDK.Models.SquadDefinition`) | 1 public method (`Validate`) | Pure-logic validation; no interface deps | Add tests for required fields (`Id`, `DisplayName`) and the `MaxSize < MinSize` constraint. |
| 4 | `src/SDK/Models/UnitDefinition.cs` (`DINOForge.SDK.Models.UnitDefinition`) / nested `UnitStats` | 1 public method on `UnitDefinition` (`Validate`) + 9 public properties on `UnitStats` | Pure-logic validation + data model; no interface deps | Add tests for required fields, `Stats.Hp`, `Stats.Accuracy`, and nested model defaults. |
| 5 | `src/SDK/Models/WaveDefinition.cs` (`DINOForge.SDK.Models.WaveDefinition`) / nested `DifficultyScaling` | 1 public method (`Validate`) + 3 public properties on `DifficultyScaling` | Pure-logic validation + data model; no interface deps | Add tests for `WaveNumber`, `DelaySeconds`, `DifficultyScaling` constraints, and child `SpawnGroup` validation aggregation. |

Notes:
- These classes are the best coverage targets because they are public, currently have no dedicated test file in `src/Tests`, and their behavior is deterministic enough for fast unit tests.
- All five are pure-logic/data-validation types, so they should be covered with standard unit tests only; Moq is not required.
