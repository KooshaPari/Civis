# FR Coverage Audit

**Generated:** 2026-09-20  
**Source inventory:** `docs/audits/_id_inventory_v3.json`  
**Total IDs scanned:** 1455

## Status legend

| Status | Meaning |
|--------|---------|
| `COVERED` | spec/trace + code + real test all present |
| `STUB-TEST-ONLY` | spec/trace + code present, but the only test reference is a placeholder (TDD-red stub or legacy 'Epic: auto-generated'). The test file exists but does not exercise anything FR-specific, so treat coverage as unverified. Fan-out agents target this bucket. |
| `TEST-NO-CODE-REF` | spec/trace + real test present, but no ID-tagged code reference. A test exercises the requirement yet no source file carries the ID, so the implementation cannot be located from the ID alone. |
| `IMPL-NO-TEST` | spec/trace + code present, no test reference |
| `SPEC-ONLY` | spec/trace present, no implementing code found |
| `CODE-ONLY-no-spec` | code present, no spec/traceability reference |

## Summary

| Status | Count | % |
|--------|------:|--:|
| `COVERED` | 960 | 66.0 |
| `STUB-TEST-ONLY` | 10 | 0.7 |
| `TEST-NO-CODE-REF` | 155 | 10.7 |
| `IMPL-NO-TEST` | 95 | 6.5 |
| `SPEC-ONLY` | 234 | 16.1 |
| `CODE-ONLY-no-spec` | 1 | 0.1 |
| **Total** | **1455** | **100.0** |

## Coverage by epic

| Epic | Total | COVERED | STUB-TEST-ONLY | TEST-NO-CODE-REF | IMPL-NO-TEST | SPEC-ONLY | CODE-ONLY-no-spec |
|------|------:|---------|----------------|------------------|--------------|-----------|-------------------|
| FR-AI | 7 | 7 | 0 | 0 | 0 | 0 | 0 |
| FR-API | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-ASSET | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-ASSET-PIPELINE | 2 | 0 | 0 | 0 | 2 | 0 | 0 |
| FR-AUD | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV | 15 | 14 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-0001-TICK | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-3D | 16 | 16 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ACCESS | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ACT | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ACTOR | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ACTOR-001-LIFECYCLE | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-AGENTS | 17 | 17 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-AGGRESSION | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-AI | 15 | 15 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ARCH | 9 | 8 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-ARCH-A | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ARCH-B | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ARCH-C | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ARCH-D | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ARCH-NOSVG | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ASSET | 20 | 8 | 0 | 0 | 0 | 12 | 0 |
| FR-CIV-ASSET-MANI | 2 | 0 | 0 | 0 | 2 | 0 | 0 |
| FR-CIV-ASSET-QUAL | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-AUDIO | 12 | 8 | 0 | 0 | 0 | 4 | 0 |
| FR-CIV-BELIEF | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-BEVY | 21 | 12 | 0 | 0 | 8 | 1 | 0 |
| FR-CIV-BIO | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-BRUSH | 13 | 13 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-BUILD | 14 | 14 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-CA | 11 | 10 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-CARAVAN | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-CLIENT | 3 | 1 | 0 | 0 | 2 | 0 | 0 |
| FR-CIV-CLIENT-GODOT | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-CLIMATE | 7 | 7 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-COHESION | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-CONSTRUCTION | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-CONTENT | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-CORE | 21 | 20 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-CORE-DET | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-CULT | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-CULTURE | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-DET | 7 | 1 | 0 | 6 | 0 | 0 | 0 |
| FR-CIV-DIFFUSION | 16 | 16 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-DIPLO | 16 | 16 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-DIPLO-001-RELATIONS | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-DIPLO-002-SHADOW | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-DIPLOMACY | 2 | 1 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-ECON | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ECON-001-MARKET | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ECON-002-JOULE | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-ECON-FOCUS | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-EMERG | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-EMERGE-DASH | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-EMERGENCE | 25 | 10 | 0 | 0 | 0 | 15 | 0 |
| FR-CIV-EMERGENCE-N10 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-N11 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-N12 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-N13 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-RELIGION | 2 | 0 | 0 | 0 | 0 | 2 | 0 |
| FR-CIV-EMERGENT-MIGRATION | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-ENGINE-INT | 10 | 10 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ENGINE-REPLAY | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-ERA | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-FAMINE | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-FEST | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-FOG | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-GAME | 3 | 1 | 0 | 0 | 2 | 0 | 0 |
| FR-CIV-GENETICS | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-GENETICS-SEED | 3 | 0 | 0 | 0 | 3 | 0 | 0 |
| FR-CIV-GEO | 10 | 0 | 0 | 0 | 0 | 10 | 0 |
| FR-CIV-GODOT-ATTACH | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-GODOT-F3D0 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-GODOT-UX | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-GODTOOL | 8 | 2 | 0 | 5 | 1 | 0 | 0 |
| FR-CIV-GOV | 8 | 7 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-HUD | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-IDEOLOGY | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-INFOVIEW | 20 | 20 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-INFRA | 13 | 13 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-INSPECT | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-INSTITUTIONS | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-INT | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-L10N | 4 | 2 | 0 | 0 | 2 | 0 | 0 |
| FR-CIV-L5 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LANG | 10 | 10 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LAWS | 10 | 10 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS | 9 | 8 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-LEGENDS-BROWSER | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-CAUSAL | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-CONFIG | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-GAP | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-GRAPH | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-INGEST | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-INSPECT | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-NARRATOR | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PERF | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PERSIST | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PRESIM | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PRODUCER | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-QUERY | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-RESOLVE | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-SCALE | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-SIG | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-LIFE | 20 | 19 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-LLM | 6 | 0 | 0 | 6 | 0 | 0 | 0 |
| FR-CIV-MARKET | 8 | 8 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-MCP | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-METRICS | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-METRICS-001-TIMESERIES | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-MIGRATION | 5 | 0 | 0 | 0 | 5 | 0 | 0 |
| FR-CIV-MOD | 21 | 21 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-NEEDS-DECAY | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-NOTIFY | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PBR | 11 | 10 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-PERF | 20 | 1 | 0 | 19 | 0 | 0 | 0 |
| FR-CIV-PERF-BUILD | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PERF-RT | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PERF-WEB | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PLANET | 12 | 12 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-POLITY | 8 | 8 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PROTO | 15 | 15 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PROTO3D | 19 | 19 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-PSYCHE | 29 | 12 | 0 | 0 | 0 | 17 | 0 |
| FR-CIV-PSYCHE-N11 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-QOL | 14 | 0 | 0 | 14 | 0 | 0 | 0 |
| FR-CIV-REL | 5 | 3 | 0 | 1 | 1 | 0 | 0 |
| FR-CIV-RELIGION | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RENDER | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RES | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RESEARCH | 13 | 13 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-001-SCENARIO | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-002-SNAPSHOT | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-003-EXPORT | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-004-REPLAY | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-ROAD | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RTS | 15 | 0 | 0 | 15 | 0 | 0 | 0 |
| FR-CIV-RTS-NATION | 2 | 0 | 0 | 2 | 0 | 0 | 0 |
| FR-CIV-RTS-RENDER | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-RTS-ZOOM | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-SAVE | 4 | 2 | 0 | 0 | 2 | 0 | 0 |
| FR-CIV-SCALE | 8 | 8 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-SERVER | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-SERVER-001-WS | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-CIV-SERVER-002-PROTO | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-SOCIAL | 2 | 0 | 0 | 0 | 0 | 2 | 0 |
| FR-CIV-SOCIAL-001-INSTITUTIONS | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-CIV-SOCIAL-002-IDEOLOGY | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-SPECIES | 48 | 12 | 0 | 0 | 0 | 36 | 0 |
| FR-CIV-TACTICS | 63 | 47 | 0 | 15 | 1 | 0 | 0 |
| FR-CIV-TECH | 21 | 0 | 0 | 0 | 0 | 21 | 0 |
| FR-CIV-TERRAIN | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-TEST | 7 | 0 | 0 | 7 | 0 | 0 | 0 |
| FR-CIV-TRAFFIC-LANE | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-UI | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-UNREST | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-UX | 7 | 7 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-VEHICLE | 26 | 26 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-VERIFY | 10 | 10 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-VOXEL | 18 | 18 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-VOXEL-DIRTY | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-WAR | 15 | 14 | 0 | 0 | 0 | 1 | 0 |
| FR-CIV-WAR-001-UNITS | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-WAR-002-COMBAT | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-CIV-WARFARE | 4 | 0 | 0 | 0 | 4 | 0 | 0 |
| FR-CIV-WEB | 9 | 5 | 0 | 0 | 0 | 4 | 0 |
| FR-CLIENT | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-CLIM | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-CORE | 10 | 8 | 0 | 2 | 0 | 0 | 0 |
| FR-DET | 7 | 0 | 0 | 7 | 0 | 0 | 0 |
| FR-DIP | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-DIPL | 7 | 7 | 0 | 0 | 0 | 0 | 0 |
| FR-DOC | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-ECO | 10 | 0 | 0 | 10 | 0 | 0 | 0 |
| FR-ECON | 10 | 10 | 0 | 0 | 0 | 0 | 0 |
| FR-ECON-EMERGE | 3 | 0 | 0 | 0 | 3 | 0 | 0 |
| FR-EMG | 24 | 9 | 0 | 0 | 15 | 0 | 0 |
| FR-FR-CORE | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-GUARD | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-INST | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-INT | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-LANGUAGE | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FR-LOD | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-MET | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-METRICS | 5 | 3 | 0 | 2 | 0 | 0 | 0 |
| FR-MOD | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-MUSIC | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-NET | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| FR-NFR-CIV-DET | 3 | 0 | 0 | 0 | 0 | 3 | 0 |
| FR-NFR-CIV-DEV-HYGIENE | 1 | 0 | 1 | 0 | 0 | 0 | 0 |
| FR-NFR-CIV-PERF | 11 | 0 | 0 | 0 | 0 | 10 | 1 |
| FR-NFR-CIV-PORT | 3 | 0 | 3 | 0 | 0 | 0 | 0 |
| FR-NFR-R | 6 | 0 | 0 | 0 | 0 | 6 | 0 |
| FR-NFR-S | 6 | 0 | 6 | 0 | 0 | 0 | 0 |
| FR-PERF | 5 | 5 | 0 | 0 | 0 | 0 | 0 |
| FR-PROT | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-PROTO | 5 | 0 | 0 | 5 | 0 | 0 | 0 |
| FR-REP | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| FR-REPLAY | 2 | 1 | 0 | 1 | 0 | 0 | 0 |
| FR-SAVE | 25 | 15 | 0 | 0 | 0 | 10 | 0 |
| FR-SESS | 6 | 6 | 0 | 0 | 0 | 0 | 0 |
| FR-SESSION | 33 | 33 | 0 | 0 | 0 | 0 | 0 |
| FR-SOC-CIV | 2 | 2 | 0 | 0 | 0 | 0 | 0 |
| FR-SOC-COH | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-SOC-DET | 2 | 0 | 0 | 2 | 0 | 0 | 0 |
| FR-SOC-FAC | 2 | 0 | 0 | 2 | 0 | 0 | 0 |
| FR-SOC-HLT | 5 | 0 | 0 | 5 | 0 | 0 | 0 |
| FR-SOC-IDE | 6 | 0 | 0 | 6 | 0 | 0 | 0 |
| FR-SOC-INS | 7 | 7 | 0 | 0 | 0 | 0 | 0 |
| FR-SOC-INT | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| FR-SOC-INTG | 7 | 7 | 0 | 0 | 0 | 0 | 0 |
| FR-SOCI | 6 | 5 | 0 | 1 | 0 | 0 | 0 |
| FR-STOR | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-TEST | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-THRY | 4 | 0 | 0 | 4 | 0 | 0 | 0 |
| FR-UX | 27 | 5 | 0 | 0 | 0 | 22 | 0 |
| FR-VAL | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| FR-VIEWPORT | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| NFR-C | 7 | 0 | 0 | 0 | 6 | 1 | 0 |
| NFR-CIV | 13 | 0 | 0 | 0 | 0 | 13 | 0 |
| NFR-CIV-ACC | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| NFR-CIV-AI | 3 | 2 | 0 | 0 | 0 | 1 | 0 |
| NFR-CIV-DET | 4 | 4 | 0 | 0 | 0 | 0 | 0 |
| NFR-CIV-DEV-HYGIENE | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| NFR-CIV-LEGENDS-CONFIG | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| NFR-CIV-LEGENDS-LOUD | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-PERF | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| NFR-CIV-LEGENDS-SCALE | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| NFR-CIV-MAINT | 6 | 0 | 0 | 0 | 4 | 2 | 0 |
| NFR-CIV-PERF | 11 | 2 | 0 | 0 | 0 | 9 | 0 |
| NFR-CIV-PORT | 3 | 3 | 0 | 0 | 0 | 0 | 0 |
| NFR-CIV-REL | 4 | 1 | 0 | 0 | 0 | 3 | 0 |
| NFR-CIV-SCALE | 9 | 0 | 0 | 3 | 0 | 6 | 0 |
| NFR-CIV-SCALE-PERF | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| NFR-CIV-SEC | 4 | 1 | 0 | 0 | 0 | 3 | 0 |
| NFR-O | 6 | 0 | 0 | 0 | 0 | 6 | 0 |
| NFR-P | 8 | 0 | 0 | 0 | 8 | 0 | 0 |
| NFR-R | 6 | 0 | 0 | 0 | 0 | 6 | 0 |
| NFR-S | 6 | 1 | 0 | 0 | 5 | 0 | 0 |
| NFR-SCALE | 1 | 1 | 0 | 0 | 0 | 0 | 0 |

## Spec-only IDs (need implementation) (234)

- `FR-CIV-0700`
  - spec: docs/design/civ-actor-assets-fix.md:322
- `FR-CIV-ASSET-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206, docs/traceability/fr-civ-asset-002/fr-civ-asset-002-adr.md:1
- `FR-CIV-ASSET-008`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2497, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212, docs/traceability/fr-civ-asset-008/fr-civ-asset-008-adr.md:1
- `FR-CIV-ASSET-009`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2507, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213, docs/traceability/fr-civ-asset-009/fr-civ-asset-009-adr.md:1
- `FR-CIV-ASSET-010`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2425, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2517
- `FR-CIV-ASSET-011`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2529
- `FR-CIV-ASSET-012`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2539, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2917, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3216
- `FR-CIV-ASSET-013`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2549, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3217, docs/traceability/fr-civ-asset-013/fr-civ-asset-013-adr.md:1
- `FR-CIV-ASSET-014`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2559, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3218, docs/traceability/fr-civ-asset-014/fr-civ-asset-014-adr.md:1
- `FR-CIV-ASSET-015`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:1714, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2569, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3219
- `FR-CIV-ASSET-017`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2589, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3221, docs/traceability/fr-civ-asset-017/fr-civ-asset-017-adr.md:1
- `FR-CIV-ASSET-019`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2609, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223, docs/traceability/fr-civ-asset-019/fr-civ-asset-019-adr.md:1
- `FR-CIV-ASSET-020`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2619
- `FR-CIV-AUDIO-009`
  - spec: docs/design/audio-direction.md:302, docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:1, docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:6
- `FR-CIV-AUDIO-010`
  - spec: docs/design/audio-direction.md:303, docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:1, docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:6
- `FR-CIV-AUDIO-011`
  - spec: docs/design/audio-direction.md:304, docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:1, docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:6
- `FR-CIV-AUDIO-012`
  - spec: docs/design/audio-direction.md:305, docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:1, docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:6
- `FR-CIV-BEVY-021`
  - spec: docs/development-guide/p-w1-kickoff.md:83, docs/development-guide/p-w1-kickoff.md:133, docs/traceability/fr-3d-matrix.md:186
- `FR-CIV-ECON-002-JOULE`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:70, docs/guides/COPILOT_L3_AGENTS.md:92, docs/guides/COPILOT_L3_AGENTS.md:93
- `FR-CIV-EMERGENCE-100`
  - spec: docs/traceability/emergent-systems-tracelinks.md:150
- `FR-CIV-EMERGENCE-111`
  - spec: docs/traceability/emergent-systems-tracelinks.md:151
- `FR-CIV-EMERGENCE-119`
  - spec: docs/traceability/emergent-systems-tracelinks.md:152
- `FR-CIV-EMERGENCE-124`
  - spec: docs/traceability/emergent-systems-tracelinks.md:153
- `FR-CIV-EMERGENCE-132`
  - spec: docs/traceability/emergent-systems-tracelinks.md:154
- `FR-CIV-EMERGENCE-141`
  - spec: docs/traceability/emergent-systems-tracelinks.md:155
- `FR-CIV-EMERGENCE-144`
  - spec: docs/traceability/emergent-systems-tracelinks.md:156
- `FR-CIV-EMERGENCE-151`
  - spec: docs/traceability/emergent-systems-tracelinks.md:157
- `FR-CIV-EMERGENCE-168`
  - spec: docs/traceability/emergent-systems-tracelinks.md:158
- `FR-CIV-EMERGENCE-198`
  - spec: docs/traceability/emergent-systems-tracelinks.md:159
- `FR-CIV-EMERGENCE-221`
  - spec: docs/traceability/emergent-systems-tracelinks.md:160
- `FR-CIV-EMERGENCE-236`
  - spec: docs/traceability/emergent-systems-tracelinks.md:161
- `FR-CIV-EMERGENCE-239`
  - spec: docs/traceability/emergent-systems-tracelinks.md:162
- `FR-CIV-EMERGENCE-241`
  - spec: docs/traceability/emergent-systems-tracelinks.md:163
- `FR-CIV-EMERGENCE-249`
  - spec: docs/traceability/emergent-systems-tracelinks.md:164
- `FR-CIV-EMERGENCE-RELIGION-1`
  - spec: docs/design/RELIGION_EMERGENCE.md:243
- `FR-CIV-EMERGENCE-RELIGION-2`
  - spec: docs/design/RELIGION_EMERGENCE.md:456
- `FR-CIV-GEO-001`
  - spec: docs/reference/FR_TRACKER.md:22, docs/reports/STATUS_REPORT.md:95, docs/specs/CIV-0300-rts-ui-ux-spec.md:2024
- `FR-CIV-GEO-002`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2025, docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:1, docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:6
- `FR-CIV-GEO-003`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2026, docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:1, docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:6
- `FR-CIV-GEO-004`
  - spec: docs/reports/STATUS_REPORT.md:96, docs/specs/CIV-0300-rts-ui-ux-spec.md:2027, docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:1
- `FR-CIV-GEO-005`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2028, docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:1, docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:6
- `FR-CIV-GEO-006`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2029, docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:1, docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:6
- `FR-CIV-GEO-007`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2030, docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:1, docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:6
- `FR-CIV-GEO-008`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2031, docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:1, docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:6
- `FR-CIV-GEO-009`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2032, docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:1, docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:6
- `FR-CIV-GEO-010`
  - spec: docs/specs/CIV-0101-two-zoom-lod-v1.md:1580, docs/specs/CIV-0101-two-zoom-lod-v1.md:1582, docs/specs/CIV-0101-two-zoom-lod-v1.md:1584
- `FR-CIV-GODOT-UX-000`
  - spec: docs/development-guide/fr-godot-attach.md:13, docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:1, docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:6
- `FR-CIV-LEGENDS-CONFIG-04`
  - spec: docs/traceability/fr-emergence-matrix.md:259
- `FR-CIV-LEGENDS-SCALE-02`
  - spec: docs/traceability/fr-emergence-matrix.md:261
- `FR-CIV-PSYCHE-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:1, docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:6
- `FR-CIV-PSYCHE-007`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-007/fr-civ-psyche-007-adr.md:1, docs/traceability/fr-civ-psyche-007/fr-civ-psyche-007-adr.md:6
- `FR-CIV-PSYCHE-008`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-008/fr-civ-psyche-008-adr.md:1, docs/traceability/fr-civ-psyche-008/fr-civ-psyche-008-adr.md:6
- `FR-CIV-PSYCHE-010`
  - spec: docs/design/psyche-social.md:142, docs/design/psyche-social.md:255, docs/design/psyche-social.md:277
