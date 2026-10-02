# FR Traceability Index

Index of 1362 per-ID documentation directories under `docs/traceability/`, grouped by epic. 1362 have at least one artefact on disk. Generated 2026-10-02 by `scripts/traceability/gen-traceability-index.py`.

## Coverage status is deliberately not in this file

This file used to carry a hand-maintained `Status` column with values including `COVERED` and `CODE-ONLY-no-spec`. It had no generator, and it disagreed with the authoritative matrix on 435 of the 1229 rows the two shared. Spot-checking those disagreements found the matrix correct each time: `FR-AI-001` was listed `SPEC-ONLY` here while it is defined at `docs/FR.md:43`, implemented at `crates/ai/src/decision.rs:3`, and tested at `crates/ai/tests/fr_fr_ai_001.rs:1`.

Authoritative coverage is [`docs/audits/fr-matrix.json`](../audits/fr-matrix.json), derived from evidence by `scripts/traceability/check-fr-coverage.py`, which fails the build on a coverage regression.

## Artifacts

`Artifacts` counts how many of the five documentation files exist for an ID: `spec`, `intent`, `plan`, `research`, `adr`.

This is a **documentation** signal, not a coverage signal. An ID with all five artefacts can still be `SPEC-ONLY`, because a specification existing is not the behaviour being implemented.

2 of these IDs have no row in the current matrix. That is either a documentation directory with no requirement behind it, or a requirement the ID regex does not match; both are tracked in `docs/audits/digitless-id-findings-2026-10-02.md`.

## FR-AI (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-AI-001 | 5/5 | yes | yes | yes | [dir](fr-ai-001/) |
| FR-AI-002 | 5/5 | yes | yes | yes | [dir](fr-ai-002/) |
| FR-AI-003 | 5/5 | yes | yes | yes | [dir](fr-ai-003/) |
| FR-AI-004 | 5/5 | yes | yes | yes | [dir](fr-ai-004/) |
| FR-AI-005 | 5/5 | yes | yes | yes | [dir](fr-ai-005/) |
| FR-AI-006 | 5/5 | yes | yes | yes | [dir](fr-ai-006/) |
| FR-AI-007 | 5/5 | yes | yes | yes | [dir](fr-ai-007/) |

## FR-API (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-API-001 | 5/5 | yes | yes | yes | [dir](fr-api-001/) |
| FR-API-002 | 5/5 | yes | yes | yes | [dir](fr-api-002/) |
| FR-API-003 | 5/5 | yes | yes | yes | [dir](fr-api-003/) |
| FR-API-004 | 5/5 | yes | yes | yes | [dir](fr-api-004/) |

## FR-ASSET (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-ASSET-001 | 5/5 | yes | yes | yes | [dir](fr-asset-001/) |
| FR-ASSET-002 | 5/5 | yes | yes | yes | [dir](fr-asset-002/) |
| FR-ASSET-003 | 5/5 | yes | yes | yes | [dir](fr-asset-003/) |
| FR-ASSET-004 | 5/5 | yes | yes | yes | [dir](fr-asset-004/) |

## FR-ASSET-PIPELINE (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-ASSET-PIPELINE-001 | 1/5 | no | no | no | [dir](fr-asset-pipeline-001/) |
| FR-ASSET-PIPELINE-002 | 1/5 | no | no | no | [dir](fr-asset-pipeline-002/) |

## FR-AUD (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-AUD-001 | 5/5 | yes | yes | yes | [dir](fr-aud-001/) |
| FR-AUD-002 | 5/5 | yes | yes | yes | [dir](fr-aud-002/) |
| FR-AUD-003 | 5/5 | yes | yes | yes | [dir](fr-aud-003/) |

## FR-CIV (12)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-0001 | 5/5 | yes | yes | yes | [dir](fr-civ-0001/) |
| FR-CIV-0104-001 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-001/) |
| FR-CIV-0104-002 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-002/) |
| FR-CIV-0104-003 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-003/) |
| FR-CIV-0104-004 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-004/) |
| FR-CIV-0104-005 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-005/) |
| FR-CIV-0104-006 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-006/) |
| FR-CIV-0104-007 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-007/) |
| FR-CIV-0104-008 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-008/) |
| FR-CIV-0104-009 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-009/) |
| FR-CIV-0104-010 | 5/5 | yes | yes | yes | [dir](fr-civ-0104-010/) |
| FR-CIV-014 | 1/5 | no | no | no | [dir](fr-civ-014/) |

## FR-CIV-0001-TICK (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-0001-TICK | 5/5 | yes | yes | yes | [dir](fr-civ-0001-tick/) |

## FR-CIV-3D (15)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-3D-001 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-001/) |
| FR-CIV-3D-002 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-002/) |
| FR-CIV-3D-003 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-003/) |
| FR-CIV-3D-004 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-004/) |
| FR-CIV-3D-005 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-005/) |
| FR-CIV-3D-006 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-006/) |
| FR-CIV-3D-007 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-007/) |
| FR-CIV-3D-008 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-008/) |
| FR-CIV-3D-009 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-009/) |
| FR-CIV-3D-010 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-010/) |
| FR-CIV-3D-011 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-011/) |
| FR-CIV-3D-012 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-012/) |
| FR-CIV-3D-013 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-013/) |
| FR-CIV-3D-014 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-014/) |
| FR-CIV-3D-015 | 5/5 | yes | yes | yes | [dir](fr-civ-3d-015/) |

## FR-CIV-ACT (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ACT-001 | 5/5 | yes | yes | yes | [dir](fr-civ-act-001/) |
| FR-CIV-ACT-003 | 5/5 | yes | yes | yes | [dir](fr-civ-act-003/) |
| FR-CIV-ACT-004 | 5/5 | yes | yes | yes | [dir](fr-civ-act-004/) |
| FR-CIV-ACT-005 | 5/5 | yes | yes | yes | [dir](fr-civ-act-005/) |

## FR-CIV-ACTOR (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ACTOR-001 | 5/5 | yes | yes | yes | [dir](fr-civ-actor-001/) |
| FR-CIV-ACTOR-002 | 5/5 | yes | yes | yes | [dir](fr-civ-actor-002/) |

## FR-CIV-ACTOR-001-LIFECYCLE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ACTOR-001-LIFECYCLE | 5/5 | yes | yes | yes | [dir](fr-civ-actor-001-lifecycle/) |

## FR-CIV-AGENTS (17)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-AGENTS-000 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-000/) |
| FR-CIV-AGENTS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-001/) |
| FR-CIV-AGENTS-002 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-002/) |
| FR-CIV-AGENTS-003 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-003/) |
| FR-CIV-AGENTS-010 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-010/) |
| FR-CIV-AGENTS-011 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-011/) |
| FR-CIV-AGENTS-020 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-020/) |
| FR-CIV-AGENTS-021 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-021/) |
| FR-CIV-AGENTS-022 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-022/) |
| FR-CIV-AGENTS-023 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-023/) |
| FR-CIV-AGENTS-024 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-024/) |
| FR-CIV-AGENTS-025 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-025/) |
| FR-CIV-AGENTS-030 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-030/) |
| FR-CIV-AGENTS-031 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-031/) |
| FR-CIV-AGENTS-032 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-032/) |
| FR-CIV-AGENTS-033 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-033/) |
| FR-CIV-AGENTS-034 | 5/5 | yes | yes | yes | [dir](fr-civ-agents-034/) |

## FR-CIV-AGGRESSION (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-AGGRESSION-001 | 1/5 | no | no | no | [dir](fr-civ-aggression-001/) |

## FR-CIV-AI (15)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-AI-001 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-001/) |
| FR-CIV-AI-002 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-002/) |
| FR-CIV-AI-003 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-003/) |
| FR-CIV-AI-004 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-004/) |
| FR-CIV-AI-005 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-005/) |
| FR-CIV-AI-006 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-006/) |
| FR-CIV-AI-007 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-007/) |
| FR-CIV-AI-008 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-008/) |
| FR-CIV-AI-009 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-009/) |
| FR-CIV-AI-010 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-010/) |
| FR-CIV-AI-011 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-011/) |
| FR-CIV-AI-012 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-012/) |
| FR-CIV-AI-013 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-013/) |
| FR-CIV-AI-014 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-014/) |
| FR-CIV-AI-015 | 5/5 | yes | yes | yes | [dir](fr-civ-ai-015/) |

## FR-CIV-ARCH (9)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ARCH-00 | 1/5 | no | no | no | [dir](fr-civ-arch-00/) |
| FR-CIV-ARCH-001 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-001/) |
| FR-CIV-ARCH-002 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-002/) |
| FR-CIV-ARCH-003 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-003/) |
| FR-CIV-ARCH-004 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-004/) |
| FR-CIV-ARCH-005 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-005/) |
| FR-CIV-ARCH-006 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-006/) |
| FR-CIV-ARCH-007 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-007/) |
| FR-CIV-ARCH-008 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-008/) |

## FR-CIV-ARCH-A (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ARCH-A-001 | 1/5 | no | no | no | [dir](fr-civ-arch-a-001/) |
| FR-CIV-ARCH-A-002 | 1/5 | no | no | no | [dir](fr-civ-arch-a-002/) |
| FR-CIV-ARCH-A-003 | 1/5 | no | no | no | [dir](fr-civ-arch-a-003/) |

## FR-CIV-ARCH-B (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ARCH-B-001 | 1/5 | no | no | no | [dir](fr-civ-arch-b-001/) |
| FR-CIV-ARCH-B-002 | 1/5 | no | no | no | [dir](fr-civ-arch-b-002/) |
| FR-CIV-ARCH-B-003 | 1/5 | no | no | no | [dir](fr-civ-arch-b-003/) |
| FR-CIV-ARCH-B-004 | 1/5 | no | no | no | [dir](fr-civ-arch-b-004/) |

## FR-CIV-ARCH-C (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ARCH-C-001 | 1/5 | no | no | no | [dir](fr-civ-arch-c-001/) |
| FR-CIV-ARCH-C-002 | 1/5 | no | no | no | [dir](fr-civ-arch-c-002/) |
| FR-CIV-ARCH-C-003 | 1/5 | no | no | no | [dir](fr-civ-arch-c-003/) |
| FR-CIV-ARCH-C-004 | 1/5 | no | no | no | [dir](fr-civ-arch-c-004/) |

## FR-CIV-ARCH-D (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ARCH-D-001 | 1/5 | no | no | no | [dir](fr-civ-arch-d-001/) |
| FR-CIV-ARCH-D-002 | 1/5 | no | no | no | [dir](fr-civ-arch-d-002/) |
| FR-CIV-ARCH-D-003 | 1/5 | no | no | no | [dir](fr-civ-arch-d-003/) |
| FR-CIV-ARCH-D-004 | 1/5 | no | no | no | [dir](fr-civ-arch-d-004/) |

## FR-CIV-ARCH-NOSVG (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ARCH-NOSVG-001 | 5/5 | yes | yes | yes | [dir](fr-civ-arch-nosvg-001/) |

## FR-CIV-ASSET (20)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ASSET-001 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-001/) |
| FR-CIV-ASSET-002 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-002/) |
| FR-CIV-ASSET-003 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-003/) |
| FR-CIV-ASSET-004 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-004/) |
| FR-CIV-ASSET-005 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-005/) |
| FR-CIV-ASSET-006 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-006/) |
| FR-CIV-ASSET-007 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-007/) |
| FR-CIV-ASSET-008 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-008/) |
| FR-CIV-ASSET-009 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-009/) |
| FR-CIV-ASSET-010 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-010/) |
| FR-CIV-ASSET-011 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-011/) |
| FR-CIV-ASSET-012 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-012/) |
| FR-CIV-ASSET-013 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-013/) |
| FR-CIV-ASSET-014 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-014/) |
| FR-CIV-ASSET-015 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-015/) |
| FR-CIV-ASSET-016 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-016/) |
| FR-CIV-ASSET-017 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-017/) |
| FR-CIV-ASSET-018 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-018/) |
| FR-CIV-ASSET-019 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-019/) |
| FR-CIV-ASSET-020 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-020/) |

## FR-CIV-ASSET-MANI (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ASSET-MANI-001 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-mani-001/) |
| FR-CIV-ASSET-MANI-002 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-mani-002/) |

## FR-CIV-ASSET-QUAL (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ASSET-QUAL-001 | 5/5 | yes | yes | yes | [dir](fr-civ-asset-qual-001/) |

## FR-CIV-AUDIO (12)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-AUDIO-001 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-001/) |
| FR-CIV-AUDIO-002 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-002/) |
| FR-CIV-AUDIO-003 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-003/) |
| FR-CIV-AUDIO-004 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-004/) |
| FR-CIV-AUDIO-005 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-005/) |
| FR-CIV-AUDIO-006 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-006/) |
| FR-CIV-AUDIO-007 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-007/) |
| FR-CIV-AUDIO-008 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-008/) |
| FR-CIV-AUDIO-009 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-009/) |
| FR-CIV-AUDIO-010 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-010/) |
| FR-CIV-AUDIO-011 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-011/) |
| FR-CIV-AUDIO-012 | 5/5 | yes | yes | yes | [dir](fr-civ-audio-012/) |

## FR-CIV-BELIEF (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-BELIEF-001 | 1/5 | no | no | no | [dir](fr-civ-belief-001/) |

## FR-CIV-BEVY (21)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-BEVY-001 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-001/) |
| FR-CIV-BEVY-002 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-002/) |
| FR-CIV-BEVY-003 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-003/) |
| FR-CIV-BEVY-013 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-013/) |
| FR-CIV-BEVY-014 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-014/) |
| FR-CIV-BEVY-015 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-015/) |
| FR-CIV-BEVY-016 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-016/) |
| FR-CIV-BEVY-017 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-017/) |
| FR-CIV-BEVY-018 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-018/) |
| FR-CIV-BEVY-019 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-019/) |
| FR-CIV-BEVY-020 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-020/) |
| FR-CIV-BEVY-021 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-021/) |
| FR-CIV-BEVY-022 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-022/) |
| FR-CIV-BEVY-023 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-023/) |
| FR-CIV-BEVY-024 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-024/) |
| FR-CIV-BEVY-025 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-025/) |
| FR-CIV-BEVY-026 | 5/5 | yes | yes | yes | [dir](fr-civ-bevy-026/) |
| FR-CIV-BEVY-028 | 1/5 | no | no | no | [dir](fr-civ-bevy-028/) |
| FR-CIV-BEVY-034 | 1/5 | no | no | no | [dir](fr-civ-bevy-034/) |
| FR-CIV-BEVY-035 | 1/5 | no | no | no | [dir](fr-civ-bevy-035/) |
| FR-CIV-BEVY-036 | 1/5 | no | no | no | [dir](fr-civ-bevy-036/) |

## FR-CIV-BIO (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-BIO-001 | 5/5 | yes | yes | yes | [dir](fr-civ-bio-001/) |
| FR-CIV-BIO-002 | 5/5 | yes | yes | yes | [dir](fr-civ-bio-002/) |
| FR-CIV-BIO-003 | 5/5 | yes | yes | yes | [dir](fr-civ-bio-003/) |

