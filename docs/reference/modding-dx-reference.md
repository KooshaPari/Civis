# Modding DX/UX Reference Analysis

**Date**: 2026-03-09
**Purpose**: Reference games and patterns to emulate for DINOForge modding library design.

---

## Tier 1: Best Official Modding DX

### Factorio
**Copy**: API shape, manifests, dependency/version handling, distribution, contract discipline.

- Documented API with stable contract
- Official mod portal with in-ecosystem distribution
- Explicit publish/upload APIs
- Modder workflow: "author against a known contract, package, publish, update"
- Not "hack files until it works"

### Minecraft Bedrock Add-Ons
**Copy**: Pack structure, capability declarations, folder conventions.

- Resource packs + behavior packs + manifest structure
- Content organization that makes generation and validation easy
- Machine-readable pack schemas

### Bethesda Creation Kit
**Copy**: Editor-centric world/content creation, asset authoring pipelines.

- "Ship an editor and treat mods as first-class data packages"
- Historically strongest mod ecosystems

### UEFN / Roblox Creator Stack
**Copy**: End-to-end creation pipeline concept (editor, scripting, assets, testing, publishing, docs, discoverability).

- Not classic modding but best modern creator DX
- Creation treated as full pipeline

---

## Tier 2: Best Community-Rescued DX

### Satisfactory + SML/SMM
**Copy**: Mod loaders, plugin bootstrap, community tooling, install/dependency UX.

- Satisfactory Mod Manager makes install smooth
- SML docs cover dev environment setup
- Relevant because DINOForge lives in same world: semi-official feeling, layered over shipped game

### RimWorld + Harmony
**Copy**: Split between declarative content and imperative code patches.

- XML-heavy approach + C#/Harmony extension
- Easy things are data, hard things are code, invasive things are patch-based
- Critical conceptual model for DINOForge

### Unity/BepInEx/Thunderstore Ecosystems
**Copy**: Reality of mod loaders on shipped Unity games.

- Cities: Skylines II community stack
- More "effective" than elegant but proven

---

## Key Design Patterns to Steal

### 1. Stable Package Shape
Every mod has manifest, declared version, dependencies, predictable folder layout.

### 2. Declarative-First Authoring
Most mods are data edits: content packs, stat changes, loot tables, spawn rules, UI definitions, config bundles.

### 3. Code Only for Hard Cases
Narrow surface: event hooks, patch points, extension interfaces, ECS system registration, custom components.

### 4. One-Click Install/Update
If system requires dragging DLLs into mystery folders, UX is already mediocre.

### 5. Validation Before Runtime
Reduce "launch game, crash, guess why." Lint manifests, dependency graphs, schema validity, asset references, ECS registration conflicts before game boots.

### 6. Discoverability and Docs in Happy Path
Docs part of workflow, not buried in Discord archaeology.

---

## What NOT to Copy

- Raw BepInEx "drop DLL in plugins and pray"
- Undocumented patch soup
- Hidden load order rules
- Mods defined by arbitrary code instead of typed schemas
- Asset imports with no validation
- Discord-as-documentation

---

## DINOForge Hybrid Model

| Aspect | Source |
|--------|--------|
| API shape, manifests, versioning | Factorio |
| Declarative content + code escape hatch | RimWorld |
| Mod loaders, plugin bootstrap | Satisfactory/BepInEx |
| Pack schemas, folder conventions | Minecraft Bedrock |
| End-to-end pipeline concept | UEFN/Roblox |

---

## Agentic Modding Pipeline (from Star Wars Conversation)

The scaffold converts natural-language requests into a deterministic build pipeline:

1. **Request -> Typed Mod Spec** (machine-validated contract, not loose prompt)
2. **Orchestrator** spawns specialist agents
3. **Game Adapter** handles per-game inspection, loader, asset discovery, patching
4. **Universe Pack** encodes faction taxonomies, era rules, naming/style guides
5. **Asset Pipeline** handles discovery, permission checking, generation, conversion
6. **Validation** checks completeness, coherence, stability, packaging
7. **Recipe Library** provides reusable recipes (replace unit family, add hero, retheme buildings)
8. **Knowledge Base** provides persistent queryable knowledge for game systems, prior mods, style guides

### Quality Formula
```
quality = model capability x process maturity x tool coverage x knowledge depth x validation rigor x library depth
```

Once model is "good enough," dominant terms are process, library, validation, tools.
