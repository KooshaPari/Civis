# DINOForge Worklog

## 2026-03-20 - Runtime Integration Regression Fix

**[Detailed worklog](./2026-03-20.md)**

Investigated and fixed F9/F10 overlay regression caused by Harmony patch interference with the two-boot cycle. Root cause: Harmony patches (LazyPatch + DeltaTimeResurrectionPatch) were destroying the runtime root despite HideAndDontSave + DontDestroyOnLoad flags. Solution: Removed all Harmony patches, restored pure native persistence mechanism (commit df3b55e).

**Documentation updates**:
- Enhanced M8 roadmap section with root cause and fix details
- Added "Runtime Execution Flow" diagram to architecture.md
- Added "The Two-Boot Cycle" section explaining persistence mechanism
- Added Desktop Companion (WinUI 3, M9) to products section
- Updated CLAUDE.md with agent governance rules and tooling evolution guidance

**Next**: In-game validation of F9/F10 stability across scene transitions.

---

## 2026-03-09 - Project Inception

### Research Phase
- Analyzed 3 ChatGPT planning conversations covering:
  - Star Wars modding framework / agentic production pipeline design
  - Best modding DX/UX reference games (Factorio, RimWorld, Satisfactory, Minecraft Bedrock, UEFN)
  - Futuristic warfare total conversion architecture (5-layer architecture, faction archetypes, unit role matrix)
- Researched kooshapari GitHub projects for engineering conventions:
  - CLAUDE.md governance, CHANGELOG format, polyrepo-hexagonal architecture
  - BDD/SDD/TDD testing philosophy, spec-driven development
  - 17-mode SPARC framework, memory-centric agent coordination
- Researched DINO game modding landscape:
  - Unity ECS game with Burst compilation
  - BepInEx loader via modified `ecs_plugins` path
  - Harmony performance warning (halves framerate)
  - Small Nexus Mods community (~handful of mods)
  - No official mod API documentation
  - Steam Community guide (id:3348001330) as primary modding reference

### Deliverables Created
- [x] Git repository initialized
- [x] CLAUDE.md - agent governance and project overview
- [x] docs/PRD.md - full product requirements document
- [x] ADR-001 through ADR-006 - core architectural decisions
- [x] CHANGELOG.md
- [x] docs/WORKLOG.md (this file)
- [x] docs/warfare/ - warfare domain specification
- [x] docs/reference/ - modding DX/UX reference analysis
- [x] schemas/ - initial pack manifest and faction schemas

### Key Decisions
1. Product name: **DINOForge**
2. Three-product architecture: Runtime -> SDK -> Packs
3. Warfare as first domain plugin (not hardcoded)
4. Declarative-first: YAML/JSON manifests over C# patches
5. Agent-first repo design with legal move classes
6. ECS-native modding preferred over Harmony
7. Pack system with explicit manifests, deps, conflicts
8. 3 faction archetypes (Order, Industrial Swarm, Asymmetric) across 5 factions
9. Build order: Modern warfare first, Star Wars second, Guerrilla third

### Open Questions (from inception -- most now resolved)
- [ ] Exact DINO Unity version
- [x] ~~Mono vs IL2CPP status~~ -> **Mono confirmed**
- [x] ~~Component dump format~~ -> Entity Dumper and System Enumerator implemented in Runtime
- [ ] Asset replacement feasibility (models, textures, audio)
- [x] ~~Steam Workshop integration potential~~ -> Maps only via May 2025 Map Editor update; no code mod support
- [x] ~~BepInEx version compatibility~~ -> BepInEx 5.4.23.5 with modified Doorstop fork confirmed working
