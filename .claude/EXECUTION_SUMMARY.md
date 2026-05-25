# Star Wars Clone Wars Mod - Execution Summary & Dispatch Brief

**Status:** ✓ Ready for Parallel Agent Execution
**Generated:** 2026-03-13
**For:** kooshapari (Koosha Paridehpour)

---

## What's Completed (73% Done)

✓ **All 27 units** have YAML definitions (stats, costs, roles)
✓ **All 10 buildings** have YAML definitions (costs, production)
✓ **Vanilla crosswalk** complete (militia → clone_trooper, etc.)
✓ **10 Sketchfab asset discoveries** with polycount/license data
✓ **Framework:** Asset pipeline, ContentLoader, ECS bridge ready

---

## What Remains (8 Phases, 22-Day Timeline)

| Phase | Work | Duration | Agents | Blocker |
|-------|------|----------|--------|---------|
| **1** | Download + validate 3 core assets (Clone, B1, AAT) | 3 days | 1 | None |
| **2A** | Source Clone infantry (6 units) from Sketchfab | 4 days | 1 | Phase 1 |
| **2B** | Source vehicles + buildings (9 items) | 4 days | 1 | Phase 1 |
| **3A** | Import + LOD Clone units (6 units) | 4 days | 2 | Phase 2A |
| **3B** | Import + LOD Droid units (6 units) | 4 days | 2 | Phase 2A |
| **3C** | Import + LOD vehicles (6 units) | 4 days | 2 | Phase 2B |
| **4** | Import + LOD buildings (10 buildings) | 5 days | 2 | Phase 2B |
| **5** | Generate prefabs + Addressables catalog | 4 days | 2 | Phase 3+4 |
| **6** | Map YAML to visual assets + prefabs | 5 days | 2 | Phase 5 |
| **7** | QA testing (unit + integration tests) | 4 days | 3 | Phase 6 |
| **8** | Release docs + sign-off | 3 days | 2 | Phase 7 |

**Total:** 8 phases, 22 days critical path, ~20 agents, 77 agent-days effort

---

## Three Key Documents

1. **`STARWARS_COMPLETION_PLAN.md`** (20 pages)
   - Complete phase breakdown with asset sourcing strategy
   - Unit/building distribution by phase
   - Risk mitigation + success criteria
   - Reference for all work details

2. **`PHASE_DEPENDENCY_DAG.md`** (visual reference)
   - Mermaid DAG showing phase dependencies
   - Wall-clock timeline + agent dispatch waves
   - Resource utilization graph
   - Dependency matrix

3. **`AGENT_ASSIGNMENTS.md`** (tactical dispatch)
   - Per-phase agent assignments (20 agents total)
   - Detailed checklists for each agent
   - Items covered, durations, dependencies
   - Quick copy-paste task briefs

---

## Critical Path (No Slack)

```
Phase 1 (Download) → Phase 2A/2B (Source) → Phase 3A/3B/3C/4 (LOD)
→ Phase 5 (Prefabs) → Phase 6 (YAML Mapping) → Phase 7 (QA) → Phase 8 (Release)
= 22 days minimum
```

**Any delay in Phase 1-2 pushes entire timeline.**

---

## Asset Sourcing Strategy (100% Wrapped)

### Priority: Wrap → Modify → Create

| Asset Type | Count | Strategy | Risk |
|------------|-------|----------|------|
| **Infantry** (Clone, Droid) | 12 | Wrap from Sketchfab (search "lowpoly") | Medium |
| **Vehicles** (BARC, AAT, V-19) | 6 | Wrap from Sketchfab | Medium |
| **Buildings** (Barracks, Defense) | 10 | Wrap from Sketchfab + Tatooine searches | Medium-High |
| **Elite/Hero** (Jedi, Grievous) | 4 | Wrap from Sketchfab (existing models) | Medium |
| **Utility** (Probe, Medic) | 3 | Wrap from Sketchfab (reuse/adapt) | Low |
| **Custom Creation** | 0 | **GOAL: ZERO** (all wrapped) | — |

