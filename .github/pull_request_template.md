## Summary

<!-- Briefly describe what this PR does and why. Keep it concise (1-2 sentences). -->

## Type

- [ ] Feature
- [ ] Bug Fix
- [ ] Documentation
- [ ] Refactoring
- [ ] Performance
- [ ] Security
- [ ] Infrastructure / CI-CD
- [ ] Other

## Related Issues

<!-- Link to GitHub issues if applicable. Format: Closes #123, Resolves #456 -->
- Closes #
- Related: #

## Changes

<!-- List the main changes made in this PR -->
- 
- 
- 

## Testing

<!-- How were these changes tested? Include steps to reproduce if applicable. -->

### Local Testing
- [ ] `dotnet build src/DINOForge.sln` succeeds
- [ ] `dotnet test src/DINOForge.sln` passes
- [ ] `dotnet format src/DINOForge.sln --verify-no-changes` passes (code style)

### Additional Testing
- [ ] Pack validation passes (if packs changed)
- [ ] Schema validation passes (if schemas changed)
- [ ] Asset pipeline tests pass (if assets changed)
- [ ] No new warnings or errors introduced
- [ ] Game automation tests pass (if game integration changed)

## Checklist

### Code Quality
- [ ] Tests added/updated for new functionality
- [ ] Test coverage maintained at 95%+
- [ ] Follows C# 12+ style guide and nullable reference types
- [ ] No `var` used for non-obvious types
- [ ] XML doc comments added for public APIs

### Content & Schemas
- [ ] Schemas validated (if modified)
- [ ] No hardcoded content IDs in engine glue
- [ ] Follows registry pattern for extensible domains
- [ ] Pack manifests validate against schemas

### Documentation
- [ ] CHANGELOG.md updated
- [ ] README.md updated (if public API/features changed)
- [ ] Inline documentation updated
- [ ] New public APIs documented with examples

### Compliance
- [ ] No secrets or credentials committed
- [ ] Dependencies reviewed (NuGet packages)
- [ ] Breaking changes documented (if applicable)
- [ ] Migration guide provided (if breaking)

---

**Notes for reviewers:**
<!-- Any additional context for code reviewers -->

---

**Sign-off:**
Co-Authored-By: [Your Name] <[Your Email]>
