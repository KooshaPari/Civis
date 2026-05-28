## Summary

<!-- What does this PR do, in 1-3 sentences. Why is this change needed? -->

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring
- [ ] Infrastructure / CI changes

## Changes

<!-- Bullet list of key changes -->
- 

## Testing

All tests must pass before merging. Check all that apply:

- [ ] `dotnet build src/DINOForge.CI.sln -c Release` passes
- [ ] `dotnet test src/DINOForge.CI.sln --verbosity normal` passes (100% green)
- [ ] Code formatting passes: `dotnet format src/DINOForge.CI.sln --verify-no-changes`
- [ ] Pack validation passes (if packs modified): `dotnet run --project src/Tools/PackCompiler -- validate packs/`
- [ ] In-game tested (if Runtime/Bridge changes) — describe scenario
- [ ] CHANGELOG.md updated with user-facing changes
- [ ] New public APIs have XML doc comments
- [ ] No new compiler warnings

## Impact Assessment

If your PR modifies any of these critical areas, describe the impact:

- **Runtime (`/src/Runtime/`)**: Verify netstandard2.0 compatibility and BepInEx plugin load timing
- **SDK public API (`/src/SDK/`)**: Update docs and verify backward compatibility
- **Schemas (`/schemas/`)**: Validate all packs still load; document breaking changes in CHANGELOG
- **CI/Build (`/.github/workflows/`)**: Test on CI system if possible; document any new requirements

## Related Issues

<!-- Link related issues. Use "Closes #123" to auto-close on merge. -->

Closes #

## Deployment Notes

<!-- If this is a breaking change, document migration path for users. -->

## Screenshots / Videos

<!-- For UI changes or visual fixes, attach before/after. -->

---

**Generated with [Claude Code](https://claude.com/claude-code)**