**Licensing approach:**
- Primary: CC-BY, CC0 models (legally downloadable)
- Secondary: Contact author for permission (phamill327 for Tatooine buildings)
- Fallback: Use existing scifi assets + material tweaks

---

## Key Success Metrics

### By Phase 8 Sign-Off, Must Have:

✓ **27/27 units** with GLB model + YAML + prefab + mapping
✓ **10/10 buildings** with GLB model + YAML + prefab + mapping
✓ **37+ assets** discovered → imported → optimized → integrated
✓ **95+ tests passing** (80 baseline + 15 new)
✓ **0 custom models** created (100% wrapped from Sketchfab)
✓ **0 errors** in asset pipeline validation
✓ **Docs updated:** CHANGELOG + README + factions guide
✓ **Pack tagged:** v0.2.0-warfare-starwars ready for release

---

## Parallel Execution Waves

### Wave 1: Day 1
- **Agent 1** → Phase 1 (download 3 assets)

### Wave 2: Day 2-3
- **Agents 2-3** → Phase 2A/2B (source infantry, vehicles, buildings)

### Wave 3: Day 4-8
- **Agents 4-11** → Phases 3A/3B/3C/4 (LOD + optimization)
- **Peak concurrency:** 5 agents max
- **No resource conflicts** (different file directories)

### Wave 4: Day 9-12
- **Agents 12-13** → Phase 5 (prefab generation)

### Wave 5: Day 13-17
- **Agents 14-15** → Phase 6 (YAML mapping)

### Wave 6: Day 18-21
- **Agents 16-18** → Phase 7 (QA testing, optional in-game validation)

### Wave 7: Day 22-24
- **Agents 19-20** → Phase 8 (release docs, sign-off)

---

## Per-Phase Deliverables

### Phase 1: Foundation
- ✓ 3× GLB files downloaded (Clone, B1, AAT)
- ✓ IP validation report
- ✓ `asset_pipeline.yaml` configured with LOD targets

### Phases 2A-2B: Sourcing
- ✓ 15 Sketchfab asset IDs identified
- ✓ GLB files saved to `packs/warfare-starwars/assets/raw/`
- ✓ Sourcing manifest (author, license, polycount)

### Phases 3A-C + 4: LOD Optimization
- ✓ 37 JSON model files (imported)
- ✓ 111 LOD variants (37 × 3 levels)
- ✓ All validate against polycount targets
- ✓ Material slots tested for faction colors

### Phase 5: Prefab Generation
- ✓ 81 unit prefab variants (27 units × 3 LOD)
- ✓ 30 building prefab variants (10 buildings × 3 LOD)
- ✓ Addressables catalog registered
- ✓ Material instances created

### Phase 6: YAML Integration
- ✓ 27 units updated with `visual_asset` field
- ✓ 10 buildings updated with `visual_asset` field
- ✓ All prefab_address fields valid
- ✓ Crosswalk mapping validated

### Phase 7: QA
- ✓ 15+ new asset tests written + passing
- ✓ 80+ integration tests passing
- ✓ 0 broken asset references
- ✓ Optional: In-game smoke test passing

### Phase 8: Release
- ✓ CHANGELOG.md updated with v0.2.0 details
- ✓ README.md faction roster updated
- ✓ Asset sourcing report finalized
- ✓ pack.yaml validated for distribution
- ✓ Git tag: v0.2.0-warfare-starwars

---

## Resource Allocation

### Agent-Days by Phase
```
Phase 1: 3 days (1 agent)
Phase 2A: 4 days (1 agent)
Phase 2B: 4 days (1 agent)
Phase 3A: 8 days (2 agents)
Phase 3B: 8 days (2 agents)
Phase 3C: 4 days (1 agent)
Phase 4: 10 days (2 agents)
Phase 5: 8 days (2 agents)
Phase 6: 10 days (2 agents)
Phase 7: 12 days (3 agents, some overlap)
Phase 8: 6 days (2 agents)
─────────────────────────────
TOTAL: 77 agent-days across ~20 agents
```

### Estimated Token Usage
- ~8,000-12,000 tokens per agent per phase
- 20 agents × 10,000 avg = **~200K tokens** (well within Haiku limits)

---

## How to Dispatch Agents