- `FR-CIV-PSYCHE-011`
  - spec: docs/design/psyche-social.md:191, docs/design/psyche-social.md:259, docs/design/psyche-social.md:278
- `FR-CIV-PSYCHE-020`
  - spec: docs/design/civ-culture-emergent.md:7, docs/design/psyche-social.md:153, docs/design/psyche-social.md:256
- `FR-CIV-PSYCHE-021`
  - spec: docs/design/psyche-social.md:209, docs/design/psyche-social.md:280, docs/traceability/fr-civ-psyche-021/fr-civ-psyche-021-adr.md:1
- `FR-CIV-PSYCHE-024`
  - spec: docs/design/psyche-social.md:233, docs/design/psyche-social.md:264, docs/design/psyche-social.md:281
- `FR-CIV-PSYCHE-030`
  - spec: docs/design/psyche-social.md:162, docs/design/psyche-social.md:257, docs/design/psyche-social.md:258
- `FR-CIV-PSYCHE-031`
  - spec: docs/design/psyche-social.md:174, docs/design/psyche-social.md:283, docs/traceability/fr-civ-psyche-031/fr-civ-psyche-031-adr.md:1
- `FR-CIV-PSYCHE-032`
  - spec: docs/design/psyche-social.md:203, docs/design/psyche-social.md:260, docs/design/psyche-social.md:284
- `FR-CIV-PSYCHE-033`
  - spec: docs/design/psyche-social.md:206, docs/design/psyche-social.md:261, docs/design/psyche-social.md:285
- `FR-CIV-PSYCHE-034`
  - spec: docs/design/psyche-social.md:241, docs/design/psyche-social.md:286, docs/traceability/fr-civ-psyche-034/fr-civ-psyche-034-adr.md:1
- `FR-CIV-PSYCHE-035`
  - spec: docs/design/psyche-social.md:245, docs/design/psyche-social.md:287, docs/traceability/fr-civ-psyche-035/fr-civ-psyche-035-adr.md:1
- `FR-CIV-PSYCHE-036`
  - spec: docs/design/psyche-social.md:246, docs/design/psyche-social.md:288, docs/traceability/fr-civ-psyche-036/fr-civ-psyche-036-adr.md:1
- `FR-CIV-PSYCHE-037`
  - spec: docs/design/psyche-social.md:247, docs/design/psyche-social.md:289, docs/traceability/fr-civ-psyche-037/fr-civ-psyche-037-adr.md:1
- `FR-CIV-PSYCHE-040`
  - spec: docs/design/psyche-social.md:6, docs/design/psyche-social.md:290, docs/traceability/fr-civ-psyche-040/fr-civ-psyche-040-adr.md:1
- `FR-CIV-RESEARCH-004-REPLAY`
  - spec: PLAN.md:239, docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:1, docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:6
- `FR-CIV-SOCIAL-001`
  - spec: agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:26, agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:40, agileplus-specs/civ-007-diplomacy-laws-government/spec.md:42
- `FR-CIV-SOCIAL-002`
  - spec: agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:27, agileplus-specs/civ-009-culture-diffusion/spec.md:37, docs/reference/agileplus-artifacts-index.md:73
- `FR-CIV-SPECIES-012`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-012/fr-civ-species-012-adr.md:1, docs/traceability/fr-civ-species-012/fr-civ-species-012-adr.md:6
- `FR-CIV-SPECIES-013`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-013/fr-civ-species-013-adr.md:1, docs/traceability/fr-civ-species-013/fr-civ-species-013-adr.md:6
- `FR-CIV-SPECIES-014`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-014/fr-civ-species-014-adr.md:1, docs/traceability/fr-civ-species-014/fr-civ-species-014-adr.md:6
- `FR-CIV-SPECIES-015`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-015/fr-civ-species-015-adr.md:1, docs/traceability/fr-civ-species-015/fr-civ-species-015-adr.md:6
- `FR-CIV-SPECIES-016`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-016/fr-civ-species-016-adr.md:1, docs/traceability/fr-civ-species-016/fr-civ-species-016-adr.md:6
- `FR-CIV-SPECIES-017`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-017/fr-civ-species-017-adr.md:1, docs/traceability/fr-civ-species-017/fr-civ-species-017-adr.md:6
- `FR-CIV-SPECIES-100`
  - spec: docs/design/species-sentience.md:73, docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:1, docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:6
- `FR-CIV-SPECIES-101`
  - spec: docs/design/species-sentience.md:74, docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:1, docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:6
- `FR-CIV-SPECIES-102`
  - spec: docs/design/species-sentience.md:75, docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:1, docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:6
- `FR-CIV-SPECIES-103`
  - spec: docs/design/species-sentience.md:76, docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:1, docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:6
- `FR-CIV-SPECIES-104`
  - spec: docs/design/species-sentience.md:77, docs/design/species-sentience.md:101, docs/traceability/fr-civ-species-104/fr-civ-species-104-adr.md:1
- `FR-CIV-SPECIES-105`
  - spec: docs/design/species-sentience.md:78, docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:1, docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:6
- `FR-CIV-SPECIES-200`
  - spec: docs/design/species-sentience.md:98, docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:1, docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:6
- `FR-CIV-SPECIES-201`
  - spec: docs/design/species-sentience.md:99, docs/design/species-sentience.md:206, docs/traceability/fr-civ-species-201/fr-civ-species-201-adr.md:1
- `FR-CIV-SPECIES-202`
  - spec: docs/design/species-sentience.md:100, docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:1, docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:6
- `FR-CIV-SPECIES-203`
  - spec: docs/design/species-sentience.md:101, docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:1, docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:6
- `FR-CIV-SPECIES-204`
  - spec: docs/design/species-sentience.md:102, docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:1, docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:6
- `FR-CIV-SPECIES-205`
  - spec: docs/design/species-sentience.md:103, docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:1, docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:6
- `FR-CIV-SPECIES-300`
  - spec: docs/design/species-sentience.md:120, docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:1, docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:6
- `FR-CIV-SPECIES-301`
  - spec: docs/design/species-sentience.md:121, docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:1, docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:6
- `FR-CIV-SPECIES-302`
  - spec: docs/design/species-sentience.md:122, docs/design/species-sentience.md:192, docs/traceability/fr-civ-species-302/fr-civ-species-302-adr.md:1
- `FR-CIV-SPECIES-303`
  - spec: docs/design/species-sentience.md:123, docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:1, docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:6
- `FR-CIV-SPECIES-304`
  - spec: docs/design/species-sentience.md:124, docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:1, docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:6
- `FR-CIV-SPECIES-400`
  - spec: docs/design/species-sentience.md:167, docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:1, docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:6
- `FR-CIV-SPECIES-401`
  - spec: docs/design/species-sentience.md:168, docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:1, docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:6
- `FR-CIV-SPECIES-402`
  - spec: docs/design/species-sentience.md:169, docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:1, docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:6
- `FR-CIV-SPECIES-403`
  - spec: docs/design/species-sentience.md:170, docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:1, docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:6
- `FR-CIV-SPECIES-404`
  - spec: docs/design/species-sentience.md:171, docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:1, docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:6
- `FR-CIV-SPECIES-405`
  - spec: docs/design/species-sentience.md:172, docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:1, docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:6
- `FR-CIV-SPECIES-406`
  - spec: docs/design/species-sentience.md:173, docs/design/species-sentience.md:216, docs/traceability/fr-civ-species-406/fr-civ-species-406-adr.md:1
- `FR-CIV-SPECIES-500`
  - spec: docs/design/species-sentience.md:191, docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:1, docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:6
- `FR-CIV-SPECIES-501`
  - spec: docs/design/species-sentience.md:192, docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:1, docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:6
- `FR-CIV-SPECIES-502`
  - spec: docs/design/species-sentience.md:193, docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:1, docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:6
- `FR-CIV-SPECIES-503`
  - spec: docs/design/species-sentience.md:194, docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:1, docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:6
- `FR-CIV-SPECIES-504`
  - spec: docs/design/species-sentience.md:195, docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:1, docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:6
- `FR-CIV-SPECIES-505`
  - spec: docs/design/species-sentience.md:196, docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:1, docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:6
- `FR-CIV-TECH-001`
  - spec: docs/design/tech-engineering.md:225, docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:1, docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:6
- `FR-CIV-TECH-002`
  - spec: docs/design/tech-engineering.md:226, docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:1, docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:6
- `FR-CIV-TECH-003`
  - spec: docs/design/tech-engineering.md:227, docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:1, docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:6
- `FR-CIV-TECH-004`
  - spec: docs/design/tech-engineering.md:228, docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:1, docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:6
- `FR-CIV-TECH-005`
  - spec: docs/design/tech-engineering.md:229, docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:1, docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:6
- `FR-CIV-TECH-006`
  - spec: docs/design/tech-engineering.md:230, docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:1, docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:6
- `FR-CIV-TECH-007`
  - spec: docs/design/tech-engineering.md:231, docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:1, docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:6
- `FR-CIV-TECH-008`
  - spec: docs/design/tech-engineering.md:232, docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:1, docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:6
- `FR-CIV-TECH-009`
  - spec: docs/design/tech-engineering.md:233, docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:1, docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:6
- `FR-CIV-TECH-010`
  - spec: docs/design/tech-engineering.md:234, docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:1, docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:6
- `FR-CIV-TECH-011`
  - spec: docs/design/tech-engineering.md:235, docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:1, docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:6
- `FR-CIV-TECH-012`
  - spec: docs/design/tech-engineering.md:236, docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:1, docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:6
- `FR-CIV-TECH-013`
  - spec: docs/design/tech-engineering.md:237, docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:1, docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:6
- `FR-CIV-TECH-014`
  - spec: docs/design/tech-engineering.md:238, docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:1, docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:6
- `FR-CIV-TECH-015`
  - spec: docs/design/tech-engineering.md:239, docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:1, docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:6
- `FR-CIV-TECH-016`
  - spec: docs/design/tech-engineering.md:240, docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:1, docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:6
- `FR-CIV-TECH-017`
  - spec: docs/design/tech-engineering.md:241, docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:1, docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:6
- `FR-CIV-TECH-018`
  - spec: docs/design/tech-engineering.md:242, docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:1, docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:6
- `FR-CIV-TECH-019`
  - spec: docs/design/tech-engineering.md:243, docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:1, docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:6
- `FR-CIV-TECH-020`
  - spec: docs/design/tech-engineering.md:244, docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:1, docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:6
- `FR-CIV-TECH-021`
  - spec: docs/design/tech-engineering.md:245, docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:1, docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:6
- `FR-CIV-WAR-020`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:228, docs/design/warfare.md:106, docs/design/warfare.md:195
- `FR-CIV-WEB-000`
  - spec: docs/development-guide/fr-web-spectator.md:3, docs/development-guide/fr-web-spectator.md:29, docs/development-guide/pr-296-body.md:20
- `FR-CIV-WEB-001`
  - spec: docs/development-guide/fr-web-spectator.md:30, docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:1, docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:6
- `FR-CIV-WEB-004`
  - spec: docs/development-guide/fr-web-spectator.md:33, docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:1, docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:6
- `FR-CIV-WEB-005`
  - spec: docs/development-guide/fr-web-spectator.md:34, docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:1, docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:6
- `FR-NFR-CIV-DET-001`
  - spec: docs/traceability/nfr-civ-det-001/nfr-civ-det-001-intent.md:1, docs/traceability/nfr-civ-det-001/nfr-civ-det-001-intent.md:4
- `FR-NFR-CIV-DET-002`
  - spec: docs/traceability/nfr-civ-det-002/nfr-civ-det-002-intent.md:1, docs/traceability/nfr-civ-det-002/nfr-civ-det-002-intent.md:4
- `FR-NFR-CIV-DET-003`
  - spec: docs/traceability/nfr-civ-det-001/nfr-civ-det-001-intent.md:19, docs/traceability/nfr-civ-det-002/nfr-civ-det-002-intent.md:19
- `FR-NFR-CIV-PERF-002`
  - spec: docs/traceability/nfr-civ-perf-002/nfr-civ-perf-002-intent.md:1, docs/traceability/nfr-civ-perf-002/nfr-civ-perf-002-intent.md:4
- `FR-NFR-CIV-PERF-003`
  - spec: docs/traceability/nfr-civ-perf-003/nfr-civ-perf-003-intent.md:1, docs/traceability/nfr-civ-perf-003/nfr-civ-perf-003-intent.md:4
- `FR-NFR-CIV-PERF-004`
  - spec: docs/traceability/nfr-civ-perf-004/nfr-civ-perf-004-intent.md:1, docs/traceability/nfr-civ-perf-004/nfr-civ-perf-004-intent.md:4
- `FR-NFR-CIV-PERF-005`
  - spec: docs/traceability/nfr-civ-perf-005/nfr-civ-perf-005-intent.md:1, docs/traceability/nfr-civ-perf-005/nfr-civ-perf-005-intent.md:4
- `FR-NFR-CIV-PERF-006`
  - spec: docs/traceability/nfr-civ-perf-006/nfr-civ-perf-006-intent.md:1, docs/traceability/nfr-civ-perf-006/nfr-civ-perf-006-intent.md:4
- `FR-NFR-CIV-PERF-007`
  - spec: docs/traceability/nfr-civ-perf-007/nfr-civ-perf-007-intent.md:1, docs/traceability/nfr-civ-perf-007/nfr-civ-perf-007-intent.md:4
- `FR-NFR-CIV-PERF-008`
  - spec: docs/traceability/nfr-civ-perf-008/nfr-civ-perf-008-intent.md:1, docs/traceability/nfr-civ-perf-008/nfr-civ-perf-008-intent.md:4
- `FR-NFR-CIV-PERF-900`
  - spec: docs/traceability/nfr-civ-perf-900/nfr-civ-perf-900-intent.md:1, docs/traceability/nfr-civ-perf-900/nfr-civ-perf-900-intent.md:4
