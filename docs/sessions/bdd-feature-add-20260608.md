# BDD Feature Add - 2026-06-08

## Goal

Add a second checked-in BDD feature under `src/Tests/BDD` that exercises a different SDK behavior from the initial manifest-type rule.

## What Changed

- Added a second feature file:
  - `src/Tests/BDD/Features/PackHashDeterminism.feature`
- Added a matching step definition:
  - `src/Tests/BDD/Steps/PackHashDeterminismSteps.cs`

## Behavior Covered

The new scenario pins `PackSigner.ComputePackHash()` behavior by asserting that signing artifacts do not affect the computed pack hash. The hash is computed from pack content only and should remain stable when `pack.signature` and `pack.publickey` are added.

## Validation

Ran:

`dotnet build src/Tests/BDD/DINOForge.Tests.BDD.csproj -c Release`

## Notes

- This adds a distinct behavior surface from the existing total-conversion type validation.
- The session note stays focused on the new feature only and does not repeat the earlier BDD skeleton writeup.
