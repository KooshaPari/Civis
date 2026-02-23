# Spec Completion TODOs (Agent-Added)

Goal: complete CivLab spec set to implementation-ready quality.

1. Define canonical artifact tree (`PRD`, `ADR`, `HLD`, `LLD`, `Math`, `Policy DSL`, `Scenario Catalog`, `Contracts`).
2. Freeze a single glossary/symbol table for all specs.
3. Consolidate theorem suite into one dependency-ordered document.
4. Write `Minimal Constraint Set Theorem` + assumptions + counterexamples + falsification tests.
5. Finalize `Policy DSL v1` grammar/schema/validation/migrations.
6. Finalize `World Seed Package + Handoff Contract` with deterministic replay checksum rules.
7. Finalize scheduler read/write contracts and cadence table.
8. Complete climate package: damage math, scenario families, UI telemetry contract, adaptation control levers.
9. Complete `Economy Spec v1` with conservation invariants and test vectors.
10. Complete `War/Diplomacy/Shadow Spec v1` with state transitions and enforcement/leakage coupling.
11. Complete `Social-Ideology-Health-Insurgency Spec v1` with intervention surfaces.
12. Define unified objective/Pareto evaluation protocol across scenarios.
13. Define verification plan (property tests, invariants, stress tests, calibration checks).
14. Define staged implementation roadmap (MVP -> Alpha -> Research -> Game Layer) with exit criteria.
15. Define governance change-control process for spec/version/theorem updates.

## Closure Mapping (2026-02-21)
1. Artifact tree and glossary: closed by `../civ/docs/ARTIFACT_TREE.md` and `../civ/docs/GLOSSARY_SYMBOL_TABLE.md`.
2. Theorem consolidation/minimal theorem: closed by `../civ/docs/THEOREM_CHAIN.md` and `../civ/docs/specs/CIV-0104-minimal-constraint-set-theorem.md`.
3. Policy DSL, seed handoff, scheduler: closed at planning/spec level by model specs and Track C roadmap.
4. Climate package: closed by `../civ/docs/specs/CIV-0102-climate-followup-v1.md`.
5. Economy package: closed by `../civ/docs/specs/CIV-0100-economy-v1.md`.
6. War/diplomacy/shadow closed by `../civ/docs/specs/CIV-0105-war-diplomacy-shadow-v1.md`; social/ideology/health/insurgency closed by `../civ/docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md`.
7. Unified objective, verification, implementation roadmap, governance process: closed by `../civ/docs/governance/track-c-civ-sim-closure.md`.