- `FR-NFR-CIV-PERF-901`
  - spec: docs/traceability/nfr-civ-perf-901/nfr-civ-perf-901-intent.md:1, docs/traceability/nfr-civ-perf-901/nfr-civ-perf-901-intent.md:4
- `FR-NFR-CIV-PERF-902`
  - spec: docs/traceability/nfr-civ-perf-902/nfr-civ-perf-902-intent.md:1, docs/traceability/nfr-civ-perf-902/nfr-civ-perf-902-intent.md:4
- `FR-NFR-R-01`
  - spec: docs/traceability/nfr-r-01/nfr-r-01-intent.md:1, docs/traceability/nfr-r-01/nfr-r-01-intent.md:4
- `FR-NFR-R-02`
  - spec: docs/traceability/nfr-r-02/nfr-r-02-intent.md:1, docs/traceability/nfr-r-02/nfr-r-02-intent.md:4
- `FR-NFR-R-03`
  - spec: docs/traceability/nfr-r-03/nfr-r-03-intent.md:1, docs/traceability/nfr-r-03/nfr-r-03-intent.md:4
- `FR-NFR-R-04`
  - spec: docs/traceability/nfr-r-04/nfr-r-04-intent.md:1, docs/traceability/nfr-r-04/nfr-r-04-intent.md:4
- `FR-NFR-R-05`
  - spec: docs/traceability/nfr-r-05/nfr-r-05-intent.md:1, docs/traceability/nfr-r-05/nfr-r-05-intent.md:4
- `FR-NFR-R-06`
  - spec: docs/traceability/nfr-r-06/nfr-r-06-intent.md:1, docs/traceability/nfr-r-06/nfr-r-06-intent.md:4
- `FR-SAVE-006`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2805, docs/specs/CIV-1000-save-load-persistence-spec.md:2943, docs/traceability/fr-save-006/fr-save-006-adr.md:1
- `FR-SAVE-008`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2807, docs/specs/CIV-1000-save-load-persistence-spec.md:2949, docs/traceability/fr-save-008/fr-save-008-adr.md:1
- `FR-SAVE-009`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2808, docs/traceability/fr-save-009/fr-save-009-adr.md:1, docs/traceability/fr-save-009/fr-save-009-adr.md:6
- `FR-SAVE-011`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2810, docs/specs/CIV-1000-save-load-persistence-spec.md:2963, docs/traceability/fr-save-011/fr-save-011-adr.md:1
- `FR-SAVE-012`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2811, docs/specs/CIV-1000-save-load-persistence-spec.md:2963, docs/traceability/fr-save-012/fr-save-012-adr.md:1
- `FR-SAVE-013`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2812, docs/specs/CIV-1000-save-load-persistence-spec.md:2969, docs/traceability/fr-save-013/fr-save-013-adr.md:1
- `FR-SAVE-016`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2815, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-016/fr-save-016-adr.md:1
- `FR-SAVE-017`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2816, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-017/fr-save-017-adr.md:1
- `FR-SAVE-018`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2817, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-018/fr-save-018-adr.md:1
- `FR-SAVE-019`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2818, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-019/fr-save-019-adr.md:1
- `FR-UX-006`
  - spec: docs/models/civ-sim/USER_SPEC.md:928, docs/traceability/fr-ux-006/fr-ux-006-adr.md:1, docs/traceability/fr-ux-006/fr-ux-006-adr.md:6
- `FR-UX-007`
  - spec: docs/models/civ-sim/USER_SPEC.md:931, docs/traceability/fr-ux-007/fr-ux-007-adr.md:1, docs/traceability/fr-ux-007/fr-ux-007-adr.md:6
- `FR-UX-008`
  - spec: docs/models/civ-sim/USER_SPEC.md:934, docs/traceability/fr-ux-008/fr-ux-008-adr.md:1, docs/traceability/fr-ux-008/fr-ux-008-adr.md:6
- `FR-UX-009`
  - spec: docs/models/civ-sim/USER_SPEC.md:939, docs/traceability/fr-ux-009/fr-ux-009-adr.md:1, docs/traceability/fr-ux-009/fr-ux-009-adr.md:6
- `FR-UX-010`
  - spec: docs/models/civ-sim/USER_SPEC.md:942, docs/traceability/fr-ux-010/fr-ux-010-adr.md:1, docs/traceability/fr-ux-010/fr-ux-010-adr.md:6
- `FR-UX-011`
  - spec: docs/models/civ-sim/USER_SPEC.md:945, docs/traceability/fr-ux-011/fr-ux-011-adr.md:1, docs/traceability/fr-ux-011/fr-ux-011-adr.md:6
- `FR-UX-012`
  - spec: docs/models/civ-sim/USER_SPEC.md:948, docs/traceability/fr-ux-012/fr-ux-012-adr.md:1, docs/traceability/fr-ux-012/fr-ux-012-adr.md:6
- `FR-UX-013`
  - spec: docs/models/civ-sim/USER_SPEC.md:951, docs/traceability/fr-ux-013/fr-ux-013-adr.md:1, docs/traceability/fr-ux-013/fr-ux-013-adr.md:6
- `FR-UX-014`
  - spec: docs/models/civ-sim/USER_SPEC.md:956, docs/traceability/fr-ux-014/fr-ux-014-adr.md:1, docs/traceability/fr-ux-014/fr-ux-014-adr.md:6
- `FR-UX-015`
  - spec: docs/models/civ-sim/USER_SPEC.md:959, docs/traceability/fr-ux-015/fr-ux-015-adr.md:1, docs/traceability/fr-ux-015/fr-ux-015-adr.md:6
- `FR-UX-016`
  - spec: docs/models/civ-sim/USER_SPEC.md:962, docs/traceability/fr-ux-016/fr-ux-016-adr.md:1, docs/traceability/fr-ux-016/fr-ux-016-adr.md:6
- `FR-UX-017`
  - spec: docs/models/civ-sim/USER_SPEC.md:965, docs/traceability/fr-ux-017/fr-ux-017-adr.md:1, docs/traceability/fr-ux-017/fr-ux-017-adr.md:6
- `FR-UX-018`
  - spec: docs/models/civ-sim/USER_SPEC.md:970, docs/traceability/fr-ux-018/fr-ux-018-adr.md:1, docs/traceability/fr-ux-018/fr-ux-018-adr.md:6
- `FR-UX-019`
  - spec: docs/models/civ-sim/USER_SPEC.md:973, docs/traceability/fr-ux-019/fr-ux-019-adr.md:1, docs/traceability/fr-ux-019/fr-ux-019-adr.md:6
- `FR-UX-020`
  - spec: docs/models/civ-sim/USER_SPEC.md:976, docs/traceability/fr-ux-020/fr-ux-020-adr.md:1, docs/traceability/fr-ux-020/fr-ux-020-adr.md:6
- `FR-UX-021`
  - spec: docs/models/civ-sim/USER_SPEC.md:979, docs/traceability/fr-ux-021/fr-ux-021-adr.md:1, docs/traceability/fr-ux-021/fr-ux-021-adr.md:6
- `FR-UX-022`
  - spec: docs/models/civ-sim/USER_SPEC.md:982, docs/traceability/fr-ux-022/fr-ux-022-adr.md:1, docs/traceability/fr-ux-022/fr-ux-022-adr.md:6
- `FR-UX-023`
  - spec: docs/models/civ-sim/USER_SPEC.md:987, docs/traceability/fr-ux-023/fr-ux-023-adr.md:1, docs/traceability/fr-ux-023/fr-ux-023-adr.md:6
- `FR-UX-024`
  - spec: docs/models/civ-sim/USER_SPEC.md:990, docs/traceability/fr-ux-024/fr-ux-024-adr.md:1, docs/traceability/fr-ux-024/fr-ux-024-adr.md:6
- `FR-UX-025`
  - spec: docs/models/civ-sim/USER_SPEC.md:993, docs/traceability/fr-ux-025/fr-ux-025-adr.md:1, docs/traceability/fr-ux-025/fr-ux-025-adr.md:6
- `FR-UX-026`
  - spec: docs/models/civ-sim/USER_SPEC.md:996, docs/traceability/fr-ux-026/fr-ux-026-adr.md:1, docs/traceability/fr-ux-026/fr-ux-026-adr.md:6
- `FR-UX-027`
  - spec: docs/models/civ-sim/USER_SPEC.md:999, docs/traceability/fr-ux-027/fr-ux-027-adr.md:1, docs/traceability/fr-ux-027/fr-ux-027-adr.md:6
- `NFR-C-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2042, docs/traceability/index.md:1152, docs/traceability/nfr-c-02/nfr-c-02-spec.md:1
- `NFR-CIV-001`
  - spec: docs/traceability/nfr-matrix.md:52
- `NFR-CIV-002`
  - spec: docs/traceability/nfr-matrix.md:53
- `NFR-CIV-003`
  - spec: docs/traceability/nfr-matrix.md:54
- `NFR-CIV-004`
  - spec: docs/traceability/nfr-matrix.md:55
- `NFR-CIV-005`
  - spec: docs/traceability/nfr-matrix.md:56
- `NFR-CIV-006`
  - spec: docs/traceability/nfr-matrix.md:57
- `NFR-CIV-007`
  - spec: docs/traceability/nfr-matrix.md:58
- `NFR-CIV-008`
  - spec: docs/traceability/nfr-matrix.md:59
- `NFR-CIV-009`
  - spec: docs/traceability/nfr-matrix.md:60
- `NFR-CIV-010`
  - spec: docs/traceability/nfr-matrix.md:61
- `NFR-CIV-011`
  - spec: docs/traceability/nfr-matrix.md:62
- `NFR-CIV-012`
  - spec: docs/traceability/nfr-matrix.md:63
- `NFR-CIV-013`
  - spec: docs/traceability/nfr-matrix.md:64
- `NFR-CIV-AI-002`
  - spec: docs/design/civ-ai-crate.md:49, docs/traceability/index.md:1163, docs/traceability/nfr-civ-ai-002/nfr-civ-ai-002-research.md:1
- `NFR-CIV-DEV-HYGIENE-001`
  - spec: docs/ops/history-purge-plan.md:4, docs/traceability/fr-nfr-civ-dev-hygiene-001/fr-nfr-civ-dev-hygiene-001-intent.md:34, docs/traceability/index.md:1169
- `NFR-CIV-LEGENDS-PERF-01`
  - spec: docs/design/legends-engine.md:451, docs/traceability/index.md:1172, docs/traceability/nfr-civ-legends-perf-01/nfr-civ-legends-perf-01-research.md:1
- `NFR-CIV-MAINT-005`
  - spec: docs/reference/non-functional-requirements.md:517, docs/reference/non-functional-requirements.md:584, docs/reference/non-functional-requirements.md:608
- `NFR-CIV-MAINT-006`
  - spec: docs/reference/non-functional-requirements.md:531, docs/reference/non-functional-requirements.md:585, docs/traceability/fr-nfr-matrix.md:108
- `NFR-CIV-PERF-003`
  - spec: agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:69, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:25, docs/design/civ-perf-dirty-incremental.md:9
- `NFR-CIV-PERF-004`
  - spec: docs/guides/voxel-emergent-vision-and-migration.md:189, docs/reference/non-functional-requirements.md:69, docs/reference/non-functional-requirements.md:193
- `NFR-CIV-PERF-005`
  - spec: agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:24, docs/design/civ-perf-dirty-incremental.md:10, docs/design/civ-perf-dirty-incremental.md:502
- `NFR-CIV-PERF-006`
  - spec: docs/guides/voxel-emergent-vision-and-migration.md:172, docs/reference/non-functional-requirements.md:100, docs/reference/non-functional-requirements.md:556
- `NFR-CIV-PERF-007`
  - spec: docs/reference/non-functional-requirements.md:114, docs/reference/non-functional-requirements.md:230, docs/reference/non-functional-requirements.md:557
- `NFR-CIV-PERF-008`
  - spec: docs/guides/voxel-emergent-vision-and-migration.md:171, docs/guides/voxel-emergent-vision-and-migration.md:191, docs/traceability/index.md:1187
- `NFR-CIV-PERF-900`
  - spec: docs/agileplus/epics/civ-w5-scale.md:14, docs/agileplus/epics/civ-w5-scale.md:27, docs/agileplus/README.md:24
- `NFR-CIV-PERF-901`
  - spec: docs/agileplus/epics/civ-w5-scale.md:15, docs/agileplus/epics/civ-w5-scale.md:27, docs/agileplus/README.md:24
- `NFR-CIV-PERF-902`
  - spec: docs/agileplus/epics/civ-w5-scale.md:16, docs/agileplus/epics/civ-w5-scale.md:28, docs/agileplus/README.md:24
- `NFR-CIV-REL-001`
  - spec: docs/reference/non-functional-requirements.md:236, docs/reference/non-functional-requirements.md:565, docs/reference/non-functional-requirements.md:595
- `NFR-CIV-REL-002`
  - spec: docs/reference/non-functional-requirements.md:250, docs/reference/non-functional-requirements.md:332, docs/reference/non-functional-requirements.md:566
- `NFR-CIV-REL-003`
  - spec: docs/reference/non-functional-requirements.md:264, docs/reference/non-functional-requirements.md:567, docs/reference/non-functional-requirements.md:595
- `NFR-CIV-SCALE-003`
  - spec: docs/reference/non-functional-requirements.md:220, docs/reference/non-functional-requirements.md:564, docs/reference/non-functional-requirements.md:602
- `NFR-CIV-SCALE-004`
  - spec: docs/guides/voxel-emergent-vision-and-migration.md:172, docs/traceability/index.md:1201, docs/traceability/TRACEABILITY-GAP-REPORT-20260916.md:248
- `NFR-CIV-SCALE-900`
  - spec: docs/agileplus/epics/civ-w5-scale.md:9, docs/agileplus/epics/civ-w5-scale.md:22, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-902`
  - spec: docs/agileplus/epics/civ-w5-scale.md:11, docs/agileplus/epics/civ-w5-scale.md:24, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-910`
  - spec: docs/agileplus/epics/civ-w5-scale.md:12, docs/agileplus/epics/civ-w5-scale.md:25, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-920`
  - spec: docs/agileplus/epics/civ-w5-scale.md:13, docs/agileplus/epics/civ-w5-scale.md:26, docs/agileplus/README.md:24
- `NFR-CIV-SEC-002`
  - spec: docs/reference/non-functional-requirements.md:308, docs/reference/non-functional-requirements.md:346, docs/reference/non-functional-requirements.md:570
- `NFR-CIV-SEC-003`
  - spec: docs/reference/non-functional-requirements.md:322, docs/reference/non-functional-requirements.md:571, docs/reference/non-functional-requirements.md:602
- `NFR-CIV-SEC-004`
  - spec: docs/reference/non-functional-requirements.md:336, docs/reference/non-functional-requirements.md:572, docs/reference/non-functional-requirements.md:611
- `NFR-O-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2088, docs/traceability/index.md:1211, docs/traceability/nfr-o-01/nfr-o-01-spec.md:1
- `NFR-O-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2089, docs/traceability/index.md:1212, docs/traceability/nfr-o-02/nfr-o-02-spec.md:1
- `NFR-O-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2090, docs/traceability/index.md:1213, docs/traceability/nfr-o-03/nfr-o-03-spec.md:1
- `NFR-O-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2091, docs/traceability/index.md:1214, docs/traceability/nfr-o-04/nfr-o-04-spec.md:1
- `NFR-O-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2092, docs/traceability/index.md:1215, docs/traceability/nfr-o-05/nfr-o-05-spec.md:1
- `NFR-O-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2093, docs/traceability/index.md:1216, docs/traceability/nfr-o-06/nfr-o-06-spec.md:1
- `NFR-R-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2077, docs/traceability/index.md:1225, docs/traceability/nfr-r-01/nfr-r-01-spec.md:1
- `NFR-R-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2078, docs/traceability/index.md:1226, docs/traceability/nfr-r-02/nfr-r-02-spec.md:1
- `NFR-R-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2079, docs/traceability/index.md:1227, docs/traceability/nfr-r-03/nfr-r-03-spec.md:1
- `NFR-R-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2080, docs/traceability/index.md:1228, docs/traceability/nfr-r-04/nfr-r-04-spec.md:1
- `NFR-R-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2081, docs/traceability/index.md:1229, docs/traceability/nfr-r-05/nfr-r-05-spec.md:1
- `NFR-R-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2082, docs/traceability/index.md:1230, docs/traceability/nfr-r-06/nfr-r-06-spec.md:1

## Tested IDs with no ID-tagged code (add a code reference) (155)

- `FR-CIV-ARCH-00`
  - spec: docs/traceability/fr-civ-arch-00/fr-civ-arch-00-intent.md:1, docs/traceability/fr-civ-arch-00/fr-civ-arch-00-intent.md:4, docs/traceability/fr-civ-arch-00/fr-civ-arch-00-intent.md:9
  - tests: crates/engine/tests/fr_civ_act_arch_cluster.rs:2
