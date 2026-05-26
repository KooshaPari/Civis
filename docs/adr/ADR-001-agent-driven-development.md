# ADR-001: Agent-Driven Development Model

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

DINOForge is built entirely through agent-driven development. No human reads or reviews code directly. The human acts as product owner, tester, design director, and failure reporter.

This means the codebase cannot depend on "smart coding" or human intuition. It must depend on rigid structure, typed interfaces, machine-checkable contracts, and strong observability.

## Decision

All development follows an agent-first model:

1. **Architect agents** define contracts, registries, schemas, extension points
2. **Runtime agents** handle low-level ECS/hook work
3. **Pack agents** create content packs from schemas
4. **Validation agents** run tests, static checks, pack compilation, compatibility
5. **Diagnosis agents** interpret logs/crashes/failures
6. **Governance agents** keep abstractions clean, kill duplication

### Agent Constraints

Agents MUST:
- Work through manifests
- Use generators/templates
- Update docs/contracts
- Add tests for new public surfaces
- Log failure modes
- Keep features pack-based when possible

Agents MUST NOT:
- Patch runtime internals unless assigned runtime-layer work
- Invent new registry patterns casually
- Duplicate schemas
- Bypass validators
- Hardcode content IDs in engine glue
- Add undocumented extension points
- Skip tests

### Legal Move Classes

All agent work must reduce to one of:
- create schema
- extend registry
- add content pack
- patch mapping
- write validator
- add test fixture
- add debug view
- add migration
- add compatibility rule
- add documentation manifest

## Consequences

- Every module needs strict ownership boundaries, public interfaces, generated docs, examples, invariants, self-tests, fixtures, expected logs, failure modes, rollback behavior
- Every folder needs purpose, allowed imports, patterns agents may/may not use
- Every feature needs design manifest, schema, sample usage, tests, observability hooks
- The CLAUDE.md file serves as the primary agent governance document
