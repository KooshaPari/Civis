# Star Wars Clone Wars Mod Completion Plan
## Master Index & Quick Start Guide

**Status:** ✓ Ready for Parallel Agent Execution
**Created:** 2026-03-13
**Project:** DINOForge `warfare-starwars` pack (v0.2.0)

---

## What's in This Plan?

This folder contains a **complete phased work plan** for finishing the Star Wars Clone Wars mod for DINOForge. It covers asset sourcing, 3D model import, prefab generation, YAML integration, and QA testing across 8 phases with ~20 parallel haiku agents.

**Goal:** 100% completion with 0 custom model creation (all wrapped from Sketchfab).

---

## Four Key Documents

### 1. 📋 `EXECUTION_SUMMARY.md` ← **START HERE**
**2 pages, read first**
- What's done (73%), what remains (27%)
- 8-phase timeline at a glance
- Critical path + resource allocation
- Quick dispatch instructions
- Risk mitigation playbook

**Read this if:** You want a 2-minute overview before diving into details.

---

### 2. 📊 `STARWARS_COMPLETION_PLAN.md` ← **PHASE DETAILS**
**20 pages, complete reference**
- Full breakdown of all 8 phases
- Unit/building inventory (27 units, 10 buildings)
- Asset sourcing strategy per item (Wrap → Modify → Create)
- Detailed dependencies, blockers, deliverables
- Tech stack (AssimpNet, PackCompiler, Addressables)
- Risk registry + mitigation
- Success criteria (Phase 8 sign-off)

**Read this if:** You're assigning agents or need full context.

---

### 3. 🔀 `PHASE_DEPENDENCY_DAG.md` ← **VISUAL ROADMAP**
**5 pages, flow + timeline**
- Mermaid DAG showing all phase dependencies
- Wall-clock timeline (22 days critical path)
- Agent dispatch schedule (7 waves)
- Resource utilization graph
- Unit/building distribution by phase
- Dependency matrix

**Read this if:** You want to visualize phase ordering and dispatch timing.

---

### 4. 📝 `AGENT_ASSIGNMENTS.md` ← **AGENT CHECKLISTS**
**10 pages, tactical dispatch**
- Per-phase agent assignments (20 agents total)
- Detailed checklist for each agent (what to do, commands to run, expected outputs)
- Parallel task groupings
- Items covered, dependencies, success criteria

**Read this if:** You're briefing individual agents or need task-level details.

---

## How to Use This Plan

### Scenario A: You're Reading This Alone
1. **Start:** Read `EXECUTION_SUMMARY.md` (2 min)
2. **Deepen:** Read `PHASE_DEPENDENCY_DAG.md` (5 min, visual)
3. **Reference:** Keep `STARWARS_COMPLETION_PLAN.md` nearby for details
4. **Dispatch:** Use `AGENT_ASSIGNMENTS.md` to brief agents

### Scenario B: You're Briefing an Agent
1. **For agent:** Share `EXECUTION_SUMMARY.md` + relevant phase section from `STARWARS_COMPLETION_PLAN.md`
2. **Checklist:** Point agent to their phase in `AGENT_ASSIGNMENTS.md`
3. **Context:** Agent reads full `STARWARS_COMPLETION_PLAN.md` for context

### Scenario C: You're Managing All Agents (Parallel)
1. **Wave 1 (Day 1):** Dispatch Agent-1 with Phase 1 task (download 3 assets)
2. **Wave 2 (Day 2):** Dispatch Agents 2-3 with Phase 2A/2B tasks
3. **Wave 3 (Day 4):** Dispatch Agents 4-11 with Phase 3+4 tasks (parallel LOD)
4. **Repeat:** Follow `AGENT_ASSIGNMENTS.md` dispatch schedule

---

## Quick Reference: Phase Timeline

```
Phase 1: Foundation Assets         (3 days)  ← Critical path start
Phase 2: Asset Sourcing             (4 days)
Phase 3: LOD Optimization           (4 days)  ← 5 agents in parallel
Phase 4: Building Models            (5 days)
Phase 5: Prefab Generation          (4 days)  ← Depends on 3+4
Phase 6: YAML Integration           (5 days)  ← Depends on 5
Phase 7: QA Testing                 (4 days)  ← Depends on 6
Phase 8: Release + Docs             (3 days)  ← Final sign-off
────────────────────────────────────────────
Total: 22 days critical path (week-at-a-time realistic)
```

---

## Critical Success Factors

✓ **Phase 1 must complete first** (no parallel start) — sets asset pipeline
✓ **Phases 2A/2B can run in parallel** — both source assets independently
✓ **Phases 3A/3B/3C/4 can run in parallel** — no file conflicts (different units/buildings)
✓ **Phase 5 depends on phases 3+4** — can't generate prefabs without imported models
✓ **Phase 6 depends on phase 5** — need prefab_address before mapping YAML
✓ **Phase 7 depends on phase 6** — can't test without complete integration
✓ **Phase 8 depends on phase 7** — can't release without passing QA

