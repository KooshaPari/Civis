# ADR-003: Pack System Design

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

Mods need to be composable, installable, versionable, and conflict-aware. Loose file drops ("DLL in plugins and pray") create chaos. The best modding ecosystems (Factorio, Minecraft Bedrock) treat mods as first-class packages.

## Decision

Every mod is a **pack** with explicit metadata:

```yaml
id: starwars-republic-cis
name: Republic vs CIS Pack
version: 0.1.0
framework_version: ">=0.4.0 <0.5.0"
author: DINOForge Agents
type: total_conversion  # content | balance | ruleset | total_conversion | utility
depends_on:
  - dino-core
  - dino-warfare-domain
conflicts_with:
  - modern-west-pack
loads:
  factions:
    - republic
    - cis
  doctrines:
    - elite_discipline
    - mechanized_attrition
  audio:
    - sw_blaster_pack
  visuals:
    - sw_projectiles
  localization:
    - sw_english
```

### Pack Compiler Pipeline

1. Agent authors pack files
2. Compiler validates schemas
3. Compiler resolves references
4. Compiler checks missing assets and circular dependencies
5. Test runner simulates pack
6. Compiler builds pack artifact
7. Compiler emits compatibility metadata and test plan

### Pack Categories

- **Content**: Units, buildings, projectiles, effects, icons, names, audio
- **Balance**: Stats, costs, spawn sizes, economy rates
- **Ruleset**: Research requirements, wave timings, victory conditions
- **Total Conversion**: Full faction swap with theme, replacing multiple systems
- **Utility**: Debug tools, profilers, inspectors

## Consequences

- All mods are discoverable, diffable, testable
- Pack install/uninstall is atomic
- Compatibility conflict detection is automated
- Agents reason at package level, not source-file level
- No loose file modifications outside pack system
