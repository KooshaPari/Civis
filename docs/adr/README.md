# Architecture Decision Records (ADRs)

This directory contains every Architecture Decision Record (ADR) for Civis.
ADRs are numbered, immutable Markdown files that capture significant
architectural choices: context, decision, consequences, and status.

The numbering convention follows Michael Nygard's ADR pattern as recorded in
[`0001-record-architecture-decisions.md`](0001-record-architecture-decisions.md).
Statuses used in this directory: **Accepted**, **Proposed**, **Superseded**.

## Index

Numbered ADRs (in numeric order):

| #     | Title                                                                                                       | Status            | File                                                                                       |
|-------|-------------------------------------------------------------------------------------------------------------|-------------------|--------------------------------------------------------------------------------------------|
| 0001  | Record Architecture Decisions                                                                               | Accepted          | [0001-record-architecture-decisions.md](0001-record-architecture-decisions.md)             |
| 001   | Rust Crate Structure — Workspace with Focused Modules                                                       | Accepted          | [ADR-001-rust-crate-structure.md](ADR-001-rust-crate-structure.md)                         |
| 002   | Joule Economy as Pluggable Resource Allocator                                                               | Accepted          | [ADR-002-joule-economy-as-allocator.md](ADR-002-joule-economy-as-allocator.md)             |
| 003   | Deterministic Scenario Replay — Mandatory for All Simulation Runs                                           | **Superseded** by [ADR-determinism-dropped](ADR-determinism-dropped.md) | [ADR-003-deterministic-replay.md](ADR-003-deterministic-replay.md)                         |
| 004   | *reserved / vacant* — historically referenced by ADR-005/006/007/008 as the "deterministic replay" ADR; that decision now lives in [ADR-003](ADR-003-deterministic-replay.md) (Superseded) and the active position in [ADR-determinism-dropped](ADR-determinism-dropped.md) | reserved / vacant | —                                                                                          |
| 005   | Adaptive Hybrid Voxel Substrate with Deterministic Mesh-Dirty Queue                                         | Proposed          | [ADR-005-adaptive-voxel.md](ADR-005-adaptive-voxel.md)                                     |
| 006   | LLM Outputs as Hash-Keyed Cached Events for Replay-Safe Hybrid Progression                                  | Proposed          | [ADR-006-llm-event-sourcing.md](ADR-006-llm-event-sourcing.md)                             |
| 007   | Three Reference 3D Clients (Bevy + Godot + Unreal) in Parallel                                              | Proposed          | [ADR-007-three-renderers.md](ADR-007-three-renderers.md)                                   |
| 008   | Genetics, Mutation, and Speciation Are Algorithmic (No LLM)                                                 | Proposed          | [ADR-008-algorithmic-genetics.md](ADR-008-algorithmic-genetics.md)                         |
| 009   | Web Client Strategy — Spectator-First (Not a Fourth Game Engine)                                            | Accepted (amended by [ADR-017](ADR-017-web-l2-authoring-amendment.md)) | [ADR-009-web-client-strategy.md](ADR-009-web-client-strategy.md)                           |
| 010   | CA Tick Budget Guard                                                                                        | Accepted          | [ADR-010-ca-tick-budget-guard.md](ADR-010-ca-tick-budget-guard.md)                         |
| 011   | N-Series Emergence Coupling Architecture                                                                    | Accepted          | [ADR-011-n-series-emergence-coupling.md](ADR-011-n-series-emergence-coupling.md)           |
| 012   | Keycap Palette Design System                                                                                | Accepted          | [ADR-012-keycap-palette-design-system.md](ADR-012-keycap-palette-design-system.md)         |
| 013   | Hexagonal Architecture + Wire Protocol                                                                      | Accepted          | [ADR-013-hexagonal-architecture.md](ADR-013-hexagonal-architecture.md)                     |
| 014   | Language Emergence via Phoneme Drift                                                                        | Accepted          | [ADR-014-language-emergence-via-phoneme-drift.md](ADR-014-language-emergence-via-phoneme-drift.md) |
| 015   | Faction Emergence via k-means Ideology Clustering                                                           | Accepted          | [ADR-015-faction-emergence-via-k-means-ideology-clustering.md](ADR-015-faction-emergence-via-k-means-ideology-clustering.md) |
| 016   | Religion Emergence from Needs Vector                                                                        | Accepted          | [ADR-016-religion-emergence-from-needs-vector.md](ADR-016-religion-emergence-from-needs-vector.md) |
| 017   | Web L2 Authoring (Amendment to ADR-009)                                                                     | Accepted          | [ADR-017-web-l2-authoring-amendment.md](ADR-017-web-l2-authoring-amendment.md)             |

Non-numeric ADRs (named; kept under their original filenames on purpose — see
notes below the table):

| Slug                        | Title                              | Status   | File                                                                                |
|-----------------------------|------------------------------------|----------|-------------------------------------------------------------------------------------|
| `ADR-emergence-charter`     | Civis Emergence Charter            | Accepted | [ADR-emergence-charter.md](ADR-emergence-charter.md)                                |
| `ADR-bevy-vulkan-primary-backend` | Bevy Vulkan Primary Backend  | Accepted | [ADR-bevy-vulkan-primary-backend.md](ADR-bevy-vulkan-primary-backend.md)            |
| `ADR-voxel-streaming-scale` | Voxel Streaming Scale Target       | Accepted | [ADR-voxel-streaming-scale.md](ADR-voxel-streaming-scale.md)                        |
| `ADR-determinism-dropped`   | Determinism Requirement Dropped    | **Accepted** (supersedes [ADR-003](ADR-003-deterministic-replay.md)) | [ADR-determinism-dropped.md](ADR-determinism-dropped.md) |