## FR-CIV-BRUSH (13)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-BRUSH-01 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-01/) |
| FR-CIV-BRUSH-02 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-02/) |
| FR-CIV-BRUSH-03 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-03/) |
| FR-CIV-BRUSH-04 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-04/) |
| FR-CIV-BRUSH-05 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-05/) |
| FR-CIV-BRUSH-06 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-06/) |
| FR-CIV-BRUSH-07 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-07/) |
| FR-CIV-BRUSH-08 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-08/) |
| FR-CIV-BRUSH-09 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-09/) |
| FR-CIV-BRUSH-10 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-10/) |
| FR-CIV-BRUSH-11 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-11/) |
| FR-CIV-BRUSH-12 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-12/) |
| FR-CIV-BRUSH-13 | 5/5 | yes | yes | yes | [dir](fr-civ-brush-13/) |

## FR-CIV-BUILD (14)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-BUILD-000 | 5/5 | yes | yes | yes | [dir](fr-civ-build-000/) |
| FR-CIV-BUILD-001 | 5/5 | yes | yes | yes | [dir](fr-civ-build-001/) |
| FR-CIV-BUILD-002 | 5/5 | yes | yes | yes | [dir](fr-civ-build-002/) |
| FR-CIV-BUILD-003 | 5/5 | yes | yes | yes | [dir](fr-civ-build-003/) |
| FR-CIV-BUILD-004 | 5/5 | yes | yes | yes | [dir](fr-civ-build-004/) |
| FR-CIV-BUILD-005 | 5/5 | yes | yes | yes | [dir](fr-civ-build-005/) |
| FR-CIV-BUILD-010 | 5/5 | yes | yes | yes | [dir](fr-civ-build-010/) |
| FR-CIV-BUILD-011 | 5/5 | yes | yes | yes | [dir](fr-civ-build-011/) |
| FR-CIV-BUILD-012 | 5/5 | yes | yes | yes | [dir](fr-civ-build-012/) |
| FR-CIV-BUILD-013 | 5/5 | yes | yes | yes | [dir](fr-civ-build-013/) |
| FR-CIV-BUILD-014 | 5/5 | yes | yes | yes | [dir](fr-civ-build-014/) |
| FR-CIV-BUILD-015 | 5/5 | yes | yes | yes | [dir](fr-civ-build-015/) |
| FR-CIV-BUILD-020 | 5/5 | yes | yes | yes | [dir](fr-civ-build-020/) |
| FR-CIV-BUILD-030 | 5/5 | yes | yes | yes | [dir](fr-civ-build-030/) |

## FR-CIV-CA (11)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CA-001 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-001/) |
| FR-CIV-CA-002 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-002/) |
| FR-CIV-CA-003 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-003/) |
| FR-CIV-CA-004 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-004/) |
| FR-CIV-CA-005 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-005/) |
| FR-CIV-CA-006 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-006/) |
| FR-CIV-CA-007 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-007/) |
| FR-CIV-CA-008 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-008/) |
| FR-CIV-CA-009 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-009/) |
| FR-CIV-CA-010 | 5/5 | yes | yes | yes | [dir](fr-civ-ca-010/) |
| FR-CIV-CA-011 | 1/5 | no | no | no | [dir](fr-civ-ca-011/) |

## FR-CIV-CARAVAN (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CARAVAN-001 | 1/5 | no | no | no | [dir](fr-civ-caravan-001/) |

## FR-CIV-CLIENT (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CLIENT-006 | 1/5 | no | no | no | [dir](fr-civ-client-006/) |
| FR-CIV-CLIENT-011 | 1/5 | no | no | no | [dir](fr-civ-client-011/) |
| FR-CIV-CLIENT-013 | 1/5 | no | no | no | [dir](fr-civ-client-013/) |

## FR-CIV-CLIENT-GODOT (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CLIENT-GODOT-001 | 5/5 | yes | yes | yes | [dir](fr-civ-client-godot-001/) |
| FR-CIV-CLIENT-GODOT-002 | 5/5 | yes | yes | yes | [dir](fr-civ-client-godot-002/) |

## FR-CIV-CLIMATE (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CLIMATE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-climate-001/) |
| FR-CIV-CLIMATE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-climate-002/) |
| FR-CIV-CLIMATE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-climate-003/) |
| FR-CIV-CLIMATE-1 | 1/5 | no | no | no | [dir](fr-civ-climate-1/) |
| FR-CIV-CLIMATE-2 | 1/5 | no | no | no | [dir](fr-civ-climate-2/) |
| FR-CIV-CLIMATE-3 | 1/5 | no | no | no | [dir](fr-civ-climate-3/) |
| FR-CIV-CLIMATE-4 | 1/5 | no | no | no | [dir](fr-civ-climate-4/) |

## FR-CIV-COHESION (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-COHESION-001 | 1/5 | no | no | no | [dir](fr-civ-cohesion-001/) |

## FR-CIV-CONSTRUCTION (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CONSTRUCTION-001 | 1/5 | no | no | no | [dir](fr-civ-construction-001/) |

## FR-CIV-CONTENT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CONTENT-001 | 1/5 | no | no | no | [dir](fr-civ-content-001/) |

## FR-CIV-CORE (21)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CORE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-core-001/) |
| FR-CIV-CORE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-core-002/) |
| FR-CIV-CORE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-core-003/) |
| FR-CIV-CORE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-core-004/) |
| FR-CIV-CORE-005 | 5/5 | yes | yes | yes | [dir](fr-civ-core-005/) |
| FR-CIV-CORE-006 | 5/5 | yes | yes | yes | [dir](fr-civ-core-006/) |
| FR-CIV-CORE-007 | 5/5 | yes | yes | yes | [dir](fr-civ-core-007/) |
| FR-CIV-CORE-008 | 5/5 | yes | yes | yes | [dir](fr-civ-core-008/) |
| FR-CIV-CORE-009 | 5/5 | yes | yes | yes | [dir](fr-civ-core-009/) |
| FR-CIV-CORE-010 | 5/5 | yes | yes | yes | [dir](fr-civ-core-010/) |
| FR-CIV-CORE-011 | 5/5 | yes | yes | yes | [dir](fr-civ-core-011/) |
| FR-CIV-CORE-012 | 5/5 | yes | yes | yes | [dir](fr-civ-core-012/) |
| FR-CIV-CORE-013 | 5/5 | yes | yes | yes | [dir](fr-civ-core-013/) |
| FR-CIV-CORE-014 | 5/5 | yes | yes | yes | [dir](fr-civ-core-014/) |
| FR-CIV-CORE-015 | 5/5 | yes | yes | yes | [dir](fr-civ-core-015/) |
| FR-CIV-CORE-016 | 5/5 | yes | yes | yes | [dir](fr-civ-core-016/) |
| FR-CIV-CORE-017 | 5/5 | yes | yes | yes | [dir](fr-civ-core-017/) |
| FR-CIV-CORE-018 | 5/5 | yes | yes | yes | [dir](fr-civ-core-018/) |
| FR-CIV-CORE-019 | 5/5 | yes | yes | yes | [dir](fr-civ-core-019/) |
| FR-CIV-CORE-020 | 5/5 | yes | yes | yes | [dir](fr-civ-core-020/) |
| FR-CIV-CORE-021 | 1/5 | no | no | no | [dir](fr-civ-core-021/) |

## FR-CIV-CORE-DET (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CORE-DET-001 | 5/5 | yes | yes | yes | [dir](fr-civ-core-det-001/) |
| FR-CIV-CORE-DET-002 | 5/5 | yes | yes | yes | [dir](fr-civ-core-det-002/) |
| FR-CIV-CORE-DET-003 | 5/5 | yes | yes | yes | [dir](fr-civ-core-det-003/) |

## FR-CIV-CULT (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CULT-001 | 5/5 | yes | yes | yes | [dir](fr-civ-cult-001/) |
| FR-CIV-CULT-002 | 5/5 | yes | yes | yes | [dir](fr-civ-cult-002/) |
| FR-CIV-CULT-003 | 5/5 | yes | yes | yes | [dir](fr-civ-cult-003/) |

## FR-CIV-CULTURE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-CULTURE-001 | 1/5 | no | no | no | [dir](fr-civ-culture-001/) |

## FR-CIV-DET (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-DET-001 | 5/5 | yes | yes | yes | [dir](fr-civ-det-001/) |
| FR-CIV-DET-002 | 1/5 | no | no | no | [dir](fr-civ-det-002/) |
| FR-CIV-DET-003 | 1/5 | no | no | no | [dir](fr-civ-det-003/) |
| FR-CIV-DET-004 | 1/5 | no | no | no | [dir](fr-civ-det-004/) |
| FR-CIV-DET-005 | 1/5 | no | no | no | [dir](fr-civ-det-005/) |
| FR-CIV-DET-006 | 1/5 | no | no | no | [dir](fr-civ-det-006/) |
| FR-CIV-DET-007 | 1/5 | no | no | no | [dir](fr-civ-det-007/) |

## FR-CIV-DIFFUSION (16)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-DIFFUSION-000 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-000/) |
| FR-CIV-DIFFUSION-001 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-001/) |
| FR-CIV-DIFFUSION-002 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-002/) |
| FR-CIV-DIFFUSION-003 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-003/) |
| FR-CIV-DIFFUSION-004 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-004/) |
| FR-CIV-DIFFUSION-005 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-005/) |
| FR-CIV-DIFFUSION-006 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-006/) |
| FR-CIV-DIFFUSION-007 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-007/) |
| FR-CIV-DIFFUSION-008 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-008/) |
| FR-CIV-DIFFUSION-009 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-009/) |
| FR-CIV-DIFFUSION-010 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-010/) |
| FR-CIV-DIFFUSION-011 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-011/) |
| FR-CIV-DIFFUSION-012 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-012/) |
| FR-CIV-DIFFUSION-013 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-013/) |
| FR-CIV-DIFFUSION-014 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-014/) |
| FR-CIV-DIFFUSION-015 | 5/5 | yes | yes | yes | [dir](fr-civ-diffusion-015/) |

## FR-CIV-DIPLO (16)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-DIPLO-001 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-001/) |
| FR-CIV-DIPLO-002 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-002/) |
| FR-CIV-DIPLO-003 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-003/) |
| FR-CIV-DIPLO-003-006 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-006/) |
| FR-CIV-DIPLO-003-01 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-01/) |
| FR-CIV-DIPLO-003-02 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-02/) |
| FR-CIV-DIPLO-003-03 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-03/) |
| FR-CIV-DIPLO-003-04 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-04/) |
| FR-CIV-DIPLO-003-05 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-05/) |
| FR-CIV-DIPLO-003-06 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-06/) |
| FR-CIV-DIPLO-003-07 | 1/5 | no | no | no | [dir](fr-civ-diplo-003-07/) |
| FR-CIV-DIPLO-004 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-004/) |
| FR-CIV-DIPLO-005 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-005/) |
| FR-CIV-DIPLO-006 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-006/) |
| FR-CIV-DIPLO-007 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-007/) |
| FR-CIV-DIPLO-008 | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-008/) |

## FR-CIV-DIPLO-001-RELATIONS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-DIPLO-001-RELATIONS | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-001-relations/) |

## FR-CIV-DIPLO-002-SHADOW (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-DIPLO-002-SHADOW | 5/5 | yes | yes | yes | [dir](fr-civ-diplo-002-shadow/) |

## FR-CIV-DIPLOMACY (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-DIPLOMACY-001 | 1/5 | no | no | no | [dir](fr-civ-diplomacy-001/) |
| FR-CIV-DIPLOMACY-004 | 1/5 | no | no | no | [dir](fr-civ-diplomacy-004/) |

## FR-CIV-ECON (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ECON-001 | 5/5 | yes | yes | yes | [dir](fr-civ-econ-001/) |
| FR-CIV-ECON-002 | 5/5 | yes | yes | yes | [dir](fr-civ-econ-002/) |
| FR-CIV-ECON-003 | 5/5 | yes | yes | yes | [dir](fr-civ-econ-003/) |
| FR-CIV-ECON-004 | 5/5 | yes | yes | yes | [dir](fr-civ-econ-004/) |
| FR-CIV-ECON-010 | 1/5 | no | no | no | [dir](fr-civ-econ-010/) |
| FR-CIV-ECON-015 | 5/5 | yes | yes | yes | [dir](fr-civ-econ-015/) |

## FR-CIV-ECON-001-MARKET (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ECON-001-MARKET | 5/5 | yes | yes | yes | [dir](fr-civ-econ-001-market/) |

## FR-CIV-ECON-002-JOULE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ECON-002-JOULE | 5/5 | yes | yes | yes | [dir](fr-civ-econ-002-joule/) |

## FR-CIV-ECON-FOCUS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ECON-FOCUS-001 | 1/5 | no | no | no | [dir](fr-civ-econ-focus-001/) |

## FR-CIV-EMERG (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-EMERG-001 | 5/5 | yes | yes | yes | [dir](fr-civ-emerg-001/) |
| FR-CIV-EMERG-002 | 5/5 | yes | yes | yes | [dir](fr-civ-emerg-002/) |
| FR-CIV-EMERG-003 | 5/5 | yes | yes | yes | [dir](fr-civ-emerg-003/) |
| FR-CIV-EMERG-004 | 5/5 | yes | yes | yes | [dir](fr-civ-emerg-004/) |
| FR-CIV-EMERG-005 | 5/5 | yes | yes | yes | [dir](fr-civ-emerg-005/) |

## FR-CIV-EMERGE-DASH (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-EMERGE-DASH-001 | 1/5 | no | no | no | [dir](fr-civ-emerge-dash-001/) |

## FR-CIV-EMERGENCE (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-EMERGENCE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-001/) |
| FR-CIV-EMERGENCE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-002/) |
| FR-CIV-EMERGENCE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-003/) |
| FR-CIV-EMERGENCE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-004/) |
| FR-CIV-EMERGENCE-005 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-005/) |
| FR-CIV-EMERGENCE-006 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-006/) |
| FR-CIV-EMERGENCE-010 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-010/) |
| FR-CIV-EMERGENCE-011 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-011/) |
| FR-CIV-EMERGENCE-012 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-012/) |
| FR-CIV-EMERGENCE-013 | 5/5 | yes | yes | yes | [dir](fr-civ-emergence-013/) |

## FR-CIV-EMERGENT-MIGRATION (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-EMERGENT-MIGRATION-001 | 1/5 | no | no | no | [dir](fr-civ-emergent-migration-001/) |

## FR-CIV-ENGINE-INT (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ENGINE-INT-001 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-001/) |
| FR-CIV-ENGINE-INT-002 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-002/) |
| FR-CIV-ENGINE-INT-003 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-003/) |
| FR-CIV-ENGINE-INT-005 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-005/) |
| FR-CIV-ENGINE-INT-010 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-010/) |
| FR-CIV-ENGINE-INT-011 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-011/) |
| FR-CIV-ENGINE-INT-012 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-012/) |
| FR-CIV-ENGINE-INT-013 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-013/) |
| FR-CIV-ENGINE-INT-014 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-014/) |
| FR-CIV-ENGINE-INT-015 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-int-015/) |

## FR-CIV-ENGINE-REPLAY (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ENGINE-REPLAY-001 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-replay-001/) |
| FR-CIV-ENGINE-REPLAY-002 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-replay-002/) |
| FR-CIV-ENGINE-REPLAY-003 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-replay-003/) |
| FR-CIV-ENGINE-REPLAY-004 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-replay-004/) |
| FR-CIV-ENGINE-REPLAY-005 | 5/5 | yes | yes | yes | [dir](fr-civ-engine-replay-005/) |

## FR-CIV-ERA (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ERA-001 | 1/5 | no | no | no | [dir](fr-civ-era-001/) |

## FR-CIV-FAMINE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-FAMINE-001 | 1/5 | no | no | no | [dir](fr-civ-famine-001/) |

