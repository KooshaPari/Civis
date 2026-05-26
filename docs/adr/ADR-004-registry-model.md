# ADR-004: Registry Model

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

Without a central registration model, agents will scatter content definitions across arbitrary files and hardcode IDs into engine glue. This creates unmaintainable spaghetti.

## Decision

Everything goes through registries. Agents add or override entries in registries. They do not just "edit the game."

### Registry Domains

| Registry | Contents |
|----------|----------|
| Units | Unit definitions with role, stats, behavior, visuals |
| Buildings | Building definitions with role, cost, tech requirements |
| Weapons | Weapon definitions with damage model, projectile type |
| Projectiles | Projectile definitions with VFX, speed, behavior |
| Effects | Visual/audio effects |
| Audio Packs | Sound replacement sets |
| UI Skins | UI theme definitions |
| Doctrines | Faction-level gameplay modifiers |
| Factions | Faction definitions with roster, economy, visuals |
| Wave Templates | Enemy wave composition and timing |
| Behaviors | AI behavior tags and rules |
| Scenario Scripts | Mission events and triggers |
| Tech Nodes | Research tree entries |
| Localization | String bundles per language |

### Registry Contract

Each registry entry has:
- Unique string ID (namespaced: `pack_id:entry_id`)
- Schema-validated data
- Override priority (base < framework < domain < pack)
- Source pack reference
- Validation status

### Override Model

Layered content overrides:
1. Base game defaults (discovered via introspection)
2. Framework defaults
3. Domain plugin additions
4. Pack-level overrides

## Consequences

- Composability: multiple packs can contribute to same registry
- Diffability: registry state is inspectable
- Testability: mock registries for testing
- Pack install/uninstall: registry entries are scoped to packs
- Validation: duplicate ID detection, missing reference detection
- Compatibility: conflict detection when packs override same entries
