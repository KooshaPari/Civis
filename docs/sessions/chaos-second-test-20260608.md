# Chaos Second Test - 2026-06-08

## Goal

Add a second Chaos canary under `src/Tests/Chaos` that exercises a different bridge-receipt failure path than the tampered-HMAC case.

## Files Changed

- `src/Tests/Chaos/DINOForge.Tests.Chaos.csproj`
  - Switched the Chaos test project to explicit compile items and added the new replay-path test file.
- `src/Tests/Chaos/BridgeReceiptReplayChaosTests.cs`
  - Added a second `[Trait("Category", "Chaos")]` fact that verifies a valid receipt once, then replays the same receipt to trigger the `world_frame` regression rejection path.

## Behavior Covered

The new Chaos fact exercises a different negative path from the original tampered-receipt test:

- first verification succeeds and advances `lastFrame`
- second verification of the same receipt is rejected
- the failure reason contains `regressed`

## Validation

Ran:

```powershell
dotnet build src/Tests/Chaos/DINOForge.Tests.Chaos.csproj -c Release --no-restore -p:BuildProjectReferences=false -m:1 --disable-build-servers
```

Result:

- `Build succeeded.`
- `0 Warning(s), 0 Error(s)` for the Chaos test project build

Reference project builds used for the scoped compile:

```powershell
dotnet build src/Bridge/Client/DINOForge.Bridge.Client.csproj -c Release -m:1 --disable-build-servers
dotnet build src/SDK/DINOForge.SDK.csproj -c Release -m:1 --disable-build-servers
```

## Notes

- This note is intentionally separate from `docs/sessions/chaos-tests-green-20260608.md` and `docs/sessions/chaos-test-add-20260608.md`.
- The new Chaos fact is replay-based rather than HMAC-tamper-based, so it covers a distinct verifier rejection path.
