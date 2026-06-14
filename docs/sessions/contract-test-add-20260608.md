# Contract Test Add - 2026-06-08

## Summary

Added one focused consumer-driven contract test under `src/Tests/Contract` for the bridge `connect` handshake response shape.

## What Changed

- Added `src/Tests/Contract/DINOForge.Tests.Contract.csproj`.
- Added `src/Tests/Contract/BridgeRpcHandshakeContractTests.cs`.
- Wired the new project into `src/DINOForge.sln`.
- Excluded the new `Contract` folder from `src/Tests/DINOForge.Tests.csproj` so the parent test project does not glob the new subproject.

## Contract Coverage

- The test asserts the public JSON-RPC envelope shape:
  - top-level `jsonrpc`
  - top-level `id`
  - top-level `result`
- The handshake `result` is pinned to:
  - `session_id`
  - `session_key_b64`
- The test also asserts that `error` and `bridge_receipt` are absent on the success path.

## Validation

- Targeted build succeeded:
  - `dotnet build src/Tests/Contract/DINOForge.Tests.Contract.csproj -p:UseSharedCompilation=false`
- Targeted test run succeeded:
  - `dotnet test src/Tests/Contract/DINOForge.Tests.Contract.csproj --no-build`

## Notes

- This is intentionally narrow and does not duplicate the existing bridge handshake tests in `src/Tests/Bridge`.
- I used a simple JSON shape match rather than introducing a larger Pact dependency.