- `FR-CIV-CORE-021`
  - spec: docs/traceability/fr-civ-core-021/fr-civ-core-021-intent.md:1, docs/traceability/fr-civ-core-021/fr-civ-core-021-intent.md:4
  - tests: crates/build/tests/fr_matrix_batch12.rs:765
- `FR-CIV-DET-002`
  - spec: docs/traceability/fr-civ-det-002/fr-civ-det-002-intent.md:1, docs/traceability/fr-civ-det-002/fr-civ-det-002-intent.md:4, docs/traceability/fr-civ-det-002/fr-civ-det-002-intent.md:22
  - tests: crates/engine/tests/fr_fr_civ_det_002.rs:1, crates/engine/tests/fr_fr_civ_det_002.rs:4, crates/engine/tests/fr_fr_civ_det_002.rs:9
- `FR-CIV-DET-003`
  - spec: docs/traceability/fr-civ-det-003/fr-civ-det-003-intent.md:1, docs/traceability/fr-civ-det-003/fr-civ-det-003-intent.md:4, docs/traceability/fr-civ-det-003/fr-civ-det-003-intent.md:20
  - tests: crates/engine/tests/fr_fr_civ_det_003.rs:1, crates/engine/tests/fr_fr_civ_det_003.rs:4, crates/engine/tests/fr_fr_civ_det_003.rs:9
- `FR-CIV-DET-004`
  - spec: docs/traceability/fr-civ-det-004/fr-civ-det-004-intent.md:1, docs/traceability/fr-civ-det-004/fr-civ-det-004-intent.md:4, docs/traceability/fr-civ-det-004/fr-civ-det-004-intent.md:19
  - tests: crates/engine/tests/fr_fr_civ_det_004.rs:1, crates/engine/tests/fr_fr_civ_det_004.rs:4, crates/engine/tests/fr_fr_civ_det_004.rs:9
- `FR-CIV-DET-005`
  - spec: docs/traceability/fr-civ-det-005/fr-civ-det-005-intent.md:1, docs/traceability/fr-civ-det-005/fr-civ-det-005-intent.md:4, docs/traceability/fr-civ-det-005/fr-civ-det-005-intent.md:20
  - tests: crates/engine/tests/fr_fr_civ_det_005.rs:1, crates/engine/tests/fr_fr_civ_det_005.rs:4, crates/engine/tests/fr_fr_civ_det_005.rs:9
- `FR-CIV-DET-006`
  - spec: docs/traceability/fr-civ-det-006/fr-civ-det-006-intent.md:1, docs/traceability/fr-civ-det-006/fr-civ-det-006-intent.md:4, docs/traceability/fr-civ-det-006/fr-civ-det-006-intent.md:21
  - tests: crates/engine/tests/fr_fr_civ_det_006.rs:1, crates/engine/tests/fr_fr_civ_det_006.rs:4, crates/engine/tests/fr_fr_civ_det_006.rs:9
- `FR-CIV-DET-007`
  - spec: docs/traceability/fr-civ-det-007/fr-civ-det-007-intent.md:1, docs/traceability/fr-civ-det-007/fr-civ-det-007-intent.md:4, docs/traceability/fr-civ-det-007/fr-civ-det-007-intent.md:21
  - tests: crates/engine/tests/fr_fr_civ_det_007.rs:1, crates/engine/tests/fr_fr_civ_det_007.rs:4, crates/engine/tests/fr_fr_civ_det_007.rs:9
- `FR-CIV-DIPLO-002-SHADOW`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:215, PLAN.md:209, PLAN.md:210
  - tests: crates/diplomacy/tests/fr_civ_diplo_tests.rs:3, crates/diplomacy/tests/fr_civ_diplo_tests.rs:8, crates/diplomacy/tests/fr_civ_diplo_tests.rs:11
- `FR-CIV-GODTOOL-910`
  - spec: docs/agileplus/epics/civ-w1-voxel-render.md:9, docs/agileplus/epics/civ-w1-voxel-render.md:19, docs/agileplus/README.md:20
  - tests: crates/engine/tests/fr_civ_godtool_cluster.rs:2, crates/engine/tests/fr_civ_godtool_cluster.rs:173, crates/engine/tests/fr_civ_godtool_cluster.rs:176
- `FR-CIV-GODTOOL-911`
  - spec: docs/agileplus/epics/civ-w1-voxel-render.md:10, docs/agileplus/epics/civ-w1-voxel-render.md:20, docs/agileplus/README.md:20
  - tests: crates/engine/tests/fr_civ_godtool_cluster.rs:429, crates/engine/tests/fr_civ_godtool_cluster.rs:432, crates/engine/tests/fr_civ_godtool_cluster.rs:473
- `FR-CIV-GODTOOL-912`
  - spec: docs/agileplus/epics/civ-w1-voxel-render.md:11, docs/agileplus/epics/civ-w1-voxel-render.md:21, docs/agileplus/README.md:20
  - tests: crates/engine/tests/fr_civ_godtool_cluster.rs:612, crates/engine/tests/fr_civ_godtool_cluster.rs:615, crates/engine/tests/fr_civ_godtool_cluster.rs:654
- `FR-CIV-GODTOOL-920`
  - spec: docs/agileplus/epics/civ-w1-voxel-render.md:12, docs/agileplus/epics/civ-w1-voxel-render.md:22, docs/agileplus/epics/civ-w6-ui.md:10
  - tests: crates/engine/tests/fr_civ_godtool_cluster.rs:763, crates/engine/tests/fr_civ_godtool_cluster.rs:766, crates/engine/tests/fr_civ_godtool_cluster.rs:802
- `FR-CIV-GODTOOL-921`
  - spec: docs/agileplus/epics/civ-w1-voxel-render.md:13, docs/agileplus/epics/civ-w1-voxel-render.md:23, docs/agileplus/epics/civ-w6-ui.md:11
  - tests: crates/engine/tests/fr_civ_godtool_cluster.rs:886, crates/engine/tests/fr_civ_godtool_cluster.rs:889, crates/engine/tests/fr_civ_godtool_cluster.rs:940
- `FR-CIV-INT-001`
  - spec: docs/traceability/fr-civ-int-001/fr-civ-int-001-intent.md:1, docs/traceability/fr-civ-int-001/fr-civ-int-001-intent.md:4, docs/traceability/fr-civ-int-001/fr-civ-int-001-intent.md:36
  - tests: crates/engine/tests/fr_engine_replay_integrity_tests.rs:5, crates/engine/tests/fr_engine_replay_integrity_tests.rs:113, crates/engine/tests/fr_engine_replay_integrity_tests.rs:116
- `FR-CIV-LEGENDS-CAUSAL-06`
  - spec: docs/design/legends-engine.md:441, docs/traceability/fr-civ-legends-causal-06/fr-civ-legends-causal-06-adr.md:1, docs/traceability/fr-civ-legends-causal-06/fr-civ-legends-causal-06-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_causal_06.rs:1, crates/legends/tests/fr_fr_civ_legends_causal_06.rs:10, crates/legends/tests/fr_fr_civ_legends_causal_06.rs:23
- `FR-CIV-LEGENDS-INSPECT-08`
  - spec: docs/design/legends-engine.md:443, docs/traceability/fr-civ-legends-inspect-08/fr-civ-legends-inspect-08-adr.md:1, docs/traceability/fr-civ-legends-inspect-08/fr-civ-legends-inspect-08-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:1, crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:10, crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:16
- `FR-CIV-LEGENDS-NARRATOR-13`
  - spec: docs/design/legends-engine.md:448, docs/traceability/fr-civ-legends-narrator-13/fr-civ-legends-narrator-13-adr.md:1, docs/traceability/fr-civ-legends-narrator-13/fr-civ-legends-narrator-13-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:1, crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:10, crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:21
- `FR-CIV-LEGENDS-RESOLVE-04`
  - spec: docs/design/legends-engine.md:439, docs/traceability/fr-civ-legends-resolve-04/fr-civ-legends-resolve-04-adr.md:1, docs/traceability/fr-civ-legends-resolve-04/fr-civ-legends-resolve-04-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:1, crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:10, crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:26
- `FR-CIV-LLM-001`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-001/fr-civ-llm-001-adr.md:1, docs/traceability/fr-civ-llm-001/fr-civ-llm-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_001.rs:1, crates/engine/tests/fr_fr_civ_llm_001.rs:5, crates/engine/tests/fr_fr_civ_llm_001.rs:9
- `FR-CIV-LLM-002`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-002/fr-civ-llm-002-adr.md:1, docs/traceability/fr-civ-llm-002/fr-civ-llm-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_002.rs:1, crates/engine/tests/fr_fr_civ_llm_002.rs:5, crates/engine/tests/fr_fr_civ_llm_002.rs:9
- `FR-CIV-LLM-003`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-003/fr-civ-llm-003-adr.md:1, docs/traceability/fr-civ-llm-003/fr-civ-llm-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_003.rs:1, crates/engine/tests/fr_fr_civ_llm_003.rs:5, crates/engine/tests/fr_fr_civ_llm_003.rs:9
- `FR-CIV-LLM-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-004/fr-civ-llm-004-adr.md:1, docs/traceability/fr-civ-llm-004/fr-civ-llm-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_004.rs:1, crates/engine/tests/fr_fr_civ_llm_004.rs:5, crates/engine/tests/fr_fr_civ_llm_004.rs:9
- `FR-CIV-LLM-005`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-005/fr-civ-llm-005-adr.md:1, docs/traceability/fr-civ-llm-005/fr-civ-llm-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_005.rs:1, crates/engine/tests/fr_fr_civ_llm_005.rs:5, crates/engine/tests/fr_fr_civ_llm_005.rs:9
- `FR-CIV-LLM-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-006/fr-civ-llm-006-adr.md:1, docs/traceability/fr-civ-llm-006/fr-civ-llm-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_006.rs:1, crates/engine/tests/fr_fr_civ_llm_006.rs:5, crates/engine/tests/fr_fr_civ_llm_006.rs:9
- `FR-CIV-PERF-002`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1926, docs/traceability/fr-civ-perf-002/fr-civ-perf-002-adr.md:1, docs/traceability/fr-civ-perf-002/fr-civ-perf-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_002.rs:1, crates/engine/tests/fr_fr_civ_perf_002.rs:5, crates/engine/tests/fr_fr_civ_perf_002.rs:9
- `FR-CIV-PERF-003`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1931, docs/traceability/fr-civ-perf-003/fr-civ-perf-003-adr.md:1, docs/traceability/fr-civ-perf-003/fr-civ-perf-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_003.rs:1, crates/engine/tests/fr_fr_civ_perf_003.rs:5, crates/engine/tests/fr_fr_civ_perf_003.rs:9
- `FR-CIV-PERF-004`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1936, docs/traceability/fr-civ-perf-004/fr-civ-perf-004-adr.md:1, docs/traceability/fr-civ-perf-004/fr-civ-perf-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_004.rs:1, crates/engine/tests/fr_fr_civ_perf_004.rs:5, crates/engine/tests/fr_fr_civ_perf_004.rs:9
- `FR-CIV-PERF-005`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1941, docs/traceability/fr-civ-perf-005/fr-civ-perf-005-adr.md:1, docs/traceability/fr-civ-perf-005/fr-civ-perf-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_005.rs:1, crates/engine/tests/fr_fr_civ_perf_005.rs:5, crates/engine/tests/fr_fr_civ_perf_005.rs:9
- `FR-CIV-PERF-006`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1946, docs/traceability/fr-civ-perf-006/fr-civ-perf-006-adr.md:1, docs/traceability/fr-civ-perf-006/fr-civ-perf-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_006.rs:1, crates/engine/tests/fr_fr_civ_perf_006.rs:5, crates/engine/tests/fr_fr_civ_perf_006.rs:9
- `FR-CIV-PERF-007`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1951, docs/traceability/fr-civ-perf-007/fr-civ-perf-007-adr.md:1, docs/traceability/fr-civ-perf-007/fr-civ-perf-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_007.rs:1, crates/engine/tests/fr_fr_civ_perf_007.rs:5, crates/engine/tests/fr_fr_civ_perf_007.rs:9
- `FR-CIV-PERF-008`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1956, docs/traceability/fr-civ-perf-008/fr-civ-perf-008-adr.md:1, docs/traceability/fr-civ-perf-008/fr-civ-perf-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_008.rs:1, crates/engine/tests/fr_fr_civ_perf_008.rs:5, crates/engine/tests/fr_fr_civ_perf_008.rs:9
- `FR-CIV-PERF-009`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1961, docs/traceability/fr-civ-perf-009/fr-civ-perf-009-adr.md:1, docs/traceability/fr-civ-perf-009/fr-civ-perf-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_009.rs:1, crates/engine/tests/fr_fr_civ_perf_009.rs:5, crates/engine/tests/fr_fr_civ_perf_009.rs:9
- `FR-CIV-PERF-010`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1966, docs/traceability/fr-civ-perf-010/fr-civ-perf-010-adr.md:1, docs/traceability/fr-civ-perf-010/fr-civ-perf-010-adr.md:6
  - tests: crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:1176, crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:1184, crates/engine/tests/fr_fr_civ_perf_010.rs:1
- `FR-CIV-PERF-011`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1971, docs/traceability/fr-civ-perf-011/fr-civ-perf-011-adr.md:1, docs/traceability/fr-civ-perf-011/fr-civ-perf-011-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_011.rs:1, crates/engine/tests/fr_fr_civ_perf_011.rs:5, crates/engine/tests/fr_fr_civ_perf_011.rs:9
- `FR-CIV-PERF-012`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1976, docs/traceability/fr-civ-perf-012/fr-civ-perf-012-adr.md:1, docs/traceability/fr-civ-perf-012/fr-civ-perf-012-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_012.rs:1, crates/engine/tests/fr_fr_civ_perf_012.rs:5, crates/engine/tests/fr_fr_civ_perf_012.rs:9
- `FR-CIV-PERF-013`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1981, docs/traceability/fr-civ-perf-013/fr-civ-perf-013-adr.md:1, docs/traceability/fr-civ-perf-013/fr-civ-perf-013-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_013.rs:1, crates/engine/tests/fr_fr_civ_perf_013.rs:5, crates/engine/tests/fr_fr_civ_perf_013.rs:9
- `FR-CIV-PERF-014`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1986, docs/traceability/fr-civ-perf-014/fr-civ-perf-014-adr.md:1, docs/traceability/fr-civ-perf-014/fr-civ-perf-014-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_014.rs:1, crates/engine/tests/fr_fr_civ_perf_014.rs:5, crates/engine/tests/fr_fr_civ_perf_014.rs:9
- `FR-CIV-PERF-015`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1991, docs/traceability/fr-civ-perf-015/fr-civ-perf-015-adr.md:1, docs/traceability/fr-civ-perf-015/fr-civ-perf-015-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_015.rs:1, crates/engine/tests/fr_fr_civ_perf_015.rs:5, crates/engine/tests/fr_fr_civ_perf_015.rs:9
- `FR-CIV-PERF-016`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1996, docs/traceability/fr-civ-perf-016/fr-civ-perf-016-adr.md:1, docs/traceability/fr-civ-perf-016/fr-civ-perf-016-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_016.rs:1, crates/engine/tests/fr_fr_civ_perf_016.rs:5, crates/engine/tests/fr_fr_civ_perf_016.rs:9
- `FR-CIV-PERF-017`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2001, docs/traceability/fr-civ-perf-017/fr-civ-perf-017-adr.md:1, docs/traceability/fr-civ-perf-017/fr-civ-perf-017-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_017.rs:1, crates/engine/tests/fr_fr_civ_perf_017.rs:5, crates/engine/tests/fr_fr_civ_perf_017.rs:9
- `FR-CIV-PERF-018`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2006, docs/traceability/fr-civ-perf-018/fr-civ-perf-018-adr.md:1, docs/traceability/fr-civ-perf-018/fr-civ-perf-018-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_018.rs:1, crates/engine/tests/fr_fr_civ_perf_018.rs:5, crates/engine/tests/fr_fr_civ_perf_018.rs:9
- `FR-CIV-PERF-019`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2011, docs/traceability/fr-civ-perf-019/fr-civ-perf-019-adr.md:1, docs/traceability/fr-civ-perf-019/fr-civ-perf-019-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_019.rs:1, crates/engine/tests/fr_fr_civ_perf_019.rs:5, crates/engine/tests/fr_fr_civ_perf_019.rs:9
- `FR-CIV-PERF-020`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2016, docs/traceability/fr-civ-perf-020/fr-civ-perf-020-adr.md:1, docs/traceability/fr-civ-perf-020/fr-civ-perf-020-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_020.rs:1, crates/engine/tests/fr_fr_civ_perf_020.rs:5, crates/engine/tests/fr_fr_civ_perf_020.rs:9
- `FR-CIV-QOL-100`
  - spec: docs/design/onboarding-qol.md:37, docs/traceability/fr-civ-qol-100/fr-civ-qol-100-adr.md:1, docs/traceability/fr-civ-qol-100/fr-civ-qol-100-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_100.rs:1, crates/engine/tests/fr_fr_civ_qol_100.rs:6
