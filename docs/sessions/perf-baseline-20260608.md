# Perf Baseline 2026-06-08

## Scope

- Added a minimal BenchmarkDotNet skeleton under `src/Tests/Perf`.
- Benchmarked one bridge-path operation: `BridgeReceiptVerifier.ComputeReceiptHmac`.
- Persisted a single artifact at `src/Tests/Perf/Result.txt`.

## Run Command

```powershell
dotnet run --project src\Tests\Perf\DINOForge.Tests.Perf.csproj -c Release --no-build -- --filter *ComputeReceiptHmac*
```

## Measured Result

- `ComputeReceiptHmac`: `1670.17 ns/op`

## Artifact

- `src/Tests/Perf/Result.txt`