## Numeric gaps and renumber history

- **ADR-004 is reserved / vacant.** It is historically referenced by
  ADR-005 / ADR-006 / ADR-007 / ADR-008 as the "deterministic replay" ADR. The
  decision they reference now lives under ADR-003
  ([ADR-003-deterministic-replay.md](ADR-003-deterministic-replay.md), status
  **Superseded**) with the active position recorded in
  [ADR-determinism-dropped.md](ADR-determinism-dropped.md) (status
  **Accepted**). The ADR-004 slot is left vacant rather than backfilled so
  existing cross-references in ADR-005..008 stay readable; do not introduce a
  new ADR-004 without first renumbering those references.
- **ADR-009 collision (resolved).** Two files originally shared the `ADR-009-`
  filename prefix:
  `ADR-009-web-client-strategy.md` (the canonical ADR-009) and
  `ADR-009-amendment-web-l2-authoring.md` (an amendment). The amendment has
  been **renumbered to ADR-017** as
  [ADR-017-web-l2-authoring-amendment.md](ADR-017-web-l2-authoring-amendment.md).
  ADR-009 now unambiguously refers to the web client strategy.

## Non-numeric filename note

Some ADRs (`ADR-emergence-charter.md`, `ADR-bevy-vulkan-primary-backend.md`,
`ADR-voxel-streaming-scale.md`, `ADR-determinism-dropped.md`) intentionally
keep their original non-numeric filenames. They were proposed under those
names and reviewed under those names; renaming them to numeric ADR numbers
would have orphaned external links. They are full first-class ADRs and
participate in supersedes/links as the table above shows.

The only exception to "keep original filename" would be if a future pass
allocates them proper numbers and updates all in-repo references in the same
commit — that is left as a deliberate follow-up so this pass stays focused
on recovery and the ADR-009 collision.

## Supersedes graph

- [ADR-determinism-dropped](ADR-determinism-dropped.md) **supersedes**
  [ADR-003-deterministic-replay](ADR-003-deterministic-replay.md).
- [ADR-017-web-l2-authoring-amendment](ADR-017-web-l2-authoring-amendment.md)
  **amends** the default in
  [ADR-009-web-client-strategy](ADR-009-web-client-strategy.md).
- [ADR-010-ca-tick-budget-guard](ADR-010-ca-tick-budget-guard.md) is the
  Phase 1 safety ceiling referenced by
  [ADR-011-n-series-emergence-coupling](ADR-011-n-series-emergence-coupling.md)
  as the upgrade path for N10+ couplings.
- [ADR-005-adaptive-voxel](ADR-005-adaptive-voxel.md),
  [ADR-006-llm-event-sourcing](ADR-006-llm-event-sourcing.md),
  [ADR-007-three-renderers](ADR-007-three-renderers.md), and
  [ADR-008-algorithmic-genetics](ADR-008-algorithmic-genetics.md) reference
  ADR-004 ("deterministic replay") — that ADR is vacant; their effective
  determinism contract is now ADR-003 (Superseded) / ADR-determinism-dropped
  (Accepted).
- [ADR-voxel-streaming-scale](ADR-voxel-streaming-scale.md) cross-references
  [ADR-005-adaptive-voxel](ADR-005-adaptive-voxel.md).
- [ADR-bevy-vulkan-primary-backend](ADR-bevy-vulkan-primary-backend.md) and
  [ADR-voxel-streaming-scale](ADR-voxel-streaming-scale.md) both anchor the
  Bevy/Vulkan and voxel streaming rows of the 3D FR matrix.
- [ADR-emergence-charter](ADR-emergence-charter.md) is the umbrella decision
  that ADR-011, ADR-014, ADR-015, and ADR-016 implement.

## Recovery / renumber pass (2026-06)

This README and the underlying ADR set were repaired in a single docs-only
pass (`docs/adr-recovery-renumber` branch, PR
"docs(adr): recover deleted ADRs 010-013 + fix 009 collision + determinism
supersede"):

1. **Restored** ADR-010, ADR-011, ADR-012, ADR-013 from git history (they had
   been deleted by a botched renumber).
2. **Resolved** the ADR-003 / ADR-determinism-dropped contradiction by
   marking ADR-003 as *Superseded by ADR-determinism-dropped* and giving
   ADR-determinism-dropped status *Accepted* with explicit
   Context/Decision/Consequences. Both files are kept so the historical
   decision trail is preserved.
3. **Renumbered** the duplicate `ADR-009-amendment-web-l2-authoring.md` to
   `ADR-017-web-l2-authoring-amendment.md` (using `git mv`) and updated its
   in-file header. ADR-009 now unambiguously refers to the web client
   strategy.
4. **Rebuilt** this README as the canonical index of every ADR in
   `docs/adr/`.

No code changes; this is docs-only.