- `FR-CIV-QOL-110`
  - spec: docs/design/onboarding-qol.md:74, docs/traceability/fr-civ-qol-110/fr-civ-qol-110-adr.md:1, docs/traceability/fr-civ-qol-110/fr-civ-qol-110-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_110.rs:1, crates/engine/tests/fr_fr_civ_qol_110.rs:6
- `FR-CIV-QOL-120`
  - spec: docs/design/onboarding-qol.md:90, docs/traceability/fr-civ-qol-120/fr-civ-qol-120-adr.md:1, docs/traceability/fr-civ-qol-120/fr-civ-qol-120-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_120.rs:1, crates/engine/tests/fr_fr_civ_qol_120.rs:6
- `FR-CIV-QOL-130`
  - spec: docs/design/onboarding-qol.md:104, docs/traceability/fr-civ-qol-130/fr-civ-qol-130-adr.md:1, docs/traceability/fr-civ-qol-130/fr-civ-qol-130-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_130.rs:1, crates/engine/tests/fr_fr_civ_qol_130.rs:6
- `FR-CIV-QOL-140`
  - spec: docs/design/onboarding-qol.md:120, docs/traceability/fr-civ-qol-140/fr-civ-qol-140-adr.md:1, docs/traceability/fr-civ-qol-140/fr-civ-qol-140-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_140.rs:1, crates/engine/tests/fr_fr_civ_qol_140.rs:7
- `FR-CIV-QOL-150`
  - spec: docs/design/onboarding-qol.md:136, docs/traceability/fr-civ-qol-150/fr-civ-qol-150-adr.md:1, docs/traceability/fr-civ-qol-150/fr-civ-qol-150-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_150.rs:1
- `FR-CIV-QOL-160`
  - spec: docs/design/onboarding-qol.md:146, docs/traceability/fr-civ-qol-160/fr-civ-qol-160-adr.md:1, docs/traceability/fr-civ-qol-160/fr-civ-qol-160-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_160.rs:1
- `FR-CIV-QOL-170`
  - spec: docs/design/onboarding-qol.md:162, docs/traceability/fr-civ-qol-170/fr-civ-qol-170-adr.md:1, docs/traceability/fr-civ-qol-170/fr-civ-qol-170-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_170.rs:1
- `FR-CIV-QOL-180`
  - spec: docs/design/onboarding-qol.md:172, docs/traceability/fr-civ-qol-180/fr-civ-qol-180-adr.md:1, docs/traceability/fr-civ-qol-180/fr-civ-qol-180-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_180.rs:1, crates/engine/tests/fr_fr_civ_qol_180.rs:7
- `FR-CIV-QOL-190`
  - spec: docs/design/onboarding-qol.md:191, docs/traceability/fr-civ-qol-190/fr-civ-qol-190-adr.md:1, docs/traceability/fr-civ-qol-190/fr-civ-qol-190-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_190.rs:1
- `FR-CIV-QOL-200`
  - spec: docs/design/onboarding-qol.md:205, docs/traceability/fr-civ-qol-200/fr-civ-qol-200-adr.md:1, docs/traceability/fr-civ-qol-200/fr-civ-qol-200-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_200.rs:1
- `FR-CIV-QOL-210`
  - spec: docs/design/onboarding-qol.md:222, docs/traceability/fr-civ-qol-210/fr-civ-qol-210-adr.md:1, docs/traceability/fr-civ-qol-210/fr-civ-qol-210-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_210.rs:1
- `FR-CIV-QOL-220`
  - spec: docs/design/onboarding-qol.md:240, docs/traceability/fr-civ-qol-220/fr-civ-qol-220-adr.md:1, docs/traceability/fr-civ-qol-220/fr-civ-qol-220-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_220.rs:1
- `FR-CIV-QOL-230`
  - spec: docs/design/onboarding-qol.md:249, docs/traceability/fr-civ-qol-230/fr-civ-qol-230-adr.md:1, docs/traceability/fr-civ-qol-230/fr-civ-qol-230-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_230.rs:1, crates/engine/tests/fr_fr_civ_qol_230.rs:7
- `FR-CIV-REL-007`
  - spec: docs/traceability/fr-civ-rel-007/fr-civ-rel-007-intent.md:1, docs/traceability/fr-civ-rel-007/fr-civ-rel-007-intent.md:4
  - tests: crates/engine/tests/fr_civ_religion_007_phase_belief.rs:1
- `FR-CIV-RTS-001`
  - spec: docs/reference/FR_TRACKER.md:16, docs/reports/STATUS_REPORT.md:93, docs/specs/CIV-0300-rts-ui-ux-spec.md:1313
  - tests: crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:7, crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:128, crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:131
- `FR-CIV-RTS-002`
  - spec: docs/reports/STATUS_REPORT.md:94, docs/specs/CIV-0300-rts-ui-ux-spec.md:1314, docs/specs/CIV-0300-rts-ui-ux-spec.md:1315
  - tests: crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:8, crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:370, crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:373
- `FR-CIV-RTS-003`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:1116, docs/specs/CIV-0300-rts-ui-ux-spec.md:1318, docs/specs/CIV-0300-rts-ui-ux-spec.md:1344
  - tests: crates/engine/tests/fr_fr_civ_rts_003.rs:1, crates/engine/tests/fr_fr_civ_rts_003.rs:5
- `FR-CIV-RTS-004`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:1143, docs/specs/CIV-0300-rts-ui-ux-spec.md:1317, docs/specs/CIV-0300-rts-ui-ux-spec.md:2007
  - tests: crates/engine/tests/fr_fr_civ_rts_004.rs:1, crates/engine/tests/fr_fr_civ_rts_004.rs:5
- `FR-CIV-RTS-005`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2008, docs/traceability/fr-civ-rts-005/fr-civ-rts-005-adr.md:1, docs/traceability/fr-civ-rts-005/fr-civ-rts-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_005.rs:1, crates/engine/tests/fr_fr_civ_rts_005.rs:5
- `FR-CIV-RTS-006`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2009, docs/traceability/fr-civ-rts-006/fr-civ-rts-006-adr.md:1, docs/traceability/fr-civ-rts-006/fr-civ-rts-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_006.rs:1, crates/engine/tests/fr_fr_civ_rts_006.rs:5
- `FR-CIV-RTS-007`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2010, docs/traceability/fr-civ-rts-007/fr-civ-rts-007-adr.md:1, docs/traceability/fr-civ-rts-007/fr-civ-rts-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_007.rs:1, crates/engine/tests/fr_fr_civ_rts_007.rs:5
- `FR-CIV-RTS-008`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2011, docs/traceability/fr-civ-rts-008/fr-civ-rts-008-adr.md:1, docs/traceability/fr-civ-rts-008/fr-civ-rts-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_008.rs:1, crates/engine/tests/fr_fr_civ_rts_008.rs:5
- `FR-CIV-RTS-009`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2012, docs/traceability/fr-civ-rts-009/fr-civ-rts-009-adr.md:1, docs/traceability/fr-civ-rts-009/fr-civ-rts-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_009.rs:1, crates/engine/tests/fr_fr_civ_rts_009.rs:5
- `FR-CIV-RTS-010`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2013, docs/traceability/fr-civ-rts-010/fr-civ-rts-010-adr.md:1, docs/traceability/fr-civ-rts-010/fr-civ-rts-010-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_010.rs:1, crates/engine/tests/fr_fr_civ_rts_010.rs:5
- `FR-CIV-RTS-011`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2014, docs/traceability/fr-civ-rts-011/fr-civ-rts-011-adr.md:1, docs/traceability/fr-civ-rts-011/fr-civ-rts-011-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_011.rs:1, crates/engine/tests/fr_fr_civ_rts_011.rs:5
- `FR-CIV-RTS-012`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2015, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2532, docs/traceability/fr-civ-rts-012/fr-civ-rts-012-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_rts_012.rs:1, crates/engine/tests/fr_fr_civ_rts_012.rs:5
- `FR-CIV-RTS-013`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2016, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2531, docs/traceability/fr-civ-rts-013/fr-civ-rts-013-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_rts_013.rs:1, crates/engine/tests/fr_fr_civ_rts_013.rs:5
- `FR-CIV-RTS-014`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2017, docs/specs/CIV-0400-ai-npc-behavior-spec.md:14, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2514
  - tests: crates/engine/tests/fr_fr_civ_rts_014.rs:1, crates/engine/tests/fr_fr_civ_rts_014.rs:5
- `FR-CIV-RTS-015`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2018, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2533, docs/traceability/fr-civ-rts-015/fr-civ-rts-015-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_rts_015.rs:1, crates/engine/tests/fr_fr_civ_rts_015.rs:5
- `FR-CIV-RTS-NATION-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3209, docs/traceability/fr-civ-rts-nation-001/fr-civ-rts-nation-001-adr.md:1, docs/traceability/fr-civ-rts-nation-001/fr-civ-rts-nation-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_nation_001.rs:1, crates/engine/tests/fr_fr_civ_rts_nation_001.rs:4, crates/engine/tests/fr_fr_civ_rts_nation_001.rs:11
- `FR-CIV-RTS-NATION-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3220, docs/traceability/fr-civ-rts-nation-002/fr-civ-rts-nation-002-adr.md:1, docs/traceability/fr-civ-rts-nation-002/fr-civ-rts-nation-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_nation_002.rs:1, crates/engine/tests/fr_fr_civ_rts_nation_002.rs:4, crates/engine/tests/fr_fr_civ_rts_nation_002.rs:11
- `FR-CIV-SERVER-001-WS`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:221, agileplus-specs/civ-021-recovered-requirements/spec.md:222, PLAN.md:174
  - tests: crates/server/tests/fr_civ_server_tests.rs:3, crates/server/tests/fr_civ_server_tests.rs:18, crates/server/tests/fr_fr_civ_server_001_ws.rs:1
- `FR-CIV-TACTICS-051`
  - spec: docs/development-guide/p-w1-kickoff.md:53, docs/traceability/fr-3d-matrix.md:141, docs/traceability/full-traceability-matrix.md:246
  - tests: crates/tactics/tests/fr_civ_tactics_tests.rs:18, crates/tactics/tests/fr_civ_tactics_tests.rs:26, crates/tactics/tests/fr_fr_civ_tactics_051.rs:1
- `FR-CIV-TACTICS-058`
  - spec: docs/development-guide/p-w1-kickoff.md:61, docs/traceability/fr-3d-matrix.md:149, docs/traceability/full-traceability-matrix.md:253
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:12, crates/mod-host/tests/fr_matrix_batch10.rs:48, crates/mod-host/tests/fr_matrix_batch10.rs:49
- `FR-CIV-TACTICS-060`
  - spec: docs/development-guide/p-w1-kickoff.md:63, docs/traceability/fr-3d-matrix.md:151, docs/traceability/full-traceability-matrix.md:255
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:13, crates/mod-host/tests/fr_matrix_batch10.rs:94, crates/mod-host/tests/fr_matrix_batch10.rs:95
- `FR-CIV-TACTICS-062`
  - spec: docs/development-guide/p-w1-kickoff.md:64, docs/traceability/fr-3d-matrix.md:152, docs/traceability/full-traceability-matrix.md:257
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:13, crates/mod-host/tests/fr_matrix_batch10.rs:115, crates/mod-host/tests/fr_matrix_batch10.rs:116
- `FR-CIV-TACTICS-064`
  - spec: docs/development-guide/p-w1-kickoff.md:66, docs/traceability/fr-3d-matrix.md:154, docs/traceability/full-traceability-matrix.md:259
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:13, crates/mod-host/tests/fr_matrix_batch10.rs:141, crates/mod-host/tests/fr_matrix_batch10.rs:142
- `FR-CIV-TACTICS-065`
  - spec: docs/development-guide/p-w1-kickoff.md:67, docs/traceability/fr-3d-matrix.md:155, docs/traceability/full-traceability-matrix.md:260
  - tests: crates/tactics/tests/fr_civ_tactics_tests.rs:33, crates/tactics/tests/fr_fr_civ_tactics_065.rs:1, crates/tactics/tests/fr_fr_civ_tactics_065.rs:6
- `FR-CIV-TACTICS-067`
  - spec: docs/development-guide/p-w1-kickoff.md:69, docs/traceability/fr-3d-matrix.md:157, docs/traceability/full-traceability-matrix.md:262
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:14, crates/mod-host/tests/fr_matrix_batch10.rs:171, crates/mod-host/tests/fr_matrix_batch10.rs:172
- `FR-CIV-TACTICS-069`
  - spec: docs/development-guide/p-w1-kickoff.md:71, docs/traceability/fr-3d-matrix.md:159, docs/traceability/full-traceability-matrix.md:264
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:14, crates/mod-host/tests/fr_matrix_batch10.rs:195, crates/mod-host/tests/fr_matrix_batch10.rs:196
- `FR-CIV-TACTICS-070`
  - spec: docs/development-guide/p-w1-kickoff.md:72, docs/traceability/fr-3d-matrix.md:160, docs/traceability/full-traceability-matrix.md:265
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:14, crates/mod-host/tests/fr_matrix_batch10.rs:225, crates/mod-host/tests/fr_matrix_batch10.rs:226
- `FR-CIV-TACTICS-072`
  - spec: docs/development-guide/p-w1-kickoff.md:74, docs/traceability/fr-3d-matrix.md:162, docs/traceability/full-traceability-matrix.md:267
  - tests: crates/mod-host/tests/fr_matrix_batch10.rs:15, crates/mod-host/tests/fr_matrix_batch10.rs:248, crates/mod-host/tests/fr_matrix_batch10.rs:249
- `FR-CIV-TACTICS-073`
  - spec: docs/development-guide/p-w1-kickoff.md:75, docs/traceability/fr-3d-matrix.md:163, docs/traceability/full-traceability-matrix.md:268
  - tests: crates/tactics/tests/fr_fr_civ_tactics_073.rs:1, crates/tactics/tests/fr_fr_civ_tactics_073.rs:6, crates/tactics/tests/fr_fr_civ_tactics_073.rs:16
- `FR-CIV-TACTICS-076`
  - spec: docs/development-guide/p-w1-kickoff.md:78, docs/traceability/fr-3d-matrix.md:166, docs/traceability/full-traceability-matrix.md:271
  - tests: crates/tactics/tests/fr_civ_tactics_tests.rs:41, crates/tactics/tests/fr_civ_tactics_tests.rs:49, crates/tactics/tests/fr_fr_civ_tactics_076.rs:1
- `FR-CIV-TACTICS-100`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:36, docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:1, docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_100.rs:1, crates/tactics/tests/fr_fr_civ_tactics_100.rs:6, crates/tactics/tests/fr_fr_civ_tactics_100.rs:14
- `FR-CIV-TACTICS-101`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:40, docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:1, docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_101.rs:1, crates/tactics/tests/fr_fr_civ_tactics_101.rs:6, crates/tactics/tests/fr_fr_civ_tactics_101.rs:14
- `FR-CIV-TACTICS-102`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:43, docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:1, docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_102.rs:1, crates/tactics/tests/fr_fr_civ_tactics_102.rs:6, crates/tactics/tests/fr_fr_civ_tactics_102.rs:14
- `FR-CIV-TEST-001`
  - spec: docs/traceability/fr-civ-test-001/fr-civ-test-001-intent.md:1, docs/traceability/fr-civ-test-001/fr-civ-test-001-intent.md:4, docs/traceability/fr-civ-test-001/fr-civ-test-001-intent.md:26
  - tests: crates/engine/tests/n_series_coverage.rs:3
- `FR-CIV-TEST-002`
  - spec: docs/traceability/fr-civ-test-002/fr-civ-test-002-intent.md:1, docs/traceability/fr-civ-test-002/fr-civ-test-002-intent.md:4, docs/traceability/fr-civ-test-002/fr-civ-test-002-intent.md:21
  - tests: crates/civis-mcp/tests/mcp_integration.rs:1
- `FR-CIV-TEST-006`
  - spec: docs/traceability/fr-civ-test-006/fr-civ-test-006-intent.md:1, docs/traceability/fr-civ-test-006/fr-civ-test-006-intent.md:4, docs/traceability/fr-civ-test-006/fr-civ-test-006-intent.md:24
  - tests: crates/economy/tests/economy_coverage.rs:1, crates/economy/tests/economy_coverage.rs:18, crates/economy/tests/economy_coverage.rs:51
- `FR-CIV-TEST-007`
  - spec: docs/traceability/fr-civ-test-007/fr-civ-test-007-intent.md:1, docs/traceability/fr-civ-test-007/fr-civ-test-007-intent.md:4, docs/traceability/fr-civ-test-007/fr-civ-test-007-intent.md:22
  - tests: crates/server/tests/server_coverage.rs:1
- `FR-CIV-TEST-008`
  - spec: docs/traceability/fr-civ-test-008/fr-civ-test-008-intent.md:1, docs/traceability/fr-civ-test-008/fr-civ-test-008-intent.md:4, docs/traceability/fr-civ-test-008/fr-civ-test-008-intent.md:25
  - tests: crates/civ-emergence-metrics/tests/emergence_coverage.rs:2, crates/civis-mcp/tests/mcp_coverage.rs:1