**No slack in critical path.** Any delay in 1→2→3/4→5→6→7→8 pushes full timeline.

---

## What Each Agent Does (High-Level)

| Agent | Phase | Work | Items | Days |
|-------|-------|------|-------|------|
| 1 | 1 | Download 3 core assets | Clone, B1, AAT | 3 |
| 2 | 2A | Find Clone infantry models on Sketchfab | 6 units | 4 |
| 3 | 2B | Find vehicles + buildings on Sketchfab | 9 items | 4 |
| 4-5 | 3A | Import Clone units + generate LOD variants | 6 units | 4 |
| 6-7 | 3B | Import CIS droid units + LOD variants | 6 units | 4 |
| 8-9 | 3C | Import vehicles + LOD variants | 6 units | 4 |
| 10-11 | 4 | Import buildings + LOD variants | 10 buildings | 5 |
| 12-13 | 5 | Generate prefabs + Addressables catalog | 37 assets | 4 |
| 14-15 | 6 | Map YAML definitions to prefabs | 37 assets | 5 |
| 16-18 | 7 | QA testing (unit + integration) | All | 4 |
| 19-20 | 8 | Docs + release sign-off | Manifest | 3 |

---

## Key Technologies

- **Game:** Diplomacy is Not an Option (Unity 2021.3.45f2, ECS, Mono)
- **Build:** PackCompiler (C#, ASP.NET tooling)
- **Assets:** AssimpNet (GLB import), AssetsTools.NET (bundle I/O)
- **Validation:** NJsonSchema (YAML/JSON schema validation)
- **Testing:** xUnit + FluentAssertions
- **CI/CD:** GitHub Actions + .NET 8

---

## Expected Outputs (Phase 8 Sign-Off)

✓ 27 unit models (GLB → JSON → .prefab)
✓ 10 building models (GLB → JSON → .prefab)
✓ Full Addressables catalog with faction colors
✓ Complete YAML mapping (unit.yaml + building.yaml)
✓ 95+ unit tests passing
✓ CHANGELOG.md + README.md updated
✓ Asset sourcing report (Sketchfab credits)
✓ Git tag: `v0.2.0-warfare-starwars` (ready for distribution)

---

## Files & Locations

### Working Directories
```
packs/warfare-starwars/
├── units/                          (27 YAML definitions)
│   ├── republic_units.yaml         ✓ DONE
│   └── cis_units.yaml              ✓ DONE
├── buildings/                      (10 YAML definitions)
│   ├── republic_buildings.yaml     ✓ DONE
│   └── cis_buildings.yaml          ✓ DONE
├── assets/
│   ├── raw/                        (GLB files → import here)
│   ├── registry/
│   │   ├── asset_index.json        (discovered assets)
│   │   └── provenance_index.json   (IP tracking)
│   └── SKETCHFAB_MODELS.json       (search results)
├── asset_pipeline.yaml             (to update Phase 1)
├── doctrines/                      (8 CIS/Republic doctrines)
├── factions/                       (2 factions: Republic, CIS)
├── waves/                          (Clone Wars scenarios)
└── pack.yaml                       (manifest → update v0.2.0)

universes/
├── star-wars-clone-wars/
│   └── crosswalk.yaml              (vanilla ↔ themed mapping) ✓ DONE

docs/
├── warfare/
│   └── factions.md                 (to update Phase 8)
└── asset-sourcing/
    └── STARWARS_CREDITS.md         (to create Phase 8)
```

### Reference Docs
- `CHANGELOG.md` (to update Phase 8)
- `README.md` (to update Phase 8)
- `MASTER_SYNTHESIS.md` (user's memory file)

---

## Asset Sourcing Strategy

### Goal: 100% Wrapped (0 Custom Creation)

1. **Wrap (Primary):** Find existing Sketchfab model
   - Example: Search "clone trooper lowpoly" → use existing GLB as-is
   - Fastest, lowest risk, maximizes feature coverage

2. **Modify (Secondary):** Adapt existing model with Blender
   - Example: Clone Trooper base + add custom armor piece
   - Used for variants (Clone Heavy = Clone Trooper + armor)
   - ~10-20% effort vs full creation

3. **Create (Last Resort):** Handmake 3D model
   - Example: Alien creature with no real-world equivalent
   - **NOT USED** for Star Wars pack (everything wrappable)
   - Reserved only if absolutely no model exists

### Sourcing Queries (Phase 2A/2B)

```
Phase 2A (Infantry):
  "clone trooper lowpoly" → find 2-3 candidates
  "clone sharpshooter" OR "clone marksman"
  "clone heavy trooper"
  "clone medic" OR "medical trooper"
  "ARF trooper" OR "reconnaissance trooper"

Phase 2B (Vehicles + Buildings):
  "BARC speeder" (fast vehicle)
  "AT-TE walker" (heavy walker)
  "V-19 starfighter" (fighter jet)
  "tatooine building" (civilian/command structure)
  "geonosis droid factory" (industrial facility)
  "AAT tank" (already have Phase 1)
  "STAP speeder" (light vehicle)
  "DSD1 dwarf spider" (walking tank)
```

---

## Running Commands (Reference)

### Asset Validation
```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars
```

### Asset Optimization (LOD generation)
```bash
dotnet run --project src/Tools/PackCompiler -- assets optimize packs/warfare-starwars --lod-targets 50,25
```

### Prefab Generation
```bash
dotnet run --project src/Tools/PackCompiler -- assets generate packs/warfare-starwars
```

### Full Pipeline Build
```bash
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars
```

### Run Tests
```bash
dotnet test src/DINOForge.sln --verbosity normal
# Target: ≥95 tests passing
```

---

## Success Criteria Checklist (Phase 8)

- [ ] All 27 units have `.prefab` + YAML + Addressables mapping
- [ ] All 10 buildings have `.prefab` + YAML + Addressables mapping
- [ ] `dotnet run -- validate packs/warfare-starwars` = 0 errors, 0 warnings
- [ ] `dotnet run -- build packs/warfare-starwars` = ✓ Ready for distribution
- [ ] `dotnet test src/DINOForge.sln` = ≥95 tests passing
- [ ] CHANGELOG.md updated with v0.2.0 entry
- [ ] README.md faction roster updated
- [ ] `docs/asset-sourcing/STARWARS_CREDITS.md` created with Sketchfab authors
- [ ] `docs/warfare/factions.md` updated with unit/building descriptions
- [ ] Git tag `v0.2.0-warfare-starwars` created
- [ ] Asset sourcing report: 0 custom models (100% wrapped)

---

## Dispatch Instructions

### For kooshapari (You):
1. **Day 1:** Read `EXECUTION_SUMMARY.md` (2 min)
2. **Day 1:** Read `PHASE_DEPENDENCY_DAG.md` (5 min)
3. **Day 1:** Dispatch Agent-1 with Phase 1 task
4. **Day 2:** Dispatch Agents 2-3 with Phase 2A/2B tasks
5. **Day 4:** Dispatch Agents 4-11 with Phase 3+4 tasks
6. **Continue:** Follow dispatch schedule in `AGENT_ASSIGNMENTS.md`

### For Each Agent:
1. Read assigned phase section in `STARWARS_COMPLETION_PLAN.md` (full context)
2. Find your phase checklist in `AGENT_ASSIGNMENTS.md` (what to do)
3. Execute checklist + run test commands
4. Commit results to worktree
5. Report completion with test output
6. Next phase agents can start (if dependencies met)

---

## Common Issues & Fixes

### Q: Can I start Phase 2A before Phase 1 finishes?
**A:** No. Phase 1 outputs (asset_pipeline.yaml, assetIndex) are inputs to Phase 2. Wait until Phase 1 commits.

### Q: Can I run Phases 3A, 3B, 3C in parallel?
**A:** Yes! They operate on different units (Clone, Droid, Vehicle) with no conflicts. All 3 agents can work simultaneously.

### Q: What if a Sketchfab model is unavailable?
**A:** Try alternative search term (e.g., "droid", "android", "robot" instead of exact model name). Contact author for permission. Last resort: use lower-quality substitute + note for future improvement.

### Q: How do I test if my LOD looks good?
**A:** Compare polycount (expect ~50% reduction per LOD), verify silhouette is preserved, check in-game that model renders without obvious degradation.

### Q: Do I need Blender to modify models?
**A:** For most work, no. GLB files can be validated without Blender. If modification needed (e.g., add armor piece), Blender import/export is helpful but not required for basic sourcing.

---

## Token Budget

- **Phase 1 agent:** ~8K tokens (download, validate, docs)
- **Phase 2A/2B agents:** ~10K tokens each (sourcing, manifests)
- **Phase 3A-C/4 agents:** ~12K tokens each (import, LOD, validation)
- **Phase 5-6 agents:** ~10K tokens each (prefab gen, mapping)
- **Phase 7 agents:** ~8K tokens each (testing, reports)
- **Phase 8 agents:** ~6K tokens each (docs, release)

**Total: ~200K tokens across 20 agents** (well within Haiku budget)

---

## Questions?

Refer to:
- **Context:** `STARWARS_COMPLETION_PLAN.md` (20 pages of details)
- **Timeline:** `PHASE_DEPENDENCY_DAG.md` (visual + schedule)
- **Tasks:** `AGENT_ASSIGNMENTS.md` (phase checklists)
- **Overview:** This file (README)

---

## Status

✓ Plan complete
✓ Dependencies mapped
✓ Agent assignments prepared
✓ Risk mitigation planned
✓ Success criteria defined

**Ready for dispatch. Begin Phase 1.**

---

**Generated:** 2026-03-13
**For:** kooshapari (Koosha Paridehpour)
**Project:** DINOForge Star Wars Clone Wars Mod
**Version:** 0.2.0 (Complete)
