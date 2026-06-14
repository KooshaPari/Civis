# Chaos Test Add - 2026-06-08

## Goal

Add the smallest xUnit Chaos canary that proves tampered bridge receipts fail verification, and wire it into the existing Chaos trait filter path.

## Files Changed

- `src/Tests/Chaos/DINOForge.Tests.Chaos.csproj`
  - Added a dedicated net8.0 xUnit Chaos test project with the bridge client dependency set needed for receipt verification.
- `src/Tests/Chaos/BridgeReceiptChaosTests.cs`
  - Added one `[Trait("Category", "Chaos")]` fact that builds a valid receipt and then tampers the HMAC before verification.
- `src/Tests/DINOForge.Tests.csproj`
  - Excluded `Chaos\**` so the parent unit project does not compile the nested Chaos project sources twice.
- `src/DINOForge.sln`
  - Added the Chaos project to the main solution graph.
- `src/DINOForge.CI.sln`
  - Added the Chaos project to the CI solution graph.

## Behavior Covered

The new Chaos fact exercises the Bridge receipt verifier on a tampered receipt and asserts:

- `VerificationResult.Valid` is `false`
- the failure reason contains `hmac mismatch`
- `lastFrame` does not advance on rejection

## Validation

Ran:

```powershell
dotnet test src/Tests/Chaos/DINOForge.Tests.Chaos.csproj --configuration Release --filter "Category=Chaos" --verbosity minimal
```

Result:

- `Passed: 1, Failed: 0, Skipped: 0`

## Notes

- This doc is intentionally separate from `docs/sessions/chaos-tests-green-20260608.md`.
- The filter wiring is trait-based: the fact is tagged with `Category=Chaos`, so the Chaos scaffold can select it with `--filter "Category=Chaos"`.
