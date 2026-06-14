# BDD Bridge Regression Suite - 2026-06-08

## Goal

Add a two-feature Reqnroll regression slice under `src/Tests/BDD` that exercises the bridge handshake happy path and rejects a replayed receipt/frame.

## What Changed

- Updated `src/Tests/BDD/DINOForge.Tests.BDD.csproj` to reference:
  - `src/Bridge/Client/DINOForge.Bridge.Client.csproj`
  - `src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj`
- Added two bridge BDD features:
  - `src/Tests/BDD/Features/BridgeHandshakeHappyPath.feature`
  - `src/Tests/BDD/Features/BridgeReplayAttackRejection.feature`
- Added shared BDD support for a small in-process named-pipe bridge server:
  - `src/Tests/BDD/Support/BridgeRegressionServer.cs`
- Added Reqnroll step wiring for the new bridge scenarios:
  - `src/Tests/BDD/Steps/BridgeRegressionSteps.cs`

## Behavior Covered

- The handshake feature proves the client can complete `connect`, cache the issued session material, and successfully verify a signed follow-up `ping`.
- The replay feature proves a repeated signed `ping` response with the same world frame is rejected as a replay/regression case.

## Validation

Ran:

`dotnet build src/Tests/BDD/DINOForge.Tests.BDD.csproj -c Release`

Result:

- Build succeeded with 0 errors and 0 warnings.
- Reqnroll integrated the new feature files successfully through the BDD project build.

## Notes

- This note only covers the bridge regression slice and does not repeat the earlier BDD skeleton or pack-hash feature writeups.