### Step 1: Create Worktree
```bash
# For Phase 1 agent
git worktree add .claude/worktrees/phase1-foundation main
# Switch agent into that worktree
```

### Step 2: Assign Task (Example for Agent 1)
```
Phase: 1 (Foundation Assets)
Duration: 3 days
Task: Download Clone Trooper, B1, AAT models from Sketchfab
      Validate IP licensing, create asset_pipeline.yaml config
Files to modify:
  - packs/warfare-starwars/asset_pipeline.yaml (new)
  - packs/warfare-starwars/assets/raw/ (new GLB files)
  - .claude/sw_search/ (sourcing reports)
Tests: dotnet run --project src/Tools/PackCompiler -- assets validate
Checklist: See AGENT_ASSIGNMENTS.md → Phase 1 → Agent-1 Checklist
Done when: AssetIndex.json updated, pack.yaml validates, 0 errors
```

### Step 3: Agent Works Independently
- Agent reads `STARWARS_COMPLETION_PLAN.md` for context
- Agent reads `AGENT_ASSIGNMENTS.md` for specific checklist
- Agent commits to worktree
- Agent reports completion with test output

### Step 4: Merge to Main
- Code review (if needed)
- Merge worktree back to main
- Delete worktree
- Dispatch next wave

---

## Risk Mitigation Playbook

### If Sketchfab Model Unavailable (Phase 1-2)
1. Search for alternative keywords (e.g., "star wars military android" instead of "B1")
2. Check multiple Sketchfab results (go past top 3)
3. **Fallback:** Use conceptually similar lowpoly model + modify in Blender
4. **Last resort:** Placeholder geometry (cube/sphere) + note for manual creation later

### If LOD Optimization Degrades Silhouette (Phase 3)
1. Manual edge preservation in decimation tool
2. Test LOD0/LOD1 swap if LOD2 is too degraded
3. Increase LOD1 polycount target (e.g., 60% instead of 50%)
4. **Flag for manual review** if >10% quality loss

### If Prefab Serialization Fails (Phase 5)
1. Validate JSON model file structure
2. Check material slot names match expected keys
3. Run unit test: `PrefabGenerationService.GenerateUnitPrefabs()` with debug output
4. Review error log for missing references

### If YAML Mapping Breaks Tests (Phase 6)
1. Validate all `prefab_address` keys exist in Addressables catalog
2. Check crosswalk mapping for typos
3. Run: `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars`
4. Review error messages for specific broken reference

### If In-Game Spawn Fails (Phase 7)
1. Enable DebugOverlay (F9) to check asset load errors
2. Verify StatModifierSystem applied multipliers (check console logs)
3. Check ECS bridge: `ComponentMap` has correct vanilla→themed mappings
4. Review BepInEx mod menu for pack load errors

---

## Quality Gates (Must Pass Before Phase 8)

### Asset Pipeline Validation
```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars
# Expected: 0 errors, 0 warnings
```

### Full Build
```bash
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars
# Expected: ✓ Pack validation complete. Ready for distribution.
```

### Unit Test Suite
```bash
dotnet test src/DINOForge.sln --verbosity normal
# Expected: ≥95 tests passing (80 baseline + 15 new)
```

### Pack.yaml Completeness
```yaml
# packs/warfare-starwars/pack.yaml must have:
✓ id: warfare-starwars
✓ version: 0.2.0
✓ loads: { units: 27, buildings: 10, ... }
✓ No unresolved dependencies
✓ No conflicts listed
```

---

## Documentation to Update (Phase 8)

1. **`CHANGELOG.md`**
   - Add v0.2.0 section
   - List: units added, buildings added, assets integrated
   - Technical notes: asset sourcing, pipeline phases

2. **`README.md`**
   - Add Star Wars pack to pack examples
   - Update faction table (add Republic, CIS)

3. **`docs/warfare/factions.md`** (new or update)
   - Galactic Republic: 10 units, 10 buildings
   - Confederacy of Independent Systems: 9 units, 10 buildings
   - Doctrine descriptions + special mechanics

4. **`docs/asset-sourcing/STARWARS_CREDITS.md`** (new)
   - Sketchfab model links (27 items)
   - Author credits + licenses
   - Modification log (if any)

