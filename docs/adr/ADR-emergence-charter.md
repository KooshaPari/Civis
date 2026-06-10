# ADR: Civis Emergence Charter

**Status:** Accepted
**Date:** 2026-05-30

## Context

Civis is framed as an emergence-first simulation. The authoritative charter already states that the engine hardcodes only the environmental and physical substrate, while life, society, economy, polity, language, and technology are expected to arise from those rules. See:

- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md)
- [`docs/guides/voxel-emergent-vision-and-migration.md`](../guides/voxel-emergent-vision-and-migration.md)
- [`docs/specs/requirements/index.md`](../specs/requirements/index.md)

## Decision

Civis will hardcode only Layer-0 laws:

- physics / voxel material-fluid rules
- chemistry, energy, and material constraints
- climate / planet rules
- genomics primitives

Everything above Layer-0 must be represented as emergent state or derived annotation. In particular, species, psyches, ideologies, cultures, markets, polities, roads, and engineering systems are not to be authored as fixed enums or scripted outcomes.

## Consequences

- The simulation stays open-ended and consistent with the project’s emergence charter.
- New systems must model constraints and mechanisms, not outcomes.
- Hardcoded taxonomies become a design smell and should be replaced by runtime discovery or clustering when possible.
- Specs and requirements need to distinguish authored substrate from emergent phenomena explicitly.

