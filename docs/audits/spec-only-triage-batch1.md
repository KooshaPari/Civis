# Spec-Only Triage — Batch 1

Source: first 60 `SPEC-ONLY` rows in `docs/audits/fr-matrix.json` (matrix order)

Total rows: `60`  
Verdict counts:

- `BUILD-NEXT`: `36`
- `DEFER`: `14`
- `ARCHIVE`: `10`

Legend for notes:

- `ARCHIVE` notes include superseding ID where explicitly present in spec text.
- `BUILD-NEXT` notes include matching parity benchmark top-20 gap where relevant.

| # | FR ID | Epic | Spec | Verdict | Notes |
|---|---|---|---|---|---|
| 1 | FR-AI-001 | FR-AI | `docs/FR.md` | DEFER | Foundation AI scope not mapped to 1.0 parity gaps; broad spec without implementation plan. |
| 2 | FR-AI-002 | FR-AI | `docs/FR.md` | DEFER | Faction AI planning remains post-1.0 relative to parity top-20. |
| 3 | FR-AI-003 | FR-AI | `docs/FR.md` | DEFER | Core ideology AI behavior is not directly required in top-20 parity closure. |
| 4 | FR-AI-004 | FR-AI | `docs/FR.md` | DEFER | Event-system breadth is post-1.0 polish versus shipped baseline. |
| 5 | FR-AI-005 | FR-AI | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0400` AI cap/decision planning line. |
| 6 | FR-AI-006 | FR-AI | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0400` replay-decision telemetry path. |
| 7 | FR-AI-007 | FR-AI | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0400` MCTS budgeting scope. |
| 8 | FR-ASSET-001 | FR-ASSET | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0600` 2D asset pipeline model. |
| 9 | FR-ASSET-002 | FR-ASSET | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0600` atlas pipeline model. |
| 10 | FR-ASSET-003 | FR-ASSET | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0600` event emission model. |
| 11 | FR-ASSET-004 | FR-ASSET | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0601` glTF lazy-load direction. |
| 12 | FR-AUD-001 | FR-AUD | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0800` adaptive music bus model. |
| 13 | FR-AUD-002 | FR-AUD | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0800` adaptive mix spec. |
| 14 | FR-AUD-003 | FR-AUD | `docs/traceability/TRACEABILITY_MATRIX.md` | ARCHIVE | Superseded by `CIV-0800` event-to-SFX spec. |
| 15 | FR-CIV-ARCH-001 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 parity gap #12 (procedural build/parity variety); canonical architecture direction needed. |
| 16 | FR-CIV-ARCH-002 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; required to keep BuildingGraph/grammar parity stable with WFC. |
| 17 | FR-CIV-ARCH-003 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; supports culture-era style variation. |
| 18 | FR-CIV-ARCH-004 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; deterministic parcel style generation is core next-step architecture work. |
| 19 | FR-CIV-ARCH-005 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; building output determinism and mesh continuity are required for playable city-loop. |
| 20 | FR-CIV-ARCH-006 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; freehand/grammar parity is required to close tool-to-structure consistency. |
| 21 | FR-CIV-ARCH-007 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; primitive-to-style preset path is a needed v1 visual/simulation deliverable. |
| 22 | FR-CIV-ARCH-008 | FR-CIV-ARCH | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #12; façade divergence test closes architecture polish gap for v1. |
| 23 | FR-CIV-CA-001 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14 (ecology/creature & weather depth). |
| 24 | FR-CIV-CA-002 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; CA thermodynamic core is missing for ecology parity. |
| 25 | FR-CIV-CA-003 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; water retention mechanics directly affect world-life behavior. |
| 26 | FR-CIV-CA-004 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; weather/evaporation depth contributes to ecology bar. |
| 27 | FR-CIV-CA-005 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; heat/phase behavior is required simulation parity. |
| 28 | FR-CIV-CA-006 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; stable boundary contracts needed before large-scale ecology simulation. |
| 29 | FR-CIV-CA-007 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; sea-level and hydro dynamics are gameplay-critical. |
| 30 | FR-CIV-CA-008 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; required for performance-sane CA stepping. |
| 31 | FR-CIV-CA-009 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; integration into phase loop required for first 1.0 ecology slice. |
| 32 | FR-CIV-CA-010 | FR-CIV-CA | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #14; fixed micro-fixture coverage needed to unblock ecological depth. |
| 33 | FR-CIV-DIPLO-004 | FR-CIV-DIPLO | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #18 (first-class diplomacy UI/data); treaty concession model is prerequisite. |
| 34 | FR-CIV-DIPLO-005 | FR-CIV-DIPLO | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #18; formal transcript structure needed before diplomacy UX surfaces. |
| 35 | FR-CIV-DIPLO-006 | FR-CIV-DIPLO | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #18; LLM exclusion rule enables player-readable diplomacy integrity. |
| 36 | FR-CIV-DIPLO-007 | FR-CIV-DIPLO | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #18; polity identity model is required for stable war-goal browsing. |
| 37 | FR-CIV-DIPLO-008 | FR-CIV-DIPLO | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #18; treaty accept/reject/counter behavior needs test coverage for parity. |
| 38 | FR-CIV-ECON-015 | FR-CIV-ECON | `PRD.md` | BUILD-NEXT | Top-20 gap #1 (city-scale chain simulation); references EC/ON chain contract family. |
| 39 | FR-CIV-LANG-001 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Language pipeline is non-blocking for 1.0 loop and likely post-1.0 polish. |
| 40 | FR-CIV-LANG-002 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | UI/atlas localization plumbing is not currently in parity top-20 closure. |
| 41 | FR-CIV-LANG-003 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Phonology core is foundational deep-sim work; not required for first parity slice. |
| 42 | FR-CIV-LANG-004 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Lexicon generation is a long-lead feature and post-1.0 in scope. |
| 43 | FR-CIV-LANG-005 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Civil language drift/cross-contact split can be deferred to v2. |
| 44 | FR-CIV-LANG-006 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Orthography/lag model is quality work outside immediate parity blockers. |
| 45 | FR-CIV-LANG-007 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Glyph pipeline implementation is non-blocking relative to MVP roadmap. |
| 46 | FR-CIV-LANG-008 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Storage model for linguistic drift can wait until after gameplay parity MVP. |
| 47 | FR-CIV-LANG-009 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Non-LLM generation constraint is not a 1.0 gameplay blocker. |
| 48 | FR-CIV-LANG-010 | FR-CIV-LANG | `FUNCTIONAL_REQUIREMENTS.md` | DEFER | Legal isolation contingency belongs to cleanup/refactor track after core parity. |
| 49 | FR-CIV-LEGENDS-001 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9 (storyteller/event narrative layer). |
| 50 | FR-CIV-LEGENDS-002 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; rumor/chronicle logic supports narrative parity objective. |
| 51 | FR-CIV-LEGENDS-003 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; rumor mutation pipeline adds depth expected by high-impact player loop. |
| 52 | FR-CIV-LEGENDS-004 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; literature surface quality is part of storyteller closure. |
| 53 | FR-CIV-LEGENDS-005 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; legend graph compatibility keeps narratives coherent across systems. |
| 54 | FR-CIV-LEGENDS-006 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; graceful gap handling is required for production storyteller UX. |
| 55 | FR-CIV-LEGENDS-007 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; register/UI separation is needed for narrative readability. |
| 56 | FR-CIV-LEGENDS-008 | FR-CIV-LEGENDS | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Top-20 gap #9; culturally variant naming is needed for immersion parity. |
| 57 | FR-CIV-LLM-001 | FR-CIV-LLM | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Player-visible narrative loops now have a cache contract lock for deterministic flavor delivery and replay-friendly AI affordances. |
| 58 | FR-CIV-LLM-002 | FR-CIV-LLM | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Prompt tags/hashes are part of visible generation replayability and now have direct coverage in tests. |
| 59 | FR-CIV-LLM-003 | FR-CIV-LLM | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Call-budget/env caps are now covered to keep AI throughput behavior deterministic under load. |
| 60 | FR-CIV-LLM-004 | FR-CIV-LLM | `FUNCTIONAL_REQUIREMENTS.md` | BUILD-NEXT | Allowed-use boundaries are now asserted to avoid silent cross-mode provider misuse during gameplay. |