## FR-CIV-FEST (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-FEST-001 | 1/5 | no | no | no | [dir](fr-civ-fest-001/) |

## FR-CIV-FOG (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-FOG-001 | 5/5 | yes | yes | yes | [dir](fr-civ-fog-001/) |
| FR-CIV-FOG-002 | 5/5 | yes | yes | yes | [dir](fr-civ-fog-002/) |
| FR-CIV-FOG-003 | 5/5 | yes | yes | yes | [dir](fr-civ-fog-003/) |
| FR-CIV-FOG-004 | 5/5 | yes | yes | yes | [dir](fr-civ-fog-004/) |
| FR-CIV-FOG-005 | 5/5 | yes | yes | yes | [dir](fr-civ-fog-005/) |

## FR-CIV-GAME (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GAME-001 | 1/5 | no | no | no | [dir](fr-civ-game-001/) |
| FR-CIV-GAME-002 | 1/5 | no | no | no | [dir](fr-civ-game-002/) |
| FR-CIV-GAME-003 | 1/5 | no | no | no | [dir](fr-civ-game-003/) |

## FR-CIV-GENETICS (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GENETICS-000 | 5/5 | yes | yes | yes | [dir](fr-civ-genetics-000/) |
| FR-CIV-GENETICS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-genetics-001/) |
| FR-CIV-GENETICS-002 | 5/5 | yes | yes | yes | [dir](fr-civ-genetics-002/) |
| FR-CIV-GENETICS-010 | 5/5 | yes | yes | yes | [dir](fr-civ-genetics-010/) |
| FR-CIV-GENETICS-011 | 5/5 | yes | yes | yes | [dir](fr-civ-genetics-011/) |
| FR-CIV-GENETICS-012 | 5/5 | yes | yes | yes | [dir](fr-civ-genetics-012/) |

## FR-CIV-GENETICS-SEED (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GENETICS-SEED-001 | 1/5 | no | no | no | [dir](fr-civ-genetics-seed-001/) |
| FR-CIV-GENETICS-SEED-002 | 1/5 | no | no | no | [dir](fr-civ-genetics-seed-002/) |
| FR-CIV-GENETICS-SEED-003 | 1/5 | no | no | no | [dir](fr-civ-genetics-seed-003/) |

## FR-CIV-GEO (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GEO-001 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-001/) |
| FR-CIV-GEO-002 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-002/) |
| FR-CIV-GEO-003 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-003/) |
| FR-CIV-GEO-004 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-004/) |
| FR-CIV-GEO-005 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-005/) |
| FR-CIV-GEO-006 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-006/) |
| FR-CIV-GEO-007 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-007/) |
| FR-CIV-GEO-008 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-008/) |
| FR-CIV-GEO-009 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-009/) |
| FR-CIV-GEO-010 | 5/5 | yes | yes | yes | [dir](fr-civ-geo-010/) |

## FR-CIV-GODOT-ATTACH (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GODOT-ATTACH-000 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-attach-000/) |
| FR-CIV-GODOT-ATTACH-001 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-attach-001/) |
| FR-CIV-GODOT-ATTACH-002 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-attach-002/) |
| FR-CIV-GODOT-ATTACH-003 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-attach-003/) |
| FR-CIV-GODOT-ATTACH-004 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-attach-004/) |

## FR-CIV-GODOT-F3D0 (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GODOT-F3D0 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-f3d0/) |

## FR-CIV-GODOT-UX (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GODOT-UX-000 | 5/5 | yes | yes | yes | [dir](fr-civ-godot-ux-000/) |

## FR-CIV-GODTOOL (8)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GODTOOL-001 | 1/5 | no | no | no | [dir](fr-civ-godtool-001/) |
| FR-CIV-GODTOOL-900 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-900/) |
| FR-CIV-GODTOOL-901 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-901/) |
| FR-CIV-GODTOOL-910 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-910/) |
| FR-CIV-GODTOOL-911 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-911/) |
| FR-CIV-GODTOOL-912 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-912/) |
| FR-CIV-GODTOOL-920 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-920/) |
| FR-CIV-GODTOOL-921 | 5/5 | yes | yes | yes | [dir](fr-civ-godtool-921/) |

## FR-CIV-GOV (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-GOV-001 | 5/5 | yes | yes | yes | [dir](fr-civ-gov-001/) |
| FR-CIV-GOV-002 | 5/5 | yes | yes | yes | [dir](fr-civ-gov-002/) |
| FR-CIV-GOV-003 | 1/5 | no | no | no | [dir](fr-civ-gov-003/) |
| FR-CIV-GOV-010 | 1/5 | no | no | no | [dir](fr-civ-gov-010/) |
| FR-CIV-GOV-020 | 1/5 | no | no | no | [dir](fr-civ-gov-020/) |
| FR-CIV-GOV-100 | 1/5 | no | no | no | [dir](fr-civ-gov-100/) |
| FR-CIV-GOV-200 | 1/5 | no | no | no | [dir](fr-civ-gov-200/) |

## FR-CIV-HUD (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-HUD-001 | 5/5 | yes | yes | yes | [dir](fr-civ-hud-001/) |
| FR-CIV-HUD-002 | 5/5 | yes | yes | yes | [dir](fr-civ-hud-002/) |
| FR-CIV-HUD-003 | 5/5 | yes | yes | yes | [dir](fr-civ-hud-003/) |
| FR-CIV-HUD-004 | 5/5 | yes | yes | yes | [dir](fr-civ-hud-004/) |
| FR-CIV-HUD-005 | 5/5 | yes | yes | yes | [dir](fr-civ-hud-005/) |

## FR-CIV-IDEOLOGY (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-IDEOLOGY-001 | 1/5 | no | no | no | [dir](fr-civ-ideology-001/) |

## FR-CIV-INFOVIEW (20)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-INFOVIEW-900 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-900/) |
| FR-CIV-INFOVIEW-901 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-901/) |
| FR-CIV-INFOVIEW-902 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-902/) |
| FR-CIV-INFOVIEW-903 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-903/) |
| FR-CIV-INFOVIEW-904 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-904/) |
| FR-CIV-INFOVIEW-905 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-905/) |
| FR-CIV-INFOVIEW-906 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-906/) |
| FR-CIV-INFOVIEW-910 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-910/) |
| FR-CIV-INFOVIEW-911 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-911/) |
| FR-CIV-INFOVIEW-912 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-912/) |
| FR-CIV-INFOVIEW-913 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-913/) |
| FR-CIV-INFOVIEW-914 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-914/) |
| FR-CIV-INFOVIEW-915 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-915/) |
| FR-CIV-INFOVIEW-916 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-916/) |
| FR-CIV-INFOVIEW-917 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-917/) |
| FR-CIV-INFOVIEW-918 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-918/) |
| FR-CIV-INFOVIEW-919 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-919/) |
| FR-CIV-INFOVIEW-920 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-920/) |
| FR-CIV-INFOVIEW-921 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-921/) |
| FR-CIV-INFOVIEW-930 | 5/5 | yes | yes | yes | [dir](fr-civ-infoview-930/) |

## FR-CIV-INFRA (13)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-INFRA-001 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-001/) |
| FR-CIV-INFRA-010 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-010/) |
| FR-CIV-INFRA-011 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-011/) |
| FR-CIV-INFRA-020 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-020/) |
| FR-CIV-INFRA-021 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-021/) |
| FR-CIV-INFRA-022 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-022/) |
| FR-CIV-INFRA-030 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-030/) |
| FR-CIV-INFRA-040 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-040/) |
| FR-CIV-INFRA-050 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-050/) |
| FR-CIV-INFRA-060 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-060/) |
| FR-CIV-INFRA-070 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-070/) |
| FR-CIV-INFRA-071 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-071/) |
| FR-CIV-INFRA-072 | 5/5 | yes | yes | yes | [dir](fr-civ-infra-072/) |

## FR-CIV-INSPECT (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-INSPECT-900 | 5/5 | yes | yes | yes | [dir](fr-civ-inspect-900/) |
| FR-CIV-INSPECT-901 | 5/5 | yes | yes | yes | [dir](fr-civ-inspect-901/) |
| FR-CIV-INSPECT-902 | 5/5 | yes | yes | yes | [dir](fr-civ-inspect-902/) |
| FR-CIV-INSPECT-903 | 5/5 | yes | yes | yes | [dir](fr-civ-inspect-903/) |
| FR-CIV-INSPECT-910 | 5/5 | yes | yes | yes | [dir](fr-civ-inspect-910/) |
| FR-CIV-INSPECT-920 | 5/5 | yes | yes | yes | [dir](fr-civ-inspect-920/) |

## FR-CIV-INSTITUTIONS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-INSTITUTIONS-001 | 1/5 | no | no | no | [dir](fr-civ-institutions-001/) |

## FR-CIV-INT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-INT-001 | 1/5 | no | no | no | [dir](fr-civ-int-001/) |

## FR-CIV-L5 (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-L5 | 5/5 | yes | yes | yes | [dir](fr-civ-l5/) |

## FR-CIV-LANG (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LANG-001 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-001/) |
| FR-CIV-LANG-002 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-002/) |
| FR-CIV-LANG-003 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-003/) |
| FR-CIV-LANG-004 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-004/) |
| FR-CIV-LANG-005 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-005/) |
| FR-CIV-LANG-006 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-006/) |
| FR-CIV-LANG-007 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-007/) |
| FR-CIV-LANG-008 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-008/) |
| FR-CIV-LANG-009 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-009/) |
| FR-CIV-LANG-010 | 5/5 | yes | yes | yes | [dir](fr-civ-lang-010/) |

## FR-CIV-LAWS (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LAWS-000 | 5/5 | yes | yes | yes | [dir](fr-civ-laws-000/) |
| FR-CIV-LAWS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-laws-001/) |
| FR-CIV-LAWS-002 | 5/5 | yes | yes | yes | [dir](fr-civ-laws-002/) |
| FR-CIV-LAWS-003 | 5/5 | yes | yes | yes | [dir](fr-civ-laws-003/) |
| FR-CIV-LAWS-004 | 5/5 | yes | yes | yes | [dir](fr-civ-laws-004/) |
| FR-CIV-LAWS-005 | 5/5 | yes | yes | yes | [dir](fr-civ-laws-005/) |

## FR-CIV-LEGENDS (9)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-001/) |
| FR-CIV-LEGENDS-002 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-002/) |
| FR-CIV-LEGENDS-003 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-003/) |
| FR-CIV-LEGENDS-004 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-004/) |
| FR-CIV-LEGENDS-005 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-005/) |
| FR-CIV-LEGENDS-006 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-006/) |
| FR-CIV-LEGENDS-007 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-007/) |
| FR-CIV-LEGENDS-008 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-008/) |
| FR-CIV-LEGENDS-010 | 1/5 | no | no | no | [dir](fr-civ-legends-010/) |

## FR-CIV-LEGENDS-BROWSER (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-BROWSER-09 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-browser-09/) |

## FR-CIV-LEGENDS-CAUSAL (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-CAUSAL-06 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-causal-06/) |

## FR-CIV-LEGENDS-GAP (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-GAP-12 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-gap-12/) |

## FR-CIV-LEGENDS-GRAPH (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-GRAPH-01 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-graph-01/) |

## FR-CIV-LEGENDS-INGEST (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-INGEST-02 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-ingest-02/) |

## FR-CIV-LEGENDS-INSPECT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-INSPECT-08 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-inspect-08/) |

## FR-CIV-LEGENDS-NARRATOR (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-NARRATOR-13 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-narrator-13/) |

## FR-CIV-LEGENDS-PERSIST (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-PERSIST-11 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-persist-11/) |

## FR-CIV-LEGENDS-PRESIM (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-PRESIM-10 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-presim-10/) |

## FR-CIV-LEGENDS-PRODUCER (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-PRODUCER-03 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-producer-03/) |

## FR-CIV-LEGENDS-QUERY (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-QUERY-07 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-query-07/) |

## FR-CIV-LEGENDS-RESOLVE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-RESOLVE-04 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-resolve-04/) |

## FR-CIV-LEGENDS-SIG (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LEGENDS-SIG-05 | 5/5 | yes | yes | yes | [dir](fr-civ-legends-sig-05/) |

## FR-CIV-LIFE (19)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LIFE-000 | 5/5 | yes | yes | yes | [dir](fr-civ-life-000/) |
| FR-CIV-LIFE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-life-001/) |
| FR-CIV-LIFE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-life-002/) |
| FR-CIV-LIFE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-life-003/) |
| FR-CIV-LIFE-010 | 5/5 | yes | yes | yes | [dir](fr-civ-life-010/) |
| FR-CIV-LIFE-011 | 5/5 | yes | yes | yes | [dir](fr-civ-life-011/) |
| FR-CIV-LIFE-012 | 5/5 | yes | yes | yes | [dir](fr-civ-life-012/) |
| FR-CIV-LIFE-013 | 5/5 | yes | yes | yes | [dir](fr-civ-life-013/) |
| FR-CIV-LIFE-014 | 5/5 | yes | yes | yes | [dir](fr-civ-life-014/) |
| FR-CIV-LIFE-015 | 5/5 | yes | yes | yes | [dir](fr-civ-life-015/) |
| FR-CIV-LIFE-016 | 5/5 | yes | yes | yes | [dir](fr-civ-life-016/) |
| FR-CIV-LIFE-020 | 5/5 | yes | yes | yes | [dir](fr-civ-life-020/) |
| FR-CIV-LIFE-021 | 5/5 | yes | yes | yes | [dir](fr-civ-life-021/) |
| FR-CIV-LIFE-022 | 5/5 | yes | yes | yes | [dir](fr-civ-life-022/) |
| FR-CIV-LIFE-023 | 5/5 | yes | yes | yes | [dir](fr-civ-life-023/) |
| FR-CIV-LIFE-024 | 5/5 | yes | yes | yes | [dir](fr-civ-life-024/) |
| FR-CIV-LIFE-025 | 5/5 | yes | yes | yes | [dir](fr-civ-life-025/) |
| FR-CIV-LIFE-030 | 5/5 | yes | yes | yes | [dir](fr-civ-life-030/) |
| FR-CIV-LIFE-035 | 5/5 | yes | yes | yes | [dir](fr-civ-life-035/) |

## FR-CIV-LLM (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-LLM-001 | 5/5 | yes | yes | yes | [dir](fr-civ-llm-001/) |
| FR-CIV-LLM-002 | 5/5 | yes | yes | yes | [dir](fr-civ-llm-002/) |
| FR-CIV-LLM-003 | 5/5 | yes | yes | yes | [dir](fr-civ-llm-003/) |
| FR-CIV-LLM-004 | 5/5 | yes | yes | yes | [dir](fr-civ-llm-004/) |
| FR-CIV-LLM-005 | 5/5 | yes | yes | yes | [dir](fr-civ-llm-005/) |
| FR-CIV-LLM-006 | 5/5 | yes | yes | yes | [dir](fr-civ-llm-006/) |

## FR-CIV-MARKET (8)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-MARKET-001 | 5/5 | yes | yes | yes | [dir](fr-civ-market-001/) |
| FR-CIV-MARKET-002 | 5/5 | yes | yes | yes | [dir](fr-civ-market-002/) |
| FR-CIV-MARKET-003 | 5/5 | yes | yes | yes | [dir](fr-civ-market-003/) |
| FR-CIV-MARKET-004 | 5/5 | yes | yes | yes | [dir](fr-civ-market-004/) |
| FR-CIV-MARKET-005 | 5/5 | yes | yes | yes | [dir](fr-civ-market-005/) |
| FR-CIV-MARKET-006 | 5/5 | yes | yes | yes | [dir](fr-civ-market-006/) |
| FR-CIV-MARKET-007 | 5/5 | yes | yes | yes | [dir](fr-civ-market-007/) |
| FR-CIV-MARKET-008 | 5/5 | yes | yes | yes | [dir](fr-civ-market-008/) |