- `FR-CIV-TEST-009`
  - spec: docs/traceability/fr-civ-test-009/fr-civ-test-009-intent.md:1, docs/traceability/fr-civ-test-009/fr-civ-test-009-intent.md:4, docs/traceability/fr-civ-test-009/fr-civ-test-009-intent.md:22
  - tests: crates/protocol-3d/tests/protocol_coverage.rs:1
- `FR-CIV-TEST-021`
  - spec: docs/traceability/fr-civ-test-021/fr-civ-test-021-intent.md:1, docs/traceability/fr-civ-test-021/fr-civ-test-021-intent.md:4, docs/traceability/fr-civ-test-021/fr-civ-test-021-intent.md:25
  - tests: crates/server/tests/save_load_e2e.rs:1
- `FR-CORE-003`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-001-core-simulation-engine/spec.md:26, agileplus-specs/civ-002-economy-joule-system/spec.md:42
  - tests: crates/engine/tests/fr_fr_core_003.rs:1
- `FR-CORE-008`
  - spec: docs/adr/ADR-022-runtime-representation-deviations.md:20, docs/adr/ADR-022-runtime-representation-deviations.md:46, docs/adr/ADR-022-runtime-representation-deviations.md:70
  - tests: crates/engine/tests/fr_fr_core_008.rs:1
- `FR-DET-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:439, docs/traceability/fr-det-001/fr-det-001-adr.md:1, docs/traceability/fr-det-001/fr-det-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_001.rs:1, crates/engine/tests/fr_fr_det_001.rs:5, crates/engine/tests/fr_fr_det_001.rs:9
- `FR-DET-002`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:290, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:440, docs/traceability/fr-det-002/fr-det-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_det_002.rs:1, crates/engine/tests/fr_fr_det_002.rs:5, crates/engine/tests/fr_fr_det_002.rs:9
- `FR-DET-003`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:441, docs/traceability/fr-det-003/fr-det-003-adr.md:1, docs/traceability/fr-det-003/fr-det-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_003.rs:1, crates/engine/tests/fr_fr_det_003.rs:5, crates/engine/tests/fr_fr_det_003.rs:9
- `FR-DET-004`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:442, docs/traceability/fr-det-004/fr-det-004-adr.md:1, docs/traceability/fr-det-004/fr-det-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_004.rs:1, crates/engine/tests/fr_fr_det_004.rs:5, crates/engine/tests/fr_fr_det_004.rs:9
- `FR-DET-005`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:443, docs/traceability/fr-det-005/fr-det-005-adr.md:1, docs/traceability/fr-det-005/fr-det-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_005.rs:1, crates/engine/tests/fr_fr_det_005.rs:5, crates/engine/tests/fr_fr_det_005.rs:9
- `FR-DET-006`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:342, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:444, docs/traceability/fr-det-006/fr-det-006-adr.md:1
  - tests: crates/engine/tests/fr_fr_det_006.rs:1, crates/engine/tests/fr_fr_det_006.rs:5, crates/engine/tests/fr_fr_det_006.rs:9
- `FR-DET-007`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:445, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:451, docs/traceability/fr-det-007/fr-det-007-adr.md:1
  - tests: crates/engine/tests/fr_fr_det_007.rs:1, crates/engine/tests/fr_fr_det_007.rs:5, crates/engine/tests/fr_fr_det_007.rs:9
- `FR-ECO-001`
  - spec: docs/traceability/fr-eco-001/fr-eco-001-adr.md:1, docs/traceability/fr-eco-001/fr-eco-001-adr.md:6, docs/traceability/fr-eco-001/fr-eco-001-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1651
- `FR-ECO-002`
  - spec: docs/traceability/fr-eco-002/fr-eco-002-adr.md:1, docs/traceability/fr-eco-002/fr-eco-002-adr.md:6, docs/traceability/fr-eco-002/fr-eco-002-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1662
- `FR-ECO-003`
  - spec: docs/traceability/fr-eco-003/fr-eco-003-adr.md:1, docs/traceability/fr-eco-003/fr-eco-003-adr.md:6, docs/traceability/fr-eco-003/fr-eco-003-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1682
- `FR-ECO-004`
  - spec: docs/traceability/fr-eco-004/fr-eco-004-adr.md:1, docs/traceability/fr-eco-004/fr-eco-004-adr.md:6, docs/traceability/fr-eco-004/fr-eco-004-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1696
- `FR-ECO-005`
  - spec: docs/traceability/fr-eco-005/fr-eco-005-adr.md:1, docs/traceability/fr-eco-005/fr-eco-005-adr.md:6, docs/traceability/fr-eco-005/fr-eco-005-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1719
- `FR-ECO-006`
  - spec: docs/traceability/fr-eco-006/fr-eco-006-adr.md:1, docs/traceability/fr-eco-006/fr-eco-006-adr.md:6, docs/traceability/fr-eco-006/fr-eco-006-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1739
- `FR-ECO-007`
  - spec: docs/traceability/fr-eco-007/fr-eco-007-adr.md:1, docs/traceability/fr-eco-007/fr-eco-007-adr.md:6, docs/traceability/fr-eco-007/fr-eco-007-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1766
- `FR-ECO-008`
  - spec: docs/traceability/fr-eco-008/fr-eco-008-adr.md:1, docs/traceability/fr-eco-008/fr-eco-008-adr.md:6, docs/traceability/fr-eco-008/fr-eco-008-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1792
- `FR-ECO-009`
  - spec: docs/traceability/fr-eco-009/fr-eco-009-adr.md:1, docs/traceability/fr-eco-009/fr-eco-009-adr.md:6, docs/traceability/fr-eco-009/fr-eco-009-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1821
- `FR-ECO-010`
  - spec: docs/traceability/fr-eco-010/fr-eco-010-adr.md:1, docs/traceability/fr-eco-010/fr-eco-010-adr.md:6, docs/traceability/fr-eco-010/fr-eco-010-adr.md:11
  - tests: docs/specs/CIV-0100-economy-v1.md:1845
- `FR-FR-CORE-009`
  - spec: docs/traceability/fr-fr-core-009/fr-fr-core-009-intent.md:1, docs/traceability/fr-fr-core-009/fr-fr-core-009-intent.md:4, docs/traceability/fr-fr-core-009/fr-fr-core-009-intent.md:37
  - tests: crates/engine/tests/fr_core_cluster.rs:1, crates/engine/tests/fr_core_cluster.rs:15, crates/engine/tests/fr_core_cluster.rs:184
- `FR-INT-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1739, docs/traceability/fr-int-001/fr-int-001-adr.md:1, docs/traceability/fr-int-001/fr-int-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_int_001.rs:1, crates/engine/tests/fr_fr_int_001.rs:8, crates/engine/tests/fr_fr_int_001.rs:19
- `FR-MET-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1203, docs/traceability/fr-met-001/fr-met-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_met_001.rs:1, crates/engine/tests/fr_fr_met_001.rs:8, crates/engine/tests/fr_fr_met_001.rs:15
- `FR-METRICS-004`
  - spec: docs/FR.md:36, docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:1, docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_metrics_004.rs:1, crates/engine/tests/fr_fr_metrics_004.rs:8, crates/engine/tests/fr_fr_metrics_004.rs:18
- `FR-METRICS-005`
  - spec: docs/FR.md:37, docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:1, docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_metrics_005.rs:1, crates/engine/tests/fr_fr_metrics_005.rs:8, crates/engine/tests/fr_fr_metrics_005.rs:16
- `FR-PROTO-001`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-010-multi-client-protocol/spec.md:25, agileplus-specs/civ-014-terrain-playable-hardening/spec.md:67
  - tests: crates/server/tests/ws_smoke.rs:1687, crates/server/tests/ws_smoke.rs:1688
- `FR-PROTO-002`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-010-multi-client-protocol/spec.md:26, agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:81
  - tests: crates/engine/tests/fr_fr_proto_002.rs:1, crates/engine/tests/fr_fr_proto_002.rs:5, crates/engine/tests/fr_fr_proto_002.rs:9
- `FR-PROTO-003`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-010-multi-client-protocol/spec.md:27, agileplus-specs/civ-011-bevy-primary-client/spec.md:42
  - tests: crates/engine/tests/fr_fr_proto_003.rs:1, crates/engine/tests/fr_fr_proto_003.rs:5, crates/engine/tests/fr_fr_proto_003.rs:9
- `FR-PROTO-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-010-multi-client-protocol/spec.md:28, agileplus-specs/civ-011-bevy-primary-client/spec.md:43
  - tests: crates/engine/tests/fr_fr_proto_004.rs:1, crates/engine/tests/fr_fr_proto_004.rs:5, crates/engine/tests/fr_fr_proto_004.rs:9
- `FR-PROTO-005`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-010-multi-client-protocol/spec.md:29, docs/reference/agileplus-artifacts-index.md:185
  - tests: crates/engine/tests/fr_fr_proto_005.rs:1, crates/engine/tests/fr_fr_proto_005.rs:5, crates/engine/tests/fr_fr_proto_005.rs:9
- `FR-REPLAY-002`
  - spec: FUNCTIONAL_REQUIREMENTS.md, agileplus-specs/civ-013-research-api/plan.md:11, agileplus-specs/civ-013-research-api/spec.md:30
  - tests: crates/engine/tests/fr_engine_metrics_replay_tests.rs:4, crates/engine/tests/fr_engine_metrics_replay_tests.rs:101, crates/engine/tests/fr_engine_metrics_replay_tests.rs:104
- `FR-SOC-DET-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1495, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1498, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1984
  - tests: crates/engine/tests/fr_fr_soc_det_001.rs:1, crates/engine/tests/fr_fr_soc_det_001.rs:5, crates/engine/tests/fr_fr_soc_det_001.rs:9
- `FR-SOC-DET-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1516, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1985, docs/traceability/fr-soc-det-002/fr-soc-det-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_det_002.rs:1, crates/engine/tests/fr_fr_soc_det_002.rs:5, crates/engine/tests/fr_fr_soc_det_002.rs:9
- `FR-SOC-FAC-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4277, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4568, docs/traceability/fr-soc-fac-001/fr-soc-fac-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_fac_001.rs:1, crates/engine/tests/fr_fr_soc_fac_001.rs:5, crates/engine/tests/fr_fr_soc_fac_001.rs:9
- `FR-SOC-FAC-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4300, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4569, docs/traceability/fr-soc-fac-002/fr-soc-fac-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_fac_002.rs:1, crates/engine/tests/fr_fr_soc_fac_002.rs:5, crates/engine/tests/fr_fr_soc_fac_002.rs:9
- `FR-SOC-HLT-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1643, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1994, docs/traceability/fr-soc-hlt-001/fr-soc-hlt-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_001.rs:1, crates/engine/tests/fr_fr_soc_hlt_001.rs:5, crates/engine/tests/fr_fr_soc_hlt_001.rs:9
- `FR-SOC-HLT-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1657, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1995, docs/traceability/fr-soc-hlt-002/fr-soc-hlt-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_002.rs:1, crates/engine/tests/fr_fr_soc_hlt_002.rs:5, crates/engine/tests/fr_fr_soc_hlt_002.rs:9
- `FR-SOC-HLT-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1667, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1996, docs/traceability/fr-soc-hlt-003/fr-soc-hlt-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_003.rs:1, crates/engine/tests/fr_fr_soc_hlt_003.rs:5, crates/engine/tests/fr_fr_soc_hlt_003.rs:9
- `FR-SOC-HLT-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1675, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1997, docs/traceability/fr-soc-hlt-004/fr-soc-hlt-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_004.rs:1, crates/engine/tests/fr_fr_soc_hlt_004.rs:5, crates/engine/tests/fr_fr_soc_hlt_004.rs:9
- `FR-SOC-HLT-005`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4412, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4574, docs/traceability/fr-soc-hlt-005/fr-soc-hlt-005-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_005.rs:1, crates/engine/tests/fr_fr_soc_hlt_005.rs:5, crates/engine/tests/fr_fr_soc_hlt_005.rs:9
- `FR-SOC-IDE-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1588, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1990, docs/traceability/fr-soc-ide-001/fr-soc-ide-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_001.rs:1, crates/engine/tests/fr_fr_soc_ide_001.rs:5, crates/engine/tests/fr_fr_soc_ide_001.rs:9
- `FR-SOC-IDE-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1601, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1991, docs/traceability/fr-soc-ide-002/fr-soc-ide-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_002.rs:1, crates/engine/tests/fr_fr_soc_ide_002.rs:5, crates/engine/tests/fr_fr_soc_ide_002.rs:9
- `FR-SOC-IDE-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1612, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1992, docs/traceability/fr-soc-ide-003/fr-soc-ide-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_003.rs:1, crates/engine/tests/fr_fr_soc_ide_003.rs:5, crates/engine/tests/fr_fr_soc_ide_003.rs:9
- `FR-SOC-IDE-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1624, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1993, docs/traceability/fr-soc-ide-004/fr-soc-ide-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_004.rs:1, crates/engine/tests/fr_fr_soc_ide_004.rs:5, crates/engine/tests/fr_fr_soc_ide_004.rs:9
- `FR-SOC-IDE-005`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4373, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4572, docs/traceability/fr-soc-ide-005/fr-soc-ide-005-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_005.rs:1, crates/engine/tests/fr_fr_soc_ide_005.rs:5, crates/engine/tests/fr_fr_soc_ide_005.rs:9
- `FR-SOC-IDE-006`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4390, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4573, docs/traceability/fr-soc-ide-006/fr-soc-ide-006-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_006.rs:1, crates/engine/tests/fr_fr_soc_ide_006.rs:5, crates/engine/tests/fr_fr_soc_ide_006.rs:9
- `FR-SOCI-001`
  - spec: docs/IMPLEMENTATION_STATUS.md:80, docs/IMPLEMENTATION_STATUS.md:117, docs/traceability/TRACEABILITY_MATRIX.md:147
  - tests: crates/engine/tests/fr_fr_soci_001.rs:1, crates/engine/tests/fr_fr_soci_001.rs:5, crates/engine/tests/fr_fr_soci_001.rs:9
- `FR-STOR-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1931, docs/traceability/fr-stor-001/fr-stor-001-adr.md:1, docs/traceability/fr-stor-001/fr-stor-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_stor_001.rs:1, crates/engine/tests/fr_fr_stor_001.rs:5, crates/engine/tests/fr_fr_stor_001.rs:9
- `FR-TEST-001`
  - spec: docs/FR_DETAILED.md:341, docs/traceability/fr-test-001/fr-test-001-adr.md:1, docs/traceability/fr-test-001/fr-test-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_test_001.rs:1, crates/engine/tests/fr_fr_test_001.rs:5, crates/engine/tests/fr_fr_test_001.rs:9
- `FR-THRY-001`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:118, docs/traceability/fr-thry-001/fr-thry-001-adr.md:1, docs/traceability/fr-thry-001/fr-thry-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_thry_001.rs:1, crates/engine/tests/fr_fr_thry_001.rs:3, crates/engine/tests/fr_fr_thry_001.rs:16
- `FR-THRY-002`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:119, docs/traceability/fr-thry-002/fr-thry-002-adr.md:1, docs/traceability/fr-thry-002/fr-thry-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_thry_002.rs:1, crates/engine/tests/fr_fr_thry_002.rs:3, crates/engine/tests/fr_fr_thry_002.rs:15
- `FR-THRY-003`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:120, docs/traceability/fr-thry-003/fr-thry-003-adr.md:1, docs/traceability/fr-thry-003/fr-thry-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_thry_003.rs:1, crates/engine/tests/fr_fr_thry_003.rs:3, crates/engine/tests/fr_fr_thry_003.rs:15
- `FR-THRY-004`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:121, docs/traceability/fr-thry-004/fr-thry-004-adr.md:1, docs/traceability/fr-thry-004/fr-thry-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_thry_004.rs:1, crates/engine/tests/fr_fr_thry_004.rs:3, crates/engine/tests/fr_fr_thry_004.rs:16
- `FR-VAL-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:170, docs/traceability/fr-val-001/fr-val-001-adr.md:1, docs/traceability/fr-val-001/fr-val-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_val_001.rs:1, crates/engine/tests/fr_fr_val_001.rs:5, crates/engine/tests/fr_fr_val_001.rs:9
- `NFR-CIV-SCALE-001`
  - spec: docs/reference/non-functional-requirements.md:82, docs/reference/non-functional-requirements.md:188, docs/reference/non-functional-requirements.md:562
  - tests: crates/protocol-3d/tests/fr_perf_005_frame3d_timing.rs:85
- `NFR-CIV-SCALE-002`
  - spec: docs/guides/voxel-emergent-vision-and-migration.md:96, docs/guides/voxel-emergent-vision-and-migration.md:152, docs/reference/non-functional-requirements.md:110
  - tests: crates/voxel/tests/fr_civ_render_001_chunk_stream_radius.rs:6
- `NFR-CIV-SCALE-901`
  - spec: docs/agileplus/epics/civ-w5-scale.md:10, docs/agileplus/epics/civ-w5-scale.md:23, docs/agileplus/README.md:24
  - tests: crates/voxel/tests/fr_nfr_civ_scale_901.rs:1

## Stub-test IDs (replace placeholder tests with real FR assertions) (10)

