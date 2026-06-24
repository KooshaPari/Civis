# Spec-Only Triage — Batch 2

Source: rows `61-120` in `docs/audits/fr-matrix.json` (matrix order), skipping batch 1 first 60 IDs

Total rows: `60`  
Verdict counts:

- `BUILD-NEXT`: `20`
- `DEFER`: `26`
- `ARCHIVE`: `14`

Legend for notes:

- `ARCHIVE` notes include superseding ID where explicitly present in spec text.
- `BUILD-NEXT` notes include matching parity benchmark top-20 gap where relevant.

| # | FR ID | Epic | Spec | Verdict | Notes |
|---|---|---|---|---|---|
| 61 | FR-CIV-LLM-005 | FR-CIV-LLM | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Cache-policy details are infra depth and can follow core gameplay parity delivery. |
| 62 | FR-CIV-LLM-006 | FR-CIV-LLM | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Prompt safety/branding policy is a quality hardening track, not a first-pass blocker. |
| 63 | FR-CIV-PBR-001 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 parity gap #3 (`Modern GFX`) — GI/SSR baseline visual parity family starts here. |
| 64 | FR-CIV-PBR-002 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; supports realistic material/lighting parity without blocking core sim architecture. |
| 65 | FR-CIV-PBR-003 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; volumetric and reflection behavior are core rendering milestones. |
| 66 | FR-CIV-PBR-004 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; post FX closure is needed to complete visual-complexity lane. |
| 67 | FR-CIV-PBR-005 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; expected for parity against CS2/Anno visual class references. |
| 68 | FR-CIV-PBR-006 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; depth pass that unlocks coherent advanced materials plan. |
| 69 | FR-CIV-PBR-007 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; required before advanced rendering polish can be considered complete. |
| 70 | FR-CIV-PBR-008 | FR-CIV-PBR | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #3; closes remaining cinematic pipeline spec debt. |
| 71 | FR-CIV-PSYCHE-004 | FR-CIV-PSYCHE | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Early psyche loop behavior is prelude polish and not in the immediate parity closure set. |
| 72 | FR-CIV-PSYCHE-007 | FR-CIV-PSYCHE | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Personality edge-case policy work is important but outside the first visible 1.0 closure scope. |
| 73 | FR-CIV-PSYCHE-008 | FR-CIV-PSYCHE | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Non-blocking lore/quality layer for long-lead narrative simulation depth. |
| 74 | FR-CIV-SAVE-001 | FR-CIV-SAVE | `docs/traceability/civis-tracelinks.md` | BUILD-NEXT | Top-20 gap #11 (Save-slot UI + browser) — visible UX layer required for parity UX closure. |
| 75 | FR-CIV-SAVE-002 | FR-CIV-SAVE | `docs/traceability/civis-tracelinks.md` | BUILD-NEXT | Top-20 gap #11; save-slot workflow UX required in user-facing loop. |
| 76 | FR-CIV-SAVE-003 | FR-CIV-SAVE | `docs/traceability/civis-tracelinks.md` | BUILD-NEXT | Top-20 gap #11; cloud-sync/multi-slot behavior should be tracked as shipping scope. |
| 77 | FR-CIV-SAVE-004 | FR-CIV-SAVE | `docs/traceability/civis-tracelinks.md` | BUILD-NEXT | Top-20 gap #11; close spec debt in save lifecycle path needed for parity completeness. |
| 78 | FR-CIV-SCALE-001 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15 (Plausible large-world streaming + chunk IO); core residency limit remains 1.0-critical. |
| 79 | FR-CIV-SCALE-002 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; world-size scalability contract should continue with streaming roadmap. |
| 80 | FR-CIV-SCALE-003 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; LOD ring behavior is needed for performant large-window loop. |
| 81 | FR-CIV-SCALE-004 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; sim-LOD gestalt must be locked for stable large-terrain render scale. |
| 82 | FR-CIV-SCALE-005 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; prefetch contract should reduce frame-time spikes in travel windows. |
| 83 | FR-CIV-SCALE-006 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; chunk residency controls are part of baseline large-world parity. |
| 84 | FR-CIV-SCALE-007 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; streaming boundaries are not just perf, they are route-planning prerequisites. |
| 85 | FR-CIV-SCALE-008 | FR-CIV-SCALE | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #15; final large-world stream spec for parity continuity. |
| 86 | FR-CIV-SPECIES-012 | FR-CIV-SPECIES | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Species micro-behavior expansion is a quality layer with no direct top-20 closure coupling. |
| 87 | FR-CIV-SPECIES-013 | FR-CIV-SPECIES | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Data and behavioral detail for this tranche can be deferred behind core loop completion. |
| 88 | FR-CIV-SPECIES-014 | FR-CIV-SPECIES | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Long-tail simulation elaboration; not required for immediate parity slice. |
| 89 | FR-CIV-SPECIES-015 | FR-CIV-SPECIES | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Not in current parity closure set; defer to post-1.0 polish stream. |
| 90 | FR-CIV-SPECIES-016 | FR-CIV-SPECIES | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Non-essential expansion of species model for first hardening pass. |
| 91 | FR-CIV-SPECIES-017 | FR-CIV-SPECIES | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Deferred as a detail track once core sim and streaming gates clear. |
| 92 | FR-CIV-TACTICS-001- | FR-CIV-TACTICS | `docs/traceability/fr-3d-matrix.md` | ARCHIVE | Superseded by `FR-CIV-TACTICS-001-int` implementation trace in parity benchmark baseline. |
| 93 | FR-CLIM-001 | FR-CLIM | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-CLIMATE-001` in current CIV-0102 alignment. |
| 94 | FR-CLIM-002 | FR-CLIM | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-CLIMATE-002`; this prefix is legacy in traceability naming. |
| 95 | FR-CLIM-003 | FR-CLIM | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-CLIMATE-003`; older naming can be archived. |
| 96 | FR-CLIM-004 | FR-CLIM | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by newer CIV climate implementation contract family. |
| 97 | FR-CLIM-005 | FR-CLIM | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Legacy climate cascade row; superseded by `FR-CIV-CLIMATE-*` migration target. |
| 98 | FR-CLIM-006 | FR-CLIM | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Legacy adaptation row; covered in newer CIV climate specs. |
| 99 | FR-CORE-008 | FR-CORE | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Engine-world modeling detail that can follow parity closure workstreams. |
| 100 | FR-CORE-009 | FR-CORE | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Coordinate representation refactor is foundational but not in visible parity closure now. |
| 101 | FR-CORE-010 | FR-CORE | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Numeric format hardening is valuable but non-1.0 gating versus functional parity gaps. |
| 102 | FR-DIPL-001 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-DIPLO-001` in newer diplomacy surface contract. |
| 103 | FR-DIPL-002 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-DIPLO-002`; legacy prefix should be retired. |
| 104 | FR-DIPL-003 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-DIPLO-003`; archive to avoid duplicate scope. |
| 105 | FR-DIPL-004 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `FR-CIV-DIPLO-004` in active diplomacy contract set. |
| 106 | FR-DIPL-005 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by active CIV diplomacy IDs; retained only as historical trace. |
| 107 | FR-DIPL-006 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Legacy diplomacy eventing row; replaced by FR-CIV-DIPLO family. |
| 108 | FR-DIPL-007 | FR-DIPL | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Legacy ID superseded by `FR-CIV-DIPLO-007`; no independent build-next demand. |
| 109 | FR-DOC-001 | FR-DOC | `docs/FR_DETAILED.md` | DEFER | Documentation completeness is ongoing; not in top-20 gameplay parity close. |
| 110 | FR-ECON-006 | FR-ECON | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Economy telemetry depth is downstream of the primary chain visualizer and market loop. |
| 111 | FR-ECON-007 | FR-ECON | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Trade transfer detail is useful but can follow core market contract stabilization. |
| 112 | FR-ECON-008 | FR-ECON | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | District collapse edge-case is behavior polish and not immediate closure requirement. |
| 113 | FR-ECON-009 | FR-ECON | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Subsistence fallback model can be staged after core economy throughput parity. |
| 114 | FR-ECON-010 | FR-ECON | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Economic extension row can be deferred to follow active chain work. |
| 115 | FR-INST-001 | FR-INST | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Institution taxonomy is non-blocking versus simulation-first parity scope. |
| 116 | FR-INST-002 | FR-INST | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Capturing dynamics can be deferred to a later governance hardening cycle. |
| 117 | FR-INST-003 | FR-INST | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Governance event thresholds are detail work after milestone closure. |
| 118 | FR-INST-004 | FR-INST | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Transition model is desirable but not in first parity closure. |
| 119 | FR-INST-005 | FR-INST | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Metrics persistence for institutions is a follow-up observability track. |
| 120 | FR-INST-006 | FR-INST | `docs/traceability/TRACEABILITY_MATRIX.md` | DEFER | Lifecycle-coupled behavior coupling can be sequenced after core economy/governance basics. |