## FR-CIV-MCP (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-MCP-001 | 5/5 | yes | yes | yes | [dir](fr-civ-mcp-001/) |
| FR-CIV-MCP-002 | 5/5 | yes | yes | yes | [dir](fr-civ-mcp-002/) |
| FR-CIV-MCP-003 | 5/5 | yes | yes | yes | [dir](fr-civ-mcp-003/) |
| FR-CIV-MCP-004 | 5/5 | yes | yes | yes | [dir](fr-civ-mcp-004/) |
| FR-CIV-MCP-005 | 5/5 | yes | yes | yes | [dir](fr-civ-mcp-005/) |
| FR-CIV-MCP-006 | 5/5 | yes | yes | yes | [dir](fr-civ-mcp-006/) |

## FR-CIV-METRICS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-METRICS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-metrics-001/) |

## FR-CIV-METRICS-001-TIMESERIES (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-METRICS-001-TIMESERIES | 5/5 | yes | yes | yes | [dir](fr-civ-metrics-001-timeseries/) |

## FR-CIV-MOD (21)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-MOD-000 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-000/) |
| FR-CIV-MOD-001 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-001/) |
| FR-CIV-MOD-002 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-002/) |
| FR-CIV-MOD-003 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-003/) |
| FR-CIV-MOD-004 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-004/) |
| FR-CIV-MOD-005 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-005/) |
| FR-CIV-MOD-006 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-006/) |
| FR-CIV-MOD-007 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-007/) |
| FR-CIV-MOD-008 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-008/) |
| FR-CIV-MOD-009 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-009/) |
| FR-CIV-MOD-010 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-010/) |
| FR-CIV-MOD-011 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-011/) |
| FR-CIV-MOD-012 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-012/) |
| FR-CIV-MOD-013 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-013/) |
| FR-CIV-MOD-014 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-014/) |
| FR-CIV-MOD-015 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-015/) |
| FR-CIV-MOD-016 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-016/) |
| FR-CIV-MOD-017 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-017/) |
| FR-CIV-MOD-018 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-018/) |
| FR-CIV-MOD-019 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-019/) |
| FR-CIV-MOD-020 | 5/5 | yes | yes | yes | [dir](fr-civ-mod-020/) |

## FR-CIV-NEEDS-DECAY (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-NEEDS-DECAY-01 | 1/5 | no | no | no | [dir](fr-civ-needs-decay-01/) |

## FR-CIV-NOTIFY (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-NOTIFY-900 | 5/5 | yes | yes | yes | [dir](fr-civ-notify-900/) |
| FR-CIV-NOTIFY-901 | 5/5 | yes | yes | yes | [dir](fr-civ-notify-901/) |
| FR-CIV-NOTIFY-910 | 5/5 | yes | yes | yes | [dir](fr-civ-notify-910/) |
| FR-CIV-NOTIFY-911 | 5/5 | yes | yes | yes | [dir](fr-civ-notify-911/) |
| FR-CIV-NOTIFY-920 | 5/5 | yes | yes | yes | [dir](fr-civ-notify-920/) |
| FR-CIV-NOTIFY-921 | 5/5 | yes | yes | yes | [dir](fr-civ-notify-921/) |

## FR-CIV-PBR (11)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PBR-001 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-001/) |
| FR-CIV-PBR-002 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-002/) |
| FR-CIV-PBR-003 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-003/) |
| FR-CIV-PBR-004 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-004/) |
| FR-CIV-PBR-005 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-005/) |
| FR-CIV-PBR-006 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-006/) |
| FR-CIV-PBR-007 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-007/) |
| FR-CIV-PBR-008 | 5/5 | yes | yes | yes | [dir](fr-civ-pbr-008/) |
| FR-CIV-PBR-009 | 1/5 | no | no | no | [dir](fr-civ-pbr-009/) |
| FR-CIV-PBR-010 | 1/5 | no | no | no | [dir](fr-civ-pbr-010/) |
| FR-CIV-PBR-011 | 1/5 | no | no | no | [dir](fr-civ-pbr-011/) |

## FR-CIV-PERF (20)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PERF-001 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-001/) |
| FR-CIV-PERF-002 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-002/) |
| FR-CIV-PERF-003 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-003/) |
| FR-CIV-PERF-004 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-004/) |
| FR-CIV-PERF-005 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-005/) |
| FR-CIV-PERF-006 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-006/) |
| FR-CIV-PERF-007 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-007/) |
| FR-CIV-PERF-008 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-008/) |
| FR-CIV-PERF-009 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-009/) |
| FR-CIV-PERF-010 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-010/) |
| FR-CIV-PERF-011 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-011/) |
| FR-CIV-PERF-012 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-012/) |
| FR-CIV-PERF-013 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-013/) |
| FR-CIV-PERF-014 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-014/) |
| FR-CIV-PERF-015 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-015/) |
| FR-CIV-PERF-016 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-016/) |
| FR-CIV-PERF-017 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-017/) |
| FR-CIV-PERF-018 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-018/) |
| FR-CIV-PERF-019 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-019/) |
| FR-CIV-PERF-020 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-020/) |

## FR-CIV-PERF-BUILD (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PERF-BUILD-001 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-build-001/) |

## FR-CIV-PERF-RT (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PERF-RT-001 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-rt-001/) |
| FR-CIV-PERF-RT-002 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-rt-002/) |
| FR-CIV-PERF-RT-003 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-rt-003/) |

## FR-CIV-PERF-WEB (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PERF-WEB-001 | 5/5 | yes | yes | yes | [dir](fr-civ-perf-web-001/) |

## FR-CIV-PLANET (12)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PLANET-000 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-000/) |
| FR-CIV-PLANET-001 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-001/) |
| FR-CIV-PLANET-002 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-002/) |
| FR-CIV-PLANET-003 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-003/) |
| FR-CIV-PLANET-004 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-004/) |
| FR-CIV-PLANET-005 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-005/) |
| FR-CIV-PLANET-010 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-010/) |
| FR-CIV-PLANET-020 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-020/) |
| FR-CIV-PLANET-030 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-030/) |
| FR-CIV-PLANET-040 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-040/) |
| FR-CIV-PLANET-050 | 1/5 | no | no | no | [dir](fr-civ-planet-050/) |
| FR-CIV-PLANET-060 | 5/5 | yes | yes | yes | [dir](fr-civ-planet-060/) |

## FR-CIV-POLITY (8)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-POLITY-001 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-001/) |
| FR-CIV-POLITY-002 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-002/) |
| FR-CIV-POLITY-003 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-003/) |
| FR-CIV-POLITY-004 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-004/) |
| FR-CIV-POLITY-005 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-005/) |
| FR-CIV-POLITY-006 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-006/) |
| FR-CIV-POLITY-007 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-007/) |
| FR-CIV-POLITY-008 | 5/5 | yes | yes | yes | [dir](fr-civ-polity-008/) |

## FR-CIV-PROTO (15)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PROTO-001 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-001/) |
| FR-CIV-PROTO-002 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-002/) |
| FR-CIV-PROTO-003 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-003/) |
| FR-CIV-PROTO-004 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-004/) |
| FR-CIV-PROTO-005 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-005/) |
| FR-CIV-PROTO-006 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-006/) |
| FR-CIV-PROTO-007 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-007/) |
| FR-CIV-PROTO-008 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-008/) |
| FR-CIV-PROTO-009 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-009/) |
| FR-CIV-PROTO-010 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-010/) |
| FR-CIV-PROTO-011 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-011/) |
| FR-CIV-PROTO-012 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-012/) |
| FR-CIV-PROTO-013 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-013/) |
| FR-CIV-PROTO-014 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-014/) |
| FR-CIV-PROTO-015 | 5/5 | yes | yes | yes | [dir](fr-civ-proto-015/) |

## FR-CIV-PROTO3D (19)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PROTO3D | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d/) |
| FR-CIV-PROTO3D-000 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-000/) |
| FR-CIV-PROTO3D-001 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-001/) |
| FR-CIV-PROTO3D-002 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-002/) |
| FR-CIV-PROTO3D-003 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-003/) |
| FR-CIV-PROTO3D-004 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-004/) |
| FR-CIV-PROTO3D-005 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-005/) |
| FR-CIV-PROTO3D-006 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-006/) |
| FR-CIV-PROTO3D-007 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-007/) |
| FR-CIV-PROTO3D-008 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-008/) |
| FR-CIV-PROTO3D-009 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-009/) |
| FR-CIV-PROTO3D-010 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-010/) |
| FR-CIV-PROTO3D-011 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-011/) |
| FR-CIV-PROTO3D-012 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-012/) |
| FR-CIV-PROTO3D-013 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-013/) |
| FR-CIV-PROTO3D-014 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-014/) |
| FR-CIV-PROTO3D-015 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-015/) |
| FR-CIV-PROTO3D-016 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-016/) |
| FR-CIV-PROTO3D-017 | 5/5 | yes | yes | yes | [dir](fr-civ-proto3d-017/) |

## FR-CIV-PSYCHE (29)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PSYCHE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-001/) |
| FR-CIV-PSYCHE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-002/) |
| FR-CIV-PSYCHE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-003/) |
| FR-CIV-PSYCHE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-004/) |
| FR-CIV-PSYCHE-005 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-005/) |
| FR-CIV-PSYCHE-006 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-006/) |
| FR-CIV-PSYCHE-007 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-007/) |
| FR-CIV-PSYCHE-008 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-008/) |
| FR-CIV-PSYCHE-010 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-010/) |
| FR-CIV-PSYCHE-011 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-011/) |
| FR-CIV-PSYCHE-020 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-020/) |
| FR-CIV-PSYCHE-021 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-021/) |
| FR-CIV-PSYCHE-024 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-024/) |
| FR-CIV-PSYCHE-030 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-030/) |
| FR-CIV-PSYCHE-031 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-031/) |
| FR-CIV-PSYCHE-032 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-032/) |
| FR-CIV-PSYCHE-033 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-033/) |
| FR-CIV-PSYCHE-034 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-034/) |
| FR-CIV-PSYCHE-035 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-035/) |
| FR-CIV-PSYCHE-036 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-036/) |
| FR-CIV-PSYCHE-037 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-037/) |
| FR-CIV-PSYCHE-040 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-040/) |
| FR-CIV-PSYCHE-900 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-900/) |
| FR-CIV-PSYCHE-901 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-901/) |
| FR-CIV-PSYCHE-910 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-910/) |
| FR-CIV-PSYCHE-911 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-911/) |
| FR-CIV-PSYCHE-912 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-912/) |
| FR-CIV-PSYCHE-920 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-920/) |
| FR-CIV-PSYCHE-921 | 5/5 | yes | yes | yes | [dir](fr-civ-psyche-921/) |

## FR-CIV-PSYCHE-N11 (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-PSYCHE-N11 | 1/5 | no | no | no | [dir](fr-civ-psyche-n11/) |

## FR-CIV-QOL (14)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-QOL-100 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-100/) |
| FR-CIV-QOL-110 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-110/) |
| FR-CIV-QOL-120 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-120/) |
| FR-CIV-QOL-130 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-130/) |
| FR-CIV-QOL-140 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-140/) |
| FR-CIV-QOL-150 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-150/) |
| FR-CIV-QOL-160 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-160/) |
| FR-CIV-QOL-170 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-170/) |
| FR-CIV-QOL-180 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-180/) |
| FR-CIV-QOL-190 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-190/) |
| FR-CIV-QOL-200 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-200/) |
| FR-CIV-QOL-210 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-210/) |
| FR-CIV-QOL-220 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-220/) |
| FR-CIV-QOL-230 | 5/5 | yes | yes | yes | [dir](fr-civ-qol-230/) |

## FR-CIV-REL (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-REL-007 | 1/5 | no | no | no | [dir](fr-civ-rel-007/) |

## FR-CIV-RENDER (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RENDER-001 | 5/5 | yes | yes | yes | [dir](fr-civ-render-001/) |
| FR-CIV-RENDER-002 | 5/5 | yes | yes | yes | [dir](fr-civ-render-002/) |

## FR-CIV-RES (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RES-001 | 5/5 | yes | yes | yes | [dir](fr-civ-res-001/) |

## FR-CIV-RESEARCH (13)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RESEARCH-000 | 5/5 | yes | yes | yes | [dir](fr-civ-research-000/) |
| FR-CIV-RESEARCH-001 | 5/5 | yes | yes | yes | [dir](fr-civ-research-001/) |
| FR-CIV-RESEARCH-002 | 5/5 | yes | yes | yes | [dir](fr-civ-research-002/) |
| FR-CIV-RESEARCH-003 | 5/5 | yes | yes | yes | [dir](fr-civ-research-003/) |
| FR-CIV-RESEARCH-004 | 5/5 | yes | yes | yes | [dir](fr-civ-research-004/) |
| FR-CIV-RESEARCH-010 | 5/5 | yes | yes | yes | [dir](fr-civ-research-010/) |
| FR-CIV-RESEARCH-011 | 5/5 | yes | yes | yes | [dir](fr-civ-research-011/) |
| FR-CIV-RESEARCH-012 | 5/5 | yes | yes | yes | [dir](fr-civ-research-012/) |
| FR-CIV-RESEARCH-020 | 5/5 | yes | yes | yes | [dir](fr-civ-research-020/) |
| FR-CIV-RESEARCH-030 | 5/5 | yes | yes | yes | [dir](fr-civ-research-030/) |
| FR-CIV-RESEARCH-031 | 5/5 | yes | yes | yes | [dir](fr-civ-research-031/) |
| FR-CIV-RESEARCH-032 | 5/5 | yes | yes | yes | [dir](fr-civ-research-032/) |
| FR-CIV-RESEARCH-033 | 5/5 | yes | yes | yes | [dir](fr-civ-research-033/) |

## FR-CIV-RESEARCH-001-SCENARIO (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RESEARCH-001-SCENARIO | 5/5 | yes | yes | yes | [dir](fr-civ-research-001-scenario/) |

## FR-CIV-RESEARCH-002-SNAPSHOT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RESEARCH-002-SNAPSHOT | 5/5 | yes | yes | yes | [dir](fr-civ-research-002-snapshot/) |

## FR-CIV-RESEARCH-003-EXPORT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RESEARCH-003-EXPORT | 5/5 | yes | yes | yes | [dir](fr-civ-research-003-export/) |

## FR-CIV-RESEARCH-004-REPLAY (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RESEARCH-004-REPLAY | 5/5 | yes | yes | yes | [dir](fr-civ-research-004-replay/) |

## FR-CIV-ROAD (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-ROAD-900 | 5/5 | yes | yes | yes | [dir](fr-civ-road-900/) |
| FR-CIV-ROAD-901 | 5/5 | yes | yes | yes | [dir](fr-civ-road-901/) |
| FR-CIV-ROAD-902 | 5/5 | yes | yes | yes | [dir](fr-civ-road-902/) |
| FR-CIV-ROAD-910 | 5/5 | yes | yes | yes | [dir](fr-civ-road-910/) |
| FR-CIV-ROAD-920 | 5/5 | yes | yes | yes | [dir](fr-civ-road-920/) |
| FR-CIV-ROAD-921 | 5/5 | yes | yes | yes | [dir](fr-civ-road-921/) |

