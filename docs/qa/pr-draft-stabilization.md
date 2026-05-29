## Summary

- **Stabilization (2026-05-23)** — CI green offline: DumpTools pipe drain, PollingHelper virtual clock, DebugLog temp fallback, NDJSON bridge framing (`UseMessageFraming=false`), prove-features validate-only gate, SPEC-002/004/007 tests, SDK/PackCompiler coverage, journey-viewer `@phenotype/journey-viewer@0.1.1`, click-routing H1 (`EnsureEventSystemAlive`), warfare-starwars URP Lit metadata, Pester/pytest contracts.
- **GameLaunch harness** — Fixture resolves Steam install dir → exe, 90s bootstrap, `DINO_GAME_ALREADY_RUNNING` attach mode; live run: 14/28 pass with bridge up (remaining failures tracked for follow-up).
- **Evidence (local, gitignored)** — `live-bridge-journey-capture` (3 steps), asset-swap screenshot; phenotype visual acceptance remains PARTIAL until human review.

## Test plan

- [ ] `dotnet test src/DINOForge.CI.sln -c Release`
- [ ] `pwsh scripts/game/prove-features-gate.ps1 -ValidateOnly`
- [ ] `pwsh scripts/qa/run-unit-pester.ps1`
- [ ] `pytest scripts/video/tests -q`
- [ ] `dotnet test --filter NativeMenu` / `KeyInputSystem`
- [ ] GameLaunch (optional): `$env:DINO_GAME_ALREADY_RUNNING='1'` + game with BepInEx bridge
- [ ] Manual: rendering audit in-game, DesktopCompanion WinUI build (MSVC)

## Notes

- Branch diverged from `main`; expect merge conflicts in bridge/runtime/workflows.
- `docs/qa/evidence/` is gitignored (interim captures).
- `scenario-tutorial` → `packs/scenario-tutorial.disabled/`.
