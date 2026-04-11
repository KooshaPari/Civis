# Polyglot Integration Surface

## Research Summary

Investigation conducted: 2026-04-11

### Project Status

**PhenoCompose**
- **Status**: Not found locally or referenced in codebase
- **Local Path Checked**: `C:\Users\koosh\phenocompose` — NOT FOUND
- **GitHub References**: No mentions in DINOForge git history or docs
- **Codebase References**: No `git grep` matches for "phenocompose" or "PhenoCompose"

**NVMS**
- **Status**: Not found locally or referenced in codebase
- **Local Path Checked**: `C:\Users\koosh\nvms` — NOT FOUND
- **GitHub References**: No mentions in DINOForge git history or docs
- **Codebase References**: No `git grep` matches for "nvms" or "NVMS"

### Context Found

The MEMORY.md file contains a single reference to **"PhenoDocs pattern"** (line 35):
- **Meaning**: VitePress + Mermaid documentation setup (dark theme, GitHub Pages deploy via Actions)
- **Not a tool/library**: This is a documentation *convention* adopted by the DINOForge project
- **Related to**: The current VitePress site in `docs/` directory
- **No relation to**: A separate tool called "PhenoCompose" or "NVMS"

### Search Scope

The following searches were performed:
1. ✓ Local filesystem scans (C:\Users\koosh\)
2. ✓ Git grep across DINOForge repository
3. ✓ Recursive grep in docs/ directory
4. ✓ MEMORY.md and MASTER_SYNTHESIS.md review
5. ✓ CLAUDE.md governance review

**Result**: Zero matches for PhenoCompose or NVMS in any searchable context.

---

## Hypothesis

Given that:
- Neither project exists locally
- Neither is referenced in the codebase
- No integration patterns are documented
- The user's memory files make no mention of them

**Possible explanations:**
1. **External projects** — These may be external tools/libraries the user is evaluating for potential integration (not yet integrated)
2. **Future roadmap** — They may appear in upcoming project plans or external specs
3. **Community projects** — They may be related to the DINO modding ecosystem but not yet relevant to DINOForge
4. **Typo/misremembering** — The project names may be slightly different than specified

---

## Recommendation

**No integration points can be documented without:**
1. Clarification on what PhenoCompose and NVMS are
2. Their GitHub URLs or local paths
3. Their purpose and use case relative to DINOForge
4. Whether they should be:
   - Vendored (git submodule)
   - Consumed (NuGet package)
   - Wrapped (thin adapter layer)
   - Composed (alongside DINOForge)

---

## Next Steps

To proceed with polyglot integration mapping:

### If evaluating external tools:
- Provide GitHub URLs or project links
- Define the intended role (e.g., asset pipeline, docs generation, testing)
- Specify integration type (MCP tool, CLI command, library wrapper, etc.)

### If internal projects:
- Provide local paths or repository locations
- Share README.md or project specifications
- Document the architectural relationship to DINOForge

### If unrelated/future roadmap:
- Add as a note to `CLAUDE.md` under "Future Polyglot Integrations" section
- Schedule for M12+ planning

---

## Document Template (If/When Integration Confirmed)

Once projects are clarified, this template can be filled:

```markdown
## [ProjectName]
- **Status**: [local/GitHub/external]
- **Purpose**: [1-2 sentence description]
- **Integration Type**: MCP Tool | CLI Command | Library | Submodule | Other
- **Primary Use Case**: [how it serves DINOForge]
- **Dependencies**: [any other projects/tools it requires]
- **Integration Method**: 
  - Wrapper: [if applicable]
  - Location: `[path or URL]`
  - Invocation**: `[how agents/users call it]`
- **Compatibility**: [version constraints, .NET/Node/Python version]
- **Testing Surface**: [how to verify integration works]
```

---

## Related Files

- `CLAUDE.md` — Agent governance and architecture
- `docs/` — VitePress documentation (uses PhenoDocs convention, not a tool)
- `MEMORY.md` — Project context and milestones
- `src/Tools/DinoforgeMcp/` — MCP server (existing polyglot integration)

---

**Document Status**: Research Complete · No Integration Surface Found  
**Date**: 2026-04-11  
**Awaiting**: User clarification on PhenoCompose and NVMS scope