## FR-CIV-RTS (15)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RTS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-001/) |
| FR-CIV-RTS-002 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-002/) |
| FR-CIV-RTS-003 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-003/) |
| FR-CIV-RTS-004 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-004/) |
| FR-CIV-RTS-005 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-005/) |
| FR-CIV-RTS-006 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-006/) |
| FR-CIV-RTS-007 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-007/) |
| FR-CIV-RTS-008 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-008/) |
| FR-CIV-RTS-009 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-009/) |
| FR-CIV-RTS-010 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-010/) |
| FR-CIV-RTS-011 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-011/) |
| FR-CIV-RTS-012 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-012/) |
| FR-CIV-RTS-013 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-013/) |
| FR-CIV-RTS-014 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-014/) |
| FR-CIV-RTS-015 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-015/) |

## FR-CIV-RTS-NATION (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RTS-NATION-001 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-nation-001/) |
| FR-CIV-RTS-NATION-002 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-nation-002/) |

## FR-CIV-RTS-RENDER (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RTS-RENDER-001 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-render-001/) |
| FR-CIV-RTS-RENDER-002 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-render-002/) |
| FR-CIV-RTS-RENDER-003 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-render-003/) |
| FR-CIV-RTS-RENDER-004 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-render-004/) |
| FR-CIV-RTS-RENDER-005 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-render-005/) |

## FR-CIV-RTS-ZOOM (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-RTS-ZOOM-001 | 5/5 | yes | yes | yes | [dir](fr-civ-rts-zoom-001/) |

## FR-CIV-SAVE (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SAVE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-save-001/) |
| FR-CIV-SAVE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-save-002/) |
| FR-CIV-SAVE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-save-003/) |
| FR-CIV-SAVE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-save-004/) |

## FR-CIV-SCALE (8)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SCALE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-001/) |
| FR-CIV-SCALE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-002/) |
| FR-CIV-SCALE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-003/) |
| FR-CIV-SCALE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-004/) |
| FR-CIV-SCALE-005 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-005/) |
| FR-CIV-SCALE-006 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-006/) |
| FR-CIV-SCALE-007 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-007/) |
| FR-CIV-SCALE-008 | 5/5 | yes | yes | yes | [dir](fr-civ-scale-008/) |

## FR-CIV-SERVER (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SERVER-001 | 5/5 | yes | yes | yes | [dir](fr-civ-server-001/) |
| FR-CIV-SERVER-002 | 5/5 | yes | yes | yes | [dir](fr-civ-server-002/) |
| FR-CIV-SERVER-003 | 1/5 | no | no | no | [dir](fr-civ-server-003/) |

## FR-CIV-SERVER-001-WS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SERVER-001-WS | 5/5 | yes | yes | yes | [dir](fr-civ-server-001-ws/) |

## FR-CIV-SERVER-002-PROTO (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SERVER-002-PROTO | 5/5 | yes | yes | yes | [dir](fr-civ-server-002-proto/) |

## FR-CIV-SOCIAL (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SOCIAL-001 | 5/5 | yes | yes | yes | [dir](fr-civ-social-001/) |
| FR-CIV-SOCIAL-002 | 5/5 | yes | yes | yes | [dir](fr-civ-social-002/) |

## FR-CIV-SOCIAL-001-INSTITUTIONS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SOCIAL-001-INSTITUTIONS | 5/5 | yes | yes | yes | [dir](fr-civ-social-001-institutions/) |

## FR-CIV-SOCIAL-002-IDEOLOGY (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SOCIAL-002-IDEOLOGY | 5/5 | yes | yes | yes | [dir](fr-civ-social-002-ideology/) |

## FR-CIV-SPECIES (48)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-SPECIES-000 | 5/5 | yes | yes | yes | [dir](fr-civ-species-000/) |
| FR-CIV-SPECIES-001 | 5/5 | yes | yes | yes | [dir](fr-civ-species-001/) |
| FR-CIV-SPECIES-002 | 5/5 | yes | yes | yes | [dir](fr-civ-species-002/) |
| FR-CIV-SPECIES-003 | 5/5 | yes | yes | yes | [dir](fr-civ-species-003/) |
| FR-CIV-SPECIES-004 | 5/5 | yes | yes | yes | [dir](fr-civ-species-004/) |
| FR-CIV-SPECIES-005 | 5/5 | yes | yes | yes | [dir](fr-civ-species-005/) |
| FR-CIV-SPECIES-006 | 5/5 | yes | yes | yes | [dir](fr-civ-species-006/) |
| FR-CIV-SPECIES-007 | 5/5 | yes | yes | yes | [dir](fr-civ-species-007/) |
| FR-CIV-SPECIES-008 | 5/5 | yes | yes | yes | [dir](fr-civ-species-008/) |
| FR-CIV-SPECIES-009 | 5/5 | yes | yes | yes | [dir](fr-civ-species-009/) |
| FR-CIV-SPECIES-010 | 5/5 | yes | yes | yes | [dir](fr-civ-species-010/) |
| FR-CIV-SPECIES-011 | 5/5 | yes | yes | yes | [dir](fr-civ-species-011/) |
| FR-CIV-SPECIES-012 | 5/5 | yes | yes | yes | [dir](fr-civ-species-012/) |
| FR-CIV-SPECIES-013 | 5/5 | yes | yes | yes | [dir](fr-civ-species-013/) |
| FR-CIV-SPECIES-014 | 5/5 | yes | yes | yes | [dir](fr-civ-species-014/) |
| FR-CIV-SPECIES-015 | 5/5 | yes | yes | yes | [dir](fr-civ-species-015/) |
| FR-CIV-SPECIES-016 | 5/5 | yes | yes | yes | [dir](fr-civ-species-016/) |
| FR-CIV-SPECIES-017 | 5/5 | yes | yes | yes | [dir](fr-civ-species-017/) |
| FR-CIV-SPECIES-100 | 5/5 | yes | yes | yes | [dir](fr-civ-species-100/) |
| FR-CIV-SPECIES-101 | 5/5 | yes | yes | yes | [dir](fr-civ-species-101/) |
| FR-CIV-SPECIES-102 | 5/5 | yes | yes | yes | [dir](fr-civ-species-102/) |
| FR-CIV-SPECIES-103 | 5/5 | yes | yes | yes | [dir](fr-civ-species-103/) |
| FR-CIV-SPECIES-104 | 5/5 | yes | yes | yes | [dir](fr-civ-species-104/) |
| FR-CIV-SPECIES-105 | 5/5 | yes | yes | yes | [dir](fr-civ-species-105/) |
| FR-CIV-SPECIES-200 | 5/5 | yes | yes | yes | [dir](fr-civ-species-200/) |
| FR-CIV-SPECIES-201 | 5/5 | yes | yes | yes | [dir](fr-civ-species-201/) |
| FR-CIV-SPECIES-202 | 5/5 | yes | yes | yes | [dir](fr-civ-species-202/) |
| FR-CIV-SPECIES-203 | 5/5 | yes | yes | yes | [dir](fr-civ-species-203/) |
| FR-CIV-SPECIES-204 | 5/5 | yes | yes | yes | [dir](fr-civ-species-204/) |
| FR-CIV-SPECIES-205 | 5/5 | yes | yes | yes | [dir](fr-civ-species-205/) |
| FR-CIV-SPECIES-300 | 5/5 | yes | yes | yes | [dir](fr-civ-species-300/) |
| FR-CIV-SPECIES-301 | 5/5 | yes | yes | yes | [dir](fr-civ-species-301/) |
| FR-CIV-SPECIES-302 | 5/5 | yes | yes | yes | [dir](fr-civ-species-302/) |
| FR-CIV-SPECIES-303 | 5/5 | yes | yes | yes | [dir](fr-civ-species-303/) |
| FR-CIV-SPECIES-304 | 5/5 | yes | yes | yes | [dir](fr-civ-species-304/) |
| FR-CIV-SPECIES-400 | 5/5 | yes | yes | yes | [dir](fr-civ-species-400/) |
| FR-CIV-SPECIES-401 | 5/5 | yes | yes | yes | [dir](fr-civ-species-401/) |
| FR-CIV-SPECIES-402 | 5/5 | yes | yes | yes | [dir](fr-civ-species-402/) |
| FR-CIV-SPECIES-403 | 5/5 | yes | yes | yes | [dir](fr-civ-species-403/) |
| FR-CIV-SPECIES-404 | 5/5 | yes | yes | yes | [dir](fr-civ-species-404/) |
| FR-CIV-SPECIES-405 | 5/5 | yes | yes | yes | [dir](fr-civ-species-405/) |
| FR-CIV-SPECIES-406 | 5/5 | yes | yes | yes | [dir](fr-civ-species-406/) |
| FR-CIV-SPECIES-500 | 5/5 | yes | yes | yes | [dir](fr-civ-species-500/) |
| FR-CIV-SPECIES-501 | 5/5 | yes | yes | yes | [dir](fr-civ-species-501/) |
| FR-CIV-SPECIES-502 | 5/5 | yes | yes | yes | [dir](fr-civ-species-502/) |
| FR-CIV-SPECIES-503 | 5/5 | yes | yes | yes | [dir](fr-civ-species-503/) |
| FR-CIV-SPECIES-504 | 5/5 | yes | yes | yes | [dir](fr-civ-species-504/) |
| FR-CIV-SPECIES-505 | 5/5 | yes | yes | yes | [dir](fr-civ-species-505/) |

## FR-CIV-TACTICS (65)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-TACTICS-000 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-000/) |
| FR-CIV-TACTICS-001 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-001/) |
| FR-CIV-TACTICS-001- | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-001-/) |
| FR-CIV-TACTICS-002 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-002/) |
| FR-CIV-TACTICS-003 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-003/) |
| FR-CIV-TACTICS-010 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-010/) |
| FR-CIV-TACTICS-011 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-011/) |
| FR-CIV-TACTICS-020 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-020/) |
| FR-CIV-TACTICS-021 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-021/) |
| FR-CIV-TACTICS-022 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-022/) |
| FR-CIV-TACTICS-023 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-023/) |
| FR-CIV-TACTICS-024 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-024/) |
| FR-CIV-TACTICS-025 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-025/) |
| FR-CIV-TACTICS-025- | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-025-/) |
| FR-CIV-TACTICS-030 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-030/) |
| FR-CIV-TACTICS-031 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-031/) |
| FR-CIV-TACTICS-032 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-032/) |
| FR-CIV-TACTICS-033 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-033/) |
| FR-CIV-TACTICS-034 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-034/) |
| FR-CIV-TACTICS-035 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-035/) |
| FR-CIV-TACTICS-036 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-036/) |
| FR-CIV-TACTICS-037 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-037/) |
| FR-CIV-TACTICS-038 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-038/) |
| FR-CIV-TACTICS-039 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-039/) |
| FR-CIV-TACTICS-040 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-040/) |
| FR-CIV-TACTICS-041 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-041/) |
| FR-CIV-TACTICS-042 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-042/) |
| FR-CIV-TACTICS-043 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-043/) |
| FR-CIV-TACTICS-044 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-044/) |
| FR-CIV-TACTICS-045 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-045/) |
| FR-CIV-TACTICS-046 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-046/) |
| FR-CIV-TACTICS-047 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-047/) |
| FR-CIV-TACTICS-048 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-048/) |
| FR-CIV-TACTICS-049 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-049/) |
| FR-CIV-TACTICS-050 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-050/) |
| FR-CIV-TACTICS-051 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-051/) |
| FR-CIV-TACTICS-052 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-052/) |
| FR-CIV-TACTICS-053 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-053/) |
| FR-CIV-TACTICS-054 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-054/) |
| FR-CIV-TACTICS-055 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-055/) |
| FR-CIV-TACTICS-056 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-056/) |
| FR-CIV-TACTICS-057 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-057/) |
| FR-CIV-TACTICS-058 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-058/) |
| FR-CIV-TACTICS-059 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-059/) |
| FR-CIV-TACTICS-060 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-060/) |
| FR-CIV-TACTICS-061 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-061/) |
| FR-CIV-TACTICS-062 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-062/) |
| FR-CIV-TACTICS-063 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-063/) |
| FR-CIV-TACTICS-064 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-064/) |
| FR-CIV-TACTICS-065 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-065/) |
| FR-CIV-TACTICS-066 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-066/) |
| FR-CIV-TACTICS-067 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-067/) |
| FR-CIV-TACTICS-068 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-068/) |
| FR-CIV-TACTICS-069 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-069/) |
| FR-CIV-TACTICS-070 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-070/) |
| FR-CIV-TACTICS-071 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-071/) |
| FR-CIV-TACTICS-072 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-072/) |
| FR-CIV-TACTICS-073 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-073/) |
| FR-CIV-TACTICS-074 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-074/) |
| FR-CIV-TACTICS-075 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-075/) |
| FR-CIV-TACTICS-076 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-076/) |
| FR-CIV-TACTICS-077 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-077/) |
| FR-CIV-TACTICS-100 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-100/) |
| FR-CIV-TACTICS-101 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-101/) |
| FR-CIV-TACTICS-102 | 5/5 | yes | yes | yes | [dir](fr-civ-tactics-102/) |

## FR-CIV-TECH (21)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-TECH-001 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-001/) |
| FR-CIV-TECH-002 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-002/) |
| FR-CIV-TECH-003 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-003/) |
| FR-CIV-TECH-004 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-004/) |
| FR-CIV-TECH-005 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-005/) |
| FR-CIV-TECH-006 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-006/) |
| FR-CIV-TECH-007 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-007/) |
| FR-CIV-TECH-008 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-008/) |
| FR-CIV-TECH-009 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-009/) |
| FR-CIV-TECH-010 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-010/) |
| FR-CIV-TECH-011 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-011/) |
| FR-CIV-TECH-012 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-012/) |
| FR-CIV-TECH-013 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-013/) |
| FR-CIV-TECH-014 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-014/) |
| FR-CIV-TECH-015 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-015/) |
| FR-CIV-TECH-016 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-016/) |
| FR-CIV-TECH-017 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-017/) |
| FR-CIV-TECH-018 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-018/) |
| FR-CIV-TECH-019 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-019/) |
| FR-CIV-TECH-020 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-020/) |
| FR-CIV-TECH-021 | 5/5 | yes | yes | yes | [dir](fr-civ-tech-021/) |

## FR-CIV-TERRAIN (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-TERRAIN-001 | 5/5 | yes | yes | yes | [dir](fr-civ-terrain-001/) |
| FR-CIV-TERRAIN-002 | 5/5 | yes | yes | yes | [dir](fr-civ-terrain-002/) |
| FR-CIV-TERRAIN-003 | 5/5 | yes | yes | yes | [dir](fr-civ-terrain-003/) |
| FR-CIV-TERRAIN-004 | 5/5 | yes | yes | yes | [dir](fr-civ-terrain-004/) |
| FR-CIV-TERRAIN-005 | 5/5 | yes | yes | yes | [dir](fr-civ-terrain-005/) |
| FR-CIV-TERRAIN-006 | 5/5 | yes | yes | yes | [dir](fr-civ-terrain-006/) |

