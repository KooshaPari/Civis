# Work Audit — Civis


**Category: ARCHITECTURE**
**Index:** See `../WORKLOG.md` for the active project log and this file for the category index.

## Purpose

This log records research completions, architectural decisions, issues discovered (duplication, performance, governance), work completions, and planning artifacts (fork candidates, migration plans) for Civis.

## Worklog Categories

All entries MUST be organized into one of these categories:

| Category | File | Purpose |
|----------|------|---------|
| ARCHITECTURE | worklogs/ARCHITECTURE.md | ADRs, library extraction, major refactors |
| DUPLICATION | worklogs/DUPLICATION.md | Cross-project code duplication |
| DEPENDENCIES | worklogs/DEPENDENCIES.md | External deps, forks, modernization |
| INTEGRATION | worklogs/INTEGRATION.md | External integrations, bridge contracts |
| PERFORMANCE | worklogs/PERFORMANCE.md | Optimization, benchmarking, profiling |
| RESEARCH | worklogs/RESEARCH.md | Starred repo analysis, technology research |
| GOVERNANCE | worklogs/GOVERNANCE.md | Policy, evidence, quality gates, process |

## When to Write

Write an entry for:
- Research completions (starred repos, tech eval, design exploration)
- Significant decisions made (why we chose X over Y)
- Issues found (duplication >50 LOC, performance bottlenecks, governance gaps)
- Work completions (feature shipped, large refactor finished)
- Planning (fork candidates, migration roadmaps, deprecation notices)

## Format

Each entry should include:
- **Date** (YYYY-MM-DD)
- **Category** (from table above)
- **Title** (concise, searchable)
- **Context** (what prompted this work)
- **Finding/Decision** (what was discovered or decided)
- **Impact** (how it affects this project or others)
- **Project Tags** (e.g., `Civis`, `[cross-repo]`)

## Example

```markdown
### 2026-04-24 | DUPLICATION | Config loader duplication in 5 projects

**Context:** Cross-project audit identified identical config loading patterns.
**Finding:** 50+ LOC duplicated in BytePort, Civis, Dino, Eidolon, FocalPoint.
**Decision:** Extract into phenotype-config-core shared crate.
**Impact:** -250 LOC across 5 repos; single source of truth for config.
**Tags:** `[cross-repo]` `[DUPLICATION]`
```

---

See `../WORKLOG.md` for the active log. For docs navigation, use `../README.md`.
