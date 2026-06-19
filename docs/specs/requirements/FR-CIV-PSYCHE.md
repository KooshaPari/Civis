# FR-CIV-PSYCHE — Psyche, Social Networks & Emergent History

**Owner:** Life-Sim Lead. **Source gap:** Feature Matrix §2 (psyche, social networks, emergent histories = **BLIND**; needs INCOMPLETE).
**Reference bar:** RimWorld thoughts/moods/mental-breaks; Dwarf Fortress relationships + legends/myths (gold standard for emergent history); RimWorld AI Storyteller pacing (study, do NOT hardcode a scripted director).
**Emergence note:** **[EMERGENT]** — psyche, bonds, grudges, beliefs, and recorded history MUST arise from Layer-0 (genomics→temperament) + agent experience over kinship/contact networks. NO hardcoded personality enums, NO scripted story beats. The "storyteller" is an analysis/UI lens over emergent events, not an authored event injector.

## Requirements

| ID | Requirement | Acceptance Criteria | Tag |
|---|---|---|---|
| FR-CIV-PSYCHE-900 | Each agent SHALL carry an emergent psyche state: drives, temperament (genomically seeded), mood, and bounded memory. | Temperament derives deterministically from DNA (`civ-genetics`→`civ-species`); identical DNA+history → identical psyche; no enum. | EMERGENT |
| FR-CIV-PSYCHE-901 | Mood SHALL be a measured function of need satisfaction, recent memory, environment, and social events (not a fixed table the player edits). | Mood recomputed each Hot tick from contributing measured factors; factors inspectable (FR-CIV-INSPECT-901). | EMERGENT |
| FR-CIV-PSYCHE-910 | A kinship + contact social graph SHALL emerge from co-location, reproduction, and interaction frequency. | Edges form/decay from sim events; bonds/grudges are weighted edges; queryable per agent. | EMERGENT |
| FR-CIV-PSYCHE-911 | Beliefs/norms SHALL drift and diffuse across the social graph (cultural evolution), producing ideology/culture clusters. | Clusters measurable as graph communities; drift deterministic under fixed seed; feeds INFOVIEW-913. | EMERGENT |
| FR-CIV-PSYCHE-912 | Language SHALL emerge and drift over contact networks; dialects/creoles arise from contact between divergent groups. | Distinct language regions detectable; contact zones show mixing; no authored language table. | EMERGENT |
| FR-CIV-PSYCHE-920 | The engine SHALL record significant emergent events into a queryable chronicle (births, deaths, migrations, conflicts, foundings, first-contacts). | Chronicle is append-only, deterministic, replay-stable; queryable per agent/place/polity → "legends mode". | EMERGENT+UI |
| FR-CIV-PSYCHE-921 | Psyche/social/history SHALL run LOD-tiered (full at Hot, statistical at Cold). | Cold agents use aggregate models; promoting Cold→Hot reconstructs plausibly without breaking determinism contract (NFR-CIV-DET). | EMERGENT+NFR |

**Validation:** determinism test (same seed → identical psyche/graph/chronicle); cluster-detection test; LOD promotion/demotion round-trip test.
