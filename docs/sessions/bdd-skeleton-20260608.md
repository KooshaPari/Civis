# BDD Skeleton - 2026-06-08

## Goal

Add the smallest checked-in BDD slice under `src/Tests/BDD` without duplicating the existing chaos-scaffold verification note.

## What Changed

- Added a checked-in BDD test project at `src/Tests/BDD/DINOForge.Tests.BDD.csproj`.
- Added one feature file:
  - `src/Tests/BDD/Features/TotalConversionManifestValidation.feature`
- Added one Reqnroll/xUnit step definition:
  - `src/Tests/BDD/Steps/TotalConversionManifestValidationSteps.cs`
- Updated `src/Tests/DINOForge.Tests.csproj` to exclude the new `BDD\**` subtree so the parent test project does not compile the nested BDD sources.

## Behavior Covered

The skeleton exercises the existing SDK validation rule on `TotalConversionManifest.Validate()` by asserting that a manifest with `Type = "content"` fails validation and reports a `type` error.

## Validation

Ran:

`dotnet build src/Tests/BDD/DINOForge.Tests.BDD.csproj -c Release`

Result:

- The BDD project restored and built successfully.
- Reqnroll generated the expected feature/assembly artifacts under `src/Tests/BDD/obj/Release/net8.0`.

## Notes

- This is intentionally minimal: one feature, one step file, one behavior.
- The doc avoids repeating the separate chaos-scaffold verification writeup.