## FR-CIV-TEST (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-TEST-001 | 1/5 | no | no | no | [dir](fr-civ-test-001/) |
| FR-CIV-TEST-002 | 1/5 | no | no | no | [dir](fr-civ-test-002/) |
| FR-CIV-TEST-006 | 1/5 | no | no | no | [dir](fr-civ-test-006/) |
| FR-CIV-TEST-007 | 1/5 | no | no | no | [dir](fr-civ-test-007/) |
| FR-CIV-TEST-008 | 1/5 | no | no | no | [dir](fr-civ-test-008/) |
| FR-CIV-TEST-009 | 1/5 | no | no | no | [dir](fr-civ-test-009/) |
| FR-CIV-TEST-021 | 1/5 | no | no | no | [dir](fr-civ-test-021/) |

## FR-CIV-TRAFFIC-LANE (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-TRAFFIC-LANE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-traffic-lane-001/) |
| FR-CIV-TRAFFIC-LANE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-traffic-lane-002/) |
| FR-CIV-TRAFFIC-LANE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-traffic-lane-003/) |
| FR-CIV-TRAFFIC-LANE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-traffic-lane-004/) |

## FR-CIV-UI (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-UI-001 | 5/5 | yes | yes | yes | [dir](fr-civ-ui-001/) |
| FR-CIV-UI-002 | 5/5 | yes | yes | yes | [dir](fr-civ-ui-002/) |
| FR-CIV-UI-003 | 5/5 | yes | yes | yes | [dir](fr-civ-ui-003/) |

## FR-CIV-UNREST (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-UNREST-002 | 1/5 | no | no | no | [dir](fr-civ-unrest-002/) |

## FR-CIV-UX (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-UX-000 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-000/) |
| FR-CIV-UX-001 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-001/) |
| FR-CIV-UX-002 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-002/) |
| FR-CIV-UX-003 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-003/) |
| FR-CIV-UX-004 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-004/) |
| FR-CIV-UX-005 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-005/) |
| FR-CIV-UX-006 | 5/5 | yes | yes | yes | [dir](fr-civ-ux-006/) |

## FR-CIV-VEHICLE (26)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-VEHICLE-001 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-001/) |
| FR-CIV-VEHICLE-002 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-002/) |
| FR-CIV-VEHICLE-003 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-003/) |
| FR-CIV-VEHICLE-004 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-004/) |
| FR-CIV-VEHICLE-005 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-005/) |
| FR-CIV-VEHICLE-010 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-010/) |
| FR-CIV-VEHICLE-011 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-011/) |
| FR-CIV-VEHICLE-012 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-012/) |
| FR-CIV-VEHICLE-013 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-013/) |
| FR-CIV-VEHICLE-014 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-014/) |
| FR-CIV-VEHICLE-020 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-020/) |
| FR-CIV-VEHICLE-021 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-021/) |
| FR-CIV-VEHICLE-022 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-022/) |
| FR-CIV-VEHICLE-023 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-023/) |
| FR-CIV-VEHICLE-024 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-024/) |
| FR-CIV-VEHICLE-030 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-030/) |
| FR-CIV-VEHICLE-040 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-040/) |
| FR-CIV-VEHICLE-041 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-041/) |
| FR-CIV-VEHICLE-042 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-042/) |
| FR-CIV-VEHICLE-043 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-043/) |
| FR-CIV-VEHICLE-044 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-044/) |
| FR-CIV-VEHICLE-045 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-045/) |
| FR-CIV-VEHICLE-046 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-046/) |
| FR-CIV-VEHICLE-047 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-047/) |
| FR-CIV-VEHICLE-050 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-050/) |
| FR-CIV-VEHICLE-060 | 5/5 | yes | yes | yes | [dir](fr-civ-vehicle-060/) |

## FR-CIV-VERIFY (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-VERIFY-001 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-001/) |
| FR-CIV-VERIFY-002 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-002/) |
| FR-CIV-VERIFY-003 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-003/) |
| FR-CIV-VERIFY-004 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-004/) |
| FR-CIV-VERIFY-005 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-005/) |
| FR-CIV-VERIFY-006 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-006/) |
| FR-CIV-VERIFY-007 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-007/) |
| FR-CIV-VERIFY-008 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-008/) |
| FR-CIV-VERIFY-009 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-009/) |
| FR-CIV-VERIFY-010 | 5/5 | yes | yes | yes | [dir](fr-civ-verify-010/) |

## FR-CIV-VOXEL (18)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-VOXEL-000 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-000/) |
| FR-CIV-VOXEL-001 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-001/) |
| FR-CIV-VOXEL-002 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-002/) |
| FR-CIV-VOXEL-003 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-003/) |
| FR-CIV-VOXEL-004 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-004/) |
| FR-CIV-VOXEL-005 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-005/) |
| FR-CIV-VOXEL-006 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-006/) |
| FR-CIV-VOXEL-007 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-007/) |
| FR-CIV-VOXEL-010 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-010/) |
| FR-CIV-VOXEL-020 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-020/) |
| FR-CIV-VOXEL-021 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-021/) |
| FR-CIV-VOXEL-022 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-022/) |
| FR-CIV-VOXEL-023 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-023/) |
| FR-CIV-VOXEL-024 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-024/) |
| FR-CIV-VOXEL-025 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-025/) |
| FR-CIV-VOXEL-030 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-030/) |
| FR-CIV-VOXEL-031 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-031/) |
| FR-CIV-VOXEL-032 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-032/) |

## FR-CIV-VOXEL-DIRTY (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-VOXEL-DIRTY-001 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-dirty-001/) |
| FR-CIV-VOXEL-DIRTY-002 | 5/5 | yes | yes | yes | [dir](fr-civ-voxel-dirty-002/) |

## FR-CIV-WAR (15)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-WAR-001 | 5/5 | yes | yes | yes | [dir](fr-civ-war-001/) |
| FR-CIV-WAR-002 | 5/5 | yes | yes | yes | [dir](fr-civ-war-002/) |
| FR-CIV-WAR-003 | 5/5 | yes | yes | yes | [dir](fr-civ-war-003/) |
| FR-CIV-WAR-004 | 5/5 | yes | yes | yes | [dir](fr-civ-war-004/) |
| FR-CIV-WAR-010 | 5/5 | yes | yes | yes | [dir](fr-civ-war-010/) |
| FR-CIV-WAR-011 | 5/5 | yes | yes | yes | [dir](fr-civ-war-011/) |
| FR-CIV-WAR-012 | 5/5 | yes | yes | yes | [dir](fr-civ-war-012/) |
| FR-CIV-WAR-013 | 5/5 | yes | yes | yes | [dir](fr-civ-war-013/) |
| FR-CIV-WAR-020 | 5/5 | yes | yes | yes | [dir](fr-civ-war-020/) |
| FR-CIV-WAR-021 | 5/5 | yes | yes | yes | [dir](fr-civ-war-021/) |
| FR-CIV-WAR-022 | 5/5 | yes | yes | yes | [dir](fr-civ-war-022/) |
| FR-CIV-WAR-030 | 5/5 | yes | yes | yes | [dir](fr-civ-war-030/) |
| FR-CIV-WAR-040 | 5/5 | yes | yes | yes | [dir](fr-civ-war-040/) |
| FR-CIV-WAR-041 | 5/5 | yes | yes | yes | [dir](fr-civ-war-041/) |
| FR-CIV-WAR-042 | 5/5 | yes | yes | yes | [dir](fr-civ-war-042/) |

## FR-CIV-WAR-001-UNITS (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-WAR-001-UNITS | 5/5 | yes | yes | yes | [dir](fr-civ-war-001-units/) |

## FR-CIV-WAR-002-COMBAT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-WAR-002-COMBAT | 5/5 | yes | yes | yes | [dir](fr-civ-war-002-combat/) |

## FR-CIV-WARFARE (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-WARFARE-001 | 1/5 | no | no | no | [dir](fr-civ-warfare-001/) |
| FR-CIV-WARFARE-002 | 1/5 | no | no | no | [dir](fr-civ-warfare-002/) |
| FR-CIV-WARFARE-003 | 1/5 | no | no | no | [dir](fr-civ-warfare-003/) |
| FR-CIV-WARFARE-004 | 1/5 | no | no | no | [dir](fr-civ-warfare-004/) |

## FR-CIV-WEB (9)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CIV-WEB-000 | 5/5 | yes | yes | yes | [dir](fr-civ-web-000/) |
| FR-CIV-WEB-001 | 5/5 | yes | yes | yes | [dir](fr-civ-web-001/) |
| FR-CIV-WEB-002 | 5/5 | yes | yes | yes | [dir](fr-civ-web-002/) |
| FR-CIV-WEB-003 | 5/5 | yes | yes | yes | [dir](fr-civ-web-003/) |
| FR-CIV-WEB-004 | 5/5 | yes | yes | yes | [dir](fr-civ-web-004/) |
| FR-CIV-WEB-005 | 5/5 | yes | yes | yes | [dir](fr-civ-web-005/) |
| FR-CIV-WEB-006 | 5/5 | yes | yes | yes | [dir](fr-civ-web-006/) |
| FR-CIV-WEB-007 | 5/5 | yes | yes | yes | [dir](fr-civ-web-007/) |
| FR-CIV-WEB-008 | 5/5 | yes | yes | yes | [dir](fr-civ-web-008/) |

## FR-CLIENT (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CLIENT-001 | 5/5 | yes | yes | yes | [dir](fr-client-001/) |
| FR-CLIENT-002 | 5/5 | yes | yes | yes | [dir](fr-client-002/) |
| FR-CLIENT-003 | 5/5 | yes | yes | yes | [dir](fr-client-003/) |

## FR-CLIM (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CLIM-001 | 5/5 | yes | yes | yes | [dir](fr-clim-001/) |
| FR-CLIM-002 | 5/5 | yes | yes | yes | [dir](fr-clim-002/) |
| FR-CLIM-003 | 5/5 | yes | yes | yes | [dir](fr-clim-003/) |
| FR-CLIM-004 | 5/5 | yes | yes | yes | [dir](fr-clim-004/) |
| FR-CLIM-005 | 5/5 | yes | yes | yes | [dir](fr-clim-005/) |
| FR-CLIM-006 | 5/5 | yes | yes | yes | [dir](fr-clim-006/) |

## FR-CORE (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-CORE-001 | 5/5 | yes | yes | yes | [dir](fr-core-001/) |
| FR-CORE-002 | 5/5 | yes | yes | yes | [dir](fr-core-002/) |
| FR-CORE-003 | 5/5 | yes | yes | yes | [dir](fr-core-003/) |
| FR-CORE-004 | 5/5 | yes | yes | yes | [dir](fr-core-004/) |
| FR-CORE-005 | 5/5 | yes | yes | yes | [dir](fr-core-005/) |
| FR-CORE-006 | 5/5 | yes | yes | yes | [dir](fr-core-006/) |
| FR-CORE-007 | 5/5 | yes | yes | yes | [dir](fr-core-007/) |
| FR-CORE-008 | 5/5 | yes | yes | yes | [dir](fr-core-008/) |
| FR-CORE-009 | 5/5 | yes | yes | yes | [dir](fr-core-009/) |
| FR-CORE-010 | 5/5 | yes | yes | yes | [dir](fr-core-010/) |

## FR-DET (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-DET-001 | 5/5 | yes | yes | yes | [dir](fr-det-001/) |
| FR-DET-002 | 5/5 | yes | yes | yes | [dir](fr-det-002/) |
| FR-DET-003 | 5/5 | yes | yes | yes | [dir](fr-det-003/) |
| FR-DET-004 | 5/5 | yes | yes | yes | [dir](fr-det-004/) |
| FR-DET-005 | 5/5 | yes | yes | yes | [dir](fr-det-005/) |
| FR-DET-006 | 5/5 | yes | yes | yes | [dir](fr-det-006/) |
| FR-DET-007 | 5/5 | yes | yes | yes | [dir](fr-det-007/) |

## FR-DIP (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-DIP-002 | 1/5 | no | no | no | [dir](fr-dip-002/) |

## FR-DIPL (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-DIPL-001 | 5/5 | yes | yes | yes | [dir](fr-dipl-001/) |
| FR-DIPL-002 | 5/5 | yes | yes | yes | [dir](fr-dipl-002/) |
| FR-DIPL-003 | 5/5 | yes | yes | yes | [dir](fr-dipl-003/) |
| FR-DIPL-004 | 5/5 | yes | yes | yes | [dir](fr-dipl-004/) |
| FR-DIPL-005 | 5/5 | yes | yes | yes | [dir](fr-dipl-005/) |
| FR-DIPL-006 | 5/5 | yes | yes | yes | [dir](fr-dipl-006/) |
| FR-DIPL-007 | 5/5 | yes | yes | yes | [dir](fr-dipl-007/) |

## FR-DOC (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-DOC-001 | 5/5 | yes | yes | yes | [dir](fr-doc-001/) |

## FR-ECO (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-ECO-001 | 5/5 | yes | yes | yes | [dir](fr-eco-001/) |
| FR-ECO-002 | 5/5 | yes | yes | yes | [dir](fr-eco-002/) |
| FR-ECO-003 | 5/5 | yes | yes | yes | [dir](fr-eco-003/) |
| FR-ECO-004 | 5/5 | yes | yes | yes | [dir](fr-eco-004/) |
| FR-ECO-005 | 5/5 | yes | yes | yes | [dir](fr-eco-005/) |
| FR-ECO-006 | 5/5 | yes | yes | yes | [dir](fr-eco-006/) |
| FR-ECO-007 | 5/5 | yes | yes | yes | [dir](fr-eco-007/) |
| FR-ECO-008 | 5/5 | yes | yes | yes | [dir](fr-eco-008/) |
| FR-ECO-009 | 5/5 | yes | yes | yes | [dir](fr-eco-009/) |
| FR-ECO-010 | 5/5 | yes | yes | yes | [dir](fr-eco-010/) |

## FR-ECON (10)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-ECON-001 | 5/5 | yes | yes | yes | [dir](fr-econ-001/) |
| FR-ECON-002 | 5/5 | yes | yes | yes | [dir](fr-econ-002/) |
| FR-ECON-003 | 5/5 | yes | yes | yes | [dir](fr-econ-003/) |
| FR-ECON-004 | 5/5 | yes | yes | yes | [dir](fr-econ-004/) |
| FR-ECON-005 | 5/5 | yes | yes | yes | [dir](fr-econ-005/) |
| FR-ECON-006 | 5/5 | yes | yes | yes | [dir](fr-econ-006/) |
| FR-ECON-007 | 5/5 | yes | yes | yes | [dir](fr-econ-007/) |
| FR-ECON-008 | 5/5 | yes | yes | yes | [dir](fr-econ-008/) |
| FR-ECON-009 | 5/5 | yes | yes | yes | [dir](fr-econ-009/) |
| FR-ECON-010 | 5/5 | yes | yes | yes | [dir](fr-econ-010/) |

## FR-ECON-EMERGE (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-ECON-EMERGE-001 | 1/5 | no | no | no | [dir](fr-econ-emerge-001/) |
| FR-ECON-EMERGE-002 | 1/5 | no | no | no | [dir](fr-econ-emerge-002/) |
| FR-ECON-EMERGE-004 | 1/5 | no | no | no | [dir](fr-econ-emerge-004/) |

