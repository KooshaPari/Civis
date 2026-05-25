# ADR-005: ECS Integration Strategy

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

DINO uses Unity ECS with Burst compilation. Community guidance explicitly warns that:

1. The BepInEx loader for DINO uses a modified setup with `BepInEx/ecs_plugins/` path
2. Harmony-based patching has severe performance costs (removing Burst-generated DLL halves frame rate)
3. ECS-native modding is the recommended path
4. There is no official API documentation
5. The existing mod ecosystem is small (~handful of Nexus mods)

Sources: Steam Community guide (id:3348001330), Nexus Mods DINO page.

## Decision

### ECS-Native Over Harmony

Prefer writing ECS-based mods directly over Harmony method patching. Harmony is allowed only as a last resort for systems that cannot be reached through ECS.

### Isolation Architecture

```
[Mod Packs] --> [SDK/Registries] --> [Runtime Integration Layer] --> [DINO ECS]
                                            ^
                                    (Isolated, version-sensitive,
                                     fewest agents touch this)
```

The Runtime Integration Layer is:
- The ugliest, most brittle part
- Handles BepInEx/ECS bootstrapping, entity queries, prefab replacement, component patching, stat injection
- Version-gated against specific DINO builds
- Treated as unstable and highly version-sensitive

### Performance Rules

- No per-entity heavy patch logic every frame
- Patch systems, not individual objects
- Cache ECS queries
- Batch stat application
- Avoid Harmony hot paths unless proven safe

### Version Compatibility

- Maintain component dumps per game version
- Version gate runtime hooks
- Provide fallback/degradation when hooks break
- Auto-detect game version at bootstrap

## Consequences

- Runtime layer is small, isolated, touched by few agents
- SDK consumers never interact with raw ECS
- Game updates break Runtime layer only, not pack content
- Performance remains viable for large-scale mods
- Reverse engineering investment is captured and reused