---

## Timeline Visualization

```
Week 1                Week 2                Week 3
Mon-Tue  Wed-Thu     Mon-Tue  Wed-Thu      Mon-Tue  Wed-Thu  Fri
┌────────┬──────┐    ┌────────┬──────┐     ┌────────┬──────┐
│ Phase 1│2A+2B │ →  │3+4/LOD │Phase5│ →   │Phase 6 │Phase7│ Phase 8
│  Found │Src   │    │Optim   │Prefab│     │YAML    │ QA   │ Release
└────────┴──────┘    └────────┴──────┘     └────────┴──────┘
                     │                                       │
                     Critical Path ◄──────────────────────────
```

---

## Next: Immediate Actions

### For You (kooshapari):
1. Read all three documents:
   - `STARWARS_COMPLETION_PLAN.md` (full context)
   - `PHASE_DEPENDENCY_DAG.md` (visual roadmap)
   - `AGENT_ASSIGNMENTS.md` (dispatch checklists)

2. Verify Phase 1 assignment matches your capacity (can agent-1 start tomorrow?)

3. Create worktree(s) for Phase 1 agent if using parallel subagent model

4. Confirm: Do you want all 20 agents dispatched ASAP, or staggered by phase?

### For Phase 1 Agent:
1. Create worktree (if parallel model)
2. Read `STARWARS_COMPLETION_PLAN.md` (Phase 1 section)
3. Read `AGENT_ASSIGNMENTS.md` (Phase 1 checklist)
4. Execute Phase 1 checklist (3 days)
5. Commit results, report completion
6. Phase 2A/2B agents can start immediately after Phase 1 outputs are available

---

## Success Definition

**When Phase 8 completes:**
- ✓ Clone Wars mod is **100% complete** and **playable**
- ✓ All units/buildings have visual models + proper stats
- ✓ Mod can be distributed (pack.yaml validated)
- ✓ Test suite passes (95+ tests)
- ✓ Documentation complete (changelog, credits, roster)
- ✓ **GOAL ACHIEVED: 0 custom models created, 100% wrapped from Sketchfab**

---

## Files to Track

### Core Working Directories
- `/c/Users/koosh/Dino/packs/warfare-starwars/` — Pack root
- `/c/Users/koosh/Dino/packs/warfare-starwars/assets/raw/` — Downloaded GLB files
- `/c/Users/koosh/Dino/packs/warfare-starwars/units/` — Unit YAML
- `/c/Users/koosh/Dino/packs/warfare-starwars/buildings/` — Building YAML

### Key Reference Files
- `/c/Users/koosh/Dino/universes/star-wars-clone-wars/crosswalk.yaml` — Vanilla mappings
- `/c/Users/koosh/Dino/packs/warfare-starwars/pack.yaml` — Pack manifest (to update v0.2.0)
- `/c/Users/koosh/Dino/src/Tools/PackCompiler/` — Build tool

### Documentation to Update
- `/c/Users/koosh/Dino/CHANGELOG.md`
- `/c/Users/koosh/Dino/docs/warfare/factions.md`

---

## References

- **Game Details:** DINO 2021.3.45f2, Unity ECS, Mono runtime, 45K entities
- **Asset Tool:** AssetsTools.NET (MIT) for bundle reading
- **Validation:** NJsonSchema for YAML/JSON
- **Testing:** xUnit + FluentAssertions
- **CI/CD:** GitHub Actions (build, test, lint)

---

## Ready to Dispatch

✓ Phase 1 assignment: Clear
✓ Phase 2A/2B sourcing strategy: Defined
✓ Phases 3-4 LOD optimization: Specified
✓ Phase 5 prefab generation: Planned
✓ Phase 6 YAML integration: Mapped
✓ Phase 7 QA validation: Scoped
✓ Phase 8 release: Outlined

**All documentation prepared for parallel agent execution.**

---

**Plan Created By:** DINOForge Agent Coordinator
**For:** kooshapari
**Status:** ✓ READY FOR DISPATCH
**Next Step:** Assign Phase 1 agent and begin
