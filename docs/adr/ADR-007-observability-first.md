# ADR-007: Observability as First-Class Feature

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

Since no human inspects code, the framework must expose truth through tools. If agents can't see the system clearly, they will thrash. This is non-negotiable for agent-driven development.

## Decision

DINOForge requires comprehensive observability at every layer.

### Runtime Logs
- What loaded, what failed
- What got patched
- Missing references
- Fallback activations
- Version mismatches
- Performance warnings

### In-Game Debug Overlay
- Loaded packs
- Active faction definitions
- Entity counts
- Unit mappings
- Wave composition preview
- Missing asset markers
- Component values
- Spawn diagnostics

### Dump Tools
- Entity/component dump
- Prefab mapping dump
- Localization table dump
- Research tree dump
- Wave table dump

### Validation Reports
- Pack validity
- Unresolved references
- Duplicate IDs
- Incompatible overrides
- Asset holes
- Unsupported game version

## Consequences

- Every registry operation is logged
- Every pack load emits a structured report
- Every validation failure is machine-parseable
- Debug overlay can be toggled at runtime
- Dump tools can be invoked by agents to understand game state
- Agents can diagnose failures without reading source code
