# ADR-002: Declarative-First Architecture

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

DINO modding requires reverse engineering and ECS-aware patching. Without guardrails, agents will write arbitrary imperative C# patches against game internals, creating unmaintainable code.

The best modding ecosystems (Factorio, RimWorld, Minecraft Bedrock) all use declarative-first authoring: easy things are data, hard things are code, invasive things are controlled patches.

## Decision

DINOForge adopts a **5-layer content model**:

### Layer 1: Content Packs (Declarative)
JSON/YAML/TOML data defining factions, units, weapons, projectiles, audio, UI labels, tech trees, AI params, spawn tables, VFX references.

### Layer 2: Asset Packs (Declarative + Binary)
Bundled art/audio/prefab references with explicit manifests and compatibility metadata.

### Layer 3: Code Plugins (Narrow API)
C# plugin API for custom systems, ECS queries, hooks, behaviors. Must go through SDK interfaces.

### Layer 4: Patch Layer (Controlled, Marked Unsafe)
Controlled patch points for base game modifications. Fenced off, clearly marked as high-fragility. Harmony/BepInEx equivalent.

### Layer 5: Tooling
CLI + GUI tools: create, validate, build, diff, package, install, generate docs, run compat tests.

## Consequences

- Agents mostly author YAML/JSON manifests and content bundles
- Raw imperative C# plugin code is rare and goes through review gates
- Pack compiler validates everything before runtime
- Most mods become data-driven, not code-driven
- "preferred" = register faction with units/costs/tech/projectiles/icons/audio/waves
- "not preferred" = write custom imperative patch code editing 14 engine systems manually
