# ADR-006: Domain Plugin Architecture

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

If warfare-specific assumptions are baked into core, the framework becomes useless for economy mods, UI mods, scenario mods, etc. The framework must support "full modding range."

## Decision

The framework uses **domain plugins** to extend capabilities without bloating core.

### Core Framework
Only generic primitives: registries, pack loader, schema validator, override model, debug surfaces, logging.

### Domain Plugin: Warfare
Adds: factions, doctrines, combat unit classes, wave logic, weapons/projectile families, battlefield roles.

### Domain Plugin: Economy
Adds: production chains, worker rules, resource tuning, taxation/upkeep/progression.

### Domain Plugin: Scenario
Adds: mission events, triggers, objectives, narrative scripting.

### Domain Plugin: UI
Adds: HUD injection, inspector overlays, faction skinning, custom tooltips.

### Plugin Contract

Each domain plugin:
- Registers its own registry types with core
- Provides its own schemas
- Has its own validation rules
- Can extend the pack manifest format
- Must not depend on other domain plugins (only on core)

## Consequences

- Warfare is a plugin, not hardcoded
- New domains added without modifying core
- Domain-specific packs only load when their domain plugin is active
- Clean separation of concerns
- Each domain plugin has its own test suite