## FR-EMG (24)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-EMG-001 | 1/5 | no | no | no | [dir](fr-emg-001/) |
| FR-EMG-002 | 1/5 | no | no | no | [dir](fr-emg-002/) |
| FR-EMG-003 | 1/5 | no | no | no | [dir](fr-emg-003/) |
| FR-EMG-004 | 1/5 | no | no | no | [dir](fr-emg-004/) |
| FR-EMG-005 | 1/5 | no | no | no | [dir](fr-emg-005/) |
| FR-EMG-006 | 1/5 | no | no | no | [dir](fr-emg-006/) |
| FR-EMG-007 | 1/5 | no | no | no | [dir](fr-emg-007/) |
| FR-EMG-008 | 1/5 | no | no | no | [dir](fr-emg-008/) |
| FR-EMG-009 | 1/5 | no | no | no | [dir](fr-emg-009/) |
| FR-EMG-010 | 1/5 | no | no | no | [dir](fr-emg-010/) |
| FR-EMG-012 | 1/5 | no | no | no | [dir](fr-emg-012/) |
| FR-EMG-013 | 1/5 | no | no | no | [dir](fr-emg-013/) |
| FR-EMG-014 | 1/5 | no | no | no | [dir](fr-emg-014/) |
| FR-EMG-015 | 1/5 | no | no | no | [dir](fr-emg-015/) |
| FR-EMG-016 | 1/5 | no | no | no | [dir](fr-emg-016/) |
| FR-EMG-017 | 1/5 | no | no | no | [dir](fr-emg-017/) |
| FR-EMG-018 | 1/5 | no | no | no | [dir](fr-emg-018/) |
| FR-EMG-019 | 1/5 | no | no | no | [dir](fr-emg-019/) |
| FR-EMG-020 | 1/5 | no | no | no | [dir](fr-emg-020/) |
| FR-EMG-021 | 1/5 | no | no | no | [dir](fr-emg-021/) |
| FR-EMG-022 | 1/5 | no | no | no | [dir](fr-emg-022/) |
| FR-EMG-023 | 1/5 | no | no | no | [dir](fr-emg-023/) |
| FR-EMG-024 | 1/5 | no | no | no | [dir](fr-emg-024/) |
| FR-EMG-025 | 1/5 | no | no | no | [dir](fr-emg-025/) |

## FR-FR-CORE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-FR-CORE-009 | 1/5 | no | no | no | [dir](fr-fr-core-009/) |

## FR-GUARD (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-GUARD-001 | 5/5 | yes | yes | yes | [dir](fr-guard-001/) |
| FR-GUARD-002 | 5/5 | yes | yes | yes | [dir](fr-guard-002/) |

## FR-INST (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-INST-001 | 5/5 | yes | yes | yes | [dir](fr-inst-001/) |
| FR-INST-002 | 5/5 | yes | yes | yes | [dir](fr-inst-002/) |
| FR-INST-003 | 5/5 | yes | yes | yes | [dir](fr-inst-003/) |
| FR-INST-004 | 5/5 | yes | yes | yes | [dir](fr-inst-004/) |
| FR-INST-005 | 5/5 | yes | yes | yes | [dir](fr-inst-005/) |
| FR-INST-006 | 5/5 | yes | yes | yes | [dir](fr-inst-006/) |

## FR-INT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-INT-001 | 5/5 | yes | yes | yes | [dir](fr-int-001/) |

## FR-LANGUAGE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-LANGUAGE-001 | 1/5 | no | no | no | [dir](fr-language-001/) |

## FR-LOD (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-LOD-001 | 5/5 | yes | yes | yes | [dir](fr-lod-001/) |
| FR-LOD-002 | 5/5 | yes | yes | yes | [dir](fr-lod-002/) |
| FR-LOD-003 | 5/5 | yes | yes | yes | [dir](fr-lod-003/) |
| FR-LOD-004 | 5/5 | yes | yes | yes | [dir](fr-lod-004/) |

## FR-MET (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-MET-001 | 5/5 | yes | yes | yes | [dir](fr-met-001/) |

## FR-METRICS (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-METRICS-001 | 5/5 | yes | yes | yes | [dir](fr-metrics-001/) |
| FR-METRICS-002 | 5/5 | yes | yes | yes | [dir](fr-metrics-002/) |
| FR-METRICS-003 | 5/5 | yes | yes | yes | [dir](fr-metrics-003/) |
| FR-METRICS-004 | 5/5 | yes | yes | yes | [dir](fr-metrics-004/) |
| FR-METRICS-005 | 5/5 | yes | yes | yes | [dir](fr-metrics-005/) |

## FR-MOD (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-MOD-001 | 5/5 | yes | yes | yes | [dir](fr-mod-001/) |
| FR-MOD-002 | 5/5 | yes | yes | yes | [dir](fr-mod-002/) |
| FR-MOD-003 | 5/5 | yes | yes | yes | [dir](fr-mod-003/) |
| FR-MOD-004 | 5/5 | yes | yes | yes | [dir](fr-mod-004/) |
| FR-MOD-005 | 5/5 | yes | yes | yes | [dir](fr-mod-005/) |

## FR-MUSIC (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-MUSIC-001 | 1/5 | no | no | no | [dir](fr-music-001/) |

## FR-NET (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-NET-001 | 5/5 | yes | yes | yes | [dir](fr-net-001/) |
| FR-NET-002 | 5/5 | yes | yes | yes | [dir](fr-net-002/) |
| FR-NET-003 | 5/5 | yes | yes | yes | [dir](fr-net-003/) |

## FR-PERF (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-PERF-001 | 5/5 | yes | yes | yes | [dir](fr-perf-001/) |
| FR-PERF-002 | 5/5 | yes | yes | yes | [dir](fr-perf-002/) |
| FR-PERF-003 | 5/5 | yes | yes | yes | [dir](fr-perf-003/) |
| FR-PERF-004 | 5/5 | yes | yes | yes | [dir](fr-perf-004/) |
| FR-PERF-005 | 5/5 | yes | yes | yes | [dir](fr-perf-005/) |

## FR-PROT (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-PROT-001 | 5/5 | yes | yes | yes | [dir](fr-prot-001/) |
| FR-PROT-002 | 5/5 | yes | yes | yes | [dir](fr-prot-002/) |
| FR-PROT-003 | 5/5 | yes | yes | yes | [dir](fr-prot-003/) |
| FR-PROT-004 | 5/5 | yes | yes | yes | [dir](fr-prot-004/) |
| FR-PROT-005 | 5/5 | yes | yes | yes | [dir](fr-prot-005/) |
| FR-PROT-006 | 5/5 | yes | yes | yes | [dir](fr-prot-006/) |

## FR-PROTO (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-PROTO-001 | 5/5 | yes | yes | yes | [dir](fr-proto-001/) |
| FR-PROTO-002 | 5/5 | yes | yes | yes | [dir](fr-proto-002/) |
| FR-PROTO-003 | 5/5 | yes | yes | yes | [dir](fr-proto-003/) |
| FR-PROTO-004 | 5/5 | yes | yes | yes | [dir](fr-proto-004/) |
| FR-PROTO-005 | 5/5 | yes | yes | yes | [dir](fr-proto-005/) |

## FR-REP (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-REP-001 | 5/5 | yes | yes | yes | [dir](fr-rep-001/) |

## FR-REPLAY (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-REPLAY-001 | 5/5 | yes | yes | yes | [dir](fr-replay-001/) |
| FR-REPLAY-002 | 5/5 | yes | yes | yes | [dir](fr-replay-002/) |

## FR-SAVE (25)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SAVE-001 | 5/5 | yes | yes | yes | [dir](fr-save-001/) |
| FR-SAVE-002 | 5/5 | yes | yes | yes | [dir](fr-save-002/) |
| FR-SAVE-003 | 5/5 | yes | yes | yes | [dir](fr-save-003/) |
| FR-SAVE-004 | 5/5 | yes | yes | yes | [dir](fr-save-004/) |
| FR-SAVE-005 | 5/5 | yes | yes | yes | [dir](fr-save-005/) |
| FR-SAVE-006 | 5/5 | yes | yes | yes | [dir](fr-save-006/) |
| FR-SAVE-007 | 5/5 | yes | yes | yes | [dir](fr-save-007/) |
| FR-SAVE-008 | 5/5 | yes | yes | yes | [dir](fr-save-008/) |
| FR-SAVE-009 | 5/5 | yes | yes | yes | [dir](fr-save-009/) |
| FR-SAVE-010 | 5/5 | yes | yes | yes | [dir](fr-save-010/) |
| FR-SAVE-011 | 5/5 | yes | yes | yes | [dir](fr-save-011/) |
| FR-SAVE-012 | 5/5 | yes | yes | yes | [dir](fr-save-012/) |
| FR-SAVE-013 | 5/5 | yes | yes | yes | [dir](fr-save-013/) |
| FR-SAVE-014 | 5/5 | yes | yes | yes | [dir](fr-save-014/) |
| FR-SAVE-015 | 5/5 | yes | yes | yes | [dir](fr-save-015/) |
| FR-SAVE-016 | 5/5 | yes | yes | yes | [dir](fr-save-016/) |
| FR-SAVE-017 | 5/5 | yes | yes | yes | [dir](fr-save-017/) |
| FR-SAVE-018 | 5/5 | yes | yes | yes | [dir](fr-save-018/) |
| FR-SAVE-019 | 5/5 | yes | yes | yes | [dir](fr-save-019/) |
| FR-SAVE-020 | 5/5 | yes | yes | yes | [dir](fr-save-020/) |
| FR-SAVE-021 | 5/5 | yes | yes | yes | [dir](fr-save-021/) |
| FR-SAVE-022 | 5/5 | yes | yes | yes | [dir](fr-save-022/) |
| FR-SAVE-023 | 5/5 | yes | yes | yes | [dir](fr-save-023/) |
| FR-SAVE-024 | 5/5 | yes | yes | yes | [dir](fr-save-024/) |
| FR-SAVE-025 | 5/5 | yes | yes | yes | [dir](fr-save-025/) |

## FR-SESS (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SESS-001 | 5/5 | yes | yes | yes | [dir](fr-sess-001/) |
| FR-SESS-002 | 5/5 | yes | yes | yes | [dir](fr-sess-002/) |
| FR-SESS-003 | 5/5 | yes | yes | yes | [dir](fr-sess-003/) |
| FR-SESS-004 | 5/5 | yes | yes | yes | [dir](fr-sess-004/) |
| FR-SESS-005 | 5/5 | yes | yes | yes | [dir](fr-sess-005/) |
| FR-SESS-006 | 5/5 | yes | yes | yes | [dir](fr-sess-006/) |

## FR-SESSION (33)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SESSION-001 | 5/5 | yes | yes | yes | [dir](fr-session-001/) |
| FR-SESSION-002 | 5/5 | yes | yes | yes | [dir](fr-session-002/) |
| FR-SESSION-003 | 5/5 | yes | yes | yes | [dir](fr-session-003/) |
| FR-SESSION-004 | 5/5 | yes | yes | yes | [dir](fr-session-004/) |
| FR-SESSION-005 | 5/5 | yes | yes | yes | [dir](fr-session-005/) |
| FR-SESSION-006 | 5/5 | yes | yes | yes | [dir](fr-session-006/) |
| FR-SESSION-007 | 5/5 | yes | yes | yes | [dir](fr-session-007/) |
| FR-SESSION-008 | 5/5 | yes | yes | yes | [dir](fr-session-008/) |
| FR-SESSION-009 | 5/5 | yes | yes | yes | [dir](fr-session-009/) |
| FR-SESSION-010 | 5/5 | yes | yes | yes | [dir](fr-session-010/) |
| FR-SESSION-011 | 5/5 | yes | yes | yes | [dir](fr-session-011/) |
| FR-SESSION-012 | 5/5 | yes | yes | yes | [dir](fr-session-012/) |
| FR-SESSION-013 | 5/5 | yes | yes | yes | [dir](fr-session-013/) |
| FR-SESSION-014 | 5/5 | yes | yes | yes | [dir](fr-session-014/) |
| FR-SESSION-015 | 5/5 | yes | yes | yes | [dir](fr-session-015/) |
| FR-SESSION-016 | 5/5 | yes | yes | yes | [dir](fr-session-016/) |
| FR-SESSION-017 | 5/5 | yes | yes | yes | [dir](fr-session-017/) |
| FR-SESSION-018 | 5/5 | yes | yes | yes | [dir](fr-session-018/) |
| FR-SESSION-019 | 5/5 | yes | yes | yes | [dir](fr-session-019/) |
| FR-SESSION-020 | 5/5 | yes | yes | yes | [dir](fr-session-020/) |
| FR-SESSION-021 | 5/5 | yes | yes | yes | [dir](fr-session-021/) |
| FR-SESSION-022 | 5/5 | yes | yes | yes | [dir](fr-session-022/) |
| FR-SESSION-023 | 5/5 | yes | yes | yes | [dir](fr-session-023/) |
| FR-SESSION-024 | 5/5 | yes | yes | yes | [dir](fr-session-024/) |
| FR-SESSION-025 | 5/5 | yes | yes | yes | [dir](fr-session-025/) |
| FR-SESSION-026 | 5/5 | yes | yes | yes | [dir](fr-session-026/) |
| FR-SESSION-027 | 5/5 | yes | yes | yes | [dir](fr-session-027/) |
| FR-SESSION-028 | 5/5 | yes | yes | yes | [dir](fr-session-028/) |
| FR-SESSION-029 | 5/5 | yes | yes | yes | [dir](fr-session-029/) |
| FR-SESSION-030 | 5/5 | yes | yes | yes | [dir](fr-session-030/) |
| FR-SESSION-031 | 5/5 | yes | yes | yes | [dir](fr-session-031/) |
| FR-SESSION-032 | 5/5 | yes | yes | yes | [dir](fr-session-032/) |
| FR-SESSION-033 | 5/5 | yes | yes | yes | [dir](fr-session-033/) |

## FR-SOC-CIV (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-CIV-001 | 5/5 | yes | yes | yes | [dir](fr-soc-civ-001/) |
| FR-SOC-CIV-002 | 5/5 | yes | yes | yes | [dir](fr-soc-civ-002/) |

## FR-SOC-COH (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-COH-001 | 5/5 | yes | yes | yes | [dir](fr-soc-coh-001/) |
| FR-SOC-COH-002 | 5/5 | yes | yes | yes | [dir](fr-soc-coh-002/) |
| FR-SOC-COH-003 | 5/5 | yes | yes | yes | [dir](fr-soc-coh-003/) |
| FR-SOC-COH-004 | 5/5 | yes | yes | yes | [dir](fr-soc-coh-004/) |

## FR-SOC-DET (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-DET-001 | 5/5 | yes | yes | yes | [dir](fr-soc-det-001/) |
| FR-SOC-DET-002 | 5/5 | yes | yes | yes | [dir](fr-soc-det-002/) |

## FR-SOC-FAC (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-FAC-001 | 5/5 | yes | yes | yes | [dir](fr-soc-fac-001/) |
| FR-SOC-FAC-002 | 5/5 | yes | yes | yes | [dir](fr-soc-fac-002/) |

## FR-SOC-HLT (5)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-HLT-001 | 5/5 | yes | yes | yes | [dir](fr-soc-hlt-001/) |
| FR-SOC-HLT-002 | 5/5 | yes | yes | yes | [dir](fr-soc-hlt-002/) |
| FR-SOC-HLT-003 | 5/5 | yes | yes | yes | [dir](fr-soc-hlt-003/) |
| FR-SOC-HLT-004 | 5/5 | yes | yes | yes | [dir](fr-soc-hlt-004/) |
| FR-SOC-HLT-005 | 5/5 | yes | yes | yes | [dir](fr-soc-hlt-005/) |

