# status

Show DINOForge project status: build health, test counts, pack validation, milestone progress.

## Steps

1. Run `git log --oneline -5` to show recent commits
2. Run `dotnet build src/DINOForge.sln --configuration Release --no-restore 2>&1 | grep -E "succeeded|failed|Error"`
3. Run `dotnet test src/DINOForge.sln --no-build --configuration Release 2>&1 | grep -E "Passed!|Failed!"`
4. Validate packs and count successes/failures
5. Show current VERSION file contents
6. Check `docs/ROADMAP.md` for current milestone (look for v0.6.0 section)
7. Report a clean summary table

This command provides a quick health check of the entire project including source build, tests, packs, and roadmap progress.
