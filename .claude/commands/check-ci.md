# check-ci

Run the full CI check suite locally to catch issues before pushing.

## Steps

1. Run: `dotnet build src/DINOForge.sln --configuration Release 2>&1`
2. Run: `dotnet test src/DINOForge.sln --configuration Release --no-build 2>&1`
3. Run: `dotnet format src/DINOForge.sln --verify-no-changes 2>&1`
4. Validate all packs: for each dir in `packs/*/`, run pack compiler validation
5. Report pass/fail summary with counts
6. If any failures, explain each one and suggest fixes

This command ensures code quality and pack integrity before committing to main.
