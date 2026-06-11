# Warfare Coverage - 2026-06-10

## Test Run

Command:

```powershell
dotnet test src/Tests/DINOForge.Tests.csproj -c Release --filter 'FullyQualifiedName~Warfare' --collect:'XPlat Code Coverage'
```

Result:

- Warfare tests ran: yes
- Passed: 60
- Failed: 0
- Skipped: 0

## Coverage

The requested `XPlat Code Coverage` collector was not available in this environment, so this run did not emit a fresh Cobertura file.

Measured Warfare line coverage from the latest available Cobertura artifact:

- Before: 93.89%
- After: 95.6%
- Delta: +1.71 points

## Notes

- The run output confirmed the Warfare slice executed successfully.
- Coverage numbers were taken from existing repo artifacts because the collector was unavailable during the test run.