- `FR-NFR-CIV-DEV-HYGIENE-001`
  - spec: docs/traceability/fr-nfr-civ-dev-hygiene-001/fr-nfr-civ-dev-hygiene-001-intent.md:1, docs/traceability/fr-nfr-civ-dev-hygiene-001/fr-nfr-civ-dev-hygiene-001-intent.md:4
- `FR-NFR-CIV-PORT-001`
  - spec: docs/traceability/fr-nfr-civ-port-001/fr-nfr-civ-port-001-intent.md:1, docs/traceability/fr-nfr-civ-port-001/fr-nfr-civ-port-001-intent.md:4
- `FR-NFR-CIV-PORT-002`
  - spec: docs/traceability/fr-nfr-civ-port-002/fr-nfr-civ-port-002-intent.md:1, docs/traceability/fr-nfr-civ-port-002/fr-nfr-civ-port-002-intent.md:4
- `FR-NFR-CIV-PORT-003`
  - spec: docs/traceability/fr-nfr-civ-port-003/fr-nfr-civ-port-003-intent.md:1, docs/traceability/fr-nfr-civ-port-003/fr-nfr-civ-port-003-intent.md:4
- `FR-NFR-S-01`
  - spec: docs/traceability/fr-nfr-s-01/fr-nfr-s-01-intent.md:1, docs/traceability/fr-nfr-s-01/fr-nfr-s-01-intent.md:4
- `FR-NFR-S-02`
  - spec: docs/traceability/fr-nfr-s-02/fr-nfr-s-02-intent.md:1, docs/traceability/fr-nfr-s-02/fr-nfr-s-02-intent.md:4
- `FR-NFR-S-03`
  - spec: docs/traceability/fr-nfr-s-03/fr-nfr-s-03-intent.md:1, docs/traceability/fr-nfr-s-03/fr-nfr-s-03-intent.md:4
- `FR-NFR-S-04`
  - spec: docs/traceability/fr-nfr-s-04/fr-nfr-s-04-intent.md:1, docs/traceability/fr-nfr-s-04/fr-nfr-s-04-intent.md:4
- `FR-NFR-S-05`
  - spec: docs/traceability/fr-nfr-s-05/fr-nfr-s-05-intent.md:1, docs/traceability/fr-nfr-s-05/fr-nfr-s-05-intent.md:4
- `FR-NFR-S-06`
  - spec: docs/traceability/fr-nfr-s-06/fr-nfr-s-06-intent.md:1, docs/traceability/fr-nfr-s-06/fr-nfr-s-06-intent.md:4

## Implemented but untested IDs (95)

- `FR-ASSET-PIPELINE-001`
  - spec: docs/traceability/fr-asset-pipeline-001/fr-asset-pipeline-001-intent.md:1, docs/traceability/fr-asset-pipeline-001/fr-asset-pipeline-001-intent.md:4, docs/traceability/fr-asset-pipeline-001/fr-asset-pipeline-001-intent.md:13
  - code: crates/asset-pipeline/src/lib.rs:3, crates/asset-pipeline/src/lib.rs:18, crates/asset-pipeline/src/lib.rs:60
- `FR-ASSET-PIPELINE-002`
  - spec: docs/traceability/fr-asset-pipeline-002/fr-asset-pipeline-002-intent.md:1, docs/traceability/fr-asset-pipeline-002/fr-asset-pipeline-002-intent.md:4
  - code: crates/asset-pipeline/Cargo.toml:8, crates/asset-pipeline/src/bin/svg_export.rs:20, crates/asset-pipeline/src/error.rs:9
- `FR-CIV-ASSET-MANI-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212, docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:1, docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:6
  - code: crates/asset-pipeline/src/manifest.rs:3, crates/asset-pipeline/src/manifest.rs:59
- `FR-CIV-ASSET-MANI-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213, docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:1, docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:6
  - code: crates/asset-pipeline/src/manifest.rs:4, crates/asset-pipeline/src/manifest.rs:88
- `FR-CIV-BELIEF-001`
  - spec: docs/traceability/fr-civ-belief-001/fr-civ-belief-001-intent.md:1, docs/traceability/fr-civ-belief-001/fr-civ-belief-001-intent.md:4, docs/traceability/fr-civ-belief-001/fr-civ-belief-001-intent.md:30
  - code: crates/engine/src/religion.rs:40, crates/engine/src/religion.rs:55, crates/engine/src/religion.rs:88
- `FR-CIV-BEVY-013`
  - spec: docs/development-guide/p-w1-kickoff.md:125, docs/traceability/fr-3d-matrix.md:178, docs/traceability/full-traceability-matrix.md:307
  - code: clients/bevy-ref/src/live_minimap.rs:6
- `FR-CIV-BEVY-014`
  - spec: docs/development-guide/p-w1-kickoff.md:82, docs/traceability/fr-3d-matrix.md:179, docs/traceability/full-traceability-matrix.md:308
  - code: clients/bevy-ref/src/live_stream.rs:3
- `FR-CIV-BEVY-015`
  - spec: docs/development-guide/p-w1-kickoff.md:127, docs/traceability/fr-3d-matrix.md:180, docs/traceability/full-traceability-matrix.md:309
  - code: clients/bevy-ref/src/live_focus.rs:3
- `FR-CIV-BEVY-017`
  - spec: docs/development-guide/p-w1-kickoff.md:129, docs/traceability/fr-3d-matrix.md:182, docs/traceability/full-traceability-matrix.md:311
  - code: clients/bevy-ref/src/lib.rs:557
- `FR-CIV-BEVY-018`
  - spec: docs/development-guide/p-w1-kickoff.md:130, docs/traceability/fr-3d-matrix.md:183, docs/traceability/full-traceability-matrix.md:312
  - code: clients/bevy-ref/src/ws_client.rs:652
- `FR-CIV-BEVY-019`
  - spec: docs/development-guide/p-w1-kickoff.md:131, docs/traceability/fr-3d-matrix.md:184, docs/traceability/full-traceability-matrix.md:313
  - code: clients/bevy-ref/src/live_pick.rs:3
- `FR-CIV-BEVY-020`
  - spec: docs/development-guide/p-w1-kickoff.md:132, docs/traceability/fr-3d-matrix.md:185, docs/traceability/full-traceability-matrix.md:314
  - code: clients/bevy-ref/src/lib.rs:1113
- `FR-CIV-BEVY-036`
  - spec: docs/traceability/fr-civ-bevy-036/fr-civ-bevy-036-intent.md:1, docs/traceability/fr-civ-bevy-036/fr-civ-bevy-036-intent.md:4, docs/traceability/fr-civ-bevy-036/fr-civ-bevy-036-intent.md:26
  - code: clients/bevy-ref/src/menus.rs:4, clients/bevy-ref/src/menus.rs:1348
- `FR-CIV-CA-011`
  - spec: docs/traceability/fr-civ-ca-011/fr-civ-ca-011-intent.md:1, docs/traceability/fr-civ-ca-011/fr-civ-ca-011-intent.md:4
  - code: crates/voxel/src/fluid_ca.rs:1360, crates/voxel/src/fluid_ca.rs:1698
- `FR-CIV-CARAVAN-001`
  - spec: docs/traceability/fr-civ-caravan-001/fr-civ-caravan-001-intent.md:1, docs/traceability/fr-civ-caravan-001/fr-civ-caravan-001-intent.md:3, docs/traceability/fr-civ-caravan-001/fr-civ-caravan-001-intent.md:5
  - code: crates/engine/src/caravan.rs:3
- `FR-CIV-CLIENT-011`
  - spec: docs/traceability/fr-civ-client-011/fr-civ-client-011-intent.md:1, docs/traceability/fr-civ-client-011/fr-civ-client-011-intent.md:4, docs/traceability/fr-civ-client-011/fr-civ-client-011-intent.md:19
  - code: clients/bevy-ref/src/tutorial.rs:3, clients/bevy-ref/src/tutorial.rs:4
- `FR-CIV-CLIENT-013`
  - spec: docs/traceability/fr-civ-client-013/fr-civ-client-013-intent.md:1, docs/traceability/fr-civ-client-013/fr-civ-client-013-intent.md:4, docs/traceability/fr-civ-client-013/fr-civ-client-013-intent.md:20
  - code: clients/bevy-ref/src/civ_history.rs:2, clients/bevy-ref/src/civ_history.rs:3
- `FR-CIV-COHESION-001`
  - spec: docs/traceability/fr-civ-cohesion-001/fr-civ-cohesion-001-intent.md:1, docs/traceability/fr-civ-cohesion-001/fr-civ-cohesion-001-intent.md:4
  - code: crates/engine/src/engine/social_settlement_phases.rs:272
- `FR-CIV-CONTENT-001`
  - spec: docs/traceability/fr-civ-content-001/fr-civ-content-001-intent.md:1, docs/traceability/fr-civ-content-001/fr-civ-content-001-intent.md:4, docs/traceability/fr-civ-content-001/fr-civ-content-001-intent.md:27
  - code: crates/engine/src/emergence_coupling.rs:471
- `FR-CIV-DIPLOMACY-004`
  - spec: docs/traceability/fr-civ-diplomacy-004/fr-civ-diplomacy-004-intent.md:1, docs/traceability/fr-civ-diplomacy-004/fr-civ-diplomacy-004-intent.md:4, docs/traceability/fr-civ-diplomacy-004/fr-civ-diplomacy-004-intent.md:39
  - code: crates/diplomacy/src/stance.rs:1, crates/engine/src/engine.rs:473, crates/engine/src/engine.rs:1045
- `FR-CIV-EMERGE-DASH-001`
  - spec: docs/traceability/fr-civ-emerge-dash-001/fr-civ-emerge-dash-001-intent.md:1, docs/traceability/fr-civ-emerge-dash-001/fr-civ-emerge-dash-001-intent.md:4
  - code: clients/bevy-ref/src/emergence_dashboard.rs:3
- `FR-CIV-EMERGENT-MIGRATION-001`
  - spec: docs/traceability/fr-civ-emergent-migration-001/fr-civ-emergent-migration-001-intent.md:1, docs/traceability/fr-civ-emergent-migration-001/fr-civ-emergent-migration-001-intent.md:3, docs/traceability/fr-civ-emergent-migration-001/fr-civ-emergent-migration-001-intent.md:5
  - code: crates/engine/src/emergent_migration.rs:1, crates/engine/src/emergent_migration.rs:25
- `FR-CIV-FAMINE-001`
  - spec: docs/traceability/fr-civ-climate-3/fr-civ-climate-3-intent.md:21, docs/traceability/fr-civ-famine-001/fr-civ-famine-001-intent.md:1, docs/traceability/fr-civ-famine-001/fr-civ-famine-001-intent.md:4
  - code: crates/engine/src/famine.rs:1
- `FR-CIV-FEST-001`
  - spec: docs/traceability/fr-civ-fest-001/fr-civ-fest-001-intent.md:1, docs/traceability/fr-civ-fest-001/fr-civ-fest-001-intent.md:4
  - code: crates/engine/src/festivals.rs:1
- `FR-CIV-GAME-001`
  - spec: docs/traceability/fr-civ-game-001/fr-civ-game-001-intent.md:1, docs/traceability/fr-civ-game-001/fr-civ-game-001-intent.md:4, docs/traceability/fr-civ-game-001/fr-civ-game-001-intent.md:29
  - code: clients/bevy-ref/src/gameplay_hud.rs:3, clients/bevy-ref/src/lib.rs:541, clients/bevy-ref/src/outcome_overlay.rs:3
- `FR-CIV-GAME-003`
  - spec: docs/traceability/fr-civ-game-003/fr-civ-game-003-intent.md:1, docs/traceability/fr-civ-game-003/fr-civ-game-003-intent.md:4, docs/traceability/fr-civ-game-003/fr-civ-game-003-intent.md:20
  - code: clients/bevy-ref/src/era_hud.rs:2, crates/engine/src/era.rs:1
- `FR-CIV-GENETICS-SEED-001`
  - spec: docs/traceability/fr-civ-genetics-seed-001/fr-civ-genetics-seed-001-intent.md:1, docs/traceability/fr-civ-genetics-seed-001/fr-civ-genetics-seed-001-intent.md:4, docs/traceability/fr-civ-genetics-seed-001/fr-civ-genetics-seed-001-intent.md:26
  - code: crates/engine/src/engine/engine_tests.rs:2941
- `FR-CIV-GENETICS-SEED-002`
  - spec: docs/traceability/fr-civ-genetics-seed-002/fr-civ-genetics-seed-002-intent.md:1, docs/traceability/fr-civ-genetics-seed-002/fr-civ-genetics-seed-002-intent.md:4, docs/traceability/fr-civ-genetics-seed-002/fr-civ-genetics-seed-002-intent.md:26
  - code: crates/engine/src/engine/engine_tests.rs:2991
- `FR-CIV-GENETICS-SEED-003`
  - spec: docs/traceability/fr-civ-genetics-seed-003/fr-civ-genetics-seed-003-intent.md:1, docs/traceability/fr-civ-genetics-seed-003/fr-civ-genetics-seed-003-intent.md:4, docs/traceability/fr-civ-genetics-seed-003/fr-civ-genetics-seed-003-intent.md:25
  - code: crates/engine/src/engine/engine_tests.rs:3037
- `FR-CIV-GODTOOL-001`
  - spec: docs/traceability/fr-civ-godtool-001/fr-civ-godtool-001-intent.md:1, docs/traceability/fr-civ-godtool-001/fr-civ-godtool-001-intent.md:4
  - code: crates/civis-mcp/src/server.rs:148
- `FR-CIV-GOV-200`
  - spec: docs/traceability/fr-civ-gov-200/fr-civ-gov-200-intent.md:1, docs/traceability/fr-civ-gov-200/fr-civ-gov-200-intent.md:3, docs/traceability/fr-civ-gov-200/fr-civ-gov-200-intent.md:5
  - code: crates/engine/src/engine/social_settlement_phases.rs:179
- `FR-CIV-L10N-010`
  - spec: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:49
  - code: crates/i18n/src/lib.rs:98
- `FR-CIV-L10N-020`
  - spec: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:50
  - code: crates/i18n/src/lib.rs:194
- `FR-CIV-LEGENDS-010`
  - spec: docs/traceability/fr-civ-legends-010/fr-civ-legends-010-intent.md:1, docs/traceability/fr-civ-legends-010/fr-civ-legends-010-intent.md:4, docs/traceability/fr-civ-legends-010/fr-civ-legends-010-intent.md:22
  - code: crates/engine/src/engine.rs:1040
- `FR-CIV-LIFE-004`
  - spec: docs/design/civ-003-emergent-lifecycle.md:101
  - code: crates/needs/src/lib.rs:285
- `FR-CIV-MIGRATION-001`
  - spec: docs/traceability/fr-emergence-matrix.md:215
  - code: crates/emergence-migration/src/lib.rs:359, crates/emergence-migration/src/lib.rs:371, crates/emergence-migration/src/lib.rs:381
- `FR-CIV-MIGRATION-002`
  - spec: docs/traceability/fr-emergence-matrix.md:216
  - code: crates/emergence-migration/src/lib.rs:384, crates/emergence-migration/src/lib.rs:492
- `FR-CIV-MIGRATION-003`
  - spec: docs/traceability/fr-emergence-matrix.md:217
  - code: crates/emergence-migration/src/lib.rs:386, crates/emergence-migration/src/lib.rs:492
- `FR-CIV-MIGRATION-004`
  - spec: docs/traceability/fr-emergence-matrix.md:218
  - code: crates/emergence-migration/src/lib.rs:317, crates/emergence-migration/src/lib.rs:345, crates/emergence-migration/src/lib.rs:388
- `FR-CIV-MIGRATION-005`
  - spec: docs/traceability/fr-emergence-matrix.md:219
  - code: crates/emergence-migration/src/lib.rs:390
- `FR-CIV-PBR-011`
  - spec: docs/traceability/fr-civ-pbr-011/fr-civ-pbr-011-intent.md:1, docs/traceability/fr-civ-pbr-011/fr-civ-pbr-011-intent.md:4
  - code: CHANGELOG.md:17, crates/voxel/src/atlas/gpu_atlas.rs:1, crates/voxel/src/atlas/gpu_atlas.rs:40
- `FR-CIV-REL-004`
  - spec: docs/traceability/fr-emergence-matrix.md:79
  - code: crates/engine/src/disasters.rs:65
- `FR-CIV-SAVE-003`
  - spec: docs/traceability/civis-tracelinks.md:66, docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:1, docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:6
  - code: crates/server/src/jsonrpc.rs:1574, crates/server/src/jsonrpc.rs:1590
- `FR-CIV-SAVE-004`
  - spec: docs/traceability/civis-tracelinks.md:67, docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:1, docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:6
  - code: crates/watch/src/saves_api.rs:180, crates/watch/src/saves_api.rs:232, crates/watch/src/saves_api.rs:250
- `FR-CIV-SOCIAL-001-INSTITUTIONS`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:225, PLAN.md:147, PLAN.md:148
  - code: crates/civ-institutions/src/policy.rs:1
