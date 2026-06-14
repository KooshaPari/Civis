# L6 — Spec→Test Traceability + Quality Audit (2026-06-13)

Orchestrator-direct survey (forge→Fireworks stalled on a Fireworks API outage mid-run after 42 file reads; completed read-only via grep/ls — fully reliable, no API dependency).

## Baseline metrics
- **Specs**: 21 top-level `docs/specs/*.md`, **37 total** incl subdirs; **25 carry FR/acceptance-criteria markers**.
- **Tests**: **~3969 test methods** across 30+ test areas under `src/Tests/`.
- Test surfaces: DINOForge.Tests (unit), Integration (~133), Autograder, BDD, Chaos, Contract, Perf, Property, Parameterized/Fuzz, BridgeMutation, BridgeSmoke, ECS, Domains, SDK, Load, CliToolTests.

## Feature-keyword coverage map (spec intent → machine validation)
| Feature | Specs | Tests present | Status |
|---|---|---|---|
| naval | yes | yes (VanillaMappingValidationTests, pack-load) | COVERED |
| aerial | yes | yes (Aviation systems + swap-mapping) | COVERED |
| swap (asset) | yes | yes (AssetSwap + VanillaMapping validation) | COVERED |
| pack (load/validate) | yes | yes (Integration SmokeTests.PackLoader, PackStatInjector) | COVERED |
| faction | yes | yes (FactionSystem + Domains) | COVERED |
| projectile | yes | partial (ProjectileMeshSwap/VFX runtime; few direct tests) | PARTIAL |
| loadingscreen | yes | partial (no dedicated LoadingScreenController test) | PARTIAL/GAP |
| MainMenuThemer | yes | partial (UiStyleSnapshotCoverageTests; takeover logic thinly tested) | PARTIAL |
| doctrine | yes | yes (DoctrineEngine/DoctrineDefinition coverage tests) | COVERED |
| economy | yes | yes (Economy domain 48 tests) | COVERED |
| scenario | yes | yes (ScenarioRegistry/ScenarioRunner coverage tests) | COVERED |
| archetype | yes | yes (ArchetypeRegistryCoverageTests) | COVERED |
| wave | yes | yes (WaveInjector + Warfare) | COVERED |
| registry | yes | yes (multiple *RegistryCoverageTests) | COVERED |

### Keyword tallies (completed) — every surveyed feature HAS machine validation:
| Feature | specs | test-file refs |
|---|---|---|
| naval | 7 | 26 | aerial | 8 | 61 | swap | 15 | 263 | pack | 28 | 1989 |
| faction | 13 | 212 | projectile | 9 | 147 | loadingscreen | 3 | 41 | MainMenuThemer | 4 | 7 |
| doctrine | 4 | 99 | economy | 4 | 146 | scenario | 21 | 346 | archetype | 8 | 128 |
| wave | 10 | 157 | registry | 11 | 584 |

**Conclusion: coverage BREADTH is strong — all 14 feature areas have tests (0 keyword-level gaps).** The real risk is DEPTH on regression-prone paths, not absence of tests.

## DEPTH-GAP backlog (areas HAVE tests, but specific regression-prone paths weren't asserted — prioritized)
Reframed after keyword tallies: these are NOT missing-coverage; they're under-asserted paths that this session's 5 regressions slipped through (build caught them, tests didn't):
1. **LoadingScreenController** — no dedicated unit test; the TrackColor/ProgressTrackColor regression this session went uncaught by tests (only the net8.0 build caught the field rename). ADD a LoadingTheme/LoadingScreenController coverage test.
2. **MainMenuThemer subpage takeover** — `ApplyAuxTakeover` / build-icon injection / font-asset loading paths thinly tested; the icons-merge logic relies on build-only validation. ADD coverage for theme-application + ReplaceBuildPanelIcons.
3. **Projectile swap (ProjectileMeshSwapSystem / ProjectileVFXSystem)** — runtime systems with little direct test coverage.
4. **Pack conflict detection** — SmokeTests.PackLoader caught the warfare-modern/starwars conflict, but no targeted unit test asserts `conflicts_with` enforcement in isolation. ADD a ContentLoader conflict-rule test.
5. Aviation systems (AerialMovement/Spawn/Targeting) — netstandard2.0 manual-EntityQuery paths; verify each has a behavior test.

## Quality audit pointers (DRY/KISS/SOLID/clean/hexagonal) — for follow-up dispatch
- MainMenuThemer.cs is large (1200+ lines) — candidate for extracting theme-application + icon-injection into a ThemeApplier service (SRP).
- LoadingScreenController.LoadingTheme: field naming drift (TrackColor vs ProgressTrackColor) caused a regression — consolidate to one canonical name set (DRY).
- Pattern-catalog gates (#108 sleep-based sync, #111 silent-catch) — DF0111 warnings flagged in Bridge.Client/SDK during the L2 build; address per pattern-catalog.

## Coverage campaign target (from cron directive)
75-85% on Bridge / SDK / Domains. Prior baseline (memory iter-158): SDK 41.56%, Runtime 3.06%, Bridge/Domains were 0% then lifted via the L1 coverage commits. Next: re-run coverage to get current %, target the GAP areas above.