## FR-SOC-IDE (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-IDE-001 | 5/5 | yes | yes | yes | [dir](fr-soc-ide-001/) |
| FR-SOC-IDE-002 | 5/5 | yes | yes | yes | [dir](fr-soc-ide-002/) |
| FR-SOC-IDE-003 | 5/5 | yes | yes | yes | [dir](fr-soc-ide-003/) |
| FR-SOC-IDE-004 | 5/5 | yes | yes | yes | [dir](fr-soc-ide-004/) |
| FR-SOC-IDE-005 | 5/5 | yes | yes | yes | [dir](fr-soc-ide-005/) |
| FR-SOC-IDE-006 | 5/5 | yes | yes | yes | [dir](fr-soc-ide-006/) |

## FR-SOC-INS (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-INS-001 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-001/) |
| FR-SOC-INS-002 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-002/) |
| FR-SOC-INS-003 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-003/) |
| FR-SOC-INS-004 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-004/) |
| FR-SOC-INS-005 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-005/) |
| FR-SOC-INS-006 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-006/) |
| FR-SOC-INS-007 | 5/5 | yes | yes | yes | [dir](fr-soc-ins-007/) |

## FR-SOC-INT (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-INT-001 | 5/5 | yes | yes | yes | [dir](fr-soc-int-001/) |
| FR-SOC-INT-002 | 5/5 | yes | yes | yes | [dir](fr-soc-int-002/) |
| FR-SOC-INT-003 | 5/5 | yes | yes | yes | [dir](fr-soc-int-003/) |
| FR-SOC-INT-004 | 5/5 | yes | yes | yes | [dir](fr-soc-int-004/) |

## FR-SOC-INTG (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOC-INTG-001 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-001/) |
| FR-SOC-INTG-002 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-002/) |
| FR-SOC-INTG-003 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-003/) |
| FR-SOC-INTG-004 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-004/) |
| FR-SOC-INTG-005 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-005/) |
| FR-SOC-INTG-006 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-006/) |
| FR-SOC-INTG-007 | 5/5 | yes | yes | yes | [dir](fr-soc-intg-007/) |

## FR-SOCI (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-SOCI-001 | 5/5 | yes | yes | yes | [dir](fr-soci-001/) |
| FR-SOCI-002 | 5/5 | yes | yes | yes | [dir](fr-soci-002/) |
| FR-SOCI-003 | 5/5 | yes | yes | yes | [dir](fr-soci-003/) |
| FR-SOCI-004 | 5/5 | yes | yes | yes | [dir](fr-soci-004/) |
| FR-SOCI-005 | 5/5 | yes | yes | yes | [dir](fr-soci-005/) |
| FR-SOCI-006 | 5/5 | yes | yes | yes | [dir](fr-soci-006/) |

## FR-STOR (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-STOR-001 | 5/5 | yes | yes | yes | [dir](fr-stor-001/) |

## FR-TEST (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-TEST-001 | 5/5 | yes | yes | yes | [dir](fr-test-001/) |

## FR-THRY (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-THRY-001 | 5/5 | yes | yes | yes | [dir](fr-thry-001/) |
| FR-THRY-002 | 5/5 | yes | yes | yes | [dir](fr-thry-002/) |
| FR-THRY-003 | 5/5 | yes | yes | yes | [dir](fr-thry-003/) |
| FR-THRY-004 | 5/5 | yes | yes | yes | [dir](fr-thry-004/) |

## FR-UX (27)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-UX-001 | 5/5 | yes | yes | yes | [dir](fr-ux-001/) |
| FR-UX-002 | 5/5 | yes | yes | yes | [dir](fr-ux-002/) |
| FR-UX-003 | 5/5 | yes | yes | yes | [dir](fr-ux-003/) |
| FR-UX-004 | 5/5 | yes | yes | yes | [dir](fr-ux-004/) |
| FR-UX-005 | 5/5 | yes | yes | yes | [dir](fr-ux-005/) |
| FR-UX-006 | 5/5 | yes | yes | yes | [dir](fr-ux-006/) |
| FR-UX-007 | 5/5 | yes | yes | yes | [dir](fr-ux-007/) |
| FR-UX-008 | 5/5 | yes | yes | yes | [dir](fr-ux-008/) |
| FR-UX-009 | 5/5 | yes | yes | yes | [dir](fr-ux-009/) |
| FR-UX-010 | 5/5 | yes | yes | yes | [dir](fr-ux-010/) |
| FR-UX-011 | 5/5 | yes | yes | yes | [dir](fr-ux-011/) |
| FR-UX-012 | 5/5 | yes | yes | yes | [dir](fr-ux-012/) |
| FR-UX-013 | 5/5 | yes | yes | yes | [dir](fr-ux-013/) |
| FR-UX-014 | 5/5 | yes | yes | yes | [dir](fr-ux-014/) |
| FR-UX-015 | 5/5 | yes | yes | yes | [dir](fr-ux-015/) |
| FR-UX-016 | 5/5 | yes | yes | yes | [dir](fr-ux-016/) |
| FR-UX-017 | 5/5 | yes | yes | yes | [dir](fr-ux-017/) |
| FR-UX-018 | 5/5 | yes | yes | yes | [dir](fr-ux-018/) |
| FR-UX-019 | 5/5 | yes | yes | yes | [dir](fr-ux-019/) |
| FR-UX-020 | 5/5 | yes | yes | yes | [dir](fr-ux-020/) |
| FR-UX-021 | 5/5 | yes | yes | yes | [dir](fr-ux-021/) |
| FR-UX-022 | 5/5 | yes | yes | yes | [dir](fr-ux-022/) |
| FR-UX-023 | 5/5 | yes | yes | yes | [dir](fr-ux-023/) |
| FR-UX-024 | 5/5 | yes | yes | yes | [dir](fr-ux-024/) |
| FR-UX-025 | 5/5 | yes | yes | yes | [dir](fr-ux-025/) |
| FR-UX-026 | 5/5 | yes | yes | yes | [dir](fr-ux-026/) |
| FR-UX-027 | 5/5 | yes | yes | yes | [dir](fr-ux-027/) |

## FR-VAL (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-VAL-001 | 5/5 | yes | yes | yes | [dir](fr-val-001/) |

## FR-VIEWPORT (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| FR-VIEWPORT-001 | 1/5 | no | no | no | [dir](fr-viewport-001/) |

## NFR-C (7)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-C-01 | 1/5 | yes | no | no | [dir](nfr-c-01/) |
| NFR-C-02 | 1/5 | yes | no | no | [dir](nfr-c-02/) |
| NFR-C-03 | 1/5 | yes | no | no | [dir](nfr-c-03/) |
| NFR-C-04 | 1/5 | yes | no | no | [dir](nfr-c-04/) |
| NFR-C-05 | 1/5 | yes | no | no | [dir](nfr-c-05/) |
| NFR-C-06 | 1/5 | yes | no | no | [dir](nfr-c-06/) |
| NFR-C-07 | 1/5 | yes | no | no | [dir](nfr-c-07/) |

## NFR-CIV-ACC (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-ACC-001 | 1/5 | yes | no | no | [dir](nfr-civ-acc-001/) |
| NFR-CIV-ACC-002 | 1/5 | yes | no | no | [dir](nfr-civ-acc-002/) |
| NFR-CIV-ACC-003 | 1/5 | yes | no | no | [dir](nfr-civ-acc-003/) |
| NFR-CIV-ACC-004 | 1/5 | yes | no | no | [dir](nfr-civ-acc-004/) |

## NFR-CIV-AI (3)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-AI-001 | 2/5 | yes | no | yes | [dir](nfr-civ-ai-001/) |
| NFR-CIV-AI-002 | 2/5 | yes | no | yes | [dir](nfr-civ-ai-002/) |
| NFR-CIV-AI-003 | 2/5 | yes | no | yes | [dir](nfr-civ-ai-003/) |

## NFR-CIV-DET (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-DET-001 | 2/5 | yes | no | no | [dir](nfr-civ-det-001/) |
| NFR-CIV-DET-002 | 2/5 | yes | no | no | [dir](nfr-civ-det-002/) |
| NFR-CIV-DET-003 | 1/5 | yes | no | no | [dir](nfr-civ-det-003/) |
| NFR-CIV-DET-004 | 1/5 | yes | no | no | [dir](nfr-civ-det-004/) |

## NFR-CIV-DEV-HYGIENE (2)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-DEV-HYGIENE-001 | 1/5 | no | no | no | [dir](fr-nfr-civ-dev-hygiene-001/) |
| NFR-CIV-DEV-HYGIENE-001 | 1/5 | yes | no | no | [dir](nfr-civ-dev-hygiene-001/) |

## NFR-CIV-LEGENDS-CONFIG (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-LEGENDS-CONFIG-04 | 2/5 | yes | no | yes | [dir](nfr-civ-legends-config-04/) |

## NFR-CIV-LEGENDS-PERF (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-LEGENDS-PERF-01 | 2/5 | yes | no | yes | [dir](nfr-civ-legends-perf-01/) |

## NFR-CIV-LEGENDS-SCALE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-LEGENDS-SCALE-02 | 3/5 | yes | yes | yes | [dir](nfr-civ-legends-scale-02/) |

## NFR-CIV-MAINT (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-MAINT-001 | 2/5 | yes | no | yes | [dir](nfr-civ-maint-001/) |
| NFR-CIV-MAINT-002 | 2/5 | yes | no | yes | [dir](nfr-civ-maint-002/) |
| NFR-CIV-MAINT-003 | 2/5 | yes | no | yes | [dir](nfr-civ-maint-003/) |
| NFR-CIV-MAINT-004 | 2/5 | yes | no | yes | [dir](nfr-civ-maint-004/) |
| NFR-CIV-MAINT-005 | 2/5 | yes | no | yes | [dir](nfr-civ-maint-005/) |
| NFR-CIV-MAINT-006 | 2/5 | yes | no | yes | [dir](nfr-civ-maint-006/) |

## NFR-CIV-PERF (11)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-PERF-001 | 2/5 | yes | no | yes | [dir](nfr-civ-perf-001/) |
| NFR-CIV-PERF-002 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-002/) |
| NFR-CIV-PERF-003 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-003/) |
| NFR-CIV-PERF-004 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-004/) |
| NFR-CIV-PERF-005 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-005/) |
| NFR-CIV-PERF-006 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-006/) |
| NFR-CIV-PERF-007 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-007/) |
| NFR-CIV-PERF-008 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-008/) |
| NFR-CIV-PERF-900 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-900/) |
| NFR-CIV-PERF-901 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-901/) |
| NFR-CIV-PERF-902 | 3/5 | yes | no | yes | [dir](nfr-civ-perf-902/) |

## NFR-CIV-PORT (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-PORT-001 | 1/5 | no | no | no | [dir](fr-nfr-civ-port-001/) |
| NFR-CIV-PORT-001 | 1/5 | yes | no | no | [dir](nfr-civ-port-001/) |
| NFR-CIV-PORT-002 | 1/5 | no | no | no | [dir](fr-nfr-civ-port-002/) |
| NFR-CIV-PORT-002 | 1/5 | yes | no | no | [dir](nfr-civ-port-002/) |
| NFR-CIV-PORT-003 | 1/5 | no | no | no | [dir](fr-nfr-civ-port-003/) |
| NFR-CIV-PORT-003 | 1/5 | yes | no | no | [dir](nfr-civ-port-003/) |

## NFR-CIV-REL (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-REL-001 | 1/5 | yes | no | no | [dir](nfr-civ-rel-001/) |
| NFR-CIV-REL-002 | 1/5 | yes | no | no | [dir](nfr-civ-rel-002/) |
| NFR-CIV-REL-003 | 1/5 | yes | no | no | [dir](nfr-civ-rel-003/) |
| NFR-CIV-REL-004 | 1/5 | yes | no | no | [dir](nfr-civ-rel-004/) |

## NFR-CIV-SCALE-PERF (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-SCALE-PERF-900 | 1/5 | no | no | no | [dir](nfr-civ-scale-perf-900/) |

## NFR-CIV-SEC (4)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-CIV-SEC-001 | 1/5 | yes | no | no | [dir](nfr-civ-sec-001/) |
| NFR-CIV-SEC-002 | 1/5 | yes | no | no | [dir](nfr-civ-sec-002/) |
| NFR-CIV-SEC-003 | 1/5 | yes | no | no | [dir](nfr-civ-sec-003/) |
| NFR-CIV-SEC-004 | 1/5 | yes | no | no | [dir](nfr-civ-sec-004/) |

## NFR-O (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-O-01 | 1/5 | yes | no | no | [dir](nfr-o-01/) |
| NFR-O-02 | 1/5 | yes | no | no | [dir](nfr-o-02/) |
| NFR-O-03 | 1/5 | yes | no | no | [dir](nfr-o-03/) |
| NFR-O-04 | 1/5 | yes | no | no | [dir](nfr-o-04/) |
| NFR-O-05 | 1/5 | yes | no | no | [dir](nfr-o-05/) |
| NFR-O-06 | 1/5 | yes | no | no | [dir](nfr-o-06/) |

## NFR-P (8)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-P-01 | 1/5 | yes | no | no | [dir](nfr-p-01/) |
| NFR-P-02 | 1/5 | yes | no | no | [dir](nfr-p-02/) |
| NFR-P-03 | 1/5 | yes | no | no | [dir](nfr-p-03/) |
| NFR-P-04 | 1/5 | yes | no | no | [dir](nfr-p-04/) |
| NFR-P-05 | 1/5 | yes | no | no | [dir](nfr-p-05/) |
| NFR-P-06 | 1/5 | yes | no | no | [dir](nfr-p-06/) |
| NFR-P-07 | 1/5 | yes | no | no | [dir](nfr-p-07/) |
| NFR-P-08 | 1/5 | yes | no | no | [dir](nfr-p-08/) |

## NFR-R (6)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-R-01 | 2/5 | yes | no | no | [dir](nfr-r-01/) |
| NFR-R-02 | 2/5 | yes | no | no | [dir](nfr-r-02/) |
| NFR-R-03 | 2/5 | yes | no | no | [dir](nfr-r-03/) |
| NFR-R-04 | 2/5 | yes | no | no | [dir](nfr-r-04/) |
| NFR-R-05 | 2/5 | yes | no | no | [dir](nfr-r-05/) |
| NFR-R-06 | 2/5 | yes | no | no | [dir](nfr-r-06/) |

## NFR-S (12)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-S-01 | 1/5 | no | no | no | [dir](fr-nfr-s-01/) |
| NFR-S-01 | 1/5 | yes | no | no | [dir](nfr-s-01/) |
| NFR-S-02 | 1/5 | no | no | no | [dir](fr-nfr-s-02/) |
| NFR-S-02 | 1/5 | yes | no | no | [dir](nfr-s-02/) |
| NFR-S-03 | 1/5 | no | no | no | [dir](fr-nfr-s-03/) |
| NFR-S-03 | 1/5 | yes | no | no | [dir](nfr-s-03/) |
| NFR-S-04 | 1/5 | no | no | no | [dir](fr-nfr-s-04/) |
| NFR-S-04 | 1/5 | yes | no | no | [dir](nfr-s-04/) |
| NFR-S-05 | 1/5 | no | no | no | [dir](fr-nfr-s-05/) |
| NFR-S-05 | 1/5 | yes | no | no | [dir](nfr-s-05/) |
| NFR-S-06 | 1/5 | no | no | no | [dir](fr-nfr-s-06/) |
| NFR-S-06 | 1/5 | yes | no | no | [dir](nfr-s-06/) |

## NFR-SCALE (1)

| FR ID | Artifacts | Spec | ADR | Research | Directory |
|-------|-----------|------|-----|----------|-----------|
| NFR-SCALE-02 | 3/5 | yes | yes | yes | [dir](nfr-scale-02/) |