- `FR-CIV-TACTICS-032`
  - spec: docs/development-guide/p-w1-kickoff.md:31, docs/traceability/fr-3d-matrix.md:119, docs/traceability/full-traceability-matrix.md:227
  - code: crates/engine/src/engine.rs:355
- `FR-CIV-WARFARE-001`
  - spec: docs/traceability/fr-civ-warfare-001/fr-civ-warfare-001-intent.md:1, docs/traceability/fr-civ-warfare-001/fr-civ-warfare-001-intent.md:4
  - code: crates/tactics/src/war_from_diplomacy.rs:1
- `FR-CIV-WARFARE-002`
  - spec: docs/traceability/fr-civ-warfare-002/fr-civ-warfare-002-intent.md:1, docs/traceability/fr-civ-warfare-002/fr-civ-warfare-002-intent.md:4
  - code: crates/tactics/src/doctrine_evolution.rs:1
- `FR-CIV-WARFARE-003`
  - spec: docs/traceability/fr-civ-warfare-003/fr-civ-warfare-003-intent.md:1, docs/traceability/fr-civ-warfare-003/fr-civ-warfare-003-intent.md:4
  - code: crates/tactics/src/war_economy.rs:1
- `FR-CIV-WARFARE-004`
  - spec: docs/traceability/fr-civ-warfare-004/fr-civ-warfare-004-intent.md:1, docs/traceability/fr-civ-warfare-004/fr-civ-warfare-004-intent.md:4
  - code: crates/tactics/src/war_legends.rs:1
- `FR-DIP-002`
  - spec: docs/traceability/fr-dip-002/fr-dip-002-intent.md:1, docs/traceability/fr-dip-002/fr-dip-002-intent.md:3, docs/traceability/fr-dip-002/fr-dip-002-intent.md:5
  - code: crates/diplomacy/src/effects.rs:1
- `FR-ECON-EMERGE-001`
  - spec: docs/traceability/fr-econ-emerge-001/fr-econ-emerge-001-intent.md:1, docs/traceability/fr-econ-emerge-001/fr-econ-emerge-001-intent.md:4, docs/traceability/fr-econ-emerge-001/fr-econ-emerge-001-intent.md:20
  - code: crates/economy/src/prices.rs:1, crates/economy/src/prices.rs:2
- `FR-ECON-EMERGE-002`
  - spec: docs/traceability/fr-econ-emerge-002/fr-econ-emerge-002-intent.md:1, docs/traceability/fr-econ-emerge-002/fr-econ-emerge-002-intent.md:4, docs/traceability/fr-econ-emerge-002/fr-econ-emerge-002-intent.md:20
  - code: crates/economy/src/trade.rs:1, crates/economy/src/trade.rs:2
- `FR-ECON-EMERGE-004`
  - spec: docs/traceability/fr-econ-emerge-004/fr-econ-emerge-004-intent.md:1, docs/traceability/fr-econ-emerge-004/fr-econ-emerge-004-intent.md:4, docs/traceability/fr-econ-emerge-004/fr-econ-emerge-004-intent.md:20
  - code: crates/economy/src/shocks.rs:1, crates/economy/src/shocks.rs:2
- `FR-EMG-009`
  - spec: docs/traceability/fr-emg-009/fr-emg-009-intent.md:1, docs/traceability/fr-emg-009/fr-emg-009-intent.md:4
  - code: crates/emergence-oracle/src/oracles/migration.rs:1, crates/emergence-oracle/src/oracles/migration.rs:17
- `FR-EMG-010`
  - spec: docs/traceability/fr-emg-010/fr-emg-010-intent.md:1, docs/traceability/fr-emg-010/fr-emg-010-intent.md:4, docs/traceability/fr-emg-010/fr-emg-010-intent.md:9
  - code: crates/emergence-oracle/src/oracles/epidemic.rs:1, crates/emergence-oracle/src/oracles/epidemic.rs:17, crates/emergence-oracle/src/oracles/trade.rs:1
- `FR-EMG-012`
  - spec: docs/traceability/fr-emg-012/fr-emg-012-intent.md:1, docs/traceability/fr-emg-012/fr-emg-012-intent.md:4
  - code: crates/emergence-oracle/src/oracles/festival.rs:1, crates/emergence-oracle/src/oracles/festival.rs:18
- `FR-EMG-013`
  - spec: docs/traceability/fr-civ-climate-3/fr-civ-climate-3-intent.md:22, docs/traceability/fr-civ-climate-4/fr-civ-climate-4-intent.md:22, docs/traceability/fr-emg-013/fr-emg-013-intent.md:1
  - code: crates/emergence-oracle/src/oracles/disaster.rs:1, crates/emergence-oracle/src/oracles/disaster.rs:17
- `FR-EMG-014`
  - spec: docs/traceability/fr-emg-014/fr-emg-014-intent.md:1, docs/traceability/fr-emg-014/fr-emg-014-intent.md:4
  - code: crates/emergence-oracle/src/oracles/mood.rs:1, crates/emergence-oracle/src/oracles/mood.rs:17
- `FR-EMG-015`
  - spec: docs/traceability/fr-emg-015/fr-emg-015-intent.md:1, docs/traceability/fr-emg-015/fr-emg-015-intent.md:4
  - code: crates/emergence-oracle/src/oracles/stratification.rs:1, crates/emergence-oracle/src/oracles/stratification.rs:18
- `FR-EMG-016`
  - spec: docs/traceability/fr-emg-016/fr-emg-016-intent.md:1, docs/traceability/fr-emg-016/fr-emg-016-intent.md:4
  - code: crates/emergence-oracle/src/oracles/religious_conflict.rs:1, crates/emergence-oracle/src/oracles/religious_conflict.rs:17
- `FR-EMG-017`
  - spec: docs/traceability/fr-emg-017/fr-emg-017-intent.md:1, docs/traceability/fr-emg-017/fr-emg-017-intent.md:4, docs/traceability/fr-emg-017/fr-emg-017-intent.md:21
  - code: crates/emergence-oracle/src/oracles/expansion.rs:1, crates/emergence-oracle/src/oracles/expansion.rs:17
- `FR-EMG-018`
  - spec: docs/traceability/fr-emg-018/fr-emg-018-intent.md:1, docs/traceability/fr-emg-018/fr-emg-018-intent.md:4
  - code: crates/emergence-oracle/src/oracles/migration_flow.rs:1, crates/emergence-oracle/src/oracles/migration_flow.rs:17
- `FR-EMG-019`
  - spec: docs/traceability/fr-emg-019/fr-emg-019-intent.md:1, docs/traceability/fr-emg-019/fr-emg-019-intent.md:4
  - code: crates/emergence-oracle/src/oracles/coastal_settlement.rs:1, crates/emergence-oracle/src/oracles/coastal_settlement.rs:17
- `FR-EMG-020`
  - spec: docs/traceability/fr-emg-020/fr-emg-020-intent.md:1, docs/traceability/fr-emg-020/fr-emg-020-intent.md:4, docs/traceability/fr-emg-020/fr-emg-020-intent.md:23
  - code: crates/emergence-oracle/src/oracles/river_trade.rs:1, crates/emergence-oracle/src/oracles/river_trade.rs:17
- `FR-EMG-021`
  - spec: docs/traceability/fr-emg-021/fr-emg-021-intent.md:1, docs/traceability/fr-emg-021/fr-emg-021-intent.md:4
  - code: crates/emergence-oracle/src/oracles/mountain_pass.rs:1, crates/emergence-oracle/src/oracles/mountain_pass.rs:17
- `FR-EMG-022`
  - spec: docs/traceability/fr-emg-022/fr-emg-022-intent.md:1, docs/traceability/fr-emg-022/fr-emg-022-intent.md:4
  - code: crates/emergence-oracle/src/oracles/desert_caravan.rs:1, crates/emergence-oracle/src/oracles/desert_caravan.rs:17
- `FR-EMG-023`
  - spec: docs/traceability/fr-emg-023/fr-emg-023-intent.md:1, docs/traceability/fr-emg-023/fr-emg-023-intent.md:4
  - code: crates/emergence-oracle/src/oracles/genetics.rs:1, crates/emergence-oracle/src/oracles/genetics.rs:60
- `FR-EMG-024`
  - spec: docs/traceability/fr-emg-024/fr-emg-024-intent.md:1, docs/traceability/fr-emg-024/fr-emg-024-intent.md:4, docs/traceability/fr-emg-024/fr-emg-024-intent.md:9
  - code: crates/emergence-oracle/src/oracles/i18n.rs:1, crates/emergence-oracle/src/oracles/i18n.rs:46, crates/emergence-oracle/src/oracles/powers.rs:1
- `FR-LANGUAGE-001`
  - spec: docs/traceability/fr-language-001/fr-language-001-intent.md:1, docs/traceability/fr-language-001/fr-language-001-intent.md:4, docs/traceability/fr-language-001/fr-language-001-intent.md:21
  - code: crates/engine/src/engine/culture_phases.rs:230, crates/engine/src/engine.rs:169, crates/engine/src/engine.rs:877
- `FR-VIEWPORT-001`
  - spec: docs/traceability/fr-viewport-001/fr-viewport-001-intent.md:1, docs/traceability/fr-viewport-001/fr-viewport-001-intent.md:4, docs/traceability/fr-viewport-001/fr-viewport-001-intent.md:27
  - code: crates/civis-cli/src/bin/three_d_quality.rs:54
- `NFR-C-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2041, docs/traceability/index.md:1151, docs/traceability/nfr-c-01/nfr-c-01-spec.md:1
  - code: crates/engine/src/hash_chain.rs:8
- `NFR-C-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2043, docs/traceability/index.md:1153, docs/traceability/nfr-c-03/nfr-c-03-spec.md:1
  - code: crates/engine/src/fixed_math.rs:8
- `NFR-C-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2044, docs/traceability/index.md:1154, docs/traceability/nfr-c-04/nfr-c-04-spec.md:1
  - code: crates/engine/src/emergence.rs:38
- `NFR-C-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2045, docs/traceability/index.md:1155, docs/traceability/nfr-c-05/nfr-c-05-spec.md:1
  - code: crates/engine/src/building_emergence.rs:3
- `NFR-C-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2046, docs/traceability/index.md:1156, docs/traceability/nfr-c-06/nfr-c-06-spec.md:1
  - code: crates/legends/src/model.rs:246
- `NFR-C-07`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2047, docs/traceability/index.md:1157, docs/traceability/nfr-c-07/nfr-c-07-spec.md:1
  - code: crates/engine/src/hash_chain.rs:83
- `NFR-CIV-LEGENDS-LOUD-03`
  - spec: docs/design/legends-engine.md:453, docs/traceability/index.md:1171, docs/traceability/TRACEABILITY-GAP-REPORT-20260916.md:306
  - code: crates/legends/src/decay.rs:166, crates/legends/src/worker.rs:40
- `NFR-CIV-MAINT-001`
  - spec: docs/reference/non-functional-requirements.md:260, docs/reference/non-functional-requirements.md:461, docs/reference/non-functional-requirements.md:499
  - code: scripts/quality/quality-gate.sh:420
- `NFR-CIV-MAINT-002`
  - spec: docs/reference/non-functional-requirements.md:475, docs/reference/non-functional-requirements.md:541, docs/reference/non-functional-requirements.md:581
  - code: scripts/quality/quality-gate.sh:514, scripts/quality/quality-gate.sh:518
- `NFR-CIV-MAINT-003`
  - spec: docs/reference/non-functional-requirements.md:489, docs/reference/non-functional-requirements.md:582, docs/traceability/fr-nfr-matrix.md:105
  - code: scripts/quality/quality-gate.sh:568, scripts/quality/quality-gate.sh:572
- `NFR-CIV-MAINT-004`
  - spec: docs/reference/non-functional-requirements.md:503, docs/reference/non-functional-requirements.md:583, docs/reference/non-functional-requirements.md:607
  - code: crates/audio/src/lib.rs:50
- `NFR-P-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2053, docs/traceability/index.md:1217, docs/traceability/nfr-p-01/nfr-p-01-spec.md:1
  - code: crates/engine/benches/tick_bench.rs:7
- `NFR-P-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2054, docs/traceability/index.md:1218, docs/traceability/nfr-p-02/nfr-p-02-spec.md:1
  - code: crates/engine/benches/tick_bench.rs:7
- `NFR-P-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2055, docs/traceability/index.md:1219, docs/traceability/nfr-p-03/nfr-p-03-spec.md:1
  - code: crates/engine/benches/tick_bench.rs:7
- `NFR-P-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2056, docs/traceability/index.md:1220, docs/traceability/nfr-p-04/nfr-p-04-spec.md:1
  - code: crates/engine/benches/tick_bench.rs:11
- `NFR-P-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2057, docs/traceability/index.md:1221, docs/traceability/nfr-p-05/nfr-p-05-spec.md:1
  - code: crates/engine/benches/tick_bench.rs:12
- `NFR-P-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2058, docs/traceability/index.md:1222, docs/traceability/nfr-p-06/nfr-p-06-spec.md:1
  - code: crates/server/src/ws_bridge.rs:407
- `NFR-P-07`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2059, docs/traceability/index.md:1223, docs/traceability/nfr-p-07/nfr-p-07-spec.md:1
  - code: crates/server/src/jsonrpc.rs:635
- `NFR-P-08`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2060, docs/traceability/index.md:1224, docs/traceability/nfr-p-08/nfr-p-08-spec.md:1
  - code: crates/server/src/ws_bridge.rs:689
- `NFR-S-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2067, docs/traceability/index.md:1232, docs/traceability/nfr-s-02/nfr-s-02-spec.md:1
  - code: crates/server/src/perf_budgets.rs:1, crates/server/src/perf_budgets.rs:11, crates/server/src/perf_budgets.rs:27
- `NFR-S-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2068, docs/traceability/index.md:1233, docs/traceability/nfr-s-03/nfr-s-03-spec.md:1
  - code: crates/server/src/perf_budgets.rs:1, crates/server/src/perf_budgets.rs:12, crates/server/src/perf_budgets.rs:31
- `NFR-S-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2069, docs/traceability/index.md:1234, docs/traceability/nfr-s-04/nfr-s-04-spec.md:1
  - code: crates/server/src/perf_budgets.rs:2, crates/server/src/perf_budgets.rs:13, crates/server/src/perf_budgets.rs:36
- `NFR-S-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2070, docs/traceability/index.md:1235, docs/traceability/nfr-s-05/nfr-s-05-spec.md:1
  - code: crates/server/src/perf_budgets.rs:2, crates/server/src/perf_budgets.rs:14, crates/server/src/perf_budgets.rs:40
- `NFR-S-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2071, docs/traceability/index.md:1236, docs/traceability/nfr-s-06/nfr-s-06-spec.md:1
  - code: crates/server/src/perf_budgets.rs:2, crates/server/src/perf_budgets.rs:15, crates/server/src/perf_budgets.rs:44

## Code-only IDs (missing spec/traceability) (1)

- `FR-NFR-CIV-PERF-001`

## Placeholder-only coverage (weakest evidence) (56)

These IDs are counted `COVERED` on tests whose file matches the auto-generated placeholder pattern above. Their tests assert properties of shared types, not the requirement, so treat the coverage as unverified until a real oracle exists.

`193` placeholder test files affect `56` IDs.

- `FR-SESSION-001`
- `FR-SESSION-002`
- `FR-SESSION-003`
- `FR-SESSION-004`
- `FR-SESSION-005`
- `FR-SESSION-006`
- `FR-SESSION-007`
- `FR-SESSION-008`
- `FR-SESSION-009`
- `FR-SESSION-010`
- `FR-SESSION-011`
- `FR-SESSION-012`
- `FR-SESSION-013`
- `FR-SESSION-015`
- `FR-SESSION-016`
- `FR-SESSION-017`
- `FR-SESSION-018`
- `FR-SESSION-019`
- `FR-SESSION-020`
- `FR-SESSION-021`
- `FR-SESSION-022`
- `FR-SESSION-023`
- `FR-SESSION-024`
- `FR-SESSION-025`
- `FR-SESSION-026`
- `FR-SESSION-027`
- `FR-SESSION-028`
- `FR-SESSION-029`
- `FR-SESSION-030`
- `FR-SESSION-031`
- `FR-SESSION-032`
- `FR-SESSION-033`
- `FR-SOC-CIV-001`
- `FR-SOC-CIV-002`
- `FR-SOC-COH-001`
- `FR-SOC-COH-002`
- `FR-SOC-COH-003`
- `FR-SOC-COH-004`
- `FR-SOC-INS-001`
- `FR-SOC-INS-002`
- `FR-SOC-INS-003`
- `FR-SOC-INS-004`
- `FR-SOC-INS-005`
- `FR-SOC-INS-006`
- `FR-SOC-INS-007`
- `FR-SOC-INT-001`
- `FR-SOC-INT-002`
- `FR-SOC-INT-003`
- `FR-SOC-INT-004`
- `FR-SOC-INTG-001`
- `FR-SOC-INTG-002`
- `FR-SOC-INTG-003`
- `FR-SOC-INTG-004`
- `FR-SOC-INTG-005`
- `FR-SOC-INTG-006`
- `FR-SOC-INTG-007`

