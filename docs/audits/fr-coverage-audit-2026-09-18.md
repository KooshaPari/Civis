# FR Coverage Audit

**Generated:** 2026-09-18  
**Source inventory:** `docs/audits/_id_inventory_v3.json`  
**Total IDs scanned:** 1508

## Status legend

| Status | Meaning |
|--------|---------|
| `COVERED` | spec/trace + code + test all present |
| `IMPL-NO-TEST` | spec/trace + code present, no test reference |
| `SPEC-ONLY` | spec/trace present, no implementing code found |
| `CODE-ONLY-no-spec` | code present, no spec/traceability reference |

## Summary

| Status | Count | % |
|--------|------:|--:|
| `COVERED` | 577 | 38.3 |
| `IMPL-NO-TEST` | 71 | 4.7 |
| `SPEC-ONLY` | 645 | 42.8 |
| `CODE-ONLY-no-spec` | 215 | 14.3 |
| **Total** | **1508** | **100.0** |

## Coverage by epic

| Epic | Total | COVERED | IMPL-NO-TEST | SPEC-ONLY | CODE-ONLY-no-spec |
|------|------:|--------:|-------------:|----------:|------------------:|
| FR-AI | 7 | 7 | 0 | 0 | 0 |
| FR-API | 4 | 4 | 0 | 0 | 0 |
| FR-ASSET | 4 | 4 | 0 | 0 | 0 |
| FR-ASSET-PIPELINE | 2 | 0 | 0 | 0 | 2 |
| FR-AUD | 3 | 3 | 0 | 0 | 0 |
| FR-CIV | 16 | 2 | 0 | 13 | 1 |
| FR-CIV-0001-TICK | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-3D | 16 | 0 | 0 | 16 | 0 |
| FR-CIV-ACCESS | 2 | 0 | 0 | 0 | 2 |
| FR-CIV-ACT | 4 | 4 | 0 | 0 | 0 |
| FR-CIV-ACTOR | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-ACTOR-001-LIFECYCLE | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-AGENTS | 17 | 17 | 0 | 0 | 0 |
| FR-CIV-AGGRESSION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-AI | 15 | 10 | 0 | 5 | 0 |
| FR-CIV-ARCH | 9 | 7 | 0 | 1 | 1 |
| FR-CIV-ARCH-A | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-ARCH-B | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-ARCH-C | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-ARCH-D | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-ARCH-NOSVG | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-ASSET | 20 | 0 | 0 | 20 | 0 |
| FR-CIV-ASSET-MANI | 2 | 0 | 0 | 2 | 0 |
| FR-CIV-ASSET-QUAL | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-AUDIO | 12 | 8 | 0 | 4 | 0 |
| FR-CIV-BELIEF | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-BEVY | 21 | 9 | 8 | 0 | 4 |
| FR-CIV-BIO | 3 | 3 | 0 | 0 | 0 |
| FR-CIV-BRUSH | 13 | 0 | 0 | 13 | 0 |
| FR-CIV-BUILD | 15 | 14 | 0 | 1 | 0 |
| FR-CIV-CA | 11 | 10 | 0 | 0 | 1 |
| FR-CIV-CARAVAN | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CLIENT | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-CLIENT-GODOT | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-CLIMATE | 7 | 3 | 0 | 0 | 4 |
| FR-CIV-COHESION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CONSTRUCTION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CONTENT | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CORE | 21 | 7 | 0 | 13 | 1 |
| FR-CIV-CORE-DET | 3 | 0 | 0 | 3 | 0 |
| FR-CIV-CULT | 3 | 3 | 0 | 0 | 0 |
| FR-CIV-CULTURE | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-DET | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-DIFFUSION | 16 | 16 | 0 | 0 | 0 |
| FR-CIV-DIPLO | 16 | 8 | 0 | 0 | 8 |
| FR-CIV-DIPLO-001-RELATIONS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-DIPLO-002-SHADOW | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-DIPLOMACY | 2 | 0 | 0 | 0 | 2 |
| FR-CIV-ECON | 6 | 5 | 0 | 0 | 1 |
| FR-CIV-ECON-001-MARKET | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-ECON-002-JOULE | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-ECON-FOCUS | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-EMERG | 5 | 3 | 0 | 2 | 0 |
| FR-CIV-EMERGE-DASH | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-EMERGENCE | 25 | 10 | 0 | 15 | 0 |
| FR-CIV-EMERGENCE-N10 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-N11 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-N12 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-N13 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-EMERGENCE-RELIGION | 2 | 0 | 0 | 2 | 0 |
| FR-CIV-EMERGENT-MIGRATION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-ENGINE-INT | 10 | 9 | 1 | 0 | 0 |
| FR-CIV-ENGINE-REPLAY | 5 | 5 | 0 | 0 | 0 |
| FR-CIV-ERA | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-FAMINE | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-FEST | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-FOG | 5 | 0 | 0 | 5 | 0 |
| FR-CIV-GAME | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-GENETICS | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-GENETICS-SEED | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-GEO | 10 | 0 | 2 | 8 | 0 |
| FR-CIV-GODOT-ATTACH | 5 | 1 | 4 | 0 | 0 |
| FR-CIV-GODOT-F3D0 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-GODOT-UX | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-GODTOOL | 8 | 7 | 0 | 0 | 1 |
| FR-CIV-GOV | 8 | 3 | 0 | 0 | 5 |
| FR-CIV-HUD | 5 | 5 | 0 | 0 | 0 |
| FR-CIV-IDEOLOGY | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-INFOVIEW | 20 | 9 | 0 | 11 | 0 |
| FR-CIV-INFRA | 13 | 13 | 0 | 0 | 0 |
| FR-CIV-INSPECT | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-INSTITUTIONS | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-INT | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-L10N | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-L5 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LANG | 10 | 5 | 0 | 5 | 0 |
| FR-CIV-LAWS | 10 | 10 | 0 | 0 | 0 |
| FR-CIV-LEGENDS | 9 | 8 | 0 | 0 | 1 |
| FR-CIV-LEGENDS-BROWSER | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-CAUSAL | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-CONFIG | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-GAP | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-GRAPH | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-INGEST | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-INSPECT | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-NARRATOR | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-PERF | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-PERSIST | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-PRESIM | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-PRODUCER | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-QUERY | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-RESOLVE | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-SCALE | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-SIG | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LIFE | 20 | 18 | 1 | 1 | 0 |
| FR-CIV-LLM | 6 | 0 | 0 | 6 | 0 |
| FR-CIV-MARKET | 8 | 0 | 0 | 8 | 0 |
| FR-CIV-MCP | 6 | 2 | 0 | 4 | 0 |
| FR-CIV-METRICS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-METRICS-001-TIMESERIES | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-MIGRATION | 5 | 0 | 0 | 5 | 0 |
| FR-CIV-MOD | 21 | 1 | 0 | 20 | 0 |
| FR-CIV-NEEDS-DECAY | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-NOTIFY | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-PBR | 11 | 8 | 0 | 0 | 3 |
| FR-CIV-PERF | 20 | 1 | 0 | 19 | 0 |
| FR-CIV-PERF-BUILD | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-PERF-RT | 3 | 0 | 0 | 3 | 0 |
| FR-CIV-PERF-WEB | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-PLANET | 12 | 9 | 2 | 0 | 1 |
| FR-CIV-POLITY | 8 | 0 | 0 | 8 | 0 |
| FR-CIV-PROTO | 15 | 2 | 0 | 13 | 0 |
| FR-CIV-PROTO3D | 20 | 19 | 1 | 0 | 0 |
| FR-CIV-PSYCHE | 29 | 6 | 2 | 21 | 0 |
| FR-CIV-PSYCHE-N11 | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-QOL | 14 | 0 | 0 | 14 | 0 |
| FR-CIV-REL | 5 | 3 | 0 | 1 | 1 |
| FR-CIV-RELIGION | 2 | 0 | 2 | 0 | 0 |
| FR-CIV-RENDER | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-RES | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH | 13 | 13 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-001-SCENARIO | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-002-SNAPSHOT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-003-EXPORT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-004-REPLAY | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-ROAD | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-RTS | 15 | 2 | 0 | 13 | 0 |
| FR-CIV-RTS-NATION | 2 | 0 | 0 | 2 | 0 |
| FR-CIV-RTS-RENDER | 5 | 0 | 0 | 5 | 0 |
| FR-CIV-RTS-ZOOM | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-SAVE | 4 | 2 | 0 | 2 | 0 |
| FR-CIV-SCALE | 8 | 8 | 0 | 0 | 0 |
| FR-CIV-SERVER | 3 | 2 | 0 | 0 | 1 |
| FR-CIV-SERVER-001-WS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-SERVER-002-PROTO | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-SOCIAL | 2 | 0 | 2 | 0 | 0 |
| FR-CIV-SOCIAL-001-INSTITUTIONS | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-SOCIAL-002-IDEOLOGY | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-SPECIES | 48 | 12 | 0 | 36 | 0 |
| FR-CIV-TACTICS | 65 | 56 | 5 | 4 | 0 |
| FR-CIV-TECH | 21 | 0 | 0 | 21 | 0 |
| FR-CIV-TERRAIN | 6 | 0 | 0 | 6 | 0 |
| FR-CIV-TEST | 7 | 0 | 0 | 0 | 7 |
| FR-CIV-TRAFFIC-LANE | 4 | 4 | 0 | 0 | 0 |
| FR-CIV-UI | 3 | 0 | 3 | 0 | 0 |
| FR-CIV-UNREST | 2 | 1 | 0 | 0 | 1 |
| FR-CIV-UX | 7 | 5 | 2 | 0 | 0 |
| FR-CIV-VEHICLE | 26 | 0 | 0 | 26 | 0 |
| FR-CIV-VERIFY | 10 | 0 | 0 | 10 | 0 |
| FR-CIV-VOXEL | 18 | 17 | 1 | 0 | 0 |
| FR-CIV-VOXEL-DIRTY | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-WAR | 15 | 5 | 0 | 10 | 0 |
| FR-CIV-WAR-001-UNITS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-WAR-002-COMBAT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-WARFARE | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-WEB | 9 | 1 | 8 | 0 | 0 |
| FR-CLIENT | 3 | 3 | 0 | 0 | 0 |
| FR-CLIM | 6 | 6 | 0 | 0 | 0 |
| FR-CORE | 10 | 10 | 0 | 0 | 0 |
| FR-DET | 7 | 0 | 0 | 7 | 0 |
| FR-DIP | 1 | 0 | 0 | 0 | 1 |
| FR-DIPL | 7 | 6 | 0 | 1 | 0 |
| FR-DOC | 1 | 0 | 0 | 1 | 0 |
| FR-ECO | 10 | 0 | 0 | 10 | 0 |
| FR-ECON | 10 | 10 | 0 | 0 | 0 |
| FR-ECON-EMERGE | 3 | 0 | 0 | 0 | 3 |
| FR-EMG | 24 | 0 | 0 | 0 | 24 |
| FR-FR-CORE | 1 | 0 | 0 | 0 | 1 |
| FR-GUARD | 2 | 0 | 0 | 2 | 0 |
| FR-INST | 6 | 6 | 0 | 0 | 0 |
| FR-INT | 1 | 0 | 0 | 1 | 0 |
| FR-LANGUAGE | 1 | 0 | 0 | 0 | 1 |
| FR-LOD | 4 | 4 | 0 | 0 | 0 |
| FR-MET | 1 | 0 | 0 | 1 | 0 |
| FR-METRICS | 5 | 3 | 0 | 2 | 0 |
| FR-MOD | 5 | 5 | 0 | 0 | 0 |
| FR-MUSIC | 1 | 0 | 0 | 0 | 1 |
| FR-NET | 3 | 0 | 0 | 3 | 0 |
| FR-NFR-C | 7 | 0 | 0 | 0 | 7 |
| FR-NFR-CIV-ACC | 4 | 0 | 0 | 0 | 4 |
| FR-NFR-CIV-AI | 3 | 0 | 0 | 0 | 3 |
| FR-NFR-CIV-DET | 2 | 0 | 0 | 0 | 2 |
| FR-NFR-CIV-DEV-HYGIENE | 1 | 0 | 0 | 0 | 1 |
| FR-NFR-CIV-LEGENDS-CONFIG | 1 | 0 | 0 | 0 | 1 |
| FR-NFR-CIV-LEGENDS-LOUD | 1 | 0 | 0 | 0 | 1 |
| FR-NFR-CIV-LEGENDS-PERF | 1 | 0 | 0 | 0 | 1 |
| FR-NFR-CIV-LEGENDS-SCALE | 1 | 0 | 0 | 0 | 1 |
| FR-NFR-CIV-MAINT | 6 | 0 | 0 | 0 | 6 |
| FR-NFR-CIV-PERF | 11 | 0 | 0 | 0 | 11 |
| FR-NFR-CIV-PORT | 3 | 0 | 0 | 0 | 3 |
| FR-NFR-CIV-REL | 3 | 0 | 0 | 0 | 3 |
| FR-NFR-CIV-SCALE | 9 | 0 | 0 | 0 | 9 |
| FR-NFR-CIV-SEC | 4 | 0 | 0 | 0 | 4 |
| FR-NFR-O | 6 | 0 | 0 | 0 | 6 |
| FR-NFR-P | 8 | 0 | 0 | 0 | 8 |
| FR-NFR-R | 6 | 0 | 0 | 0 | 6 |
| FR-NFR-S | 6 | 0 | 0 | 0 | 6 |
| FR-NFR-SCALE | 1 | 0 | 0 | 0 | 1 |
| FR-PERF | 5 | 4 | 0 | 1 | 0 |
| FR-PROT | 6 | 2 | 0 | 4 | 0 |
| FR-PROTO | 5 | 5 | 0 | 0 | 0 |
| FR-REP | 1 | 0 | 0 | 1 | 0 |
| FR-REPLAY | 2 | 2 | 0 | 0 | 0 |
| FR-SAVE | 25 | 3 | 0 | 22 | 0 |
| FR-SESS | 6 | 6 | 0 | 0 | 0 |
| FR-SESSION | 33 | 0 | 0 | 33 | 0 |
| FR-SOC-CIV | 2 | 0 | 0 | 2 | 0 |
| FR-SOC-COH | 4 | 0 | 0 | 4 | 0 |
| FR-SOC-DET | 2 | 0 | 0 | 2 | 0 |
| FR-SOC-FAC | 2 | 0 | 0 | 2 | 0 |
| FR-SOC-HLT | 5 | 0 | 0 | 5 | 0 |
| FR-SOC-IDE | 6 | 0 | 0 | 6 | 0 |
| FR-SOC-INS | 7 | 0 | 0 | 7 | 0 |
| FR-SOC-INT | 4 | 0 | 0 | 4 | 0 |
| FR-SOC-INTG | 7 | 0 | 0 | 7 | 0 |
| FR-SOCI | 6 | 6 | 0 | 0 | 0 |
| FR-STOR | 1 | 0 | 0 | 1 | 0 |
| FR-TEST | 1 | 0 | 0 | 1 | 0 |
| FR-THRY | 4 | 0 | 0 | 4 | 0 |
| FR-UX | 27 | 5 | 0 | 22 | 0 |
| FR-VAL | 1 | 0 | 0 | 1 | 0 |
| FR-VIEWPORT | 1 | 0 | 0 | 0 | 1 |
| NFR-C | 7 | 0 | 0 | 7 | 0 |
| NFR-CIV | 13 | 0 | 0 | 13 | 0 |
| NFR-CIV-ACC | 4 | 0 | 3 | 1 | 0 |
| NFR-CIV-AI | 3 | 0 | 2 | 1 | 0 |
| NFR-CIV-DET | 4 | 3 | 0 | 1 | 0 |
| NFR-CIV-DEV-HYGIENE | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-CONFIG | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-LOUD | 1 | 0 | 0 | 1 | 0 |
| NFR-CIV-LEGENDS-PERF | 1 | 0 | 0 | 1 | 0 |
| NFR-CIV-LEGENDS-SCALE | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-MAINT | 6 | 0 | 0 | 6 | 0 |
| NFR-CIV-PERF | 11 | 1 | 8 | 2 | 0 |
| NFR-CIV-PORT | 3 | 0 | 0 | 3 | 0 |
| NFR-CIV-REL | 4 | 0 | 0 | 4 | 0 |
| NFR-CIV-SCALE | 9 | 2 | 5 | 2 | 0 |
| NFR-CIV-SCALE-PERF | 1 | 0 | 0 | 0 | 1 |
| NFR-CIV-SEC | 4 | 0 | 0 | 4 | 0 |
| NFR-O | 6 | 0 | 0 | 6 | 0 |
| NFR-P | 8 | 0 | 0 | 8 | 0 |
| NFR-R | 6 | 0 | 0 | 6 | 0 |
| NFR-S | 6 | 0 | 0 | 6 | 0 |
| NFR-SCALE | 1 | 0 | 1 | 0 | 0 |

## Spec-only IDs (need implementation) (645)

- `FR-CIV-0100-`
  - spec: docs/traceability/emergent-systems-tracelinks.md:144, docs/traceability/fr-emergence-matrix.md:20, docs/traceability/fr-emergence-matrix.md:21
- `FR-CIV-0104-001`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1454, docs/traceability/fr-civ-0104-001/fr-civ-0104-001-adr.md:1, docs/traceability/fr-civ-0104-001/fr-civ-0104-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_001.rs:1, crates/engine/tests/fr_fr_civ_0104_001.rs:6, crates/engine/tests/fr_fr_civ_0104_001.rs:10
- `FR-CIV-0104-002`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1459, docs/traceability/fr-civ-0104-002/fr-civ-0104-002-adr.md:1, docs/traceability/fr-civ-0104-002/fr-civ-0104-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_002.rs:1, crates/engine/tests/fr_fr_civ_0104_002.rs:6, crates/engine/tests/fr_fr_civ_0104_002.rs:10
- `FR-CIV-0104-003`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1464, docs/traceability/fr-civ-0104-003/fr-civ-0104-003-adr.md:1, docs/traceability/fr-civ-0104-003/fr-civ-0104-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_003.rs:1, crates/engine/tests/fr_fr_civ_0104_003.rs:6, crates/engine/tests/fr_fr_civ_0104_003.rs:10
- `FR-CIV-0104-004`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1469, docs/traceability/fr-civ-0104-004/fr-civ-0104-004-adr.md:1, docs/traceability/fr-civ-0104-004/fr-civ-0104-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_004.rs:1, crates/engine/tests/fr_fr_civ_0104_004.rs:6, crates/engine/tests/fr_fr_civ_0104_004.rs:10
- `FR-CIV-0104-005`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1474, docs/traceability/fr-civ-0104-005/fr-civ-0104-005-adr.md:1, docs/traceability/fr-civ-0104-005/fr-civ-0104-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_005.rs:1, crates/engine/tests/fr_fr_civ_0104_005.rs:6, crates/engine/tests/fr_fr_civ_0104_005.rs:10
- `FR-CIV-0104-006`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1479, docs/traceability/fr-civ-0104-006/fr-civ-0104-006-adr.md:1, docs/traceability/fr-civ-0104-006/fr-civ-0104-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_006.rs:1, crates/engine/tests/fr_fr_civ_0104_006.rs:6, crates/engine/tests/fr_fr_civ_0104_006.rs:10
- `FR-CIV-0104-007`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1484, docs/traceability/fr-civ-0104-007/fr-civ-0104-007-adr.md:1, docs/traceability/fr-civ-0104-007/fr-civ-0104-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_007.rs:1, crates/engine/tests/fr_fr_civ_0104_007.rs:6, crates/engine/tests/fr_fr_civ_0104_007.rs:10
- `FR-CIV-0104-008`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1489, docs/traceability/fr-civ-0104-008/fr-civ-0104-008-adr.md:1, docs/traceability/fr-civ-0104-008/fr-civ-0104-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_008.rs:1, crates/engine/tests/fr_fr_civ_0104_008.rs:6, crates/engine/tests/fr_fr_civ_0104_008.rs:10
- `FR-CIV-0104-009`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1494, docs/traceability/fr-civ-0104-009/fr-civ-0104-009-adr.md:1, docs/traceability/fr-civ-0104-009/fr-civ-0104-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_009.rs:1, crates/engine/tests/fr_fr_civ_0104_009.rs:6, crates/engine/tests/fr_fr_civ_0104_009.rs:10
- `FR-CIV-0104-010`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1499, docs/traceability/fr-civ-0104-010/fr-civ-0104-010-adr.md:1, docs/traceability/fr-civ-0104-010/fr-civ-0104-010-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_0104_010.rs:1, crates/engine/tests/fr_fr_civ_0104_010.rs:6, crates/engine/tests/fr_fr_civ_0104_010.rs:10
- `FR-CIV-0104-011`
  - spec: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1504
- `FR-CIV-0700`
  - spec: docs/design/civ-actor-assets-fix.md:322
- `FR-CIV-3D`
  - spec: docs/design/civ-actor-assets-fix.md:251
- `FR-CIV-3D-001`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1888, docs/traceability/fr-civ-3d-001/fr-civ-3d-001-adr.md:1, docs/traceability/fr-civ-3d-001/fr-civ-3d-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_001.rs:1, crates/engine/tests/fr_fr_civ_3d_001.rs:6, crates/engine/tests/fr_fr_civ_3d_001.rs:10
- `FR-CIV-3D-002`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1896, docs/traceability/fr-civ-3d-002/fr-civ-3d-002-adr.md:1, docs/traceability/fr-civ-3d-002/fr-civ-3d-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_002.rs:1, crates/engine/tests/fr_fr_civ_3d_002.rs:6, crates/engine/tests/fr_fr_civ_3d_002.rs:10
- `FR-CIV-3D-003`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1904, docs/traceability/fr-civ-3d-003/fr-civ-3d-003-adr.md:1, docs/traceability/fr-civ-3d-003/fr-civ-3d-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_003.rs:1, crates/engine/tests/fr_fr_civ_3d_003.rs:6, crates/engine/tests/fr_fr_civ_3d_003.rs:10
- `FR-CIV-3D-004`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1912, docs/traceability/fr-civ-3d-004/fr-civ-3d-004-adr.md:1, docs/traceability/fr-civ-3d-004/fr-civ-3d-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_004.rs:1, crates/engine/tests/fr_fr_civ_3d_004.rs:6, crates/engine/tests/fr_fr_civ_3d_004.rs:10
- `FR-CIV-3D-005`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1920, docs/traceability/fr-civ-3d-005/fr-civ-3d-005-adr.md:1, docs/traceability/fr-civ-3d-005/fr-civ-3d-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_005.rs:1, crates/engine/tests/fr_fr_civ_3d_005.rs:6, crates/engine/tests/fr_fr_civ_3d_005.rs:10
- `FR-CIV-3D-006`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1928, docs/traceability/fr-civ-3d-006/fr-civ-3d-006-adr.md:1, docs/traceability/fr-civ-3d-006/fr-civ-3d-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_006.rs:1, crates/engine/tests/fr_fr_civ_3d_006.rs:6, crates/engine/tests/fr_fr_civ_3d_006.rs:10
- `FR-CIV-3D-007`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1936, docs/traceability/fr-civ-3d-007/fr-civ-3d-007-adr.md:1, docs/traceability/fr-civ-3d-007/fr-civ-3d-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_007.rs:1, crates/engine/tests/fr_fr_civ_3d_007.rs:6, crates/engine/tests/fr_fr_civ_3d_007.rs:10
- `FR-CIV-3D-008`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1944, docs/traceability/fr-civ-3d-008/fr-civ-3d-008-adr.md:1, docs/traceability/fr-civ-3d-008/fr-civ-3d-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_008.rs:1, crates/engine/tests/fr_fr_civ_3d_008.rs:6, crates/engine/tests/fr_fr_civ_3d_008.rs:10
- `FR-CIV-3D-009`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1952, docs/traceability/fr-civ-3d-009/fr-civ-3d-009-adr.md:1, docs/traceability/fr-civ-3d-009/fr-civ-3d-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_009.rs:1, crates/engine/tests/fr_fr_civ_3d_009.rs:6, crates/engine/tests/fr_fr_civ_3d_009.rs:10
- `FR-CIV-3D-010`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1960, docs/traceability/fr-civ-3d-010/fr-civ-3d-010-adr.md:1, docs/traceability/fr-civ-3d-010/fr-civ-3d-010-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_010.rs:1, crates/engine/tests/fr_fr_civ_3d_010.rs:6, crates/engine/tests/fr_fr_civ_3d_010.rs:10
- `FR-CIV-3D-011`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1968, docs/traceability/fr-civ-3d-011/fr-civ-3d-011-adr.md:1, docs/traceability/fr-civ-3d-011/fr-civ-3d-011-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_011.rs:1, crates/engine/tests/fr_fr_civ_3d_011.rs:6, crates/engine/tests/fr_fr_civ_3d_011.rs:10
- `FR-CIV-3D-012`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1976, docs/traceability/fr-civ-3d-012/fr-civ-3d-012-adr.md:1, docs/traceability/fr-civ-3d-012/fr-civ-3d-012-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_012.rs:1, crates/engine/tests/fr_fr_civ_3d_012.rs:6, crates/engine/tests/fr_fr_civ_3d_012.rs:10
- `FR-CIV-3D-013`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1984, docs/traceability/fr-civ-3d-013/fr-civ-3d-013-adr.md:1, docs/traceability/fr-civ-3d-013/fr-civ-3d-013-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_013.rs:1, crates/engine/tests/fr_fr_civ_3d_013.rs:6, crates/engine/tests/fr_fr_civ_3d_013.rs:10
- `FR-CIV-3D-014`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1992, docs/traceability/fr-civ-3d-014/fr-civ-3d-014-adr.md:1, docs/traceability/fr-civ-3d-014/fr-civ-3d-014-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_014.rs:1, crates/engine/tests/fr_fr_civ_3d_014.rs:6, crates/engine/tests/fr_fr_civ_3d_014.rs:10
- `FR-CIV-3D-015`
  - spec: docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:2000, docs/traceability/fr-civ-3d-015/fr-civ-3d-015-adr.md:1, docs/traceability/fr-civ-3d-015/fr-civ-3d-015-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_3d_015.rs:1, crates/engine/tests/fr_fr_civ_3d_015.rs:6, crates/engine/tests/fr_fr_civ_3d_015.rs:10
- `FR-CIV-AI-011`
  - spec: docs/design/civ-ai-crate.md:43, docs/design/civ-ai-crate.md:173, docs/design/civ-ai-crate.md:262
  - tests: crates/ai/tests/fr_fr_civ_ai_011.rs:1, crates/ai/tests/fr_fr_civ_ai_011.rs:6, crates/ai/tests/fr_fr_civ_ai_011.rs:10
- `FR-CIV-AI-012`
  - spec: docs/design/civ-ai-crate.md:44, docs/design/civ-ai-crate.md:171, docs/design/civ-ai-crate.md:263
  - tests: crates/ai/tests/fr_fr_civ_ai_012.rs:1, crates/ai/tests/fr_fr_civ_ai_012.rs:6, crates/ai/tests/fr_fr_civ_ai_012.rs:10
- `FR-CIV-AI-013`
  - spec: docs/design/civ-ai-crate.md:45, docs/design/civ-ai-crate.md:264, docs/traceability/fr-civ-ai-013/fr-civ-ai-013-adr.md:1
  - tests: crates/ai/tests/fr_fr_civ_ai_013.rs:1, crates/ai/tests/fr_fr_civ_ai_013.rs:6, crates/ai/tests/fr_fr_civ_ai_013.rs:10
- `FR-CIV-AI-014`
  - spec: docs/design/civ-ai-crate.md:46, docs/design/civ-ai-crate.md:172, docs/design/civ-ai-crate.md:265
  - tests: crates/ai/tests/fr_fr_civ_ai_014.rs:1, crates/ai/tests/fr_fr_civ_ai_014.rs:6, crates/ai/tests/fr_fr_civ_ai_014.rs:10
- `FR-CIV-AI-015`
  - spec: docs/design/civ-ai-crate.md:47, docs/design/civ-ai-crate.md:266, docs/traceability/fr-civ-ai-015/fr-civ-ai-015-adr.md:1
  - tests: crates/ai/tests/fr_fr_civ_ai_015.rs:1, crates/ai/tests/fr_fr_civ_ai_015.rs:6, crates/ai/tests/fr_fr_civ_ai_015.rs:10
- `FR-CIV-ARCH-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-arch-006/fr-civ-arch-006-adr.md:1, docs/traceability/fr-civ-arch-006/fr-civ-arch-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_arch_006.rs:1, crates/engine/tests/fr_fr_civ_arch_006.rs:6, crates/engine/tests/fr_fr_civ_arch_006.rs:10
- `FR-CIV-ARCH-NOSVG-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3218, docs/traceability/fr-civ-arch-nosvg-001/fr-civ-arch-nosvg-001-adr.md:1, docs/traceability/fr-civ-arch-nosvg-001/fr-civ-arch-nosvg-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_arch_nosvg_001.rs:1, crates/engine/tests/fr_fr_civ_arch_nosvg_001.rs:6, crates/engine/tests/fr_fr_civ_arch_nosvg_001.rs:10
- `FR-CIV-ASSET-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2425, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2427
- `FR-CIV-ASSET-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206, docs/traceability/fr-civ-asset-002/fr-civ-asset-002-adr.md:1
- `FR-CIV-ASSET-003`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2447, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3207, docs/traceability/fr-civ-asset-003/fr-civ-asset-003-adr.md:1
- `FR-CIV-ASSET-004`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2457, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3208, docs/traceability/fr-civ-asset-004/fr-civ-asset-004-adr.md:1
- `FR-CIV-ASSET-005`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2467, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3209, docs/traceability/fr-civ-asset-005/fr-civ-asset-005-adr.md:1
- `FR-CIV-ASSET-006`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2477, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3210, docs/traceability/fr-civ-asset-006/fr-civ-asset-006-adr.md:1
- `FR-CIV-ASSET-007`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2487, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3211, docs/traceability/fr-civ-asset-007/fr-civ-asset-007-adr.md:1
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
- `FR-CIV-ASSET-016`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2579, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3220, docs/traceability/fr-civ-asset-016/fr-civ-asset-016-adr.md:1
- `FR-CIV-ASSET-017`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2589, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3221, docs/traceability/fr-civ-asset-017/fr-civ-asset-017-adr.md:1
- `FR-CIV-ASSET-018`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2599, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3222, docs/traceability/fr-civ-asset-018/fr-civ-asset-018-adr.md:1
- `FR-CIV-ASSET-019`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2609, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223, docs/traceability/fr-civ-asset-019/fr-civ-asset-019-adr.md:1
- `FR-CIV-ASSET-020`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2619
- `FR-CIV-ASSET-MANI-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212, docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:1, docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:6
- `FR-CIV-ASSET-MANI-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213, docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:1, docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:6
- `FR-CIV-ASSET-QUAL-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3224, docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:1, docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:6
- `FR-CIV-AUDIO-009`
  - spec: docs/design/audio-direction.md:302, docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:1, docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:6
- `FR-CIV-AUDIO-010`
  - spec: docs/design/audio-direction.md:303, docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:1, docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:6
- `FR-CIV-AUDIO-011`
  - spec: docs/design/audio-direction.md:304, docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:1, docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:6
- `FR-CIV-AUDIO-012`
  - spec: docs/design/audio-direction.md:305, docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:1, docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:6
- `FR-CIV-BRUSH-01`
  - spec: docs/design/brush-tool-system.md:514, docs/traceability/fr-civ-brush-01/fr-civ-brush-01-adr.md:1, docs/traceability/fr-civ-brush-01/fr-civ-brush-01-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_01.rs:1, crates/engine/tests/fr_fr_civ_brush_01.rs:6, crates/engine/tests/fr_fr_civ_brush_01.rs:10
- `FR-CIV-BRUSH-02`
  - spec: docs/design/brush-tool-system.md:515, docs/traceability/fr-civ-brush-02/fr-civ-brush-02-adr.md:1, docs/traceability/fr-civ-brush-02/fr-civ-brush-02-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_02.rs:1, crates/engine/tests/fr_fr_civ_brush_02.rs:6, crates/engine/tests/fr_fr_civ_brush_02.rs:10
- `FR-CIV-BRUSH-03`
  - spec: docs/design/brush-tool-system.md:516, docs/traceability/fr-civ-brush-03/fr-civ-brush-03-adr.md:1, docs/traceability/fr-civ-brush-03/fr-civ-brush-03-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_03.rs:1, crates/engine/tests/fr_fr_civ_brush_03.rs:6, crates/engine/tests/fr_fr_civ_brush_03.rs:10
- `FR-CIV-BRUSH-04`
  - spec: docs/design/brush-tool-system.md:517, docs/traceability/fr-civ-brush-04/fr-civ-brush-04-adr.md:1, docs/traceability/fr-civ-brush-04/fr-civ-brush-04-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_04.rs:1, crates/engine/tests/fr_fr_civ_brush_04.rs:6, crates/engine/tests/fr_fr_civ_brush_04.rs:10
- `FR-CIV-BRUSH-05`
  - spec: docs/design/brush-tool-system.md:518, docs/traceability/fr-civ-brush-05/fr-civ-brush-05-adr.md:1, docs/traceability/fr-civ-brush-05/fr-civ-brush-05-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_05.rs:1, crates/engine/tests/fr_fr_civ_brush_05.rs:6, crates/engine/tests/fr_fr_civ_brush_05.rs:10
- `FR-CIV-BRUSH-06`
  - spec: docs/design/brush-tool-system.md:519, docs/traceability/fr-civ-brush-06/fr-civ-brush-06-adr.md:1, docs/traceability/fr-civ-brush-06/fr-civ-brush-06-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_06.rs:1, crates/engine/tests/fr_fr_civ_brush_06.rs:6, crates/engine/tests/fr_fr_civ_brush_06.rs:10
- `FR-CIV-BRUSH-07`
  - spec: docs/design/brush-tool-system.md:520, docs/traceability/fr-civ-brush-07/fr-civ-brush-07-adr.md:1, docs/traceability/fr-civ-brush-07/fr-civ-brush-07-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_07.rs:1, crates/engine/tests/fr_fr_civ_brush_07.rs:6, crates/engine/tests/fr_fr_civ_brush_07.rs:10
- `FR-CIV-BRUSH-08`
  - spec: docs/design/brush-tool-system.md:521, docs/traceability/fr-civ-brush-08/fr-civ-brush-08-adr.md:1, docs/traceability/fr-civ-brush-08/fr-civ-brush-08-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_08.rs:1, crates/engine/tests/fr_fr_civ_brush_08.rs:6, crates/engine/tests/fr_fr_civ_brush_08.rs:10
- `FR-CIV-BRUSH-09`
  - spec: docs/design/brush-tool-system.md:522, docs/traceability/fr-civ-brush-09/fr-civ-brush-09-adr.md:1, docs/traceability/fr-civ-brush-09/fr-civ-brush-09-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_09.rs:1, crates/engine/tests/fr_fr_civ_brush_09.rs:6, crates/engine/tests/fr_fr_civ_brush_09.rs:10
- `FR-CIV-BRUSH-10`
  - spec: docs/design/brush-tool-system.md:523, docs/traceability/fr-civ-brush-10/fr-civ-brush-10-adr.md:1, docs/traceability/fr-civ-brush-10/fr-civ-brush-10-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_10.rs:1, crates/engine/tests/fr_fr_civ_brush_10.rs:6, crates/engine/tests/fr_fr_civ_brush_10.rs:10
- `FR-CIV-BRUSH-11`
  - spec: docs/design/brush-tool-system.md:524, docs/traceability/fr-civ-brush-11/fr-civ-brush-11-adr.md:1, docs/traceability/fr-civ-brush-11/fr-civ-brush-11-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_11.rs:1, crates/engine/tests/fr_fr_civ_brush_11.rs:6, crates/engine/tests/fr_fr_civ_brush_11.rs:10
- `FR-CIV-BRUSH-12`
  - spec: docs/design/brush-tool-system.md:525, docs/traceability/fr-civ-brush-12/fr-civ-brush-12-adr.md:1, docs/traceability/fr-civ-brush-12/fr-civ-brush-12-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_12.rs:1, crates/engine/tests/fr_fr_civ_brush_12.rs:6, crates/engine/tests/fr_fr_civ_brush_12.rs:10
- `FR-CIV-BRUSH-13`
  - spec: docs/design/brush-tool-system.md:526, docs/traceability/fr-civ-brush-13/fr-civ-brush-13-adr.md:1, docs/traceability/fr-civ-brush-13/fr-civ-brush-13-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_brush_13.rs:1, crates/engine/tests/fr_fr_civ_brush_13.rs:6, crates/engine/tests/fr_fr_civ_brush_13.rs:10
- `FR-CIV-BUILD-010-`
  - spec: docs/traceability/fr-3d-matrix.md:230
- `FR-CIV-CORE-006`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:892, docs/traceability/fr-civ-core-006/fr-civ-core-006-adr.md:1, docs/traceability/fr-civ-core-006/fr-civ-core-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_006.rs:1, crates/engine/tests/fr_fr_civ_core_006.rs:6, crates/engine/tests/fr_fr_civ_core_006.rs:10
- `FR-CIV-CORE-007`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:897, docs/traceability/fr-civ-core-007/fr-civ-core-007-adr.md:1, docs/traceability/fr-civ-core-007/fr-civ-core-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_007.rs:1, crates/engine/tests/fr_fr_civ_core_007.rs:6, crates/engine/tests/fr_fr_civ_core_007.rs:10
- `FR-CIV-CORE-008`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:902, docs/traceability/fr-civ-core-008/fr-civ-core-008-adr.md:1, docs/traceability/fr-civ-core-008/fr-civ-core-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_008.rs:1, crates/engine/tests/fr_fr_civ_core_008.rs:6, crates/engine/tests/fr_fr_civ_core_008.rs:10
- `FR-CIV-CORE-009`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:907, docs/traceability/fr-civ-core-009/fr-civ-core-009-adr.md:1, docs/traceability/fr-civ-core-009/fr-civ-core-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_009.rs:1, crates/engine/tests/fr_fr_civ_core_009.rs:6, crates/engine/tests/fr_fr_civ_core_009.rs:10
- `FR-CIV-CORE-010`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:912, docs/traceability/fr-civ-core-010/fr-civ-core-010-adr.md:1, docs/traceability/fr-civ-core-010/fr-civ-core-010-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_010.rs:1, crates/engine/tests/fr_fr_civ_core_010.rs:6, crates/engine/tests/fr_fr_civ_core_010.rs:10
- `FR-CIV-CORE-011`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:1369, docs/specs/CIV-0001-core-simulation-loop.md:917, docs/traceability/fr-civ-core-011/fr-civ-core-011-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_core_011.rs:1, crates/engine/tests/fr_fr_civ_core_011.rs:6, crates/engine/tests/fr_fr_civ_core_011.rs:10
- `FR-CIV-CORE-012`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:922, docs/traceability/fr-civ-core-012/fr-civ-core-012-adr.md:1, docs/traceability/fr-civ-core-012/fr-civ-core-012-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_012.rs:1, crates/engine/tests/fr_fr_civ_core_012.rs:6, crates/engine/tests/fr_fr_civ_core_012.rs:10
- `FR-CIV-CORE-014`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:932, docs/traceability/fr-civ-core-014/fr-civ-core-014-adr.md:1, docs/traceability/fr-civ-core-014/fr-civ-core-014-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_014.rs:1, crates/engine/tests/fr_fr_civ_core_014.rs:6, crates/engine/tests/fr_fr_civ_core_014.rs:10
- `FR-CIV-CORE-015`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:937, docs/traceability/fr-civ-core-015/fr-civ-core-015-adr.md:1, docs/traceability/fr-civ-core-015/fr-civ-core-015-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_015.rs:1, crates/engine/tests/fr_fr_civ_core_015.rs:6, crates/engine/tests/fr_fr_civ_core_015.rs:10
- `FR-CIV-CORE-016`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:942, docs/traceability/fr-civ-core-016/fr-civ-core-016-adr.md:1, docs/traceability/fr-civ-core-016/fr-civ-core-016-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_016.rs:1, crates/engine/tests/fr_fr_civ_core_016.rs:6, crates/engine/tests/fr_fr_civ_core_016.rs:10
- `FR-CIV-CORE-017`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:947, docs/traceability/fr-civ-core-017/fr-civ-core-017-adr.md:1, docs/traceability/fr-civ-core-017/fr-civ-core-017-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_017.rs:1, crates/engine/tests/fr_fr_civ_core_017.rs:6, crates/engine/tests/fr_fr_civ_core_017.rs:10
- `FR-CIV-CORE-018`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:952, docs/traceability/fr-civ-core-018/fr-civ-core-018-adr.md:1, docs/traceability/fr-civ-core-018/fr-civ-core-018-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_018.rs:1, crates/engine/tests/fr_fr_civ_core_018.rs:6, crates/engine/tests/fr_fr_civ_core_018.rs:10
- `FR-CIV-CORE-020`
  - spec: docs/specs/CIV-0001-core-simulation-loop.md:962, PRD.md:263, docs/traceability/fr-civ-core-020/fr-civ-core-020-adr.md:1
  - tests: crates/build/tests/fr_matrix_batch12.rs:738, crates/build/tests/fr_matrix_batch12.rs:741
- `FR-CIV-CORE-DET-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206, docs/traceability/fr-civ-core-det-001/fr-civ-core-det-001-adr.md:1, docs/traceability/fr-civ-core-det-001/fr-civ-core-det-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_det_001.rs:1, crates/engine/tests/fr_fr_civ_core_det_001.rs:6, crates/engine/tests/fr_fr_civ_core_det_001.rs:10
- `FR-CIV-CORE-DET-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3214, docs/traceability/fr-civ-core-det-002/fr-civ-core-det-002-adr.md:1, docs/traceability/fr-civ-core-det-002/fr-civ-core-det-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_det_002.rs:1, crates/engine/tests/fr_fr_civ_core_det_002.rs:6, crates/engine/tests/fr_fr_civ_core_det_002.rs:10
- `FR-CIV-CORE-DET-003`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223, docs/traceability/fr-civ-core-det-003/fr-civ-core-det-003-adr.md:1, docs/traceability/fr-civ-core-det-003/fr-civ-core-det-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_core_det_003.rs:1, crates/engine/tests/fr_fr_civ_core_det_003.rs:6, crates/engine/tests/fr_fr_civ_core_det_003.rs:10
- `FR-CIV-DET-001`
  - spec: agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:36, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:53, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:73
  - tests: crates/engine/tests/fr_fr_civ_det_001.rs:1, crates/engine/tests/fr_fr_civ_det_001.rs:6, crates/engine/tests/fr_fr_civ_det_001.rs:10
- `FR-CIV-EMERG-004`
  - spec: agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:51, docs/traceability/fr-civ-emerg-004/fr-civ-emerg-004-adr.md:1, docs/traceability/fr-civ-emerg-004/fr-civ-emerg-004-adr.md:6
  - tests: crates/civ-emergence-metrics/tests/fr_fr_civ_emerg_004.rs:1
- `FR-CIV-EMERG-005`
  - spec: agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:55, docs/traceability/fr-civ-emerg-005/fr-civ-emerg-005-adr.md:1, docs/traceability/fr-civ-emerg-005/fr-civ-emerg-005-adr.md:6
  - tests: crates/civ-emergence-metrics/tests/fr_fr_civ_emerg_005.rs:1
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
- `FR-CIV-FOG-001`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47, docs/traceability/fr-civ-fog-001/fr-civ-fog-001-adr.md:1, docs/traceability/fr-civ-fog-001/fr-civ-fog-001-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_fog_001.rs:1, crates/tactics/tests/fr_fr_civ_fog_001.rs:6, crates/tactics/tests/fr_fr_civ_fog_001.rs:14
- `FR-CIV-FOG-002`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:51, docs/traceability/fr-civ-fog-002/fr-civ-fog-002-adr.md:1, docs/traceability/fr-civ-fog-002/fr-civ-fog-002-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_fog_002.rs:1, crates/tactics/tests/fr_fr_civ_fog_002.rs:6, crates/tactics/tests/fr_fr_civ_fog_002.rs:14
- `FR-CIV-FOG-003`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:54, docs/traceability/fr-civ-fog-003/fr-civ-fog-003-adr.md:1, docs/traceability/fr-civ-fog-003/fr-civ-fog-003-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_fog_003.rs:1, crates/tactics/tests/fr_fr_civ_fog_003.rs:6, crates/tactics/tests/fr_fr_civ_fog_003.rs:15
- `FR-CIV-FOG-004`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:57, docs/traceability/fr-civ-fog-004/fr-civ-fog-004-adr.md:1, docs/traceability/fr-civ-fog-004/fr-civ-fog-004-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_fog_004.rs:1, crates/tactics/tests/fr_fr_civ_fog_004.rs:6, crates/tactics/tests/fr_fr_civ_fog_004.rs:14
- `FR-CIV-FOG-005`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:61, docs/traceability/fr-civ-fog-005/fr-civ-fog-005-adr.md:1, docs/traceability/fr-civ-fog-005/fr-civ-fog-005-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_fog_005.rs:1, crates/tactics/tests/fr_fr_civ_fog_005.rs:6, crates/tactics/tests/fr_fr_civ_fog_005.rs:15
- `FR-CIV-GEO-002`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2025, docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:1, docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:6
- `FR-CIV-GEO-003`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2026, docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:1, docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:6
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
- `FR-CIV-INFOVIEW-902`
  - spec: docs/design/info-views.md:218, docs/traceability/fr-civ-infoview-902/fr-civ-infoview-902-adr.md:1, docs/traceability/fr-civ-infoview-902/fr-civ-infoview-902-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_902.rs:1, crates/engine/tests/fr_fr_civ_infoview_902.rs:6, crates/engine/tests/fr_fr_civ_infoview_902.rs:10
- `FR-CIV-INFOVIEW-903`
  - spec: docs/design/info-views.md:219, docs/traceability/fr-civ-infoview-903/fr-civ-infoview-903-adr.md:1, docs/traceability/fr-civ-infoview-903/fr-civ-infoview-903-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_903.rs:1, crates/engine/tests/fr_fr_civ_infoview_903.rs:6, crates/engine/tests/fr_fr_civ_infoview_903.rs:10
- `FR-CIV-INFOVIEW-904`
  - spec: docs/design/info-views.md:220, docs/traceability/fr-civ-infoview-904/fr-civ-infoview-904-adr.md:1, docs/traceability/fr-civ-infoview-904/fr-civ-infoview-904-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_904.rs:1, crates/engine/tests/fr_fr_civ_infoview_904.rs:6, crates/engine/tests/fr_fr_civ_infoview_904.rs:10
- `FR-CIV-INFOVIEW-906`
  - spec: docs/design/info-views.md:222, docs/traceability/fr-civ-infoview-906/fr-civ-infoview-906-adr.md:1, docs/traceability/fr-civ-infoview-906/fr-civ-infoview-906-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_906.rs:1, crates/engine/tests/fr_fr_civ_infoview_906.rs:6, crates/engine/tests/fr_fr_civ_infoview_906.rs:10
- `FR-CIV-INFOVIEW-915`
  - spec: docs/design/info-views.md:110, docs/traceability/fr-civ-infoview-915/fr-civ-infoview-915-adr.md:1, docs/traceability/fr-civ-infoview-915/fr-civ-infoview-915-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_915.rs:1, crates/engine/tests/fr_fr_civ_infoview_915.rs:6, crates/engine/tests/fr_fr_civ_infoview_915.rs:10
- `FR-CIV-INFOVIEW-916`
  - spec: docs/design/info-views.md:111, docs/traceability/fr-civ-infoview-916/fr-civ-infoview-916-adr.md:1, docs/traceability/fr-civ-infoview-916/fr-civ-infoview-916-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_916.rs:1, crates/engine/tests/fr_fr_civ_infoview_916.rs:6, crates/engine/tests/fr_fr_civ_infoview_916.rs:10
- `FR-CIV-INFOVIEW-917`
  - spec: docs/design/info-views.md:112, docs/traceability/fr-civ-infoview-917/fr-civ-infoview-917-adr.md:1, docs/traceability/fr-civ-infoview-917/fr-civ-infoview-917-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_917.rs:1, crates/engine/tests/fr_fr_civ_infoview_917.rs:6, crates/engine/tests/fr_fr_civ_infoview_917.rs:10
- `FR-CIV-INFOVIEW-918`
  - spec: docs/design/info-views.md:113, docs/traceability/fr-civ-infoview-918/fr-civ-infoview-918-adr.md:1, docs/traceability/fr-civ-infoview-918/fr-civ-infoview-918-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_918.rs:1, crates/engine/tests/fr_fr_civ_infoview_918.rs:6, crates/engine/tests/fr_fr_civ_infoview_918.rs:10
- `FR-CIV-INFOVIEW-919`
  - spec: docs/design/info-views.md:114, docs/traceability/fr-civ-infoview-919/fr-civ-infoview-919-adr.md:1, docs/traceability/fr-civ-infoview-919/fr-civ-infoview-919-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_919.rs:1, crates/engine/tests/fr_fr_civ_infoview_919.rs:6, crates/engine/tests/fr_fr_civ_infoview_919.rs:10
- `FR-CIV-INFOVIEW-921`
  - spec: docs/design/info-views.md:116, docs/traceability/fr-civ-infoview-921/fr-civ-infoview-921-adr.md:1, docs/traceability/fr-civ-infoview-921/fr-civ-infoview-921-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_921.rs:1, crates/engine/tests/fr_fr_civ_infoview_921.rs:6, crates/engine/tests/fr_fr_civ_infoview_921.rs:10
- `FR-CIV-INFOVIEW-930`
  - spec: docs/design/info-views.md:224, docs/traceability/fr-civ-infoview-930/fr-civ-infoview-930-adr.md:1, docs/traceability/fr-civ-infoview-930/fr-civ-infoview-930-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_infoview_930.rs:1, crates/engine/tests/fr_fr_civ_infoview_930.rs:6, crates/engine/tests/fr_fr_civ_infoview_930.rs:10
- `FR-CIV-LANG-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-lang-004/fr-civ-lang-004-adr.md:1, docs/traceability/fr-civ-lang-004/fr-civ-lang-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_lang_004.rs:1, crates/engine/tests/fr_fr_civ_lang_004.rs:6, crates/engine/tests/fr_fr_civ_lang_004.rs:10
- `FR-CIV-LANG-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-lang-006/fr-civ-lang-006-adr.md:1, docs/traceability/fr-civ-lang-006/fr-civ-lang-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_lang_006.rs:1, crates/engine/tests/fr_fr_civ_lang_006.rs:6, crates/engine/tests/fr_fr_civ_lang_006.rs:10
- `FR-CIV-LANG-007`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-lang-007/fr-civ-lang-007-adr.md:1, docs/traceability/fr-civ-lang-007/fr-civ-lang-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_lang_007.rs:1, crates/engine/tests/fr_fr_civ_lang_007.rs:6, crates/engine/tests/fr_fr_civ_lang_007.rs:10
- `FR-CIV-LANG-008`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-lang-008/fr-civ-lang-008-adr.md:1, docs/traceability/fr-civ-lang-008/fr-civ-lang-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_lang_008.rs:1, crates/engine/tests/fr_fr_civ_lang_008.rs:6, crates/engine/tests/fr_fr_civ_lang_008.rs:10
- `FR-CIV-LANG-010`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-lang-010/fr-civ-lang-010-adr.md:1, docs/traceability/fr-civ-lang-010/fr-civ-lang-010-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_lang_010.rs:1, crates/engine/tests/fr_fr_civ_lang_010.rs:6, crates/engine/tests/fr_fr_civ_lang_010.rs:10
- `FR-CIV-LEGENDS-BROWSER-09`
  - spec: docs/design/legends-engine.md:444, docs/traceability/fr-civ-legends-browser-09/fr-civ-legends-browser-09-adr.md:1, docs/traceability/fr-civ-legends-browser-09/fr-civ-legends-browser-09-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_browser_09.rs:1, crates/legends/tests/fr_fr_civ_legends_browser_09.rs:6, crates/legends/tests/fr_fr_civ_legends_browser_09.rs:10
- `FR-CIV-LEGENDS-CAUSAL-06`
  - spec: docs/design/legends-engine.md:441, docs/traceability/fr-civ-legends-causal-06/fr-civ-legends-causal-06-adr.md:1, docs/traceability/fr-civ-legends-causal-06/fr-civ-legends-causal-06-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_causal_06.rs:1, crates/legends/tests/fr_fr_civ_legends_causal_06.rs:6, crates/legends/tests/fr_fr_civ_legends_causal_06.rs:10
- `FR-CIV-LEGENDS-CONFIG-04`
  - spec: docs/traceability/fr-emergence-matrix.md:259
- `FR-CIV-LEGENDS-GAP-12`
  - spec: docs/design/legends-engine.md:447, docs/traceability/fr-civ-legends-gap-12/fr-civ-legends-gap-12-adr.md:1, docs/traceability/fr-civ-legends-gap-12/fr-civ-legends-gap-12-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_gap_12.rs:1, crates/legends/tests/fr_fr_civ_legends_gap_12.rs:6, crates/legends/tests/fr_fr_civ_legends_gap_12.rs:10
- `FR-CIV-LEGENDS-INSPECT-08`
  - spec: docs/design/legends-engine.md:443, docs/traceability/fr-civ-legends-inspect-08/fr-civ-legends-inspect-08-adr.md:1, docs/traceability/fr-civ-legends-inspect-08/fr-civ-legends-inspect-08-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:1, crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:6, crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:10
- `FR-CIV-LEGENDS-NARRATOR-13`
  - spec: docs/design/legends-engine.md:448, docs/traceability/fr-civ-legends-narrator-13/fr-civ-legends-narrator-13-adr.md:1, docs/traceability/fr-civ-legends-narrator-13/fr-civ-legends-narrator-13-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:1, crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:6, crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:10
- `FR-CIV-LEGENDS-PERF-01`
  - spec: docs/traceability/fr-emergence-matrix.md:260
- `FR-CIV-LEGENDS-PERSIST-11`
  - spec: docs/design/legends-engine.md:446, docs/traceability/fr-civ-legends-persist-11/fr-civ-legends-persist-11-adr.md:1, docs/traceability/fr-civ-legends-persist-11/fr-civ-legends-persist-11-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_persist_11.rs:1, crates/legends/tests/fr_fr_civ_legends_persist_11.rs:6, crates/legends/tests/fr_fr_civ_legends_persist_11.rs:10
- `FR-CIV-LEGENDS-PRESIM-10`
  - spec: docs/design/legends-engine.md:445, docs/traceability/fr-civ-legends-presim-10/fr-civ-legends-presim-10-adr.md:1, docs/traceability/fr-civ-legends-presim-10/fr-civ-legends-presim-10-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_presim_10.rs:1, crates/legends/tests/fr_fr_civ_legends_presim_10.rs:6, crates/legends/tests/fr_fr_civ_legends_presim_10.rs:10
- `FR-CIV-LEGENDS-PRODUCER-03`
  - spec: docs/design/legends-engine.md:438, docs/traceability/fr-civ-legends-producer-03/fr-civ-legends-producer-03-adr.md:1, docs/traceability/fr-civ-legends-producer-03/fr-civ-legends-producer-03-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_producer_03.rs:1, crates/legends/tests/fr_fr_civ_legends_producer_03.rs:6, crates/legends/tests/fr_fr_civ_legends_producer_03.rs:10
- `FR-CIV-LEGENDS-RESOLVE-04`
  - spec: docs/design/legends-engine.md:439, docs/traceability/fr-civ-legends-resolve-04/fr-civ-legends-resolve-04-adr.md:1, docs/traceability/fr-civ-legends-resolve-04/fr-civ-legends-resolve-04-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:1, crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:6, crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:10
- `FR-CIV-LEGENDS-SCALE-02`
  - spec: docs/traceability/fr-emergence-matrix.md:261
- `FR-CIV-LEGENDS-SIG-05`
  - spec: docs/design/legends-engine.md:440, docs/traceability/fr-civ-legends-sig-05/fr-civ-legends-sig-05-adr.md:1, docs/traceability/fr-civ-legends-sig-05/fr-civ-legends-sig-05-adr.md:6
  - tests: crates/legends/tests/fr_fr_civ_legends_sig_05.rs:1, crates/legends/tests/fr_fr_civ_legends_sig_05.rs:6, crates/legends/tests/fr_fr_civ_legends_sig_05.rs:10
- `FR-CIV-LIFE-004`
  - spec: docs/design/civ-003-emergent-lifecycle.md:101
- `FR-CIV-LLM-001`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-001/fr-civ-llm-001-adr.md:1, docs/traceability/fr-civ-llm-001/fr-civ-llm-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_001.rs:1, crates/engine/tests/fr_fr_civ_llm_001.rs:6, crates/engine/tests/fr_fr_civ_llm_001.rs:10
- `FR-CIV-LLM-002`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-002/fr-civ-llm-002-adr.md:1, docs/traceability/fr-civ-llm-002/fr-civ-llm-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_002.rs:1, crates/engine/tests/fr_fr_civ_llm_002.rs:6, crates/engine/tests/fr_fr_civ_llm_002.rs:10
- `FR-CIV-LLM-003`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-003/fr-civ-llm-003-adr.md:1, docs/traceability/fr-civ-llm-003/fr-civ-llm-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_003.rs:1, crates/engine/tests/fr_fr_civ_llm_003.rs:6, crates/engine/tests/fr_fr_civ_llm_003.rs:10
- `FR-CIV-LLM-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-004/fr-civ-llm-004-adr.md:1, docs/traceability/fr-civ-llm-004/fr-civ-llm-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_004.rs:1, crates/engine/tests/fr_fr_civ_llm_004.rs:6, crates/engine/tests/fr_fr_civ_llm_004.rs:10
- `FR-CIV-LLM-005`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-005/fr-civ-llm-005-adr.md:1, docs/traceability/fr-civ-llm-005/fr-civ-llm-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_005.rs:1, crates/engine/tests/fr_fr_civ_llm_005.rs:6, crates/engine/tests/fr_fr_civ_llm_005.rs:10
- `FR-CIV-LLM-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-llm-006/fr-civ-llm-006-adr.md:1, docs/traceability/fr-civ-llm-006/fr-civ-llm-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_llm_006.rs:1, crates/engine/tests/fr_fr_civ_llm_006.rs:6, crates/engine/tests/fr_fr_civ_llm_006.rs:10
- `FR-CIV-MARKET-001`
  - spec: docs/design/civ-economy-emergent-markets.md:7, docs/design/ECONOMY_EMERGENCE.md:25, docs/design/master-roadmap.md:25
  - tests: crates/economy/tests/fr_fr_civ_market_001.rs:1, crates/economy/tests/fr_fr_civ_market_001.rs:6, crates/economy/tests/fr_fr_civ_market_001.rs:10
- `FR-CIV-MARKET-002`
  - spec: docs/design/civ-economy-emergent-markets.md:46, docs/design/polities-markets.md:111, docs/traceability/fr-civ-market-002/fr-civ-market-002-adr.md:1
  - tests: crates/economy/tests/fr_fr_civ_market_002.rs:1, crates/economy/tests/fr_fr_civ_market_002.rs:6, crates/economy/tests/fr_fr_civ_market_002.rs:10
- `FR-CIV-MARKET-003`
  - spec: docs/design/polities-markets.md:122, docs/traceability/fr-civ-market-003/fr-civ-market-003-adr.md:1, docs/traceability/fr-civ-market-003/fr-civ-market-003-adr.md:6
  - tests: crates/economy/tests/fr_fr_civ_market_003.rs:1, crates/economy/tests/fr_fr_civ_market_003.rs:6, crates/economy/tests/fr_fr_civ_market_003.rs:10
- `FR-CIV-MARKET-004`
  - spec: docs/design/polities-markets.md:126, docs/traceability/fr-civ-market-004/fr-civ-market-004-adr.md:1, docs/traceability/fr-civ-market-004/fr-civ-market-004-adr.md:6
  - tests: crates/economy/tests/fr_fr_civ_market_004.rs:1, crates/economy/tests/fr_fr_civ_market_004.rs:6, crates/economy/tests/fr_fr_civ_market_004.rs:10
- `FR-CIV-MARKET-005`
  - spec: docs/design/polities-markets.md:138, docs/traceability/fr-civ-market-005/fr-civ-market-005-adr.md:1, docs/traceability/fr-civ-market-005/fr-civ-market-005-adr.md:6
  - tests: crates/economy/tests/fr_fr_civ_market_005.rs:1, crates/economy/tests/fr_fr_civ_market_005.rs:6, crates/economy/tests/fr_fr_civ_market_005.rs:10
- `FR-CIV-MARKET-006`
  - spec: docs/design/civ-economy-emergent-markets.md:109, docs/design/ECONOMY_EMERGENCE.md:67, docs/design/polities-markets.md:140
  - tests: crates/economy/tests/fr_fr_civ_market_006.rs:1, crates/economy/tests/fr_fr_civ_market_006.rs:6, crates/economy/tests/fr_fr_civ_market_006.rs:10
- `FR-CIV-MARKET-007`
  - spec: docs/design/civ-economy-emergent-markets.md:155, docs/design/polities-markets.md:144, docs/traceability/fr-civ-market-007/fr-civ-market-007-adr.md:1
  - tests: crates/economy/tests/fr_fr_civ_market_007.rs:1, crates/economy/tests/fr_fr_civ_market_007.rs:6, crates/economy/tests/fr_fr_civ_market_007.rs:10
- `FR-CIV-MARKET-008`
  - spec: docs/design/civ-economy-emergent-markets.md:85, docs/design/polities-markets.md:146, docs/traceability/fr-civ-market-008/fr-civ-market-008-adr.md:1
  - tests: crates/economy/tests/fr_fr_civ_market_008.rs:1, crates/economy/tests/fr_fr_civ_market_008.rs:6, crates/economy/tests/fr_fr_civ_market_008.rs:10
- `FR-CIV-MCP-002`
  - spec: agileplus-specs/civ-017-civis-mcp-server/spec.md:38, docs/traceability/fr-civ-mcp-002/fr-civ-mcp-002-adr.md:1, docs/traceability/fr-civ-mcp-002/fr-civ-mcp-002-adr.md:6
  - tests: crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs:1, crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs:6, crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs:10
- `FR-CIV-MCP-004`
  - spec: agileplus-specs/civ-017-civis-mcp-server/spec.md:46, docs/traceability/fr-civ-mcp-004/fr-civ-mcp-004-adr.md:1, docs/traceability/fr-civ-mcp-004/fr-civ-mcp-004-adr.md:6
  - tests: crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs:1, crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs:6, crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs:10
- `FR-CIV-MCP-005`
  - spec: agileplus-specs/civ-017-civis-mcp-server/spec.md:49, docs/traceability/fr-civ-mcp-005/fr-civ-mcp-005-adr.md:1, docs/traceability/fr-civ-mcp-005/fr-civ-mcp-005-adr.md:6
  - tests: crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs:1, crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs:6, crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs:10
- `FR-CIV-MCP-006`
  - spec: agileplus-specs/civ-017-civis-mcp-server/spec.md:52, docs/traceability/fr-civ-mcp-006/fr-civ-mcp-006-adr.md:1, docs/traceability/fr-civ-mcp-006/fr-civ-mcp-006-adr.md:6
  - tests: crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs:1, crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs:6, crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs:10
- `FR-CIV-MIGRATION-001`
  - spec: docs/traceability/fr-emergence-matrix.md:215
- `FR-CIV-MIGRATION-002`
  - spec: docs/traceability/fr-emergence-matrix.md:216
- `FR-CIV-MIGRATION-003`
  - spec: docs/traceability/fr-emergence-matrix.md:217
- `FR-CIV-MIGRATION-004`
  - spec: docs/traceability/fr-emergence-matrix.md:218
- `FR-CIV-MIGRATION-005`
  - spec: docs/traceability/fr-emergence-matrix.md:219
- `FR-CIV-MOD-000`
  - spec: docs/design/modding-platform.md:26, docs/design/modding-platform.md:89, docs/design/modding-platform.md:164
  - tests: crates/mod-host/tests/fr_fr_civ_mod_000.rs:1, crates/mod-host/tests/fr_fr_civ_mod_000.rs:6, crates/mod-host/tests/fr_fr_civ_mod_000.rs:10
- `FR-CIV-MOD-002`
  - spec: docs/design/modding-platform.md:28, docs/design/modding-platform.md:178, docs/specs/CIV-0700-modding-api-spec.md:2364
  - tests: crates/mod-host/tests/fr_fr_civ_mod_002.rs:1, crates/mod-host/tests/fr_fr_civ_mod_002.rs:6, crates/mod-host/tests/fr_fr_civ_mod_002.rs:10
- `FR-CIV-MOD-003`
  - spec: docs/design/modding-platform.md:29, docs/design/modding-platform.md:192, docs/specs/CIV-0700-modding-api-spec.md:2372
  - tests: crates/mod-host/tests/fr_fr_civ_mod_003.rs:1, crates/mod-host/tests/fr_fr_civ_mod_003.rs:6, crates/mod-host/tests/fr_fr_civ_mod_003.rs:10
- `FR-CIV-MOD-004`
  - spec: docs/design/modding-platform.md:30, docs/design/modding-platform.md:203, docs/specs/CIV-0700-modding-api-spec.md:2380
  - tests: crates/mod-host/tests/fr_fr_civ_mod_004.rs:1, crates/mod-host/tests/fr_fr_civ_mod_004.rs:6, crates/mod-host/tests/fr_fr_civ_mod_004.rs:10
- `FR-CIV-MOD-005`
  - spec: docs/design/modding-platform.md:31, docs/design/modding-platform.md:214, docs/specs/CIV-0700-modding-api-spec.md:2388
  - tests: crates/mod-host/tests/fr_fr_civ_mod_005.rs:1, crates/mod-host/tests/fr_fr_civ_mod_005.rs:6, crates/mod-host/tests/fr_fr_civ_mod_005.rs:10
- `FR-CIV-MOD-006`
  - spec: docs/design/modding-platform.md:32, docs/design/modding-platform.md:226, docs/specs/CIV-0700-modding-api-spec.md:2396
  - tests: crates/mod-host/tests/fr_fr_civ_mod_006.rs:1, crates/mod-host/tests/fr_fr_civ_mod_006.rs:6, crates/mod-host/tests/fr_fr_civ_mod_006.rs:10
- `FR-CIV-MOD-007`
  - spec: docs/design/modding-platform.md:33, docs/design/modding-platform.md:234, docs/specs/CIV-0700-modding-api-spec.md:2404
  - tests: crates/mod-host/tests/fr_fr_civ_mod_007.rs:1, crates/mod-host/tests/fr_fr_civ_mod_007.rs:6, crates/mod-host/tests/fr_fr_civ_mod_007.rs:10
- `FR-CIV-MOD-008`
  - spec: docs/design/modding-platform.md:34, docs/design/modding-platform.md:249, docs/specs/CIV-0700-modding-api-spec.md:2412
  - tests: crates/mod-host/tests/fr_fr_civ_mod_008.rs:1, crates/mod-host/tests/fr_fr_civ_mod_008.rs:6, crates/mod-host/tests/fr_fr_civ_mod_008.rs:10
- `FR-CIV-MOD-009`
  - spec: docs/design/modding-platform.md:35, docs/design/modding-platform.md:73, docs/design/modding-platform.md:260
  - tests: crates/mod-host/tests/fr_fr_civ_mod_009.rs:1, crates/mod-host/tests/fr_fr_civ_mod_009.rs:6, crates/mod-host/tests/fr_fr_civ_mod_009.rs:10
- `FR-CIV-MOD-010`
  - spec: docs/design/modding-platform.md:36, docs/design/modding-platform.md:287, docs/specs/CIV-0700-modding-api-spec.md:2428
  - tests: crates/mod-host/tests/fr_fr_civ_mod_010.rs:1, crates/mod-host/tests/fr_fr_civ_mod_010.rs:6, crates/mod-host/tests/fr_fr_civ_mod_010.rs:10
- `FR-CIV-MOD-011`
  - spec: docs/design/modding-platform.md:37, docs/design/modding-platform.md:120, docs/design/modding-platform.md:299
  - tests: crates/mod-host/tests/fr_fr_civ_mod_011.rs:1, crates/mod-host/tests/fr_fr_civ_mod_011.rs:6, crates/mod-host/tests/fr_fr_civ_mod_011.rs:10
- `FR-CIV-MOD-012`
  - spec: docs/design/modding-platform.md:38, docs/design/modding-platform.md:307, docs/specs/CIV-0700-modding-api-spec.md:2444
  - tests: crates/mod-host/tests/fr_fr_civ_mod_012.rs:1, crates/mod-host/tests/fr_fr_civ_mod_012.rs:6, crates/mod-host/tests/fr_fr_civ_mod_012.rs:10
- `FR-CIV-MOD-013`
  - spec: docs/design/modding-platform.md:39, docs/design/modding-platform.md:320, docs/specs/CIV-0700-modding-api-spec.md:2452
  - tests: crates/mod-host/tests/fr_fr_civ_mod_013.rs:1, crates/mod-host/tests/fr_fr_civ_mod_013.rs:6, crates/mod-host/tests/fr_fr_civ_mod_013.rs:10
- `FR-CIV-MOD-014`
  - spec: docs/design/modding-platform.md:40, docs/design/modding-platform.md:355, docs/specs/CIV-0700-modding-api-spec.md:2460
  - tests: crates/mod-host/tests/fr_fr_civ_mod_014.rs:1, crates/mod-host/tests/fr_fr_civ_mod_014.rs:6, crates/mod-host/tests/fr_fr_civ_mod_014.rs:10
- `FR-CIV-MOD-015`
  - spec: docs/design/modding-platform.md:41, docs/design/modding-platform.md:375, docs/specs/CIV-0700-modding-api-spec.md:2468
  - tests: crates/mod-host/tests/fr_fr_civ_mod_015.rs:1, crates/mod-host/tests/fr_fr_civ_mod_015.rs:6, crates/mod-host/tests/fr_fr_civ_mod_015.rs:10
- `FR-CIV-MOD-016`
  - spec: docs/design/modding-platform.md:42, docs/design/modding-platform.md:398, docs/traceability/fr-civ-mod-016/fr-civ-mod-016-adr.md:1
  - tests: crates/mod-host/tests/fr_fr_civ_mod_016.rs:1, crates/mod-host/tests/fr_fr_civ_mod_016.rs:6, crates/mod-host/tests/fr_fr_civ_mod_016.rs:10
- `FR-CIV-MOD-017`
  - spec: docs/design/modding-platform.md:43, docs/design/modding-platform.md:419, docs/traceability/fr-civ-mod-017/fr-civ-mod-017-adr.md:1
  - tests: crates/mod-host/tests/fr_fr_civ_mod_017.rs:1, crates/mod-host/tests/fr_fr_civ_mod_017.rs:6, crates/mod-host/tests/fr_fr_civ_mod_017.rs:10
- `FR-CIV-MOD-018`
  - spec: docs/design/modding-platform.md:44, docs/design/modding-platform.md:457, docs/traceability/fr-civ-mod-018/fr-civ-mod-018-adr.md:1
  - tests: crates/mod-host/tests/fr_fr_civ_mod_018.rs:1, crates/mod-host/tests/fr_fr_civ_mod_018.rs:6, crates/mod-host/tests/fr_fr_civ_mod_018.rs:10
- `FR-CIV-MOD-019`
  - spec: docs/design/modding-platform.md:45, docs/design/modding-platform.md:486, docs/traceability/fr-civ-mod-019/fr-civ-mod-019-adr.md:1
  - tests: crates/mod-host/tests/fr_fr_civ_mod_019.rs:1, crates/mod-host/tests/fr_fr_civ_mod_019.rs:6, crates/mod-host/tests/fr_fr_civ_mod_019.rs:10
- `FR-CIV-MOD-020`
  - spec: docs/design/modding-platform.md:46, docs/design/modding-platform.md:500, docs/traceability/fr-civ-mod-020/fr-civ-mod-020-adr.md:1
  - tests: crates/mod-host/tests/fr_fr_civ_mod_020.rs:1, crates/mod-host/tests/fr_fr_civ_mod_020.rs:6, crates/mod-host/tests/fr_fr_civ_mod_020.rs:10
- `FR-CIV-PERF-002`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1926, docs/traceability/fr-civ-perf-002/fr-civ-perf-002-adr.md:1, docs/traceability/fr-civ-perf-002/fr-civ-perf-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_002.rs:1, crates/engine/tests/fr_fr_civ_perf_002.rs:6, crates/engine/tests/fr_fr_civ_perf_002.rs:10
- `FR-CIV-PERF-003`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1931, docs/traceability/fr-civ-perf-003/fr-civ-perf-003-adr.md:1, docs/traceability/fr-civ-perf-003/fr-civ-perf-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_003.rs:1, crates/engine/tests/fr_fr_civ_perf_003.rs:6, crates/engine/tests/fr_fr_civ_perf_003.rs:10
- `FR-CIV-PERF-004`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1936, docs/traceability/fr-civ-perf-004/fr-civ-perf-004-adr.md:1, docs/traceability/fr-civ-perf-004/fr-civ-perf-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_004.rs:1, crates/engine/tests/fr_fr_civ_perf_004.rs:6, crates/engine/tests/fr_fr_civ_perf_004.rs:10
- `FR-CIV-PERF-005`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1941, docs/traceability/fr-civ-perf-005/fr-civ-perf-005-adr.md:1, docs/traceability/fr-civ-perf-005/fr-civ-perf-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_005.rs:1, crates/engine/tests/fr_fr_civ_perf_005.rs:6, crates/engine/tests/fr_fr_civ_perf_005.rs:10
- `FR-CIV-PERF-006`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1946, docs/traceability/fr-civ-perf-006/fr-civ-perf-006-adr.md:1, docs/traceability/fr-civ-perf-006/fr-civ-perf-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_006.rs:1, crates/engine/tests/fr_fr_civ_perf_006.rs:6, crates/engine/tests/fr_fr_civ_perf_006.rs:10
- `FR-CIV-PERF-007`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1951, docs/traceability/fr-civ-perf-007/fr-civ-perf-007-adr.md:1, docs/traceability/fr-civ-perf-007/fr-civ-perf-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_007.rs:1, crates/engine/tests/fr_fr_civ_perf_007.rs:6, crates/engine/tests/fr_fr_civ_perf_007.rs:10
- `FR-CIV-PERF-008`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1956, docs/traceability/fr-civ-perf-008/fr-civ-perf-008-adr.md:1, docs/traceability/fr-civ-perf-008/fr-civ-perf-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_008.rs:1, crates/engine/tests/fr_fr_civ_perf_008.rs:6, crates/engine/tests/fr_fr_civ_perf_008.rs:10
- `FR-CIV-PERF-009`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1961, docs/traceability/fr-civ-perf-009/fr-civ-perf-009-adr.md:1, docs/traceability/fr-civ-perf-009/fr-civ-perf-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_009.rs:1, crates/engine/tests/fr_fr_civ_perf_009.rs:6, crates/engine/tests/fr_fr_civ_perf_009.rs:10
- `FR-CIV-PERF-010`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1966, docs/traceability/fr-civ-perf-010/fr-civ-perf-010-adr.md:1, docs/traceability/fr-civ-perf-010/fr-civ-perf-010-adr.md:6
  - tests: crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:1176, crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:1184, crates/engine/tests/fr_fr_civ_perf_010.rs:1
- `FR-CIV-PERF-011`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1971, docs/traceability/fr-civ-perf-011/fr-civ-perf-011-adr.md:1, docs/traceability/fr-civ-perf-011/fr-civ-perf-011-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_011.rs:1, crates/engine/tests/fr_fr_civ_perf_011.rs:6, crates/engine/tests/fr_fr_civ_perf_011.rs:10
- `FR-CIV-PERF-012`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1976, docs/traceability/fr-civ-perf-012/fr-civ-perf-012-adr.md:1, docs/traceability/fr-civ-perf-012/fr-civ-perf-012-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_012.rs:1, crates/engine/tests/fr_fr_civ_perf_012.rs:6, crates/engine/tests/fr_fr_civ_perf_012.rs:10
- `FR-CIV-PERF-013`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1981, docs/traceability/fr-civ-perf-013/fr-civ-perf-013-adr.md:1, docs/traceability/fr-civ-perf-013/fr-civ-perf-013-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_013.rs:1, crates/engine/tests/fr_fr_civ_perf_013.rs:6, crates/engine/tests/fr_fr_civ_perf_013.rs:10
- `FR-CIV-PERF-014`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1986, docs/traceability/fr-civ-perf-014/fr-civ-perf-014-adr.md:1, docs/traceability/fr-civ-perf-014/fr-civ-perf-014-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_014.rs:1, crates/engine/tests/fr_fr_civ_perf_014.rs:6, crates/engine/tests/fr_fr_civ_perf_014.rs:10
- `FR-CIV-PERF-015`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1991, docs/traceability/fr-civ-perf-015/fr-civ-perf-015-adr.md:1, docs/traceability/fr-civ-perf-015/fr-civ-perf-015-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_015.rs:1, crates/engine/tests/fr_fr_civ_perf_015.rs:6, crates/engine/tests/fr_fr_civ_perf_015.rs:10
- `FR-CIV-PERF-016`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:1996, docs/traceability/fr-civ-perf-016/fr-civ-perf-016-adr.md:1, docs/traceability/fr-civ-perf-016/fr-civ-perf-016-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_016.rs:1, crates/engine/tests/fr_fr_civ_perf_016.rs:6, crates/engine/tests/fr_fr_civ_perf_016.rs:10
- `FR-CIV-PERF-017`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2001, docs/traceability/fr-civ-perf-017/fr-civ-perf-017-adr.md:1, docs/traceability/fr-civ-perf-017/fr-civ-perf-017-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_017.rs:1, crates/engine/tests/fr_fr_civ_perf_017.rs:6, crates/engine/tests/fr_fr_civ_perf_017.rs:10
- `FR-CIV-PERF-018`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2006, docs/traceability/fr-civ-perf-018/fr-civ-perf-018-adr.md:1, docs/traceability/fr-civ-perf-018/fr-civ-perf-018-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_018.rs:1, crates/engine/tests/fr_fr_civ_perf_018.rs:6, crates/engine/tests/fr_fr_civ_perf_018.rs:10
- `FR-CIV-PERF-019`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2011, docs/traceability/fr-civ-perf-019/fr-civ-perf-019-adr.md:1, docs/traceability/fr-civ-perf-019/fr-civ-perf-019-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_019.rs:1, crates/engine/tests/fr_fr_civ_perf_019.rs:6, crates/engine/tests/fr_fr_civ_perf_019.rs:10
- `FR-CIV-PERF-020`
  - spec: docs/specs/CIV-0500-performance-optimization-spec.md:2016, docs/traceability/fr-civ-perf-020/fr-civ-perf-020-adr.md:1, docs/traceability/fr-civ-perf-020/fr-civ-perf-020-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_020.rs:1, crates/engine/tests/fr_fr_civ_perf_020.rs:6, crates/engine/tests/fr_fr_civ_perf_020.rs:10
- `FR-CIV-PERF-BUILD-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3215, docs/traceability/fr-civ-perf-build-001/fr-civ-perf-build-001-adr.md:1, docs/traceability/fr-civ-perf-build-001/fr-civ-perf-build-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_build_001.rs:1, crates/engine/tests/fr_fr_civ_perf_build_001.rs:6, crates/engine/tests/fr_fr_civ_perf_build_001.rs:10
- `FR-CIV-PERF-RT-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3216, docs/traceability/fr-civ-perf-rt-001/fr-civ-perf-rt-001-adr.md:1, docs/traceability/fr-civ-perf-rt-001/fr-civ-perf-rt-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_rt_001.rs:1, crates/engine/tests/fr_fr_civ_perf_rt_001.rs:6, crates/engine/tests/fr_fr_civ_perf_rt_001.rs:10
- `FR-CIV-PERF-RT-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3217, docs/traceability/fr-civ-perf-rt-002/fr-civ-perf-rt-002-adr.md:1, docs/traceability/fr-civ-perf-rt-002/fr-civ-perf-rt-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_rt_002.rs:1, crates/engine/tests/fr_fr_civ_perf_rt_002.rs:6, crates/engine/tests/fr_fr_civ_perf_rt_002.rs:10
- `FR-CIV-PERF-RT-003`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3221, docs/traceability/fr-civ-perf-rt-003/fr-civ-perf-rt-003-adr.md:1, docs/traceability/fr-civ-perf-rt-003/fr-civ-perf-rt-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_rt_003.rs:1, crates/engine/tests/fr_fr_civ_perf_rt_003.rs:6, crates/engine/tests/fr_fr_civ_perf_rt_003.rs:10
- `FR-CIV-PERF-WEB-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3219, docs/traceability/fr-civ-perf-web-001/fr-civ-perf-web-001-adr.md:1, docs/traceability/fr-civ-perf-web-001/fr-civ-perf-web-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_perf_web_001.rs:1, crates/engine/tests/fr_fr_civ_perf_web_001.rs:6, crates/engine/tests/fr_fr_civ_perf_web_001.rs:10
- `FR-CIV-POLITY-001`
  - spec: docs/design/master-roadmap.md:25, docs/design/polities-markets.md:37, docs/traceability/fr-civ-polity-001/fr-civ-polity-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_polity_001.rs:1, crates/engine/tests/fr_fr_civ_polity_001.rs:6, crates/engine/tests/fr_fr_civ_polity_001.rs:10
- `FR-CIV-POLITY-002`
  - spec: docs/design/polities-markets.md:50, docs/traceability/fr-civ-polity-002/fr-civ-polity-002-adr.md:1, docs/traceability/fr-civ-polity-002/fr-civ-polity-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_polity_002.rs:1, crates/engine/tests/fr_fr_civ_polity_002.rs:6, crates/engine/tests/fr_fr_civ_polity_002.rs:10
- `FR-CIV-POLITY-003`
  - spec: docs/design/polities-markets.md:54, docs/traceability/fr-civ-polity-003/fr-civ-polity-003-adr.md:1, docs/traceability/fr-civ-polity-003/fr-civ-polity-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_polity_003.rs:1, crates/engine/tests/fr_fr_civ_polity_003.rs:6, crates/engine/tests/fr_fr_civ_polity_003.rs:10
- `FR-CIV-POLITY-004`
  - spec: docs/design/polities-markets.md:68, docs/traceability/fr-civ-polity-004/fr-civ-polity-004-adr.md:1, docs/traceability/fr-civ-polity-004/fr-civ-polity-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_polity_004.rs:1, crates/engine/tests/fr_fr_civ_polity_004.rs:6, crates/engine/tests/fr_fr_civ_polity_004.rs:10
- `FR-CIV-POLITY-005`
  - spec: docs/design/polities-markets.md:82, docs/traceability/fr-civ-polity-005/fr-civ-polity-005-adr.md:1, docs/traceability/fr-civ-polity-005/fr-civ-polity-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_polity_005.rs:1, crates/engine/tests/fr_fr_civ_polity_005.rs:6, crates/engine/tests/fr_fr_civ_polity_005.rs:10
- `FR-CIV-POLITY-006`
  - spec: docs/design/polities-markets.md:84, docs/traceability/fr-civ-polity-006/fr-civ-polity-006-adr.md:1, docs/traceability/fr-civ-polity-006/fr-civ-polity-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_polity_006.rs:1, crates/engine/tests/fr_fr_civ_polity_006.rs:6, crates/engine/tests/fr_fr_civ_polity_006.rs:10
- `FR-CIV-POLITY-007`
  - spec: docs/design/polities-markets.md:86, docs/traceability/fr-civ-polity-007/fr-civ-polity-007-adr.md:1, docs/traceability/fr-civ-polity-007/fr-civ-polity-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_polity_007.rs:1, crates/engine/tests/fr_fr_civ_polity_007.rs:6, crates/engine/tests/fr_fr_civ_polity_007.rs:10
- `FR-CIV-POLITY-008`
  - spec: docs/design/civ-economy-emergent-markets.md:109, docs/design/polities-markets.md:90, docs/design/polities-markets.md:140
  - tests: crates/engine/tests/fr_fr_civ_polity_008.rs:1, crates/engine/tests/fr_fr_civ_polity_008.rs:6, crates/engine/tests/fr_fr_civ_polity_008.rs:10
- `FR-CIV-PROTO-003`
  - spec: docs/specs/CIV-0200-client-protocol.md:1134, docs/traceability/fr-civ-proto-003/fr-civ-proto-003-adr.md:1, docs/traceability/fr-civ-proto-003/fr-civ-proto-003-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_003.rs:1
- `FR-CIV-PROTO-004`
  - spec: docs/specs/CIV-0200-client-protocol.md:1139, docs/traceability/fr-civ-proto-004/fr-civ-proto-004-adr.md:1, docs/traceability/fr-civ-proto-004/fr-civ-proto-004-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_004.rs:1
- `FR-CIV-PROTO-005`
  - spec: docs/specs/CIV-0200-client-protocol.md:1144, docs/traceability/fr-civ-proto-005/fr-civ-proto-005-adr.md:1, docs/traceability/fr-civ-proto-005/fr-civ-proto-005-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_005.rs:1
- `FR-CIV-PROTO-006`
  - spec: docs/specs/CIV-0200-client-protocol.md:1149, docs/traceability/fr-civ-proto-006/fr-civ-proto-006-adr.md:1, docs/traceability/fr-civ-proto-006/fr-civ-proto-006-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_006.rs:1
- `FR-CIV-PROTO-007`
  - spec: docs/specs/CIV-0200-client-protocol.md:1154, docs/traceability/fr-civ-proto-007/fr-civ-proto-007-adr.md:1, docs/traceability/fr-civ-proto-007/fr-civ-proto-007-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_007.rs:1
- `FR-CIV-PROTO-008`
  - spec: docs/specs/CIV-0200-client-protocol.md:1159, docs/traceability/fr-civ-proto-008/fr-civ-proto-008-adr.md:1, docs/traceability/fr-civ-proto-008/fr-civ-proto-008-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_008.rs:1
- `FR-CIV-PROTO-009`
  - spec: docs/specs/CIV-0200-client-protocol.md:1164, docs/traceability/fr-civ-proto-009/fr-civ-proto-009-adr.md:1, docs/traceability/fr-civ-proto-009/fr-civ-proto-009-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_009.rs:1
- `FR-CIV-PROTO-010`
  - spec: docs/specs/CIV-0200-client-protocol.md:1169, docs/traceability/fr-civ-proto-010/fr-civ-proto-010-adr.md:1, docs/traceability/fr-civ-proto-010/fr-civ-proto-010-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_010.rs:1
- `FR-CIV-PROTO-011`
  - spec: docs/specs/CIV-0200-client-protocol.md:1174, docs/traceability/fr-civ-proto-011/fr-civ-proto-011-adr.md:1, docs/traceability/fr-civ-proto-011/fr-civ-proto-011-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_011.rs:1
- `FR-CIV-PROTO-012`
  - spec: docs/specs/CIV-0200-client-protocol.md:1179, docs/traceability/fr-civ-proto-012/fr-civ-proto-012-adr.md:1, docs/traceability/fr-civ-proto-012/fr-civ-proto-012-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_012.rs:1
- `FR-CIV-PROTO-013`
  - spec: docs/specs/CIV-0200-client-protocol.md:1184, docs/traceability/fr-civ-proto-013/fr-civ-proto-013-adr.md:1, docs/traceability/fr-civ-proto-013/fr-civ-proto-013-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_013.rs:1
- `FR-CIV-PROTO-014`
  - spec: docs/specs/CIV-0200-client-protocol.md:1189, docs/traceability/fr-civ-proto-014/fr-civ-proto-014-adr.md:1, docs/traceability/fr-civ-proto-014/fr-civ-proto-014-adr.md:6
  - tests: crates/protocol-3d/tests/fr_fr_civ_proto_014.rs:1
- `FR-CIV-PROTO-015`
  - spec: docs/specs/CIV-0200-client-protocol.md:1194, PRD.md:307, docs/traceability/fr-civ-proto-015/fr-civ-proto-015-adr.md:1
  - tests: crates/protocol-3d/tests/fr_civ_proto_tests.rs:3, crates/protocol-3d/tests/fr_civ_proto_tests.rs:17, crates/protocol-3d/tests/fr_fr_civ_proto_015.rs:1
- `FR-CIV-PSYCHE-002`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/design/psyche-social.md:273, docs/traceability/fr-civ-psyche-002/fr-civ-psyche-002-adr.md:1
  - tests: crates/agents/tests/fr_civ_psyche_tests.rs:29, crates/agents/tests/fr_civ_psyche_tests.rs:32, crates/agents/tests/fr_civ_psyche_tests.rs:40
- `FR-CIV-PSYCHE-003`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/design/psyche-social.md:134, docs/design/psyche-social.md:274
  - tests: crates/agents/tests/fr_civ_psyche_tests.rs:52, crates/agents/tests/fr_civ_psyche_tests.rs:55, crates/agents/tests/fr_civ_social_tests.rs:3
- `FR-CIV-PSYCHE-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:1, docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:6
- `FR-CIV-PSYCHE-005`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/design/psyche-social.md:224, docs/design/psyche-social.md:263
  - tests: crates/agents/tests/fr_civ_psyche_tests.rs:66, crates/agents/tests/fr_civ_psyche_tests.rs:69, crates/agents/tests/fr_civ_social_tests.rs:4
- `FR-CIV-PSYCHE-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/design/psyche-social.md:225, docs/design/psyche-social.md:276
  - tests: crates/agents/tests/fr_civ_psyche_tests.rs:3, crates/agents/tests/fr_civ_psyche_tests.rs:84, crates/agents/tests/fr_civ_psyche_tests.rs:87
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
- `FR-CIV-QOL-100`
  - spec: docs/design/onboarding-qol.md:37, docs/traceability/fr-civ-qol-100/fr-civ-qol-100-adr.md:1, docs/traceability/fr-civ-qol-100/fr-civ-qol-100-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_100.rs:1, crates/engine/tests/fr_fr_civ_qol_100.rs:6, crates/engine/tests/fr_fr_civ_qol_100.rs:10
- `FR-CIV-QOL-110`
  - spec: docs/design/onboarding-qol.md:74, docs/traceability/fr-civ-qol-110/fr-civ-qol-110-adr.md:1, docs/traceability/fr-civ-qol-110/fr-civ-qol-110-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_110.rs:1, crates/engine/tests/fr_fr_civ_qol_110.rs:6, crates/engine/tests/fr_fr_civ_qol_110.rs:10
- `FR-CIV-QOL-120`
  - spec: docs/design/onboarding-qol.md:90, docs/traceability/fr-civ-qol-120/fr-civ-qol-120-adr.md:1, docs/traceability/fr-civ-qol-120/fr-civ-qol-120-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_120.rs:1, crates/engine/tests/fr_fr_civ_qol_120.rs:6, crates/engine/tests/fr_fr_civ_qol_120.rs:10
- `FR-CIV-QOL-130`
  - spec: docs/design/onboarding-qol.md:104, docs/traceability/fr-civ-qol-130/fr-civ-qol-130-adr.md:1, docs/traceability/fr-civ-qol-130/fr-civ-qol-130-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_130.rs:1, crates/engine/tests/fr_fr_civ_qol_130.rs:6, crates/engine/tests/fr_fr_civ_qol_130.rs:10
- `FR-CIV-QOL-140`
  - spec: docs/design/onboarding-qol.md:120, docs/traceability/fr-civ-qol-140/fr-civ-qol-140-adr.md:1, docs/traceability/fr-civ-qol-140/fr-civ-qol-140-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_140.rs:1, crates/engine/tests/fr_fr_civ_qol_140.rs:6, crates/engine/tests/fr_fr_civ_qol_140.rs:10
- `FR-CIV-QOL-150`
  - spec: docs/design/onboarding-qol.md:136, docs/traceability/fr-civ-qol-150/fr-civ-qol-150-adr.md:1, docs/traceability/fr-civ-qol-150/fr-civ-qol-150-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_150.rs:1, crates/engine/tests/fr_fr_civ_qol_150.rs:6, crates/engine/tests/fr_fr_civ_qol_150.rs:10
- `FR-CIV-QOL-160`
  - spec: docs/design/onboarding-qol.md:146, docs/traceability/fr-civ-qol-160/fr-civ-qol-160-adr.md:1, docs/traceability/fr-civ-qol-160/fr-civ-qol-160-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_160.rs:1, crates/engine/tests/fr_fr_civ_qol_160.rs:6, crates/engine/tests/fr_fr_civ_qol_160.rs:10
- `FR-CIV-QOL-170`
  - spec: docs/design/onboarding-qol.md:162, docs/traceability/fr-civ-qol-170/fr-civ-qol-170-adr.md:1, docs/traceability/fr-civ-qol-170/fr-civ-qol-170-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_170.rs:1, crates/engine/tests/fr_fr_civ_qol_170.rs:6, crates/engine/tests/fr_fr_civ_qol_170.rs:10
- `FR-CIV-QOL-180`
  - spec: docs/design/onboarding-qol.md:172, docs/traceability/fr-civ-qol-180/fr-civ-qol-180-adr.md:1, docs/traceability/fr-civ-qol-180/fr-civ-qol-180-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_180.rs:1, crates/engine/tests/fr_fr_civ_qol_180.rs:6, crates/engine/tests/fr_fr_civ_qol_180.rs:10
- `FR-CIV-QOL-190`
  - spec: docs/design/onboarding-qol.md:191, docs/traceability/fr-civ-qol-190/fr-civ-qol-190-adr.md:1, docs/traceability/fr-civ-qol-190/fr-civ-qol-190-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_190.rs:1, crates/engine/tests/fr_fr_civ_qol_190.rs:6, crates/engine/tests/fr_fr_civ_qol_190.rs:10
- `FR-CIV-QOL-200`
  - spec: docs/design/onboarding-qol.md:205, docs/traceability/fr-civ-qol-200/fr-civ-qol-200-adr.md:1, docs/traceability/fr-civ-qol-200/fr-civ-qol-200-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_200.rs:1, crates/engine/tests/fr_fr_civ_qol_200.rs:6, crates/engine/tests/fr_fr_civ_qol_200.rs:10
- `FR-CIV-QOL-210`
  - spec: docs/design/onboarding-qol.md:222, docs/traceability/fr-civ-qol-210/fr-civ-qol-210-adr.md:1, docs/traceability/fr-civ-qol-210/fr-civ-qol-210-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_210.rs:1, crates/engine/tests/fr_fr_civ_qol_210.rs:6, crates/engine/tests/fr_fr_civ_qol_210.rs:10
- `FR-CIV-QOL-220`
  - spec: docs/design/onboarding-qol.md:240, docs/traceability/fr-civ-qol-220/fr-civ-qol-220-adr.md:1, docs/traceability/fr-civ-qol-220/fr-civ-qol-220-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_220.rs:1, crates/engine/tests/fr_fr_civ_qol_220.rs:6, crates/engine/tests/fr_fr_civ_qol_220.rs:10
- `FR-CIV-QOL-230`
  - spec: docs/design/onboarding-qol.md:249, docs/traceability/fr-civ-qol-230/fr-civ-qol-230-adr.md:1, docs/traceability/fr-civ-qol-230/fr-civ-qol-230-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_qol_230.rs:1, crates/engine/tests/fr_fr_civ_qol_230.rs:6, crates/engine/tests/fr_fr_civ_qol_230.rs:10
- `FR-CIV-REL-004`
  - spec: docs/traceability/fr-emergence-matrix.md:79
- `FR-CIV-RTS-003`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:1116, docs/specs/CIV-0300-rts-ui-ux-spec.md:1318, docs/specs/CIV-0300-rts-ui-ux-spec.md:1344
  - tests: crates/engine/tests/fr_fr_civ_rts_003.rs:1, crates/engine/tests/fr_fr_civ_rts_003.rs:6, crates/engine/tests/fr_fr_civ_rts_003.rs:10
- `FR-CIV-RTS-004`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:1143, docs/specs/CIV-0300-rts-ui-ux-spec.md:1317, docs/specs/CIV-0300-rts-ui-ux-spec.md:2007
  - tests: crates/engine/tests/fr_fr_civ_rts_004.rs:1, crates/engine/tests/fr_fr_civ_rts_004.rs:6, crates/engine/tests/fr_fr_civ_rts_004.rs:10
- `FR-CIV-RTS-005`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2008, docs/traceability/fr-civ-rts-005/fr-civ-rts-005-adr.md:1, docs/traceability/fr-civ-rts-005/fr-civ-rts-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_005.rs:1, crates/engine/tests/fr_fr_civ_rts_005.rs:6, crates/engine/tests/fr_fr_civ_rts_005.rs:10
- `FR-CIV-RTS-006`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2009, docs/traceability/fr-civ-rts-006/fr-civ-rts-006-adr.md:1, docs/traceability/fr-civ-rts-006/fr-civ-rts-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_006.rs:1, crates/engine/tests/fr_fr_civ_rts_006.rs:6, crates/engine/tests/fr_fr_civ_rts_006.rs:10
- `FR-CIV-RTS-007`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2010, docs/traceability/fr-civ-rts-007/fr-civ-rts-007-adr.md:1, docs/traceability/fr-civ-rts-007/fr-civ-rts-007-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_007.rs:1, crates/engine/tests/fr_fr_civ_rts_007.rs:6, crates/engine/tests/fr_fr_civ_rts_007.rs:10
- `FR-CIV-RTS-008`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2011, docs/traceability/fr-civ-rts-008/fr-civ-rts-008-adr.md:1, docs/traceability/fr-civ-rts-008/fr-civ-rts-008-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_008.rs:1, crates/engine/tests/fr_fr_civ_rts_008.rs:6, crates/engine/tests/fr_fr_civ_rts_008.rs:10
- `FR-CIV-RTS-009`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2012, docs/traceability/fr-civ-rts-009/fr-civ-rts-009-adr.md:1, docs/traceability/fr-civ-rts-009/fr-civ-rts-009-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_009.rs:1, crates/engine/tests/fr_fr_civ_rts_009.rs:6, crates/engine/tests/fr_fr_civ_rts_009.rs:10
- `FR-CIV-RTS-010`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2013, docs/traceability/fr-civ-rts-010/fr-civ-rts-010-adr.md:1, docs/traceability/fr-civ-rts-010/fr-civ-rts-010-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_010.rs:1, crates/engine/tests/fr_fr_civ_rts_010.rs:6, crates/engine/tests/fr_fr_civ_rts_010.rs:10
- `FR-CIV-RTS-011`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2014, docs/traceability/fr-civ-rts-011/fr-civ-rts-011-adr.md:1, docs/traceability/fr-civ-rts-011/fr-civ-rts-011-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_011.rs:1, crates/engine/tests/fr_fr_civ_rts_011.rs:6, crates/engine/tests/fr_fr_civ_rts_011.rs:10
- `FR-CIV-RTS-012`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2015, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2532, docs/traceability/fr-civ-rts-012/fr-civ-rts-012-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_rts_012.rs:1, crates/engine/tests/fr_fr_civ_rts_012.rs:6, crates/engine/tests/fr_fr_civ_rts_012.rs:10
- `FR-CIV-RTS-013`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2016, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2531, docs/traceability/fr-civ-rts-013/fr-civ-rts-013-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_rts_013.rs:1, crates/engine/tests/fr_fr_civ_rts_013.rs:6, crates/engine/tests/fr_fr_civ_rts_013.rs:10
- `FR-CIV-RTS-014`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2017, docs/specs/CIV-0400-ai-npc-behavior-spec.md:14, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2514
  - tests: crates/engine/tests/fr_fr_civ_rts_014.rs:1, crates/engine/tests/fr_fr_civ_rts_014.rs:6, crates/engine/tests/fr_fr_civ_rts_014.rs:10
- `FR-CIV-RTS-015`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2018, docs/specs/CIV-0400-ai-npc-behavior-spec.md:2533, docs/traceability/fr-civ-rts-015/fr-civ-rts-015-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_rts_015.rs:1, crates/engine/tests/fr_fr_civ_rts_015.rs:6, crates/engine/tests/fr_fr_civ_rts_015.rs:10
- `FR-CIV-RTS-NATION-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3209, docs/traceability/fr-civ-rts-nation-001/fr-civ-rts-nation-001-adr.md:1, docs/traceability/fr-civ-rts-nation-001/fr-civ-rts-nation-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_nation_001.rs:1, crates/engine/tests/fr_fr_civ_rts_nation_001.rs:6, crates/engine/tests/fr_fr_civ_rts_nation_001.rs:10
- `FR-CIV-RTS-NATION-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3220, docs/traceability/fr-civ-rts-nation-002/fr-civ-rts-nation-002-adr.md:1, docs/traceability/fr-civ-rts-nation-002/fr-civ-rts-nation-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_nation_002.rs:1, crates/engine/tests/fr_fr_civ_rts_nation_002.rs:6, crates/engine/tests/fr_fr_civ_rts_nation_002.rs:10
- `FR-CIV-RTS-RENDER-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3205, docs/traceability/fr-civ-rts-render-001/fr-civ-rts-render-001-adr.md:1, docs/traceability/fr-civ-rts-render-001/fr-civ-rts-render-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_render_001.rs:1, crates/engine/tests/fr_fr_civ_rts_render_001.rs:6, crates/engine/tests/fr_fr_civ_rts_render_001.rs:10
- `FR-CIV-RTS-RENDER-002`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3207, docs/traceability/fr-civ-rts-render-002/fr-civ-rts-render-002-adr.md:1, docs/traceability/fr-civ-rts-render-002/fr-civ-rts-render-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_render_002.rs:1, crates/engine/tests/fr_fr_civ_rts_render_002.rs:6, crates/engine/tests/fr_fr_civ_rts_render_002.rs:10
- `FR-CIV-RTS-RENDER-003`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3208, docs/traceability/fr-civ-rts-render-003/fr-civ-rts-render-003-adr.md:1, docs/traceability/fr-civ-rts-render-003/fr-civ-rts-render-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_render_003.rs:1, crates/engine/tests/fr_fr_civ_rts_render_003.rs:6, crates/engine/tests/fr_fr_civ_rts_render_003.rs:10
- `FR-CIV-RTS-RENDER-004`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3210, docs/traceability/fr-civ-rts-render-004/fr-civ-rts-render-004-adr.md:1, docs/traceability/fr-civ-rts-render-004/fr-civ-rts-render-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_render_004.rs:1, crates/engine/tests/fr_fr_civ_rts_render_004.rs:6, crates/engine/tests/fr_fr_civ_rts_render_004.rs:10
- `FR-CIV-RTS-RENDER-005`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3211, docs/traceability/fr-civ-rts-render-005/fr-civ-rts-render-005-adr.md:1, docs/traceability/fr-civ-rts-render-005/fr-civ-rts-render-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_render_005.rs:1, crates/engine/tests/fr_fr_civ_rts_render_005.rs:6, crates/engine/tests/fr_fr_civ_rts_render_005.rs:10
- `FR-CIV-RTS-ZOOM-001`
  - spec: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3222, docs/traceability/fr-civ-rts-zoom-001/fr-civ-rts-zoom-001-adr.md:1, docs/traceability/fr-civ-rts-zoom-001/fr-civ-rts-zoom-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:1, crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:6, crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:10
- `FR-CIV-SAVE-003`
  - spec: docs/traceability/civis-tracelinks.md:66, docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:1, docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:6
- `FR-CIV-SAVE-004`
  - spec: docs/traceability/civis-tracelinks.md:67, docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:1, docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:6
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
- `FR-CIV-TACTICS-001-`
  - spec: docs/traceability/fr-3d-matrix.md:109, docs/traceability/full-traceability-matrix.md:214
- `FR-CIV-TACTICS-100`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:36, docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:1, docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_100.rs:1, crates/tactics/tests/fr_fr_civ_tactics_100.rs:6, crates/tactics/tests/fr_fr_civ_tactics_100.rs:14
- `FR-CIV-TACTICS-101`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:40, docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:1, docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_101.rs:1, crates/tactics/tests/fr_fr_civ_tactics_101.rs:6, crates/tactics/tests/fr_fr_civ_tactics_101.rs:14
- `FR-CIV-TACTICS-102`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:43, docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:1, docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_102.rs:1, crates/tactics/tests/fr_fr_civ_tactics_102.rs:6, crates/tactics/tests/fr_fr_civ_tactics_102.rs:14
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
- `FR-CIV-TERRAIN-001`
  - spec: agileplus-specs/civ-014-terrain-playable-hardening/spec.md:33, docs/traceability/fr-civ-terrain-001/fr-civ-terrain-001-adr.md:1, docs/traceability/fr-civ-terrain-001/fr-civ-terrain-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_terrain_001.rs:1, crates/engine/tests/fr_fr_civ_terrain_001.rs:6, crates/engine/tests/fr_fr_civ_terrain_001.rs:10
- `FR-CIV-TERRAIN-002`
  - spec: agileplus-specs/civ-014-terrain-playable-hardening/spec.md:38, docs/traceability/fr-civ-terrain-002/fr-civ-terrain-002-adr.md:1, docs/traceability/fr-civ-terrain-002/fr-civ-terrain-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_terrain_002.rs:1, crates/engine/tests/fr_fr_civ_terrain_002.rs:6, crates/engine/tests/fr_fr_civ_terrain_002.rs:10
- `FR-CIV-TERRAIN-003`
  - spec: agileplus-specs/civ-014-terrain-playable-hardening/spec.md:41, docs/traceability/fr-civ-terrain-003/fr-civ-terrain-003-adr.md:1, docs/traceability/fr-civ-terrain-003/fr-civ-terrain-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_terrain_003.rs:1, crates/engine/tests/fr_fr_civ_terrain_003.rs:6, crates/engine/tests/fr_fr_civ_terrain_003.rs:10
- `FR-CIV-TERRAIN-004`
  - spec: agileplus-specs/civ-014-terrain-playable-hardening/spec.md:44, docs/traceability/fr-civ-terrain-004/fr-civ-terrain-004-adr.md:1, docs/traceability/fr-civ-terrain-004/fr-civ-terrain-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_terrain_004.rs:1, crates/engine/tests/fr_fr_civ_terrain_004.rs:6, crates/engine/tests/fr_fr_civ_terrain_004.rs:10
- `FR-CIV-TERRAIN-005`
  - spec: agileplus-specs/civ-014-terrain-playable-hardening/spec.md:47, docs/traceability/fr-civ-terrain-005/fr-civ-terrain-005-adr.md:1, docs/traceability/fr-civ-terrain-005/fr-civ-terrain-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_terrain_005.rs:1, crates/engine/tests/fr_fr_civ_terrain_005.rs:6, crates/engine/tests/fr_fr_civ_terrain_005.rs:10
- `FR-CIV-TERRAIN-006`
  - spec: agileplus-specs/civ-014-terrain-playable-hardening/spec.md:50, docs/traceability/fr-civ-terrain-006/fr-civ-terrain-006-adr.md:1, docs/traceability/fr-civ-terrain-006/fr-civ-terrain-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_terrain_006.rs:1, crates/engine/tests/fr_fr_civ_terrain_006.rs:6, crates/engine/tests/fr_fr_civ_terrain_006.rs:10
- `FR-CIV-VEHICLE-001`
  - spec: docs/design/vehicles-logistics.md:101, docs/design/vehicles-logistics.md:102, docs/traceability/fr-civ-vehicle-001/fr-civ-vehicle-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_vehicle_001.rs:1, crates/engine/tests/fr_fr_civ_vehicle_001.rs:6, crates/engine/tests/fr_fr_civ_vehicle_001.rs:10
- `FR-CIV-VEHICLE-002`
  - spec: docs/design/vehicles-logistics.md:105, docs/traceability/fr-civ-vehicle-002/fr-civ-vehicle-002-adr.md:1, docs/traceability/fr-civ-vehicle-002/fr-civ-vehicle-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_002.rs:1, crates/engine/tests/fr_fr_civ_vehicle_002.rs:6, crates/engine/tests/fr_fr_civ_vehicle_002.rs:10
- `FR-CIV-VEHICLE-003`
  - spec: docs/design/vehicles-logistics.md:107, docs/traceability/fr-civ-vehicle-003/fr-civ-vehicle-003-adr.md:1, docs/traceability/fr-civ-vehicle-003/fr-civ-vehicle-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_003.rs:1, crates/engine/tests/fr_fr_civ_vehicle_003.rs:6, crates/engine/tests/fr_fr_civ_vehicle_003.rs:10
- `FR-CIV-VEHICLE-004`
  - spec: docs/design/vehicles-logistics.md:109, docs/traceability/fr-civ-vehicle-004/fr-civ-vehicle-004-adr.md:1, docs/traceability/fr-civ-vehicle-004/fr-civ-vehicle-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_004.rs:1, crates/engine/tests/fr_fr_civ_vehicle_004.rs:6, crates/engine/tests/fr_fr_civ_vehicle_004.rs:10
- `FR-CIV-VEHICLE-005`
  - spec: docs/design/vehicles-logistics.md:111, docs/traceability/fr-civ-vehicle-005/fr-civ-vehicle-005-adr.md:1, docs/traceability/fr-civ-vehicle-005/fr-civ-vehicle-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_005.rs:1, crates/engine/tests/fr_fr_civ_vehicle_005.rs:6, crates/engine/tests/fr_fr_civ_vehicle_005.rs:10
- `FR-CIV-VEHICLE-010`
  - spec: docs/design/vehicles-logistics.md:161, docs/design/vehicles-logistics.md:162, docs/traceability/fr-civ-vehicle-010/fr-civ-vehicle-010-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_vehicle_010.rs:1, crates/engine/tests/fr_fr_civ_vehicle_010.rs:6, crates/engine/tests/fr_fr_civ_vehicle_010.rs:10
- `FR-CIV-VEHICLE-011`
  - spec: docs/design/vehicles-logistics.md:164, docs/traceability/fr-civ-vehicle-011/fr-civ-vehicle-011-adr.md:1, docs/traceability/fr-civ-vehicle-011/fr-civ-vehicle-011-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_011.rs:1, crates/engine/tests/fr_fr_civ_vehicle_011.rs:6, crates/engine/tests/fr_fr_civ_vehicle_011.rs:10
- `FR-CIV-VEHICLE-012`
  - spec: docs/design/vehicles-logistics.md:166, docs/traceability/fr-civ-vehicle-012/fr-civ-vehicle-012-adr.md:1, docs/traceability/fr-civ-vehicle-012/fr-civ-vehicle-012-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_012.rs:1, crates/engine/tests/fr_fr_civ_vehicle_012.rs:6, crates/engine/tests/fr_fr_civ_vehicle_012.rs:10
- `FR-CIV-VEHICLE-013`
  - spec: docs/design/vehicles-logistics.md:168, docs/traceability/fr-civ-vehicle-013/fr-civ-vehicle-013-adr.md:1, docs/traceability/fr-civ-vehicle-013/fr-civ-vehicle-013-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_013.rs:1, crates/engine/tests/fr_fr_civ_vehicle_013.rs:6, crates/engine/tests/fr_fr_civ_vehicle_013.rs:10
- `FR-CIV-VEHICLE-014`
  - spec: docs/design/vehicles-logistics.md:170, docs/traceability/fr-civ-vehicle-014/fr-civ-vehicle-014-adr.md:1, docs/traceability/fr-civ-vehicle-014/fr-civ-vehicle-014-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_014.rs:1, crates/engine/tests/fr_fr_civ_vehicle_014.rs:6, crates/engine/tests/fr_fr_civ_vehicle_014.rs:10
- `FR-CIV-VEHICLE-020`
  - spec: docs/design/vehicles-logistics.md:198, docs/design/vehicles-logistics.md:199, docs/traceability/fr-civ-vehicle-020/fr-civ-vehicle-020-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_vehicle_020.rs:1, crates/engine/tests/fr_fr_civ_vehicle_020.rs:6, crates/engine/tests/fr_fr_civ_vehicle_020.rs:10
- `FR-CIV-VEHICLE-021`
  - spec: docs/design/vehicles-logistics.md:201, docs/traceability/fr-civ-vehicle-021/fr-civ-vehicle-021-adr.md:1, docs/traceability/fr-civ-vehicle-021/fr-civ-vehicle-021-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_021.rs:1, crates/engine/tests/fr_fr_civ_vehicle_021.rs:6, crates/engine/tests/fr_fr_civ_vehicle_021.rs:10
- `FR-CIV-VEHICLE-022`
  - spec: docs/design/vehicles-logistics.md:203, docs/traceability/fr-civ-vehicle-022/fr-civ-vehicle-022-adr.md:1, docs/traceability/fr-civ-vehicle-022/fr-civ-vehicle-022-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_022.rs:1, crates/engine/tests/fr_fr_civ_vehicle_022.rs:6, crates/engine/tests/fr_fr_civ_vehicle_022.rs:10
- `FR-CIV-VEHICLE-023`
  - spec: docs/design/vehicles-logistics.md:205, docs/traceability/fr-civ-vehicle-023/fr-civ-vehicle-023-adr.md:1, docs/traceability/fr-civ-vehicle-023/fr-civ-vehicle-023-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_023.rs:1, crates/engine/tests/fr_fr_civ_vehicle_023.rs:6, crates/engine/tests/fr_fr_civ_vehicle_023.rs:10
- `FR-CIV-VEHICLE-024`
  - spec: docs/design/vehicles-logistics.md:206, docs/traceability/fr-civ-vehicle-024/fr-civ-vehicle-024-adr.md:1, docs/traceability/fr-civ-vehicle-024/fr-civ-vehicle-024-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_024.rs:1, crates/engine/tests/fr_fr_civ_vehicle_024.rs:6, crates/engine/tests/fr_fr_civ_vehicle_024.rs:10
- `FR-CIV-VEHICLE-030`
  - spec: docs/design/vehicles-logistics.md:222, docs/traceability/fr-civ-vehicle-030/fr-civ-vehicle-030-adr.md:1, docs/traceability/fr-civ-vehicle-030/fr-civ-vehicle-030-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_030.rs:1, crates/engine/tests/fr_fr_civ_vehicle_030.rs:6, crates/engine/tests/fr_fr_civ_vehicle_030.rs:10
- `FR-CIV-VEHICLE-040`
  - spec: docs/design/vehicles-logistics.md:277, docs/design/vehicles-logistics.md:278, docs/traceability/fr-civ-vehicle-040/fr-civ-vehicle-040-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_vehicle_040.rs:1, crates/engine/tests/fr_fr_civ_vehicle_040.rs:6, crates/engine/tests/fr_fr_civ_vehicle_040.rs:10
- `FR-CIV-VEHICLE-041`
  - spec: docs/design/vehicles-logistics.md:280, docs/traceability/fr-civ-vehicle-041/fr-civ-vehicle-041-adr.md:1, docs/traceability/fr-civ-vehicle-041/fr-civ-vehicle-041-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_041.rs:1, crates/engine/tests/fr_fr_civ_vehicle_041.rs:6, crates/engine/tests/fr_fr_civ_vehicle_041.rs:10
- `FR-CIV-VEHICLE-042`
  - spec: docs/design/vehicles-logistics.md:282, docs/traceability/fr-civ-vehicle-042/fr-civ-vehicle-042-adr.md:1, docs/traceability/fr-civ-vehicle-042/fr-civ-vehicle-042-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_042.rs:1, crates/engine/tests/fr_fr_civ_vehicle_042.rs:6, crates/engine/tests/fr_fr_civ_vehicle_042.rs:10
- `FR-CIV-VEHICLE-043`
  - spec: docs/design/vehicles-logistics.md:284, docs/traceability/fr-civ-vehicle-043/fr-civ-vehicle-043-adr.md:1, docs/traceability/fr-civ-vehicle-043/fr-civ-vehicle-043-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_043.rs:1, crates/engine/tests/fr_fr_civ_vehicle_043.rs:6, crates/engine/tests/fr_fr_civ_vehicle_043.rs:10
- `FR-CIV-VEHICLE-044`
  - spec: docs/design/vehicles-logistics.md:286, docs/traceability/fr-civ-vehicle-044/fr-civ-vehicle-044-adr.md:1, docs/traceability/fr-civ-vehicle-044/fr-civ-vehicle-044-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_044.rs:1, crates/engine/tests/fr_fr_civ_vehicle_044.rs:6, crates/engine/tests/fr_fr_civ_vehicle_044.rs:10
- `FR-CIV-VEHICLE-045`
  - spec: docs/design/vehicles-logistics.md:288, docs/traceability/fr-civ-vehicle-045/fr-civ-vehicle-045-adr.md:1, docs/traceability/fr-civ-vehicle-045/fr-civ-vehicle-045-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_045.rs:1, crates/engine/tests/fr_fr_civ_vehicle_045.rs:6, crates/engine/tests/fr_fr_civ_vehicle_045.rs:10
- `FR-CIV-VEHICLE-046`
  - spec: docs/design/vehicles-logistics.md:290, docs/traceability/fr-civ-vehicle-046/fr-civ-vehicle-046-adr.md:1, docs/traceability/fr-civ-vehicle-046/fr-civ-vehicle-046-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_046.rs:1, crates/engine/tests/fr_fr_civ_vehicle_046.rs:6, crates/engine/tests/fr_fr_civ_vehicle_046.rs:10
- `FR-CIV-VEHICLE-047`
  - spec: docs/design/vehicles-logistics.md:292, docs/traceability/fr-civ-vehicle-047/fr-civ-vehicle-047-adr.md:1, docs/traceability/fr-civ-vehicle-047/fr-civ-vehicle-047-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_047.rs:1, crates/engine/tests/fr_fr_civ_vehicle_047.rs:6, crates/engine/tests/fr_fr_civ_vehicle_047.rs:10
- `FR-CIV-VEHICLE-050`
  - spec: docs/design/vehicles-logistics.md:309, docs/traceability/fr-civ-vehicle-050/fr-civ-vehicle-050-adr.md:1, docs/traceability/fr-civ-vehicle-050/fr-civ-vehicle-050-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_050.rs:1, crates/engine/tests/fr_fr_civ_vehicle_050.rs:6, crates/engine/tests/fr_fr_civ_vehicle_050.rs:10
- `FR-CIV-VEHICLE-060`
  - spec: docs/design/vehicles-logistics.md:342, docs/traceability/fr-civ-vehicle-060/fr-civ-vehicle-060-adr.md:1, docs/traceability/fr-civ-vehicle-060/fr-civ-vehicle-060-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_vehicle_060.rs:1, crates/engine/tests/fr_fr_civ_vehicle_060.rs:6, crates/engine/tests/fr_fr_civ_vehicle_060.rs:10
- `FR-CIV-VERIFY-001`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:34, agileplus-specs/civ-017-civis-mcp-server/spec.md:70, docs/traceability/fr-civ-verify-001/fr-civ-verify-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_verify_001.rs:1, crates/engine/tests/fr_fr_civ_verify_001.rs:6, crates/engine/tests/fr_fr_civ_verify_001.rs:10
- `FR-CIV-VERIFY-002`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:38, docs/traceability/fr-civ-verify-002/fr-civ-verify-002-adr.md:1, docs/traceability/fr-civ-verify-002/fr-civ-verify-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_verify_002.rs:1, crates/engine/tests/fr_fr_civ_verify_002.rs:6, crates/engine/tests/fr_fr_civ_verify_002.rs:10
- `FR-CIV-VERIFY-003`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:41, docs/traceability/fr-civ-verify-003/fr-civ-verify-003-adr.md:1, docs/traceability/fr-civ-verify-003/fr-civ-verify-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_verify_003.rs:1, crates/engine/tests/fr_fr_civ_verify_003.rs:6, crates/engine/tests/fr_fr_civ_verify_003.rs:10
- `FR-CIV-VERIFY-004`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:44, docs/traceability/fr-civ-verify-004/fr-civ-verify-004-adr.md:1, docs/traceability/fr-civ-verify-004/fr-civ-verify-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_verify_004.rs:1, crates/engine/tests/fr_fr_civ_verify_004.rs:6, crates/engine/tests/fr_fr_civ_verify_004.rs:10
- `FR-CIV-VERIFY-005`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:47, docs/traceability/fr-civ-verify-005/fr-civ-verify-005-adr.md:1, docs/traceability/fr-civ-verify-005/fr-civ-verify-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_verify_005.rs:1, crates/engine/tests/fr_fr_civ_verify_005.rs:6, crates/engine/tests/fr_fr_civ_verify_005.rs:10
- `FR-CIV-VERIFY-006`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:50, docs/traceability/fr-civ-verify-006/fr-civ-verify-006-adr.md:1, docs/traceability/fr-civ-verify-006/fr-civ-verify-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_verify_006.rs:1, crates/engine/tests/fr_fr_civ_verify_006.rs:6, crates/engine/tests/fr_fr_civ_verify_006.rs:10
- `FR-CIV-VERIFY-007`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:54, agileplus-specs/civ-018-verify-harness-extension/spec.md:31, agileplus-specs/civ-018-verify-harness-extension/spec.md:42
  - tests: crates/engine/tests/fr_fr_civ_verify_007.rs:1, crates/engine/tests/fr_fr_civ_verify_007.rs:6, crates/engine/tests/fr_fr_civ_verify_007.rs:10
- `FR-CIV-VERIFY-008`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:61, agileplus-specs/civ-018-verify-harness-extension/spec.md:30, agileplus-specs/civ-018-verify-harness-extension/spec.md:46
  - tests: crates/engine/tests/fr_fr_civ_verify_008.rs:1, crates/engine/tests/fr_fr_civ_verify_008.rs:6, crates/engine/tests/fr_fr_civ_verify_008.rs:10
- `FR-CIV-VERIFY-009`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:65, agileplus-specs/civ-018-verify-harness-extension/spec.md:50, docs/traceability/fr-civ-verify-009/fr-civ-verify-009-adr.md:1
  - tests: crates/engine/tests/fr_fr_civ_verify_009.rs:1, crates/engine/tests/fr_fr_civ_verify_009.rs:6, crates/engine/tests/fr_fr_civ_verify_009.rs:10
- `FR-CIV-VERIFY-010`
  - spec: agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:69, agileplus-specs/civ-018-verify-harness-extension/spec.md:55, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:66
  - tests: crates/engine/tests/fr_fr_civ_verify_010.rs:1, crates/engine/tests/fr_fr_civ_verify_010.rs:6, crates/engine/tests/fr_fr_civ_verify_010.rs:10
- `FR-CIV-WAR-011`
  - spec: docs/design/warfare.md:83, docs/design/warfare.md:192, docs/traceability/fr-civ-war-011/fr-civ-war-011-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_011.rs:1, crates/tactics/tests/fr_fr_civ_war_011.rs:6, crates/tactics/tests/fr_fr_civ_war_011.rs:14
- `FR-CIV-WAR-012`
  - spec: docs/design/warfare.md:86, docs/design/warfare.md:193, docs/traceability/fr-civ-war-012/fr-civ-war-012-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_012.rs:1, crates/tactics/tests/fr_fr_civ_war_012.rs:6, crates/tactics/tests/fr_fr_civ_war_012.rs:14
- `FR-CIV-WAR-013`
  - spec: docs/design/warfare.md:89, docs/design/warfare.md:194, docs/traceability/fr-civ-war-013/fr-civ-war-013-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_013.rs:1, crates/tactics/tests/fr_fr_civ_war_013.rs:6, crates/tactics/tests/fr_fr_civ_war_013.rs:15
- `FR-CIV-WAR-020`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:228, docs/design/warfare.md:106, docs/design/warfare.md:195
- `FR-CIV-WAR-021`
  - spec: docs/design/warfare.md:111, docs/design/warfare.md:196, docs/traceability/fr-civ-war-021/fr-civ-war-021-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_021.rs:1, crates/tactics/tests/fr_fr_civ_war_021.rs:6, crates/tactics/tests/fr_fr_civ_war_021.rs:15
- `FR-CIV-WAR-022`
  - spec: docs/design/warfare.md:114, docs/design/warfare.md:197, docs/traceability/fr-civ-war-022/fr-civ-war-022-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_022.rs:1, crates/tactics/tests/fr_fr_civ_war_022.rs:6, crates/tactics/tests/fr_fr_civ_war_022.rs:16
- `FR-CIV-WAR-030`
  - spec: docs/design/warfare.md:124, docs/design/warfare.md:198, docs/traceability/fr-civ-war-030/fr-civ-war-030-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_030.rs:1, crates/tactics/tests/fr_fr_civ_war_030.rs:6, crates/tactics/tests/fr_fr_civ_war_030.rs:15
- `FR-CIV-WAR-040`
  - spec: docs/design/warfare.md:144, docs/design/warfare.md:199, docs/traceability/fr-civ-war-040/fr-civ-war-040-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_040.rs:1, crates/tactics/tests/fr_fr_civ_war_040.rs:6, crates/tactics/tests/fr_fr_civ_war_040.rs:15
- `FR-CIV-WAR-041`
  - spec: docs/design/warfare.md:147, docs/design/warfare.md:200, docs/traceability/fr-civ-war-041/fr-civ-war-041-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_041.rs:1, crates/tactics/tests/fr_fr_civ_war_041.rs:6, crates/tactics/tests/fr_fr_civ_war_041.rs:15
- `FR-CIV-WAR-042`
  - spec: docs/design/warfare.md:150, docs/design/warfare.md:201, docs/traceability/fr-civ-war-042/fr-civ-war-042-adr.md:1
  - tests: crates/tactics/tests/fr_fr_civ_war_042.rs:1, crates/tactics/tests/fr_fr_civ_war_042.rs:6, crates/tactics/tests/fr_fr_civ_war_042.rs:15
- `FR-DET-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:439, docs/traceability/fr-det-001/fr-det-001-adr.md:1, docs/traceability/fr-det-001/fr-det-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_001.rs:1, crates/engine/tests/fr_fr_det_001.rs:6, crates/engine/tests/fr_fr_det_001.rs:10
- `FR-DET-002`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:290, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:440, docs/traceability/fr-det-002/fr-det-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_det_002.rs:1, crates/engine/tests/fr_fr_det_002.rs:6, crates/engine/tests/fr_fr_det_002.rs:10
- `FR-DET-003`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:441, docs/traceability/fr-det-003/fr-det-003-adr.md:1, docs/traceability/fr-det-003/fr-det-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_003.rs:1, crates/engine/tests/fr_fr_det_003.rs:6, crates/engine/tests/fr_fr_det_003.rs:10
- `FR-DET-004`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:442, docs/traceability/fr-det-004/fr-det-004-adr.md:1, docs/traceability/fr-det-004/fr-det-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_004.rs:1, crates/engine/tests/fr_fr_det_004.rs:6, crates/engine/tests/fr_fr_det_004.rs:10
- `FR-DET-005`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:443, docs/traceability/fr-det-005/fr-det-005-adr.md:1, docs/traceability/fr-det-005/fr-det-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_det_005.rs:1, crates/engine/tests/fr_fr_det_005.rs:6, crates/engine/tests/fr_fr_det_005.rs:10
- `FR-DET-006`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:342, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:444, docs/traceability/fr-det-006/fr-det-006-adr.md:1
  - tests: crates/engine/tests/fr_fr_det_006.rs:1, crates/engine/tests/fr_fr_det_006.rs:6, crates/engine/tests/fr_fr_det_006.rs:10
- `FR-DET-007`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:445, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:451, docs/traceability/fr-det-007/fr-det-007-adr.md:1
  - tests: crates/engine/tests/fr_fr_det_007.rs:1, crates/engine/tests/fr_fr_det_007.rs:6, crates/engine/tests/fr_fr_det_007.rs:10
- `FR-DIPL-007`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:137, docs/traceability/fr-dipl-007/fr-dipl-007-adr.md:1, docs/traceability/fr-dipl-007/fr-dipl-007-adr.md:6
  - tests: crates/diplomacy/tests/fr_fr_dipl_007.rs:1, crates/diplomacy/tests/fr_fr_dipl_007.rs:15, crates/diplomacy/tests/fr_fr_dipl_007.rs:36
- `FR-DOC-001`
  - spec: docs/FR_DETAILED.md:358, docs/traceability/fr-doc-001/fr-doc-001-adr.md:1, docs/traceability/fr-doc-001/fr-doc-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_doc_001.rs:1, crates/engine/tests/fr_fr_doc_001.rs:6, crates/engine/tests/fr_fr_doc_001.rs:10
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
- `FR-GUARD-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1260, docs/traceability/fr-guard-001/fr-guard-001-adr.md:1, docs/traceability/fr-guard-001/fr-guard-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_guard_001.rs:1, crates/engine/tests/fr_fr_guard_001.rs:6, crates/engine/tests/fr_fr_guard_001.rs:10
- `FR-GUARD-002`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1337, docs/traceability/fr-guard-002/fr-guard-002-adr.md:1, docs/traceability/fr-guard-002/fr-guard-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_guard_002.rs:1, crates/engine/tests/fr_fr_guard_002.rs:6, crates/engine/tests/fr_fr_guard_002.rs:10
- `FR-INT-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1739, docs/traceability/fr-int-001/fr-int-001-adr.md:1, docs/traceability/fr-int-001/fr-int-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_int_001.rs:1, crates/engine/tests/fr_fr_int_001.rs:6, crates/engine/tests/fr_fr_int_001.rs:10
- `FR-MET-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1203, docs/traceability/fr-met-001/fr-met-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_met_001.rs:1, crates/engine/tests/fr_fr_met_001.rs:6, crates/engine/tests/fr_fr_met_001.rs:10
- `FR-METRICS-004`
  - spec: docs/FR.md:36, docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:1, docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_metrics_004.rs:1, crates/engine/tests/fr_fr_metrics_004.rs:6, crates/engine/tests/fr_fr_metrics_004.rs:10
- `FR-METRICS-005`
  - spec: docs/FR.md:37, docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:1, docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_metrics_005.rs:1, crates/engine/tests/fr_fr_metrics_005.rs:6, crates/engine/tests/fr_fr_metrics_005.rs:10
- `FR-NET-001`
  - spec: docs/FR.md:52, docs/FR_DETAILED.md:267, docs/traceability/fr-net-001/fr-net-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_net_001.rs:1, crates/engine/tests/fr_fr_net_001.rs:6, crates/engine/tests/fr_fr_net_001.rs:10
- `FR-NET-002`
  - spec: docs/FR.md:53, docs/FR_DETAILED.md:281, docs/traceability/fr-net-002/fr-net-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_net_002.rs:1, crates/engine/tests/fr_fr_net_002.rs:6, crates/engine/tests/fr_fr_net_002.rs:10
- `FR-NET-003`
  - spec: docs/FR.md:54, docs/traceability/fr-net-003/fr-net-003-adr.md:1, docs/traceability/fr-net-003/fr-net-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_net_003.rs:1, crates/engine/tests/fr_fr_net_003.rs:6, crates/engine/tests/fr_fr_net_003.rs:10
- `FR-PERF-001`
  - spec: docs/FR_DETAILED.md:296, docs/traceability/TRACEABILITY_MATRIX.md:287, docs/traceability/fr-perf-001/fr-perf-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_perf_001.rs:1
- `FR-PROT-001`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:178, docs/traceability/fr-prot-001/fr-prot-001-adr.md:1, docs/traceability/fr-prot-001/fr-prot-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_prot_001.rs:1, crates/engine/tests/fr_fr_prot_001.rs:6, crates/engine/tests/fr_fr_prot_001.rs:10
- `FR-PROT-002`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:179, docs/traceability/fr-prot-002/fr-prot-002-adr.md:1, docs/traceability/fr-prot-002/fr-prot-002-adr.md:6
  - tests: crates/engine/tests/fr_fr_prot_002.rs:1, crates/engine/tests/fr_fr_prot_002.rs:6, crates/engine/tests/fr_fr_prot_002.rs:10
- `FR-PROT-003`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:180, docs/traceability/fr-prot-003/fr-prot-003-adr.md:1, docs/traceability/fr-prot-003/fr-prot-003-adr.md:6
  - tests: crates/engine/tests/fr_fr_prot_003.rs:1, crates/engine/tests/fr_fr_prot_003.rs:6, crates/engine/tests/fr_fr_prot_003.rs:10
- `FR-PROT-005`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:182, docs/traceability/fr-prot-005/fr-prot-005-adr.md:1, docs/traceability/fr-prot-005/fr-prot-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_prot_005.rs:1, crates/engine/tests/fr_fr_prot_005.rs:6, crates/engine/tests/fr_fr_prot_005.rs:10
- `FR-REP-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:489, docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:532, docs/traceability/fr-rep-001/fr-rep-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_rep_001.rs:1, crates/engine/tests/fr_fr_rep_001.rs:6, crates/engine/tests/fr_fr_rep_001.rs:10
- `FR-SAVE-001`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2800, docs/traceability/TRACEABILITY_MATRIX.md:273, docs/traceability/fr-save-001/fr-save-001-adr.md:1
  - tests: crates/save-db/tests/fr_save_tests.rs:3, crates/save-db/tests/fr_save_tests.rs:11, crates/save-db/tests/fr_save_tests.rs:121
- `FR-SAVE-004`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2803, docs/specs/CIV-1000-save-load-persistence-spec.md:2935, docs/traceability/TRACEABILITY_MATRIX.md:276
  - tests: crates/save-db/tests/fr_save_tests.rs:3, crates/save-db/tests/fr_save_tests.rs:53
- `FR-SAVE-006`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2805, docs/specs/CIV-1000-save-load-persistence-spec.md:2943, docs/traceability/fr-save-006/fr-save-006-adr.md:1
- `FR-SAVE-007`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2806, docs/specs/CIV-1000-save-load-persistence-spec.md:2943, docs/traceability/fr-save-007/fr-save-007-adr.md:1
- `FR-SAVE-008`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2807, docs/specs/CIV-1000-save-load-persistence-spec.md:2949, docs/traceability/fr-save-008/fr-save-008-adr.md:1
- `FR-SAVE-009`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2808, docs/traceability/fr-save-009/fr-save-009-adr.md:1, docs/traceability/fr-save-009/fr-save-009-adr.md:6
- `FR-SAVE-010`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:159, docs/specs/CIV-1000-save-load-persistence-spec.md:2809, docs/specs/CIV-1000-save-load-persistence-spec.md:2955
  - tests: crates/save-db/tests/fr_save_tests.rs:3, crates/save-db/tests/fr_save_tests.rs:93
- `FR-SAVE-011`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2810, docs/specs/CIV-1000-save-load-persistence-spec.md:2963, docs/traceability/fr-save-011/fr-save-011-adr.md:1
- `FR-SAVE-012`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2811, docs/specs/CIV-1000-save-load-persistence-spec.md:2963, docs/traceability/fr-save-012/fr-save-012-adr.md:1
- `FR-SAVE-013`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2812, docs/specs/CIV-1000-save-load-persistence-spec.md:2969, docs/traceability/fr-save-013/fr-save-013-adr.md:1
- `FR-SAVE-014`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2813, docs/specs/CIV-1000-save-load-persistence-spec.md:2969, docs/traceability/fr-save-014/fr-save-014-adr.md:1
- `FR-SAVE-015`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2814, docs/specs/CIV-1000-save-load-persistence-spec.md:2969, docs/traceability/fr-save-015/fr-save-015-adr.md:1
- `FR-SAVE-016`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2815, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-016/fr-save-016-adr.md:1
- `FR-SAVE-017`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2816, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-017/fr-save-017-adr.md:1
- `FR-SAVE-018`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2817, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-018/fr-save-018-adr.md:1
- `FR-SAVE-019`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2818, docs/specs/CIV-1000-save-load-persistence-spec.md:2977, docs/traceability/fr-save-019/fr-save-019-adr.md:1
- `FR-SAVE-020`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2819, docs/traceability/fr-save-020/fr-save-020-adr.md:1, docs/traceability/fr-save-020/fr-save-020-adr.md:6
- `FR-SAVE-021`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2820, docs/specs/CIV-1000-save-load-persistence-spec.md:2987, docs/traceability/fr-save-021/fr-save-021-adr.md:1
- `FR-SAVE-022`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2821, docs/specs/CIV-1000-save-load-persistence-spec.md:2993, docs/traceability/fr-save-022/fr-save-022-adr.md:1
- `FR-SAVE-023`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2822, docs/specs/CIV-1000-save-load-persistence-spec.md:3001, docs/traceability/fr-save-023/fr-save-023-adr.md:1
- `FR-SAVE-024`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2823, docs/traceability/fr-save-024/fr-save-024-adr.md:1, docs/traceability/fr-save-024/fr-save-024-adr.md:6
- `FR-SAVE-025`
  - spec: docs/specs/CIV-1000-save-load-persistence-spec.md:2824, docs/traceability/fr-save-025/fr-save-025-adr.md:1, docs/traceability/fr-save-025/fr-save-025-adr.md:6
- `FR-SESSION-001`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2020, docs/traceability/fr-session-001/fr-session-001-adr.md:1, docs/traceability/fr-session-001/fr-session-001-adr.md:6
  - tests: crates/server/tests/fr_fr_session_001.rs:1, crates/server/tests/fr_fr_session_001.rs:6, crates/server/tests/fr_fr_session_001.rs:10
- `FR-SESSION-002`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2022, docs/traceability/fr-session-002/fr-session-002-adr.md:1, docs/traceability/fr-session-002/fr-session-002-adr.md:6
  - tests: crates/server/tests/fr_fr_session_002.rs:1, crates/server/tests/fr_fr_session_002.rs:6, crates/server/tests/fr_fr_session_002.rs:10
- `FR-SESSION-003`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2024, docs/traceability/fr-session-003/fr-session-003-adr.md:1, docs/traceability/fr-session-003/fr-session-003-adr.md:6
  - tests: crates/server/tests/fr_fr_session_003.rs:1, crates/server/tests/fr_fr_session_003.rs:6, crates/server/tests/fr_fr_session_003.rs:10
- `FR-SESSION-004`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2026, docs/traceability/fr-session-004/fr-session-004-adr.md:1, docs/traceability/fr-session-004/fr-session-004-adr.md:6
  - tests: crates/server/tests/fr_fr_session_004.rs:1, crates/server/tests/fr_fr_session_004.rs:6, crates/server/tests/fr_fr_session_004.rs:10
- `FR-SESSION-005`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2028, docs/traceability/fr-session-005/fr-session-005-adr.md:1, docs/traceability/fr-session-005/fr-session-005-adr.md:6
  - tests: crates/server/tests/fr_fr_session_005.rs:1, crates/server/tests/fr_fr_session_005.rs:6, crates/server/tests/fr_fr_session_005.rs:10
- `FR-SESSION-006`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2032, docs/traceability/fr-session-006/fr-session-006-adr.md:1, docs/traceability/fr-session-006/fr-session-006-adr.md:6
  - tests: crates/server/tests/fr_fr_session_006.rs:1, crates/server/tests/fr_fr_session_006.rs:6, crates/server/tests/fr_fr_session_006.rs:10
- `FR-SESSION-007`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2034, docs/traceability/fr-session-007/fr-session-007-adr.md:1, docs/traceability/fr-session-007/fr-session-007-adr.md:6
  - tests: crates/server/tests/fr_fr_session_007.rs:1, crates/server/tests/fr_fr_session_007.rs:6, crates/server/tests/fr_fr_session_007.rs:10
- `FR-SESSION-008`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2036, docs/traceability/fr-session-008/fr-session-008-adr.md:1, docs/traceability/fr-session-008/fr-session-008-adr.md:6
  - tests: crates/server/tests/fr_fr_session_008.rs:1, crates/server/tests/fr_fr_session_008.rs:6, crates/server/tests/fr_fr_session_008.rs:10
- `FR-SESSION-009`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2038, docs/traceability/fr-session-009/fr-session-009-adr.md:1, docs/traceability/fr-session-009/fr-session-009-adr.md:6
  - tests: crates/server/tests/fr_fr_session_009.rs:1, crates/server/tests/fr_fr_session_009.rs:6, crates/server/tests/fr_fr_session_009.rs:10
- `FR-SESSION-010`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2040, docs/traceability/fr-session-010/fr-session-010-adr.md:1, docs/traceability/fr-session-010/fr-session-010-adr.md:6
  - tests: crates/server/tests/fr_fr_session_010.rs:1, crates/server/tests/fr_fr_session_010.rs:6, crates/server/tests/fr_fr_session_010.rs:10
- `FR-SESSION-011`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2044, docs/traceability/fr-session-011/fr-session-011-adr.md:1, docs/traceability/fr-session-011/fr-session-011-adr.md:6
  - tests: crates/server/tests/fr_fr_session_011.rs:1, crates/server/tests/fr_fr_session_011.rs:6, crates/server/tests/fr_fr_session_011.rs:10
- `FR-SESSION-012`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2046, docs/traceability/fr-session-012/fr-session-012-adr.md:1, docs/traceability/fr-session-012/fr-session-012-adr.md:6
  - tests: crates/server/tests/fr_fr_session_012.rs:1, crates/server/tests/fr_fr_session_012.rs:6, crates/server/tests/fr_fr_session_012.rs:10
- `FR-SESSION-013`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2048, docs/traceability/fr-session-013/fr-session-013-adr.md:1, docs/traceability/fr-session-013/fr-session-013-adr.md:6
  - tests: crates/server/tests/fr_fr_session_013.rs:1, crates/server/tests/fr_fr_session_013.rs:6, crates/server/tests/fr_fr_session_013.rs:10
- `FR-SESSION-014`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:62, agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:126, docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2050
  - tests: crates/server/tests/fr_civ_server_tests.rs:4, crates/server/tests/fr_civ_server_tests.rs:51, crates/server/tests/fr_fr_session_014.rs:1
- `FR-SESSION-015`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2052, docs/traceability/fr-session-015/fr-session-015-adr.md:1, docs/traceability/fr-session-015/fr-session-015-adr.md:6
  - tests: crates/server/tests/fr_fr_session_015.rs:1, crates/server/tests/fr_fr_session_015.rs:6, crates/server/tests/fr_fr_session_015.rs:10
- `FR-SESSION-016`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2056, docs/traceability/fr-session-016/fr-session-016-adr.md:1, docs/traceability/fr-session-016/fr-session-016-adr.md:6
  - tests: crates/server/tests/fr_fr_session_016.rs:1, crates/server/tests/fr_fr_session_016.rs:6, crates/server/tests/fr_fr_session_016.rs:10
- `FR-SESSION-017`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2058, docs/traceability/fr-session-017/fr-session-017-adr.md:1, docs/traceability/fr-session-017/fr-session-017-adr.md:6
  - tests: crates/server/tests/fr_fr_session_017.rs:1, crates/server/tests/fr_fr_session_017.rs:6, crates/server/tests/fr_fr_session_017.rs:10
- `FR-SESSION-018`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2060, docs/traceability/fr-session-018/fr-session-018-adr.md:1, docs/traceability/fr-session-018/fr-session-018-adr.md:6
  - tests: crates/server/tests/fr_fr_session_018.rs:1, crates/server/tests/fr_fr_session_018.rs:6, crates/server/tests/fr_fr_session_018.rs:10
- `FR-SESSION-019`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2062, docs/traceability/fr-session-019/fr-session-019-adr.md:1, docs/traceability/fr-session-019/fr-session-019-adr.md:6
  - tests: crates/server/tests/fr_fr_session_019.rs:1, crates/server/tests/fr_fr_session_019.rs:6, crates/server/tests/fr_fr_session_019.rs:10
- `FR-SESSION-020`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2064, docs/traceability/fr-session-020/fr-session-020-adr.md:1, docs/traceability/fr-session-020/fr-session-020-adr.md:6
  - tests: crates/server/tests/fr_fr_session_020.rs:1, crates/server/tests/fr_fr_session_020.rs:6, crates/server/tests/fr_fr_session_020.rs:10
- `FR-SESSION-021`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2068, docs/traceability/fr-session-021/fr-session-021-adr.md:1, docs/traceability/fr-session-021/fr-session-021-adr.md:6
  - tests: crates/server/tests/fr_fr_session_021.rs:1, crates/server/tests/fr_fr_session_021.rs:6, crates/server/tests/fr_fr_session_021.rs:10
- `FR-SESSION-022`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2070, docs/traceability/fr-session-022/fr-session-022-adr.md:1, docs/traceability/fr-session-022/fr-session-022-adr.md:6
  - tests: crates/server/tests/fr_fr_session_022.rs:1, crates/server/tests/fr_fr_session_022.rs:6, crates/server/tests/fr_fr_session_022.rs:10
- `FR-SESSION-023`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2072, docs/traceability/fr-session-023/fr-session-023-adr.md:1, docs/traceability/fr-session-023/fr-session-023-adr.md:6
  - tests: crates/server/tests/fr_fr_session_023.rs:1, crates/server/tests/fr_fr_session_023.rs:6, crates/server/tests/fr_fr_session_023.rs:10
- `FR-SESSION-024`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2074, docs/traceability/fr-session-024/fr-session-024-adr.md:1, docs/traceability/fr-session-024/fr-session-024-adr.md:6
  - tests: crates/server/tests/fr_fr_session_024.rs:1, crates/server/tests/fr_fr_session_024.rs:6, crates/server/tests/fr_fr_session_024.rs:10
- `FR-SESSION-025`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2076, docs/traceability/fr-session-025/fr-session-025-adr.md:1, docs/traceability/fr-session-025/fr-session-025-adr.md:6
  - tests: crates/server/tests/fr_fr_session_025.rs:1, crates/server/tests/fr_fr_session_025.rs:6, crates/server/tests/fr_fr_session_025.rs:10
- `FR-SESSION-026`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2080, docs/traceability/fr-session-026/fr-session-026-adr.md:1, docs/traceability/fr-session-026/fr-session-026-adr.md:6
  - tests: crates/server/tests/fr_fr_session_026.rs:1, crates/server/tests/fr_fr_session_026.rs:6, crates/server/tests/fr_fr_session_026.rs:10
- `FR-SESSION-027`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2082, docs/traceability/fr-session-027/fr-session-027-adr.md:1, docs/traceability/fr-session-027/fr-session-027-adr.md:6
  - tests: crates/server/tests/fr_fr_session_027.rs:1, crates/server/tests/fr_fr_session_027.rs:6, crates/server/tests/fr_fr_session_027.rs:10
- `FR-SESSION-028`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2084, docs/traceability/fr-session-028/fr-session-028-adr.md:1, docs/traceability/fr-session-028/fr-session-028-adr.md:6
  - tests: crates/server/tests/fr_fr_session_028.rs:1, crates/server/tests/fr_fr_session_028.rs:6, crates/server/tests/fr_fr_session_028.rs:10
- `FR-SESSION-029`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2086, docs/traceability/fr-session-029/fr-session-029-adr.md:1, docs/traceability/fr-session-029/fr-session-029-adr.md:6
  - tests: crates/server/tests/fr_fr_session_029.rs:1, crates/server/tests/fr_fr_session_029.rs:6, crates/server/tests/fr_fr_session_029.rs:10
- `FR-SESSION-030`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2090, docs/traceability/fr-session-030/fr-session-030-adr.md:1, docs/traceability/fr-session-030/fr-session-030-adr.md:6
  - tests: crates/server/tests/fr_fr_session_030.rs:1, crates/server/tests/fr_fr_session_030.rs:6, crates/server/tests/fr_fr_session_030.rs:10
- `FR-SESSION-031`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2092, docs/traceability/fr-session-031/fr-session-031-adr.md:1, docs/traceability/fr-session-031/fr-session-031-adr.md:6
  - tests: crates/server/tests/fr_fr_session_031.rs:1, crates/server/tests/fr_fr_session_031.rs:6, crates/server/tests/fr_fr_session_031.rs:10
- `FR-SESSION-032`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2094, docs/traceability/fr-session-032/fr-session-032-adr.md:1, docs/traceability/fr-session-032/fr-session-032-adr.md:6
  - tests: crates/server/tests/fr_fr_session_032.rs:1, crates/server/tests/fr_fr_session_032.rs:6, crates/server/tests/fr_fr_session_032.rs:10
- `FR-SESSION-033`
  - spec: docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2096, docs/traceability/fr-session-033/fr-session-033-adr.md:1, docs/traceability/fr-session-033/fr-session-033-adr.md:6
  - tests: crates/server/tests/fr_fr_session_033.rs:1, crates/server/tests/fr_fr_session_033.rs:6, crates/server/tests/fr_fr_session_033.rs:10
- `FR-SOC-CIV-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4326, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4570, docs/traceability/fr-soc-civ-001/fr-soc-civ-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_civ_001.rs:1, crates/engine/tests/fr_fr_soc_civ_001.rs:6, crates/engine/tests/fr_fr_soc_civ_001.rs:10
- `FR-SOC-CIV-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4355, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4571, docs/traceability/fr-soc-civ-002/fr-soc-civ-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_civ_002.rs:1, crates/engine/tests/fr_fr_soc_civ_002.rs:6, crates/engine/tests/fr_fr_soc_civ_002.rs:10
- `FR-SOC-COH-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1533, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1986, docs/traceability/fr-soc-coh-001/fr-soc-coh-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_coh_001.rs:1, crates/engine/tests/fr_fr_soc_coh_001.rs:6, crates/engine/tests/fr_fr_soc_coh_001.rs:10
- `FR-SOC-COH-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1546, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1987, docs/traceability/fr-soc-coh-002/fr-soc-coh-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_coh_002.rs:1, crates/engine/tests/fr_fr_soc_coh_002.rs:6, crates/engine/tests/fr_fr_soc_coh_002.rs:10
- `FR-SOC-COH-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1559, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1988, docs/traceability/fr-soc-coh-003/fr-soc-coh-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_coh_003.rs:1, crates/engine/tests/fr_fr_soc_coh_003.rs:6, crates/engine/tests/fr_fr_soc_coh_003.rs:10
- `FR-SOC-COH-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1571, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1989, docs/traceability/fr-soc-coh-004/fr-soc-coh-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_coh_004.rs:1, crates/engine/tests/fr_fr_soc_coh_004.rs:6, crates/engine/tests/fr_fr_soc_coh_004.rs:10
- `FR-SOC-DET-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1495, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1498, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1984
  - tests: crates/engine/tests/fr_fr_soc_det_001.rs:1, crates/engine/tests/fr_fr_soc_det_001.rs:6, crates/engine/tests/fr_fr_soc_det_001.rs:10
- `FR-SOC-DET-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1516, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1985, docs/traceability/fr-soc-det-002/fr-soc-det-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_det_002.rs:1, crates/engine/tests/fr_fr_soc_det_002.rs:6, crates/engine/tests/fr_fr_soc_det_002.rs:10
- `FR-SOC-FAC-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4277, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4568, docs/traceability/fr-soc-fac-001/fr-soc-fac-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_fac_001.rs:1, crates/engine/tests/fr_fr_soc_fac_001.rs:6, crates/engine/tests/fr_fr_soc_fac_001.rs:10
- `FR-SOC-FAC-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4300, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4569, docs/traceability/fr-soc-fac-002/fr-soc-fac-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_fac_002.rs:1, crates/engine/tests/fr_fr_soc_fac_002.rs:6, crates/engine/tests/fr_fr_soc_fac_002.rs:10
- `FR-SOC-HLT-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1643, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1994, docs/traceability/fr-soc-hlt-001/fr-soc-hlt-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_001.rs:1, crates/engine/tests/fr_fr_soc_hlt_001.rs:6, crates/engine/tests/fr_fr_soc_hlt_001.rs:10
- `FR-SOC-HLT-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1657, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1995, docs/traceability/fr-soc-hlt-002/fr-soc-hlt-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_002.rs:1, crates/engine/tests/fr_fr_soc_hlt_002.rs:6, crates/engine/tests/fr_fr_soc_hlt_002.rs:10
- `FR-SOC-HLT-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1667, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1996, docs/traceability/fr-soc-hlt-003/fr-soc-hlt-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_003.rs:1, crates/engine/tests/fr_fr_soc_hlt_003.rs:6, crates/engine/tests/fr_fr_soc_hlt_003.rs:10
- `FR-SOC-HLT-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1675, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1997, docs/traceability/fr-soc-hlt-004/fr-soc-hlt-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_004.rs:1, crates/engine/tests/fr_fr_soc_hlt_004.rs:6, crates/engine/tests/fr_fr_soc_hlt_004.rs:10
- `FR-SOC-HLT-005`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4412, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4574, docs/traceability/fr-soc-hlt-005/fr-soc-hlt-005-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_hlt_005.rs:1, crates/engine/tests/fr_fr_soc_hlt_005.rs:6, crates/engine/tests/fr_fr_soc_hlt_005.rs:10
- `FR-SOC-IDE-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1588, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1990, docs/traceability/fr-soc-ide-001/fr-soc-ide-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_001.rs:1, crates/engine/tests/fr_fr_soc_ide_001.rs:6, crates/engine/tests/fr_fr_soc_ide_001.rs:10
- `FR-SOC-IDE-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1601, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1991, docs/traceability/fr-soc-ide-002/fr-soc-ide-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_002.rs:1, crates/engine/tests/fr_fr_soc_ide_002.rs:6, crates/engine/tests/fr_fr_soc_ide_002.rs:10
- `FR-SOC-IDE-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1612, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1992, docs/traceability/fr-soc-ide-003/fr-soc-ide-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_003.rs:1, crates/engine/tests/fr_fr_soc_ide_003.rs:6, crates/engine/tests/fr_fr_soc_ide_003.rs:10
- `FR-SOC-IDE-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1624, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1993, docs/traceability/fr-soc-ide-004/fr-soc-ide-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_004.rs:1, crates/engine/tests/fr_fr_soc_ide_004.rs:6, crates/engine/tests/fr_fr_soc_ide_004.rs:10
- `FR-SOC-IDE-005`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4373, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4572, docs/traceability/fr-soc-ide-005/fr-soc-ide-005-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_005.rs:1, crates/engine/tests/fr_fr_soc_ide_005.rs:6, crates/engine/tests/fr_fr_soc_ide_005.rs:10
- `FR-SOC-IDE-006`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4390, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4573, docs/traceability/fr-soc-ide-006/fr-soc-ide-006-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ide_006.rs:1, crates/engine/tests/fr_fr_soc_ide_006.rs:6, crates/engine/tests/fr_fr_soc_ide_006.rs:10
- `FR-SOC-INS-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1692, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1998, docs/traceability/fr-soc-ins-001/fr-soc-ins-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_001.rs:1, crates/engine/tests/fr_fr_soc_ins_001.rs:6, crates/engine/tests/fr_fr_soc_ins_001.rs:10
- `FR-SOC-INS-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1708, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1999, docs/traceability/fr-soc-ins-002/fr-soc-ins-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_002.rs:1, crates/engine/tests/fr_fr_soc_ins_002.rs:6, crates/engine/tests/fr_fr_soc_ins_002.rs:10
- `FR-SOC-INS-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1719, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2000, docs/traceability/fr-soc-ins-003/fr-soc-ins-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_003.rs:1, crates/engine/tests/fr_fr_soc_ins_003.rs:6, crates/engine/tests/fr_fr_soc_ins_003.rs:10
- `FR-SOC-INS-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1733, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2001, docs/traceability/fr-soc-ins-004/fr-soc-ins-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_004.rs:1, crates/engine/tests/fr_fr_soc_ins_004.rs:6, crates/engine/tests/fr_fr_soc_ins_004.rs:10
- `FR-SOC-INS-005`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1752, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2002, docs/traceability/fr-soc-ins-005/fr-soc-ins-005-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_005.rs:1, crates/engine/tests/fr_fr_soc_ins_005.rs:6, crates/engine/tests/fr_fr_soc_ins_005.rs:10
- `FR-SOC-INS-006`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4432, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4575, docs/traceability/fr-soc-ins-006/fr-soc-ins-006-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_006.rs:1, crates/engine/tests/fr_fr_soc_ins_006.rs:6, crates/engine/tests/fr_fr_soc_ins_006.rs:10
- `FR-SOC-INS-007`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4453, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4576, docs/traceability/fr-soc-ins-007/fr-soc-ins-007-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_ins_007.rs:1, crates/engine/tests/fr_fr_soc_ins_007.rs:6, crates/engine/tests/fr_fr_soc_ins_007.rs:10
- `FR-SOC-INT-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1770, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2003, docs/traceability/fr-soc-int-001/fr-soc-int-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_int_001.rs:1, crates/engine/tests/fr_fr_soc_int_001.rs:6, crates/engine/tests/fr_fr_soc_int_001.rs:10
- `FR-SOC-INT-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1778, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2004, docs/traceability/fr-soc-int-002/fr-soc-int-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_int_002.rs:1, crates/engine/tests/fr_fr_soc_int_002.rs:6, crates/engine/tests/fr_fr_soc_int_002.rs:10
- `FR-SOC-INT-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1786, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2005, docs/traceability/fr-soc-int-003/fr-soc-int-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_int_003.rs:1, crates/engine/tests/fr_fr_soc_int_003.rs:6, crates/engine/tests/fr_fr_soc_int_003.rs:10
- `FR-SOC-INT-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1797, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2006, docs/traceability/fr-soc-int-004/fr-soc-int-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_int_004.rs:1, crates/engine/tests/fr_fr_soc_int_004.rs:6, crates/engine/tests/fr_fr_soc_int_004.rs:10
- `FR-SOC-INTG-001`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1812, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2007, docs/traceability/fr-soc-intg-001/fr-soc-intg-001-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_001.rs:1, crates/engine/tests/fr_fr_soc_intg_001.rs:6, crates/engine/tests/fr_fr_soc_intg_001.rs:10
- `FR-SOC-INTG-002`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1822, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2008, docs/traceability/fr-soc-intg-002/fr-soc-intg-002-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_002.rs:1, crates/engine/tests/fr_fr_soc_intg_002.rs:6, crates/engine/tests/fr_fr_soc_intg_002.rs:10
- `FR-SOC-INTG-003`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1830, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2009, docs/traceability/fr-soc-intg-003/fr-soc-intg-003-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_003.rs:1, crates/engine/tests/fr_fr_soc_intg_003.rs:6, crates/engine/tests/fr_fr_soc_intg_003.rs:10
- `FR-SOC-INTG-004`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4478, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4577, docs/traceability/fr-soc-intg-004/fr-soc-intg-004-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_004.rs:1, crates/engine/tests/fr_fr_soc_intg_004.rs:6, crates/engine/tests/fr_fr_soc_intg_004.rs:10
- `FR-SOC-INTG-005`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4495, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4578, docs/traceability/fr-soc-intg-005/fr-soc-intg-005-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_005.rs:1, crates/engine/tests/fr_fr_soc_intg_005.rs:6, crates/engine/tests/fr_fr_soc_intg_005.rs:10
- `FR-SOC-INTG-006`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4517, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4579, docs/traceability/fr-soc-intg-006/fr-soc-intg-006-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_006.rs:1, crates/engine/tests/fr_fr_soc_intg_006.rs:6, crates/engine/tests/fr_fr_soc_intg_006.rs:10
- `FR-SOC-INTG-007`
  - spec: docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4537, docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4580, docs/traceability/fr-soc-intg-007/fr-soc-intg-007-adr.md:1
  - tests: crates/engine/tests/fr_fr_soc_intg_007.rs:1, crates/engine/tests/fr_fr_soc_intg_007.rs:6, crates/engine/tests/fr_fr_soc_intg_007.rs:10
- `FR-STOR-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1931, docs/traceability/fr-stor-001/fr-stor-001-adr.md:1, docs/traceability/fr-stor-001/fr-stor-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_stor_001.rs:1, crates/engine/tests/fr_fr_stor_001.rs:6, crates/engine/tests/fr_fr_stor_001.rs:10
- `FR-TEST-001`
  - spec: docs/FR_DETAILED.md:341, docs/traceability/fr-test-001/fr-test-001-adr.md:1, docs/traceability/fr-test-001/fr-test-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_test_001.rs:1, crates/engine/tests/fr_fr_test_001.rs:6, crates/engine/tests/fr_fr_test_001.rs:10
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
- `FR-VAL-001`
  - spec: docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:170, docs/traceability/fr-val-001/fr-val-001-adr.md:1, docs/traceability/fr-val-001/fr-val-001-adr.md:6
  - tests: crates/engine/tests/fr_fr_val_001.rs:1, crates/engine/tests/fr_fr_val_001.rs:6, crates/engine/tests/fr_fr_val_001.rs:10
- `NFR-C-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2041, docs/traceability/index.md:1151, docs/traceability/nfr-c-01/nfr-c-01-spec.md:1
- `NFR-C-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2042, docs/traceability/index.md:1152, docs/traceability/nfr-c-02/nfr-c-02-spec.md:1
- `NFR-C-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2043, docs/traceability/index.md:1153, docs/traceability/nfr-c-03/nfr-c-03-spec.md:1
- `NFR-C-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2044, docs/traceability/index.md:1154, docs/traceability/nfr-c-04/nfr-c-04-spec.md:1
- `NFR-C-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2045, docs/traceability/index.md:1155, docs/traceability/nfr-c-05/nfr-c-05-spec.md:1
- `NFR-C-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2046, docs/traceability/index.md:1156, docs/traceability/nfr-c-06/nfr-c-06-spec.md:1
- `NFR-C-07`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2047, docs/traceability/index.md:1157, docs/traceability/nfr-c-07/nfr-c-07-spec.md:1
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
- `NFR-CIV-ACC-003`
  - spec: docs/reference/non-functional-requirements.md:380, docs/reference/non-functional-requirements.md:575, docs/traceability/fr-nfr-matrix.md:84
- `NFR-CIV-AI-002`
  - spec: docs/design/civ-ai-crate.md:49, docs/traceability/index.md:1163, docs/traceability/nfr-civ-ai-002/nfr-civ-ai-002-research.md:1
- `NFR-CIV-DET-004`
  - spec: docs/reference/non-functional-requirements.md:172, docs/reference/non-functional-requirements.md:561, docs/reference/non-functional-requirements.md:598
  - tests: crates/engine/tests/fr_nfr_civ_det_004.rs:1
- `NFR-CIV-LEGENDS-LOUD-03`
  - spec: docs/design/legends-engine.md:453, docs/traceability/index.md:1171, docs/traceability/nfr-civ-legends-loud-03/nfr-civ-legends-loud-03-research.md:1
- `NFR-CIV-LEGENDS-PERF-01`
  - spec: docs/design/legends-engine.md:451, docs/traceability/index.md:1172, docs/traceability/nfr-civ-legends-perf-01/nfr-civ-legends-perf-01-research.md:1
- `NFR-CIV-MAINT-001`
  - spec: docs/reference/non-functional-requirements.md:260, docs/reference/non-functional-requirements.md:461, docs/reference/non-functional-requirements.md:499
- `NFR-CIV-MAINT-002`
  - spec: docs/reference/non-functional-requirements.md:475, docs/reference/non-functional-requirements.md:541, docs/reference/non-functional-requirements.md:581
- `NFR-CIV-MAINT-003`
  - spec: docs/reference/non-functional-requirements.md:489, docs/reference/non-functional-requirements.md:582, docs/traceability/fr-nfr-matrix.md:105
- `NFR-CIV-MAINT-004`
  - spec: docs/reference/non-functional-requirements.md:503, docs/reference/non-functional-requirements.md:583, docs/reference/non-functional-requirements.md:607
- `NFR-CIV-MAINT-005`
  - spec: docs/reference/non-functional-requirements.md:517, docs/reference/non-functional-requirements.md:584, docs/reference/non-functional-requirements.md:608
- `NFR-CIV-MAINT-006`
  - spec: docs/reference/non-functional-requirements.md:531, docs/reference/non-functional-requirements.md:585, docs/traceability/fr-nfr-matrix.md:108
- `NFR-CIV-PERF-002`
  - spec: docs/reference/non-functional-requirements.md:41, docs/reference/non-functional-requirements.md:441, docs/reference/non-functional-requirements.md:552
  - tests: crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:127
- `NFR-CIV-PERF-007`
  - spec: docs/reference/non-functional-requirements.md:114, docs/reference/non-functional-requirements.md:230, docs/reference/non-functional-requirements.md:557
- `NFR-CIV-PORT-001`
  - spec: docs/reference/non-functional-requirements.md:51, docs/reference/non-functional-requirements.md:154, docs/reference/non-functional-requirements.md:410
- `NFR-CIV-PORT-002`
  - spec: docs/reference/non-functional-requirements.md:431, docs/reference/non-functional-requirements.md:578, docs/reference/non-functional-requirements.md:610
- `NFR-CIV-PORT-003`
  - spec: docs/reference/non-functional-requirements.md:445, docs/reference/non-functional-requirements.md:527, docs/reference/non-functional-requirements.md:579
- `NFR-CIV-REL-001`
  - spec: docs/reference/non-functional-requirements.md:236, docs/reference/non-functional-requirements.md:565, docs/reference/non-functional-requirements.md:595
- `NFR-CIV-REL-002`
  - spec: docs/reference/non-functional-requirements.md:250, docs/reference/non-functional-requirements.md:332, docs/reference/non-functional-requirements.md:566
- `NFR-CIV-REL-003`
  - spec: docs/reference/non-functional-requirements.md:264, docs/reference/non-functional-requirements.md:567, docs/reference/non-functional-requirements.md:595
- `NFR-CIV-REL-004`
  - spec: docs/reference/non-functional-requirements.md:278, docs/reference/non-functional-requirements.md:568, docs/reference/non-functional-requirements.md:605
  - tests: crates/engine/tests/fr_nfr_civ_rel_004.rs:1
- `NFR-CIV-SCALE-001`
  - spec: docs/reference/non-functional-requirements.md:82, docs/reference/non-functional-requirements.md:188, docs/reference/non-functional-requirements.md:562
  - tests: crates/protocol-3d/tests/fr_perf_005_frame3d_timing.rs:85
- `NFR-CIV-SCALE-003`
  - spec: docs/reference/non-functional-requirements.md:220, docs/reference/non-functional-requirements.md:564, docs/reference/non-functional-requirements.md:602
- `NFR-CIV-SEC-001`
  - spec: docs/reference/non-functional-requirements.md:294, docs/reference/non-functional-requirements.md:569, docs/reference/non-functional-requirements.md:607
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
- `NFR-P-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2053, docs/traceability/index.md:1217, docs/traceability/nfr-p-01/nfr-p-01-spec.md:1
- `NFR-P-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2054, docs/traceability/index.md:1218, docs/traceability/nfr-p-02/nfr-p-02-spec.md:1
- `NFR-P-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2055, docs/traceability/index.md:1219, docs/traceability/nfr-p-03/nfr-p-03-spec.md:1
- `NFR-P-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2056, docs/traceability/index.md:1220, docs/traceability/nfr-p-04/nfr-p-04-spec.md:1
- `NFR-P-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2057, docs/traceability/index.md:1221, docs/traceability/nfr-p-05/nfr-p-05-spec.md:1
- `NFR-P-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2058, docs/traceability/index.md:1222, docs/traceability/nfr-p-06/nfr-p-06-spec.md:1
- `NFR-P-07`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2059, docs/traceability/index.md:1223, docs/traceability/nfr-p-07/nfr-p-07-spec.md:1
- `NFR-P-08`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2060, docs/traceability/index.md:1224, docs/traceability/nfr-p-08/nfr-p-08-spec.md:1
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
- `NFR-S-01`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2066, docs/traceability/index.md:1231, docs/traceability/nfr-s-01/nfr-s-01-spec.md:1
- `NFR-S-02`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2067, docs/traceability/index.md:1232, docs/traceability/nfr-s-02/nfr-s-02-spec.md:1
- `NFR-S-03`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2068, docs/traceability/index.md:1233, docs/traceability/nfr-s-03/nfr-s-03-spec.md:1
- `NFR-S-04`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2069, docs/traceability/index.md:1234, docs/traceability/nfr-s-04/nfr-s-04-spec.md:1
- `NFR-S-05`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2070, docs/traceability/index.md:1235, docs/traceability/nfr-s-05/nfr-s-05-spec.md:1
- `NFR-S-06`
  - spec: docs/models/civ-sim/TECHNICAL_SPEC.md:2071, docs/traceability/index.md:1236, docs/traceability/nfr-s-06/nfr-s-06-spec.md:1

## Implemented but untested IDs (71)

- `FR-CIV-BEVY-013`
  - spec: docs/traceability/fr-3d-matrix.md:178, docs/traceability/full-traceability-matrix.md:307, docs/traceability/fr-civ-bevy-013/fr-civ-bevy-013-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:125
- `FR-CIV-BEVY-014`
  - spec: docs/traceability/fr-3d-matrix.md:179, docs/traceability/full-traceability-matrix.md:308, docs/traceability/fr-civ-bevy-014/fr-civ-bevy-014-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:82
- `FR-CIV-BEVY-015`
  - spec: docs/traceability/fr-3d-matrix.md:180, docs/traceability/full-traceability-matrix.md:309, docs/traceability/fr-civ-bevy-015/fr-civ-bevy-015-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:127
- `FR-CIV-BEVY-017`
  - spec: docs/traceability/fr-3d-matrix.md:182, docs/traceability/full-traceability-matrix.md:311, docs/traceability/fr-civ-bevy-017/fr-civ-bevy-017-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:129
- `FR-CIV-BEVY-018`
  - spec: docs/traceability/fr-3d-matrix.md:183, docs/traceability/full-traceability-matrix.md:312, docs/traceability/fr-civ-bevy-018/fr-civ-bevy-018-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:130
- `FR-CIV-BEVY-019`
  - spec: docs/traceability/fr-3d-matrix.md:184, docs/traceability/full-traceability-matrix.md:313, docs/traceability/fr-civ-bevy-019/fr-civ-bevy-019-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:131
- `FR-CIV-BEVY-020`
  - spec: docs/traceability/fr-3d-matrix.md:185, docs/traceability/full-traceability-matrix.md:314, docs/traceability/fr-civ-bevy-020/fr-civ-bevy-020-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:132
- `FR-CIV-BEVY-021`
  - spec: docs/traceability/fr-3d-matrix.md:186, docs/traceability/full-traceability-matrix.md:315, docs/traceability/fr-civ-bevy-021/fr-civ-bevy-021-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:83, docs/development-guide/p-w1-kickoff.md:133
- `FR-CIV-ECON-002-JOULE`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:70, docs/traceability/fr-civ-econ-002-joule/fr-civ-econ-002-joule-adr.md:1, docs/traceability/fr-civ-econ-002-joule/fr-civ-econ-002-joule-adr.md:6
  - code: docs/guides/COPILOT_L3_AGENTS.md:92, docs/guides/COPILOT_L3_AGENTS.md:93, docs/guides/COPILOT_L3_AGENTS.md:474
- `FR-CIV-ENGINE-INT-012`
  - spec: docs/traceability/fr-civ-engine-int-012/fr-civ-engine-int-012-adr.md:1, docs/traceability/fr-civ-engine-int-012/fr-civ-engine-int-012-adr.md:6, docs/traceability/fr-civ-engine-int-012/fr-civ-engine-int-012-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1235
- `FR-CIV-GEO-001`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2024, docs/traceability/fr-civ-geo-001/fr-civ-geo-001-adr.md:1, docs/traceability/fr-civ-geo-001/fr-civ-geo-001-adr.md:6
  - code: docs/reference/FR_TRACKER.md:22, docs/reports/STATUS_REPORT.md:95
- `FR-CIV-GEO-004`
  - spec: docs/specs/CIV-0300-rts-ui-ux-spec.md:2027, docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:1, docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:6
  - code: docs/reports/STATUS_REPORT.md:96
- `FR-CIV-GODOT-ATTACH-001`
  - spec: docs/traceability/fr-civ-godot-attach-001/fr-civ-godot-attach-001-adr.md:1, docs/traceability/fr-civ-godot-attach-001/fr-civ-godot-attach-001-adr.md:6, docs/traceability/fr-civ-godot-attach-001/fr-civ-godot-attach-001-adr.md:11
  - code: docs/development-guide/fr-godot-attach.md:9
- `FR-CIV-GODOT-ATTACH-002`
  - spec: docs/traceability/fr-civ-godot-attach-002/fr-civ-godot-attach-002-adr.md:1, docs/traceability/fr-civ-godot-attach-002/fr-civ-godot-attach-002-adr.md:6, docs/traceability/fr-civ-godot-attach-002/fr-civ-godot-attach-002-adr.md:11
  - code: docs/development-guide/fr-godot-attach.md:10
- `FR-CIV-GODOT-ATTACH-003`
  - spec: docs/traceability/fr-civ-godot-attach-003/fr-civ-godot-attach-003-adr.md:1, docs/traceability/fr-civ-godot-attach-003/fr-civ-godot-attach-003-adr.md:6, docs/traceability/fr-civ-godot-attach-003/fr-civ-godot-attach-003-adr.md:11
  - code: docs/development-guide/fr-godot-attach.md:11
- `FR-CIV-GODOT-ATTACH-004`
  - spec: docs/traceability/fr-civ-godot-attach-004/fr-civ-godot-attach-004-adr.md:1, docs/traceability/fr-civ-godot-attach-004/fr-civ-godot-attach-004-adr.md:6, docs/traceability/fr-civ-godot-attach-004/fr-civ-godot-attach-004-adr.md:11
  - code: docs/development-guide/fr-godot-attach.md:12
- `FR-CIV-GODOT-UX-000`
  - spec: docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:1, docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:6, docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:11
  - code: docs/development-guide/fr-godot-attach.md:13
- `FR-CIV-LIFE-035`
  - spec: docs/traceability/fr-civ-life-035/fr-civ-life-035-adr.md:1, docs/traceability/fr-civ-life-035/fr-civ-life-035-adr.md:6, docs/traceability/fr-civ-life-035/fr-civ-life-035-adr.md:11
  - code: crates/agents/src/cluster.rs:76
- `FR-CIV-PLANET-020`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:85, docs/traceability/fr-civ-planet-020/fr-civ-planet-020-adr.md:1, docs/traceability/fr-civ-planet-020/fr-civ-planet-020-adr.md:6
  - code: crates/engine/src/climate.rs:16, crates/engine/src/climate.rs:35, crates/engine/src/climate.rs:48
- `FR-CIV-PLANET-030`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:91, docs/traceability/fr-civ-planet-030/fr-civ-planet-030-adr.md:1, docs/traceability/fr-civ-planet-030/fr-civ-planet-030-adr.md:6
  - code: crates/engine/src/climate.rs:35, crates/engine/src/engine/engine_tests.rs:1850, crates/engine/src/engine.rs:820
- `FR-CIV-PROTO3D-009-`
  - spec: docs/specs/gap-audit.md:306, docs/traceability/fr-3d-matrix.md:229
  - code: docs/development-guide/p-w1-kickoff.md:116
- `FR-CIV-PSYCHE-900`
  - spec: docs/specs/requirements/FR-CIV-PSYCHE.md:11, docs/traceability/civis-tracelinks.md:142, docs/traceability/fr-civ-psyche-900/fr-civ-psyche-900-adr.md:1
  - code: crates/agents/src/psyche.rs:312, crates/agents/src/psyche.rs:424, crates/engine/src/dormant_phases.rs:3
- `FR-CIV-PSYCHE-911`
  - spec: docs/specs/build-green/engine_missing_symbols.md:13, docs/specs/requirements/FR-CIV-PSYCHE.md:14, docs/traceability/fr-civ-psyche-911/fr-civ-psyche-911-adr.md:1
  - code: crates/agents/src/psyche.rs:491, crates/engine/src/engine/ai_decision.rs:57, crates/engine/src/engine/engine_tests.rs:453
- `FR-CIV-RELIGION-001`
  - spec: docs/traceability/civis-tracelinks.md:143
  - code: crates/engine/src/emergence.rs:2436
- `FR-CIV-RELIGION-002`
  - spec: docs/traceability/fr-emergence-matrix.md:80
  - code: clients/bevy-ref/src/game_ui.rs:38, crates/emergence-oracle/src/oracles/religion.rs:8, crates/engine/src/emergence.rs:208
- `FR-CIV-RESEARCH-004-REPLAY`
  - spec: docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:1, docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:6, docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:11
  - code: PLAN.md:239
- `FR-CIV-SOCIAL-001`
  - spec: agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:26, agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:40, agileplus-specs/civ-007-diplomacy-laws-government/spec.md:42
  - code: docs/reference/agileplus-artifacts-index.md:73, docs/reference/agileplus-artifacts-index.md:270, PLAN.md:147
- `FR-CIV-SOCIAL-001-INSTITUTIONS`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:225, docs/traceability/fr-civ-social-001-institutions/fr-civ-social-001-institutions-adr.md:1, docs/traceability/fr-civ-social-001-institutions/fr-civ-social-001-institutions-adr.md:6
  - code: PLAN.md:147, PLAN.md:148
- `FR-CIV-SOCIAL-002`
  - spec: agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:27, agileplus-specs/civ-009-culture-diffusion/spec.md:37, docs/traceability/fr-civ-social-002/fr-civ-social-002-adr.md:1
  - code: docs/reference/agileplus-artifacts-index.md:73, docs/reference/agileplus-artifacts-index.md:271, PLAN.md:149
- `FR-CIV-SOCIAL-002-IDEOLOGY`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:226, docs/traceability/fr-civ-social-002-ideology/fr-civ-social-002-ideology-adr.md:1, docs/traceability/fr-civ-social-002-ideology/fr-civ-social-002-ideology-adr.md:6
  - code: PLAN.md:149, PLAN.md:150
- `FR-CIV-TACTICS-025-`
  - spec: docs/traceability/fr-3d-matrix.md:123, docs/traceability/fr-3d-matrix.md:124, docs/traceability/fr-3d-matrix.md:125
  - code: crates/engine/src/engine/engine_tests.rs:1600, crates/engine/src/engine/engine_tests.rs:1622, crates/engine/src/engine/engine_tests.rs:1646
- `FR-CIV-TACTICS-032`
  - spec: docs/traceability/fr-3d-matrix.md:119, docs/traceability/full-traceability-matrix.md:227, docs/traceability/fr-civ-tactics-032/fr-civ-tactics-032-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:31
- `FR-CIV-TACTICS-035`
  - spec: docs/traceability/fr-3d-matrix.md:122, docs/traceability/full-traceability-matrix.md:230, docs/traceability/fr-civ-tactics-035/fr-civ-tactics-035-adr.md:1
  - code: crates/engine/src/engine.rs:812, crates/tactics/src/military_phase.rs:1, docs/development-guide/p-w1-kickoff.md:34
- `FR-CIV-TACTICS-045`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:56, docs/traceability/fr-3d-matrix.md:135, docs/traceability/full-traceability-matrix.md:240
  - code: crates/engine/src/scenario.rs:99, docs/development-guide/p-w1-kickoff.md:47
- `FR-CIV-TACTICS-050`
  - spec: docs/traceability/fr-3d-matrix.md:140, docs/traceability/full-traceability-matrix.md:245, docs/traceability/fr-civ-tactics-050/fr-civ-tactics-050-adr.md:1
  - code: crates/engine/src/engine/military_phases.rs:234, crates/engine/src/scenario.rs:105, crates/engine/src/scenario.rs:189
- `FR-CIV-UI-001`
  - spec: docs/traceability/fr-civ-ui-001/fr-civ-ui-001-adr.md:1, docs/traceability/fr-civ-ui-001/fr-civ-ui-001-adr.md:6, docs/traceability/fr-civ-ui-001/fr-civ-ui-001-adr.md:11
  - code: docs/guides/voxel-emergent-vision-and-migration.md:99, docs/guides/voxel-emergent-vision-and-migration.md:155, docs/guides/voxel-emergent-vision-and-migration.md:159
- `FR-CIV-UI-002`
  - spec: docs/traceability/fr-civ-ui-002/fr-civ-ui-002-adr.md:1, docs/traceability/fr-civ-ui-002/fr-civ-ui-002-adr.md:6, docs/traceability/fr-civ-ui-002/fr-civ-ui-002-adr.md:11
  - code: docs/guides/voxel-emergent-vision-and-migration.md:99, docs/guides/voxel-emergent-vision-and-migration.md:160
- `FR-CIV-UI-003`
  - spec: docs/traceability/fr-civ-ui-003/fr-civ-ui-003-adr.md:1, docs/traceability/fr-civ-ui-003/fr-civ-ui-003-adr.md:6, docs/traceability/fr-civ-ui-003/fr-civ-ui-003-adr.md:11
  - code: docs/guides/voxel-emergent-vision-and-migration.md:99, docs/guides/voxel-emergent-vision-and-migration.md:155, docs/guides/voxel-emergent-vision-and-migration.md:161
- `FR-CIV-UX-002`
  - spec: docs/traceability/fr-civ-ux-002/fr-civ-ux-002-adr.md:1, docs/traceability/fr-civ-ux-002/fr-civ-ux-002-adr.md:6, docs/traceability/fr-civ-ux-002/fr-civ-ux-002-adr.md:11
  - code: crates/server/src/jsonrpc.rs:58, docs/development-guide/fr-godot-attach.md:14
- `FR-CIV-UX-003`
  - spec: docs/traceability/fr-civ-ux-003/fr-civ-ux-003-adr.md:1, docs/traceability/fr-civ-ux-003/fr-civ-ux-003-adr.md:6, docs/traceability/fr-civ-ux-003/fr-civ-ux-003-adr.md:11
  - code: crates/server/src/jsonrpc.rs:62, docs/development-guide/fr-godot-attach.md:15
- `FR-CIV-VOXEL-006`
  - spec: docs/traceability/fr-civ-voxel-006/fr-civ-voxel-006-adr.md:1, docs/traceability/fr-civ-voxel-006/fr-civ-voxel-006-adr.md:6, docs/traceability/fr-civ-voxel-006/fr-civ-voxel-006-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1395
- `FR-CIV-WEB-000`
  - spec: docs/traceability/fr-civ-web-000/fr-civ-web-000-adr.md:1, docs/traceability/fr-civ-web-000/fr-civ-web-000-adr.md:6, docs/traceability/fr-civ-web-000/fr-civ-web-000-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:3, docs/development-guide/fr-web-spectator.md:29, docs/development-guide/pr-296-body.md:20
- `FR-CIV-WEB-001`
  - spec: docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:1, docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:6, docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:30
- `FR-CIV-WEB-002`
  - spec: docs/traceability/fr-civ-web-002/fr-civ-web-002-adr.md:1, docs/traceability/fr-civ-web-002/fr-civ-web-002-adr.md:6, docs/traceability/fr-civ-web-002/fr-civ-web-002-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:31, web/dashboard/src/lib/civisServer.ts:1
- `FR-CIV-WEB-004`
  - spec: docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:1, docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:6, docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:33
- `FR-CIV-WEB-005`
  - spec: docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:1, docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:6, docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:34
- `FR-CIV-WEB-006`
  - spec: docs/traceability/fr-civ-web-006/fr-civ-web-006-adr.md:1, docs/traceability/fr-civ-web-006/fr-civ-web-006-adr.md:6, docs/traceability/fr-civ-web-006/fr-civ-web-006-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:35, web/dashboard/src/lib/frame3d.ts:1
- `FR-CIV-WEB-007`
  - spec: docs/traceability/fr-civ-web-007/fr-civ-web-007-adr.md:1, docs/traceability/fr-civ-web-007/fr-civ-web-007-adr.md:6, docs/traceability/fr-civ-web-007/fr-civ-web-007-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:36, docs/development-guide/pr-296-merge-readiness.md:35, docs/IMPLEMENTATION_STATUS.md:62
- `FR-CIV-WEB-008`
  - spec: docs/traceability/fr-civ-web-008/fr-civ-web-008-adr.md:1, docs/traceability/fr-civ-web-008/fr-civ-web-008-adr.md:6, docs/traceability/fr-civ-web-008/fr-civ-web-008-adr.md:11
  - code: docs/development-guide/fr-web-spectator.md:37, docs/IMPLEMENTATION_STATUS.md:63, web/dashboard/src/lib/authoring.ts:95
- `NFR-CIV-ACC-001`
  - spec: agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:108, docs/reference/non-functional-requirements.md:352, docs/reference/non-functional-requirements.md:573
  - code: docs/guides/voxel-emergent-vision-and-migration.md:173
- `NFR-CIV-ACC-002`
  - spec: docs/reference/non-functional-requirements.md:366, docs/reference/non-functional-requirements.md:574, docs/traceability/fr-nfr-matrix.md:83
  - code: docs/guides/voxel-emergent-vision-and-migration.md:99
- `NFR-CIV-ACC-004`
  - spec: docs/reference/non-functional-requirements.md:394, docs/reference/non-functional-requirements.md:576, docs/reference/non-functional-requirements.md:610
  - code: docs/guides/voxel-emergent-vision-and-migration.md:99
- `NFR-CIV-AI-001`
  - spec: docs/design/civ-ai-crate.md:48, docs/design/civ-ai-crate.md:180, docs/design/civ-ai-crate.md:216
  - code: crates/ai/src/pool.rs:1
- `NFR-CIV-AI-003`
  - spec: docs/design/civ-ai-crate.md:50, docs/design/civ-ai-crate.md:164, docs/traceability/index.md:1164
  - code: crates/ai/src/lib.rs:13, crates/ai/src/lib.rs:229
- `NFR-CIV-DEV-HYGIENE-001`
  - spec: docs/traceability/index.md:1169, docs/traceability/nfr-civ-dev-hygiene-001/nfr-civ-dev-hygiene-001-spec.md:1, docs/traceability/nfr-civ-dev-hygiene-001/nfr-civ-dev-hygiene-001-spec.md:5
  - code: docs/ops/history-purge-plan.md:4
- `NFR-CIV-LEGENDS-CONFIG-04`
  - spec: docs/design/legends-engine.md:454, docs/traceability/index.md:1170, docs/traceability/nfr-civ-legends-config-04/nfr-civ-legends-config-04-research.md:1
  - code: crates/legends/src/config.rs:1
- `NFR-CIV-LEGENDS-SCALE-02`
  - spec: docs/design/legends-engine.md:452, docs/traceability/index.md:1173, docs/traceability/nfr-civ-legends-scale-02/nfr-civ-legends-scale-02-adr.md:1
  - code: crates/legends/src/lib.rs:17
- `NFR-CIV-PERF-003`
  - spec: agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:69, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:25, docs/design/civ-perf-dirty-incremental.md:9
  - code: docs/guides/voxel-emergent-vision-and-migration.md:170, docs/guides/voxel-emergent-vision-and-migration.md:189
- `NFR-CIV-PERF-004`
  - spec: docs/reference/non-functional-requirements.md:69, docs/reference/non-functional-requirements.md:193, docs/reference/non-functional-requirements.md:194
  - code: docs/guides/voxel-emergent-vision-and-migration.md:189
- `NFR-CIV-PERF-005`
  - spec: agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:24, docs/design/civ-perf-dirty-incremental.md:10, docs/design/civ-perf-dirty-incremental.md:502
  - code: docs/guides/voxel-emergent-vision-and-migration.md:96
- `NFR-CIV-PERF-006`
  - spec: docs/reference/non-functional-requirements.md:100, docs/reference/non-functional-requirements.md:556, docs/reference/non-functional-requirements.md:596
  - code: docs/guides/voxel-emergent-vision-and-migration.md:172
- `NFR-CIV-PERF-008`
  - spec: docs/traceability/index.md:1187, docs/traceability/nfr-civ-perf-008/nfr-civ-perf-008-research.md:1, docs/traceability/nfr-civ-perf-008/nfr-civ-perf-008-research.md:4
  - code: docs/guides/voxel-emergent-vision-and-migration.md:171, docs/guides/voxel-emergent-vision-and-migration.md:191
- `NFR-CIV-PERF-900`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:15, docs/traceability/fr-nfr-matrix.md:24, docs/traceability/fr-nfr-matrix.md:148
  - code: docs/agileplus/epics/civ-w5-scale.md:14, docs/agileplus/epics/civ-w5-scale.md:27, docs/agileplus/README.md:24
- `NFR-CIV-PERF-901`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:16, docs/traceability/fr-nfr-matrix.md:25, docs/traceability/index.md:1189
  - code: docs/agileplus/epics/civ-w5-scale.md:15, docs/agileplus/epics/civ-w5-scale.md:27, docs/agileplus/README.md:24
- `NFR-CIV-PERF-902`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:17, docs/traceability/fr-nfr-matrix.md:26, docs/traceability/index.md:1190
  - code: docs/agileplus/epics/civ-w5-scale.md:16, docs/agileplus/epics/civ-w5-scale.md:28, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-004`
  - spec: docs/traceability/index.md:1201, docs/traceability/nfr-civ-scale-004/nfr-civ-scale-004-adr.md:1, docs/traceability/nfr-civ-scale-004/nfr-civ-scale-004-adr.md:6
  - code: docs/guides/voxel-emergent-vision-and-migration.md:172
- `NFR-CIV-SCALE-900`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:11, docs/traceability/fr-nfr-matrix.md:48, docs/traceability/index.md:1202
  - code: docs/agileplus/epics/civ-w5-scale.md:9, docs/agileplus/epics/civ-w5-scale.md:22, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-902`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:13, docs/traceability/fr-nfr-matrix.md:50, docs/traceability/fr-nfr-matrix.md:159
  - code: docs/agileplus/epics/civ-w5-scale.md:11, docs/agileplus/epics/civ-w5-scale.md:24, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-910`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:14, docs/traceability/fr-nfr-matrix.md:51, docs/traceability/index.md:1205
  - code: docs/agileplus/epics/civ-w5-scale.md:12, docs/agileplus/epics/civ-w5-scale.md:25, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-920`
  - spec: docs/specs/requirements/NFR-CIV-SCALE-PERF.md:18, docs/traceability/fr-nfr-matrix.md:52, docs/traceability/fr-nfr-matrix.md:159
  - code: docs/agileplus/epics/civ-w5-scale.md:13, docs/agileplus/epics/civ-w5-scale.md:26, docs/agileplus/README.md:24
- `NFR-SCALE-02`
  - spec: docs/traceability/fr-emergence-matrix.md:261, docs/traceability/index.md:1237, docs/traceability/nfr-scale-02/nfr-scale-02-adr.md:1
  - code: crates/legends/src/config.rs:18

## Code-only IDs (missing spec/traceability) (215)

- `FR-ASSET-PIPELINE-001`
  - code: crates/asset-pipeline/src/lib.rs:3, crates/asset-pipeline/src/lib.rs:18, crates/asset-pipeline/src/lib.rs:57
- `FR-ASSET-PIPELINE-002`
  - code: crates/asset-pipeline/Cargo.toml:8, crates/asset-pipeline/src/bin/svg_export.rs:20, crates/asset-pipeline/src/error.rs:9
- `FR-CIV-014`
  - code: crates/engine/src/emergence.rs:1625, crates/engine/src/engine/engine_tests.rs:3157, crates/engine/src/save.rs:335
  - tests: crates/engine/src/emergence.rs:1625, crates/engine/src/save.rs:335, crates/engine/src/save.rs:375
- `FR-CIV-ACCESS-010`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:47
- `FR-CIV-ACCESS-020`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:48
- `FR-CIV-AGGRESSION-001`
  - code: crates/engine/src/engine.rs:492, crates/engine/src/engine.rs:2191
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:3
- `FR-CIV-ARCH-00`
  - tests: crates/engine/tests/fr_civ_act_arch_cluster.rs:2
- `FR-CIV-ARCH-A-001`
  - code: crates/build/src/tiers.rs:405
  - tests: crates/build/src/tiers.rs:405
- `FR-CIV-ARCH-A-002`
  - code: crates/build/src/tiers.rs:426
  - tests: crates/build/src/tiers.rs:426
- `FR-CIV-ARCH-A-003`
  - code: crates/build/src/tiers.rs:448
  - tests: crates/build/src/tiers.rs:448
- `FR-CIV-ARCH-B-001`
  - code: crates/build/src/tiers.rs:472
  - tests: crates/build/src/tiers.rs:472
- `FR-CIV-ARCH-B-002`
  - code: crates/build/src/tiers.rs:491
  - tests: crates/build/src/tiers.rs:491
- `FR-CIV-ARCH-B-003`
  - code: crates/build/src/tiers.rs:505
  - tests: crates/build/src/tiers.rs:505
- `FR-CIV-ARCH-B-004`
  - code: crates/build/src/tiers.rs:514
  - tests: crates/build/src/tiers.rs:514
- `FR-CIV-ARCH-C-001`
  - code: crates/build/src/tiers.rs:530
  - tests: crates/build/src/tiers.rs:530
- `FR-CIV-ARCH-C-002`
  - code: crates/build/src/tiers.rs:558
  - tests: crates/build/src/tiers.rs:558
- `FR-CIV-ARCH-C-003`
  - code: crates/build/src/tiers.rs:576
  - tests: crates/build/src/tiers.rs:576
- `FR-CIV-ARCH-C-004`
  - code: crates/build/src/tiers.rs:586
  - tests: crates/build/src/tiers.rs:586
- `FR-CIV-ARCH-D-001`
  - code: crates/build/src/tiers.rs:602
  - tests: crates/build/src/tiers.rs:602
- `FR-CIV-ARCH-D-002`
  - code: crates/build/src/tiers.rs:618
  - tests: crates/build/src/tiers.rs:618
- `FR-CIV-ARCH-D-003`
  - code: crates/build/src/tiers.rs:633
  - tests: crates/build/src/tiers.rs:633
- `FR-CIV-ARCH-D-004`
  - code: crates/build/src/tiers.rs:643
  - tests: crates/build/src/tiers.rs:643
- `FR-CIV-BELIEF-001`
  - code: crates/engine/src/religion.rs:40, crates/engine/src/religion.rs:55, crates/engine/src/religion.rs:88
- `FR-CIV-BEVY-028`
  - code: crates/server/src/ws_bridge.rs:57
  - tests: crates/server/tests/ws_smoke.rs:1513, crates/server/tests/ws_smoke.rs:1522
- `FR-CIV-BEVY-034`
  - code: clients/bevy-ref/src/diplomacy_ui.rs:625
  - tests: clients/bevy-ref/src/diplomacy_ui.rs:625
- `FR-CIV-BEVY-035`
  - code: clients/bevy-ref/src/lib.rs:1973
  - tests: clients/bevy-ref/src/lib.rs:1973
- `FR-CIV-BEVY-036`
  - code: clients/bevy-ref/src/menus.rs:4, clients/bevy-ref/src/menus.rs:1348
- `FR-CIV-CA-011`
  - code: crates/voxel/src/fluid_ca.rs:1360, crates/voxel/src/fluid_ca.rs:1698
- `FR-CIV-CARAVAN-001`
  - code: crates/engine/src/caravan.rs:3
- `FR-CIV-CLIENT-006`
  - code: crates/civis-mcp/src/server.rs:346, crates/civis-mcp/src/server.rs:2264, crates/server/src/jsonrpc.rs:82
- `FR-CIV-CLIENT-011`
  - code: clients/bevy-ref/src/tutorial.rs:3
- `FR-CIV-CLIENT-013`
  - code: clients/bevy-ref/src/civ_history.rs:2
- `FR-CIV-CLIMATE-1`
  - code: crates/planet/src/seasonal.rs:181
  - tests: crates/planet/src/seasonal.rs:181
- `FR-CIV-CLIMATE-2`
  - code: crates/planet/src/seasonal.rs:193
  - tests: crates/planet/src/seasonal.rs:193
- `FR-CIV-CLIMATE-3`
  - code: crates/planet/src/seasonal.rs:209
  - tests: crates/planet/src/seasonal.rs:209
- `FR-CIV-CLIMATE-4`
  - code: crates/planet/src/seasonal.rs:218
  - tests: crates/planet/src/seasonal.rs:218
- `FR-CIV-COHESION-001`
  - code: crates/engine/src/engine/social_settlement_phases.rs:272
- `FR-CIV-CONSTRUCTION-001`
  - code: crates/engine/src/engine.rs:466, crates/engine/src/engine.rs:2182
  - tests: crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4, crates/engine/tests/persistence_replay_coverage.rs:13
- `FR-CIV-CONTENT-001`
  - code: crates/engine/src/emergence_coupling.rs:471
- `FR-CIV-CORE-021`
  - tests: crates/build/tests/fr_matrix_batch12.rs:765
- `FR-CIV-CULTURE-001`
  - code: crates/engine/src/engine.rs:478, crates/engine/src/engine.rs:2190
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:2
- `FR-CIV-DIPLO-003-006`
  - code: crates/diplomacy/src/shadow_networks.rs:660
  - tests: crates/diplomacy/src/shadow_networks.rs:660
- `FR-CIV-DIPLO-003-01`
  - code: crates/diplomacy/src/shadow_networks.rs:403, crates/diplomacy/src/shadow_networks.rs:405, crates/diplomacy/src/shadow_networks.rs:433
  - tests: crates/diplomacy/src/shadow_networks.rs:403, crates/diplomacy/src/shadow_networks.rs:405, crates/diplomacy/src/shadow_networks.rs:433
- `FR-CIV-DIPLO-003-02`
  - code: crates/diplomacy/src/shadow_networks.rs:470, crates/diplomacy/src/shadow_networks.rs:472, crates/diplomacy/src/shadow_networks.rs:509
  - tests: crates/diplomacy/src/shadow_networks.rs:470, crates/diplomacy/src/shadow_networks.rs:472, crates/diplomacy/src/shadow_networks.rs:509
- `FR-CIV-DIPLO-003-03`
  - code: crates/diplomacy/src/shadow_networks.rs:520, crates/diplomacy/src/shadow_networks.rs:522, crates/diplomacy/src/shadow_networks.rs:560
  - tests: crates/diplomacy/src/shadow_networks.rs:520, crates/diplomacy/src/shadow_networks.rs:522, crates/diplomacy/src/shadow_networks.rs:560
- `FR-CIV-DIPLO-003-04`
  - code: crates/diplomacy/src/shadow_networks.rs:594, crates/diplomacy/src/shadow_networks.rs:596
  - tests: crates/diplomacy/src/shadow_networks.rs:594, crates/diplomacy/src/shadow_networks.rs:596
- `FR-CIV-DIPLO-003-05`
  - code: crates/diplomacy/src/shadow_networks.rs:620, crates/diplomacy/src/shadow_networks.rs:622
  - tests: crates/diplomacy/src/shadow_networks.rs:620, crates/diplomacy/src/shadow_networks.rs:622
- `FR-CIV-DIPLO-003-06`
  - code: crates/diplomacy/src/shadow_networks.rs:658
  - tests: crates/diplomacy/src/shadow_networks.rs:658
- `FR-CIV-DIPLO-003-07`
  - code: crates/diplomacy/src/shadow_networks.rs:710, crates/diplomacy/src/shadow_networks.rs:712
  - tests: crates/diplomacy/src/shadow_networks.rs:710, crates/diplomacy/src/shadow_networks.rs:712
- `FR-CIV-DIPLOMACY-001`
  - code: crates/engine/src/engine.rs:2168, crates/engine/src/engine.rs:2343
  - tests: crates/engine/tests/diplomacy_flow.rs:20, crates/engine/tests/persistence_replay_coverage.rs:15
- `FR-CIV-DIPLOMACY-004`
  - code: crates/diplomacy/src/stance.rs:1, crates/engine/src/engine.rs:440, crates/engine/src/engine.rs:1003
- `FR-CIV-ECON-010`
  - code: crates/engine/src/engine.rs:532, crates/engine/src/engine.rs:723, crates/engine/src/engine.rs:2200
  - tests: crates/engine/tests/riot_migrant_taxation_persistence.rs:3
- `FR-CIV-ECON-FOCUS-001`
  - code: crates/engine/src/engine.rs:472, crates/engine/src/engine.rs:2183
  - tests: crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4, crates/engine/tests/persistence_replay_coverage.rs:14
- `FR-CIV-EMERGE-DASH-001`
  - code: clients/bevy-ref/src/emergence_dashboard.rs:3
- `FR-CIV-EMERGENT-MIGRATION-001`
  - code: crates/engine/src/emergent_migration.rs:1, crates/engine/src/emergent_migration.rs:25
- `FR-CIV-ERA-001`
  - code: crates/engine/src/engine.rs:538
  - tests: crates/engine/tests/era_emergence_significance_persistence.rs:2
- `FR-CIV-FAMINE-001`
  - code: crates/engine/src/famine.rs:1
- `FR-CIV-FEST-001`
  - code: crates/engine/src/festivals.rs:1
- `FR-CIV-GAME-001`
  - code: clients/bevy-ref/src/gameplay_hud.rs:3, clients/bevy-ref/src/lib.rs:541, clients/bevy-ref/src/outcome_overlay.rs:3
- `FR-CIV-GAME-002`
  - code: clients/bevy-ref/src/god_panel.rs:2, crates/engine/src/gameplay.rs:2, crates/engine/src/gameplay.rs:888
  - tests: crates/engine/src/gameplay.rs:888
- `FR-CIV-GAME-003`
  - code: clients/bevy-ref/src/era_hud.rs:2, crates/engine/src/era.rs:1
- `FR-CIV-GENETICS-SEED-001`
  - code: crates/engine/src/engine/engine_tests.rs:2941
- `FR-CIV-GENETICS-SEED-002`
  - code: crates/engine/src/engine/engine_tests.rs:2991
- `FR-CIV-GENETICS-SEED-003`
  - code: crates/engine/src/engine/engine_tests.rs:3037
- `FR-CIV-GODTOOL-001`
  - code: crates/civis-mcp/src/server.rs:148
- `FR-CIV-GOV-003`
  - code: crates/civ-institutions/src/lib.rs:38, crates/engine/src/engine.rs:856, crates/engine/src/engine.rs:865
  - tests: crates/engine/tests/fr_civ_gov_institutions.rs:19, crates/engine/tests/fr_civ_gov_institutions.rs:132, crates/engine/tests/fr_civ_gov_institutions.rs:133
- `FR-CIV-GOV-010`
  - code: crates/engine/src/engine.rs:3431
  - tests: crates/engine/tests/fr_civ_gov_mood.rs:1
- `FR-CIV-GOV-020`
  - code: crates/engine/src/social_types.rs:5, crates/engine/src/social_types.rs:69
  - tests: crates/engine/tests/fr_civ_gov_stratification.rs:1, crates/engine/tests/fr_civ_gov_stratification.rs:28, crates/engine/tests/fr_civ_gov_stratification.rs:29
- `FR-CIV-GOV-100`
  - code: crates/engine/src/engine.rs:870, crates/engine/src/engine.rs:875, crates/engine/src/engine.rs:880
  - tests: crates/engine/tests/fr_emergence_quality.rs:347
- `FR-CIV-GOV-200`
  - code: crates/engine/src/engine/social_settlement_phases.rs:179
- `FR-CIV-IDEOLOGY-001`
  - code: crates/engine/src/engine.rs:486, crates/engine/src/engine.rs:2190
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:2
- `FR-CIV-INSTITUTIONS-001`
  - code: crates/engine/src/engine.rs:458, crates/engine/src/engine.rs:2182
  - tests: crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:3, crates/engine/tests/persistence_replay_coverage.rs:12
- `FR-CIV-INT-001`
  - tests: crates/engine/tests/fr_engine_replay_integrity_tests.rs:5, crates/engine/tests/fr_engine_replay_integrity_tests.rs:113, crates/engine/tests/fr_engine_replay_integrity_tests.rs:116
- `FR-CIV-L10N-010`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:49
- `FR-CIV-L10N-020`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:50
- `FR-CIV-L10N-030`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:51
- `FR-CIV-L10N-040`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:52
- `FR-CIV-LEGENDS-010`
  - code: crates/engine/src/engine.rs:998
- `FR-CIV-NEEDS-DECAY-01`
  - code: crates/needs/src/decay.rs:32, crates/needs/src/decay.rs:219, crates/needs/src/decay.rs:336
  - tests: crates/needs/src/decay.rs:336, crates/needs/src/decay.rs:361, crates/needs/src/decay.rs:386
- `FR-CIV-PBR-009`
  - code: crates/voxel/src/material_pbr.rs:754, crates/voxel/src/material_pbr.rs:1461
  - tests: crates/voxel/src/material_pbr.rs:1461
- `FR-CIV-PBR-010`
  - code: CHANGELOG.md:16, crates/voxel/src/material_pbr.rs:14, crates/voxel/src/material_pbr.rs:1560
  - tests: crates/voxel/src/material_pbr.rs:1560, crates/voxel/src/material_pbr.rs:1562, crates/voxel/src/material_pbr.rs:1594
- `FR-CIV-PBR-011`
  - code: CHANGELOG.md:17, crates/voxel/src/atlas/gpu_atlas.rs:1, crates/voxel/src/atlas/gpu_atlas.rs:40
- `FR-CIV-PLANET-050`
  - code: crates/planet/src/geology.rs:651, crates/planet/src/geology.rs:658, crates/planet/src/geology.rs:665
  - tests: crates/planet/src/geology.rs:651, crates/planet/src/geology.rs:658, crates/planet/src/geology.rs:665
- `FR-CIV-PSYCHE-N11`
  - code: crates/engine/src/dormant_phases.rs:88
- `FR-CIV-REL-007`
  - tests: crates/engine/tests/fr_civ_religion_007_phase_belief.rs:1
- `FR-CIV-SERVER-003`
  - code: crates/civis-mcp/src/server.rs:2287, crates/civis-mcp/src/server.rs:2310, crates/server/src/jsonrpc.rs:84
  - tests: crates/server/src/jsonrpc.rs:4957, crates/server/src/jsonrpc.rs:4996, crates/server/src/jsonrpc.rs:5032
- `FR-CIV-TEST-001`
  - tests: crates/engine/tests/n_series_coverage.rs:3
- `FR-CIV-TEST-002`
  - tests: crates/civis-mcp/tests/mcp_integration.rs:1
- `FR-CIV-TEST-006`
  - tests: crates/economy/tests/economy_coverage.rs:1, crates/economy/tests/economy_coverage.rs:18, crates/economy/tests/economy_coverage.rs:51
- `FR-CIV-TEST-007`
  - tests: crates/server/tests/server_coverage.rs:1
- `FR-CIV-TEST-008`
  - tests: crates/civ-emergence-metrics/tests/emergence_coverage.rs:2, crates/civis-mcp/tests/mcp_coverage.rs:1
- `FR-CIV-TEST-009`
  - tests: crates/protocol-3d/tests/protocol_coverage.rs:1
- `FR-CIV-TEST-021`
  - tests: crates/server/tests/save_load_e2e.rs:1
- `FR-CIV-UNREST-002`
  - code: crates/engine/src/engine.rs:504, crates/engine/src/engine.rs:526, crates/engine/src/engine.rs:2200
  - tests: crates/engine/tests/riot_migrant_taxation_persistence.rs:2
- `FR-CIV-WARFARE-001`
  - code: crates/tactics/src/war_from_diplomacy.rs:1
- `FR-CIV-WARFARE-002`
  - code: crates/tactics/src/doctrine_evolution.rs:1
- `FR-CIV-WARFARE-003`
  - code: crates/tactics/src/war_economy.rs:1
- `FR-CIV-WARFARE-004`
  - code: crates/tactics/src/war_legends.rs:1
- `FR-DIP-002`
  - code: crates/diplomacy/src/effects.rs:1
- `FR-ECON-EMERGE-001`
  - code: crates/economy/src/prices.rs:1
- `FR-ECON-EMERGE-002`
  - code: crates/economy/src/trade.rs:1
- `FR-ECON-EMERGE-004`
  - code: crates/economy/src/shocks.rs:1
- `FR-EMG-001`
  - code: crates/emergence-oracle/src/lib.rs:15, crates/emergence-oracle/src/oracles/religion.rs:1, crates/emergence-oracle/src/oracles/religion.rs:17
  - tests: crates/engine/tests/fr_emg_oracle.rs:7, crates/engine/tests/fr_emg_oracle.rs:40, crates/engine/tests/fr_emg_oracle.rs:208
- `FR-EMG-002`
  - code: crates/emergence-oracle/src/oracles/language.rs:1, crates/emergence-oracle/src/oracles/language.rs:19
  - tests: crates/engine/tests/fr_emg_oracle.rs:8, crates/engine/tests/fr_emg_oracle.rs:53, crates/engine/tests/fr_emg_oracle.rs:211
- `FR-EMG-003`
  - code: crates/emergence-oracle/src/oracles/economy.rs:1, crates/emergence-oracle/src/oracles/economy.rs:17
  - tests: crates/engine/tests/fr_emg_oracle.rs:9, crates/engine/tests/fr_emg_oracle.rs:80, crates/engine/tests/fr_emg_oracle.rs:213
- `FR-EMG-004`
  - code: crates/emergence-oracle/src/oracles/legends.rs:1, crates/emergence-oracle/src/oracles/legends.rs:18
  - tests: crates/engine/tests/fr_emg_oracle.rs:10, crates/engine/tests/fr_emg_oracle.rs:92, crates/engine/tests/fr_emg_oracle.rs:216
- `FR-EMG-005`
  - code: crates/emergence-oracle/src/bin/oracle_report.rs:17, crates/emergence-oracle/src/oracles/diplomacy.rs:1, crates/emergence-oracle/src/oracles/diplomacy.rs:19
  - tests: crates/engine/tests/fr_emg_oracle.rs:11, crates/engine/tests/fr_emg_oracle.rs:26, crates/engine/tests/fr_emg_oracle.rs:103
- `FR-EMG-006`
  - code: crates/emergence-oracle/src/oracles/psyche.rs:1, crates/emergence-oracle/src/oracles/psyche.rs:19
  - tests: crates/engine/tests/fr_emg_oracle.rs:12, crates/engine/tests/fr_emg_oracle.rs:132, crates/engine/tests/fr_emg_oracle.rs:222
- `FR-EMG-007`
  - code: crates/emergence-oracle/src/lib.rs:158, crates/emergence-oracle/src/lib.rs:159, crates/emergence-oracle/src/oracles/architecture.rs:1
  - tests: crates/emergence-oracle/src/lib.rs:158, crates/emergence-oracle/src/lib.rs:159, crates/engine/tests/fr_emg_oracle.rs:13
- `FR-EMG-008`
  - code: crates/emergence-oracle/src/bin/oracle_report.rs:18, crates/emergence-oracle/src/oracles/creature.rs:1, crates/emergence-oracle/src/oracles/creature.rs:24
  - tests: crates/engine/tests/fr_emg_oracle.rs:14, crates/engine/tests/fr_emg_oracle.rs:26, crates/engine/tests/fr_emg_oracle.rs:175
- `FR-EMG-009`
  - code: crates/emergence-oracle/src/oracles/migration.rs:1, crates/emergence-oracle/src/oracles/migration.rs:17
- `FR-EMG-010`
  - code: crates/emergence-oracle/src/oracles/epidemic.rs:1, crates/emergence-oracle/src/oracles/epidemic.rs:17, crates/emergence-oracle/src/oracles/trade.rs:1
- `FR-EMG-012`
  - code: crates/emergence-oracle/src/oracles/festival.rs:1, crates/emergence-oracle/src/oracles/festival.rs:18
- `FR-EMG-013`
  - code: crates/emergence-oracle/src/oracles/disaster.rs:1, crates/emergence-oracle/src/oracles/disaster.rs:17
- `FR-EMG-014`
  - code: crates/emergence-oracle/src/oracles/mood.rs:1, crates/emergence-oracle/src/oracles/mood.rs:17
- `FR-EMG-015`
  - code: crates/emergence-oracle/src/oracles/stratification.rs:1, crates/emergence-oracle/src/oracles/stratification.rs:18
- `FR-EMG-016`
  - code: crates/emergence-oracle/src/oracles/religious_conflict.rs:1, crates/emergence-oracle/src/oracles/religious_conflict.rs:17
- `FR-EMG-017`
  - code: crates/emergence-oracle/src/oracles/expansion.rs:1, crates/emergence-oracle/src/oracles/expansion.rs:17
- `FR-EMG-018`
  - code: crates/emergence-oracle/src/oracles/migration_flow.rs:1, crates/emergence-oracle/src/oracles/migration_flow.rs:17
- `FR-EMG-019`
  - code: crates/emergence-oracle/src/oracles/coastal_settlement.rs:1, crates/emergence-oracle/src/oracles/coastal_settlement.rs:17
- `FR-EMG-020`
  - code: crates/emergence-oracle/src/oracles/river_trade.rs:1, crates/emergence-oracle/src/oracles/river_trade.rs:17
- `FR-EMG-021`
  - code: crates/emergence-oracle/src/oracles/mountain_pass.rs:1, crates/emergence-oracle/src/oracles/mountain_pass.rs:17
- `FR-EMG-022`
  - code: crates/emergence-oracle/src/oracles/desert_caravan.rs:1, crates/emergence-oracle/src/oracles/desert_caravan.rs:17
- `FR-EMG-023`
  - code: crates/emergence-oracle/src/oracles/genetics.rs:1, crates/emergence-oracle/src/oracles/genetics.rs:60
- `FR-EMG-024`
  - code: crates/emergence-oracle/src/oracles/i18n.rs:1, crates/emergence-oracle/src/oracles/i18n.rs:46, crates/emergence-oracle/src/oracles/powers.rs:1
- `FR-EMG-025`
  - code: crates/emergence-oracle/src/oracles/migration_pressure.rs:1, crates/emergence-oracle/src/oracles/migration_pressure.rs:16, crates/emergence-oracle/src/oracles/migration_pressure.rs:54
  - tests: crates/emergence-oracle/src/oracles/migration_pressure.rs:54
- `FR-FR-CORE-009`
  - tests: crates/engine/tests/fr_core_cluster.rs:1, crates/engine/tests/fr_core_cluster.rs:15, crates/engine/tests/fr_core_cluster.rs:184
- `FR-LANGUAGE-001`
  - code: crates/engine/src/engine/culture_phases.rs:230, crates/engine/src/engine.rs:169, crates/engine/src/engine.rs:835
- `FR-MUSIC-001`
  - code: crates/engine/src/engine/engine_tests.rs:3732
  - tests: crates/engine/src/engine/engine_tests.rs:3732
- `FR-NFR-C-01`
  - tests: crates/engine/tests/fr_nfr_c_01.rs:1, crates/engine/tests/fr_nfr_c_01.rs:6, crates/engine/tests/fr_nfr_c_01.rs:10
- `FR-NFR-C-02`
  - tests: crates/engine/tests/fr_nfr_c_02.rs:1, crates/engine/tests/fr_nfr_c_02.rs:6, crates/engine/tests/fr_nfr_c_02.rs:10
- `FR-NFR-C-03`
  - tests: crates/engine/tests/fr_nfr_c_03.rs:1, crates/engine/tests/fr_nfr_c_03.rs:6, crates/engine/tests/fr_nfr_c_03.rs:10
- `FR-NFR-C-04`
  - tests: crates/engine/tests/fr_nfr_c_04.rs:1, crates/engine/tests/fr_nfr_c_04.rs:6, crates/engine/tests/fr_nfr_c_04.rs:10
- `FR-NFR-C-05`
  - tests: crates/engine/tests/fr_nfr_c_05.rs:1, crates/engine/tests/fr_nfr_c_05.rs:6, crates/engine/tests/fr_nfr_c_05.rs:10
- `FR-NFR-C-06`
  - tests: crates/engine/tests/fr_nfr_c_06.rs:1, crates/engine/tests/fr_nfr_c_06.rs:6, crates/engine/tests/fr_nfr_c_06.rs:10
- `FR-NFR-C-07`
  - tests: crates/engine/tests/fr_nfr_c_07.rs:1, crates/engine/tests/fr_nfr_c_07.rs:6, crates/engine/tests/fr_nfr_c_07.rs:10
- `FR-NFR-CIV-ACC-001`
  - tests: crates/engine/tests/fr_nfr_civ_acc_001.rs:1, crates/engine/tests/fr_nfr_civ_acc_001.rs:6, crates/engine/tests/fr_nfr_civ_acc_001.rs:10
- `FR-NFR-CIV-ACC-002`
  - tests: crates/engine/tests/fr_nfr_civ_acc_002.rs:1, crates/engine/tests/fr_nfr_civ_acc_002.rs:6, crates/engine/tests/fr_nfr_civ_acc_002.rs:10
- `FR-NFR-CIV-ACC-003`
  - tests: crates/engine/tests/fr_nfr_civ_acc_003.rs:1, crates/engine/tests/fr_nfr_civ_acc_003.rs:6, crates/engine/tests/fr_nfr_civ_acc_003.rs:10
- `FR-NFR-CIV-ACC-004`
  - tests: crates/engine/tests/fr_nfr_civ_acc_004.rs:1, crates/engine/tests/fr_nfr_civ_acc_004.rs:6, crates/engine/tests/fr_nfr_civ_acc_004.rs:10
- `FR-NFR-CIV-AI-001`
  - tests: crates/engine/tests/fr_nfr_civ_ai_001.rs:1, crates/engine/tests/fr_nfr_civ_ai_001.rs:6, crates/engine/tests/fr_nfr_civ_ai_001.rs:10
- `FR-NFR-CIV-AI-002`
  - tests: crates/engine/tests/fr_nfr_civ_ai_002.rs:1, crates/engine/tests/fr_nfr_civ_ai_002.rs:6, crates/engine/tests/fr_nfr_civ_ai_002.rs:10
- `FR-NFR-CIV-AI-003`
  - tests: crates/engine/tests/fr_nfr_civ_ai_003.rs:1, crates/engine/tests/fr_nfr_civ_ai_003.rs:6, crates/engine/tests/fr_nfr_civ_ai_003.rs:10
- `FR-NFR-CIV-DET-001`
  - tests: crates/engine/tests/fr_nfr_civ_det_001.rs:1, crates/engine/tests/fr_nfr_civ_det_001.rs:6, crates/engine/tests/fr_nfr_civ_det_001.rs:10
- `FR-NFR-CIV-DET-002`
  - tests: crates/engine/tests/fr_nfr_civ_det_002.rs:1, crates/engine/tests/fr_nfr_civ_det_002.rs:6, crates/engine/tests/fr_nfr_civ_det_002.rs:10
- `FR-NFR-CIV-DEV-HYGIENE-001`
  - tests: crates/engine/tests/fr_nfr_civ_dev_hygiene_001.rs:1, crates/engine/tests/fr_nfr_civ_dev_hygiene_001.rs:6, crates/engine/tests/fr_nfr_civ_dev_hygiene_001.rs:10
- `FR-NFR-CIV-LEGENDS-CONFIG-04`
  - tests: crates/engine/tests/fr_nfr_civ_legends_config_04.rs:1, crates/engine/tests/fr_nfr_civ_legends_config_04.rs:6, crates/engine/tests/fr_nfr_civ_legends_config_04.rs:10
- `FR-NFR-CIV-LEGENDS-LOUD-03`
  - tests: crates/engine/tests/fr_nfr_civ_legends_loud_03.rs:1, crates/engine/tests/fr_nfr_civ_legends_loud_03.rs:6, crates/engine/tests/fr_nfr_civ_legends_loud_03.rs:10
- `FR-NFR-CIV-LEGENDS-PERF-01`
  - tests: crates/engine/tests/fr_nfr_civ_legends_perf_01.rs:1, crates/engine/tests/fr_nfr_civ_legends_perf_01.rs:6, crates/engine/tests/fr_nfr_civ_legends_perf_01.rs:10
- `FR-NFR-CIV-LEGENDS-SCALE-02`
  - tests: crates/engine/tests/fr_nfr_civ_legends_scale_02.rs:1, crates/engine/tests/fr_nfr_civ_legends_scale_02.rs:6, crates/engine/tests/fr_nfr_civ_legends_scale_02.rs:10
- `FR-NFR-CIV-MAINT-001`
  - tests: crates/engine/tests/fr_nfr_civ_maint_001.rs:1, crates/engine/tests/fr_nfr_civ_maint_001.rs:6, crates/engine/tests/fr_nfr_civ_maint_001.rs:10
- `FR-NFR-CIV-MAINT-002`
  - tests: crates/engine/tests/fr_nfr_civ_maint_002.rs:1, crates/engine/tests/fr_nfr_civ_maint_002.rs:6, crates/engine/tests/fr_nfr_civ_maint_002.rs:10
- `FR-NFR-CIV-MAINT-003`
  - tests: crates/engine/tests/fr_nfr_civ_maint_003.rs:1, crates/engine/tests/fr_nfr_civ_maint_003.rs:6, crates/engine/tests/fr_nfr_civ_maint_003.rs:10
- `FR-NFR-CIV-MAINT-004`
  - tests: crates/engine/tests/fr_nfr_civ_maint_004.rs:1, crates/engine/tests/fr_nfr_civ_maint_004.rs:6, crates/engine/tests/fr_nfr_civ_maint_004.rs:10
- `FR-NFR-CIV-MAINT-005`
  - tests: crates/engine/tests/fr_nfr_civ_maint_005.rs:1, crates/engine/tests/fr_nfr_civ_maint_005.rs:6, crates/engine/tests/fr_nfr_civ_maint_005.rs:10
- `FR-NFR-CIV-MAINT-006`
  - tests: crates/engine/tests/fr_nfr_civ_maint_006.rs:1, crates/engine/tests/fr_nfr_civ_maint_006.rs:6, crates/engine/tests/fr_nfr_civ_maint_006.rs:10
- `FR-NFR-CIV-PERF-001`
  - tests: crates/engine/tests/fr_nfr_civ_perf_001.rs:1, crates/engine/tests/fr_nfr_civ_perf_001.rs:6, crates/engine/tests/fr_nfr_civ_perf_001.rs:10
- `FR-NFR-CIV-PERF-002`
  - tests: crates/engine/tests/fr_nfr_civ_perf_002.rs:1, crates/engine/tests/fr_nfr_civ_perf_002.rs:6, crates/engine/tests/fr_nfr_civ_perf_002.rs:10
- `FR-NFR-CIV-PERF-003`
  - tests: crates/engine/tests/fr_nfr_civ_perf_003.rs:1, crates/engine/tests/fr_nfr_civ_perf_003.rs:6, crates/engine/tests/fr_nfr_civ_perf_003.rs:10
- `FR-NFR-CIV-PERF-004`
  - tests: crates/engine/tests/fr_nfr_civ_perf_004.rs:1, crates/engine/tests/fr_nfr_civ_perf_004.rs:6, crates/engine/tests/fr_nfr_civ_perf_004.rs:10
- `FR-NFR-CIV-PERF-005`
  - tests: crates/engine/tests/fr_nfr_civ_perf_005.rs:1, crates/engine/tests/fr_nfr_civ_perf_005.rs:6, crates/engine/tests/fr_nfr_civ_perf_005.rs:10
- `FR-NFR-CIV-PERF-006`
  - tests: crates/engine/tests/fr_nfr_civ_perf_006.rs:1, crates/engine/tests/fr_nfr_civ_perf_006.rs:6, crates/engine/tests/fr_nfr_civ_perf_006.rs:10
- `FR-NFR-CIV-PERF-007`
  - tests: crates/engine/tests/fr_nfr_civ_perf_007.rs:1, crates/engine/tests/fr_nfr_civ_perf_007.rs:6, crates/engine/tests/fr_nfr_civ_perf_007.rs:10
- `FR-NFR-CIV-PERF-008`
  - tests: crates/engine/tests/fr_nfr_civ_perf_008.rs:1, crates/engine/tests/fr_nfr_civ_perf_008.rs:6, crates/engine/tests/fr_nfr_civ_perf_008.rs:10
- `FR-NFR-CIV-PERF-900`
  - tests: crates/engine/tests/fr_nfr_civ_perf_900.rs:1, crates/engine/tests/fr_nfr_civ_perf_900.rs:6, crates/engine/tests/fr_nfr_civ_perf_900.rs:10
- `FR-NFR-CIV-PERF-901`
  - tests: crates/engine/tests/fr_nfr_civ_perf_901.rs:1, crates/engine/tests/fr_nfr_civ_perf_901.rs:6, crates/engine/tests/fr_nfr_civ_perf_901.rs:10
- `FR-NFR-CIV-PERF-902`
  - tests: crates/engine/tests/fr_nfr_civ_perf_902.rs:1, crates/engine/tests/fr_nfr_civ_perf_902.rs:6, crates/engine/tests/fr_nfr_civ_perf_902.rs:10
- `FR-NFR-CIV-PORT-001`
  - tests: crates/engine/tests/fr_nfr_civ_port_001.rs:1, crates/engine/tests/fr_nfr_civ_port_001.rs:6, crates/engine/tests/fr_nfr_civ_port_001.rs:10
- `FR-NFR-CIV-PORT-002`
  - tests: crates/engine/tests/fr_nfr_civ_port_002.rs:1, crates/engine/tests/fr_nfr_civ_port_002.rs:6, crates/engine/tests/fr_nfr_civ_port_002.rs:10
- `FR-NFR-CIV-PORT-003`
  - tests: crates/engine/tests/fr_nfr_civ_port_003.rs:1, crates/engine/tests/fr_nfr_civ_port_003.rs:6, crates/engine/tests/fr_nfr_civ_port_003.rs:10
- `FR-NFR-CIV-REL-001`
  - tests: crates/engine/tests/fr_nfr_civ_rel_001.rs:1, crates/engine/tests/fr_nfr_civ_rel_001.rs:6, crates/engine/tests/fr_nfr_civ_rel_001.rs:10
- `FR-NFR-CIV-REL-002`
  - tests: crates/engine/tests/fr_nfr_civ_rel_002.rs:1, crates/engine/tests/fr_nfr_civ_rel_002.rs:6, crates/engine/tests/fr_nfr_civ_rel_002.rs:10
- `FR-NFR-CIV-REL-003`
  - tests: crates/engine/tests/fr_nfr_civ_rel_003.rs:1, crates/engine/tests/fr_nfr_civ_rel_003.rs:6, crates/engine/tests/fr_nfr_civ_rel_003.rs:10
- `FR-NFR-CIV-SCALE-001`
  - tests: crates/engine/tests/fr_nfr_civ_scale_001.rs:1, crates/engine/tests/fr_nfr_civ_scale_001.rs:6, crates/engine/tests/fr_nfr_civ_scale_001.rs:10
- `FR-NFR-CIV-SCALE-002`
  - tests: crates/engine/tests/fr_nfr_civ_scale_002.rs:1, crates/engine/tests/fr_nfr_civ_scale_002.rs:6, crates/engine/tests/fr_nfr_civ_scale_002.rs:10
- `FR-NFR-CIV-SCALE-003`
  - tests: crates/engine/tests/fr_nfr_civ_scale_003.rs:1, crates/engine/tests/fr_nfr_civ_scale_003.rs:6, crates/engine/tests/fr_nfr_civ_scale_003.rs:10
- `FR-NFR-CIV-SCALE-004`
  - tests: crates/engine/tests/fr_nfr_civ_scale_004.rs:1, crates/engine/tests/fr_nfr_civ_scale_004.rs:6, crates/engine/tests/fr_nfr_civ_scale_004.rs:10
- `FR-NFR-CIV-SCALE-900`
  - tests: crates/engine/tests/fr_nfr_civ_scale_900.rs:1, crates/engine/tests/fr_nfr_civ_scale_900.rs:6, crates/engine/tests/fr_nfr_civ_scale_900.rs:10
- `FR-NFR-CIV-SCALE-901`
  - tests: crates/engine/tests/fr_nfr_civ_scale_901.rs:1, crates/engine/tests/fr_nfr_civ_scale_901.rs:6, crates/engine/tests/fr_nfr_civ_scale_901.rs:10
- `FR-NFR-CIV-SCALE-902`
  - tests: crates/engine/tests/fr_nfr_civ_scale_902.rs:1, crates/engine/tests/fr_nfr_civ_scale_902.rs:6, crates/engine/tests/fr_nfr_civ_scale_902.rs:10
- `FR-NFR-CIV-SCALE-910`
  - tests: crates/engine/tests/fr_nfr_civ_scale_910.rs:1, crates/engine/tests/fr_nfr_civ_scale_910.rs:6, crates/engine/tests/fr_nfr_civ_scale_910.rs:10
- `FR-NFR-CIV-SCALE-920`
  - tests: crates/engine/tests/fr_nfr_civ_scale_920.rs:1, crates/engine/tests/fr_nfr_civ_scale_920.rs:6, crates/engine/tests/fr_nfr_civ_scale_920.rs:10
- `FR-NFR-CIV-SEC-001`
  - tests: crates/engine/tests/fr_nfr_civ_sec_001.rs:1, crates/engine/tests/fr_nfr_civ_sec_001.rs:6, crates/engine/tests/fr_nfr_civ_sec_001.rs:10
- `FR-NFR-CIV-SEC-002`
  - tests: crates/engine/tests/fr_nfr_civ_sec_002.rs:1, crates/engine/tests/fr_nfr_civ_sec_002.rs:6, crates/engine/tests/fr_nfr_civ_sec_002.rs:10
- `FR-NFR-CIV-SEC-003`
  - tests: crates/engine/tests/fr_nfr_civ_sec_003.rs:1, crates/engine/tests/fr_nfr_civ_sec_003.rs:6, crates/engine/tests/fr_nfr_civ_sec_003.rs:10
- `FR-NFR-CIV-SEC-004`
  - tests: crates/engine/tests/fr_nfr_civ_sec_004.rs:1, crates/engine/tests/fr_nfr_civ_sec_004.rs:6, crates/engine/tests/fr_nfr_civ_sec_004.rs:10
- `FR-NFR-O-01`
  - tests: crates/engine/tests/fr_nfr_o_01.rs:1, crates/engine/tests/fr_nfr_o_01.rs:6, crates/engine/tests/fr_nfr_o_01.rs:10
- `FR-NFR-O-02`
  - tests: crates/engine/tests/fr_nfr_o_02.rs:1, crates/engine/tests/fr_nfr_o_02.rs:6, crates/engine/tests/fr_nfr_o_02.rs:10
- `FR-NFR-O-03`
  - tests: crates/engine/tests/fr_nfr_o_03.rs:1, crates/engine/tests/fr_nfr_o_03.rs:6, crates/engine/tests/fr_nfr_o_03.rs:10
- `FR-NFR-O-04`
  - tests: crates/engine/tests/fr_nfr_o_04.rs:1, crates/engine/tests/fr_nfr_o_04.rs:6, crates/engine/tests/fr_nfr_o_04.rs:10
- `FR-NFR-O-05`
  - tests: crates/engine/tests/fr_nfr_o_05.rs:1, crates/engine/tests/fr_nfr_o_05.rs:6, crates/engine/tests/fr_nfr_o_05.rs:10
- `FR-NFR-O-06`
  - tests: crates/engine/tests/fr_nfr_o_06.rs:1, crates/engine/tests/fr_nfr_o_06.rs:6, crates/engine/tests/fr_nfr_o_06.rs:10
- `FR-NFR-P-01`
  - tests: crates/engine/tests/fr_nfr_p_01.rs:1, crates/engine/tests/fr_nfr_p_01.rs:6, crates/engine/tests/fr_nfr_p_01.rs:10
- `FR-NFR-P-02`
  - tests: crates/engine/tests/fr_nfr_p_02.rs:1, crates/engine/tests/fr_nfr_p_02.rs:6, crates/engine/tests/fr_nfr_p_02.rs:10
- `FR-NFR-P-03`
  - tests: crates/engine/tests/fr_nfr_p_03.rs:1, crates/engine/tests/fr_nfr_p_03.rs:6, crates/engine/tests/fr_nfr_p_03.rs:10
- `FR-NFR-P-04`
  - tests: crates/engine/tests/fr_nfr_p_04.rs:1, crates/engine/tests/fr_nfr_p_04.rs:6, crates/engine/tests/fr_nfr_p_04.rs:10
- `FR-NFR-P-05`
  - tests: crates/engine/tests/fr_nfr_p_05.rs:1, crates/engine/tests/fr_nfr_p_05.rs:6, crates/engine/tests/fr_nfr_p_05.rs:10
- `FR-NFR-P-06`
  - tests: crates/engine/tests/fr_nfr_p_06.rs:1, crates/engine/tests/fr_nfr_p_06.rs:6, crates/engine/tests/fr_nfr_p_06.rs:10
- `FR-NFR-P-07`
  - tests: crates/engine/tests/fr_nfr_p_07.rs:1, crates/engine/tests/fr_nfr_p_07.rs:6, crates/engine/tests/fr_nfr_p_07.rs:10
- `FR-NFR-P-08`
  - tests: crates/engine/tests/fr_nfr_p_08.rs:1, crates/engine/tests/fr_nfr_p_08.rs:6, crates/engine/tests/fr_nfr_p_08.rs:10
- `FR-NFR-R-01`
  - tests: crates/engine/tests/fr_nfr_r_01.rs:1, crates/engine/tests/fr_nfr_r_01.rs:6, crates/engine/tests/fr_nfr_r_01.rs:10
- `FR-NFR-R-02`
  - tests: crates/engine/tests/fr_nfr_r_02.rs:1, crates/engine/tests/fr_nfr_r_02.rs:6, crates/engine/tests/fr_nfr_r_02.rs:10
- `FR-NFR-R-03`
  - tests: crates/engine/tests/fr_nfr_r_03.rs:1, crates/engine/tests/fr_nfr_r_03.rs:6, crates/engine/tests/fr_nfr_r_03.rs:10
- `FR-NFR-R-04`
  - tests: crates/engine/tests/fr_nfr_r_04.rs:1, crates/engine/tests/fr_nfr_r_04.rs:6, crates/engine/tests/fr_nfr_r_04.rs:10
- `FR-NFR-R-05`
  - tests: crates/engine/tests/fr_nfr_r_05.rs:1, crates/engine/tests/fr_nfr_r_05.rs:6, crates/engine/tests/fr_nfr_r_05.rs:10
- `FR-NFR-R-06`
  - tests: crates/engine/tests/fr_nfr_r_06.rs:1, crates/engine/tests/fr_nfr_r_06.rs:6, crates/engine/tests/fr_nfr_r_06.rs:10
- `FR-NFR-S-01`
  - tests: crates/engine/tests/fr_nfr_s_01.rs:1, crates/engine/tests/fr_nfr_s_01.rs:6, crates/engine/tests/fr_nfr_s_01.rs:10
- `FR-NFR-S-02`
  - tests: crates/engine/tests/fr_nfr_s_02.rs:1, crates/engine/tests/fr_nfr_s_02.rs:6, crates/engine/tests/fr_nfr_s_02.rs:10
- `FR-NFR-S-03`
  - tests: crates/engine/tests/fr_nfr_s_03.rs:1, crates/engine/tests/fr_nfr_s_03.rs:6, crates/engine/tests/fr_nfr_s_03.rs:10
- `FR-NFR-S-04`
  - tests: crates/engine/tests/fr_nfr_s_04.rs:1, crates/engine/tests/fr_nfr_s_04.rs:6, crates/engine/tests/fr_nfr_s_04.rs:10
- `FR-NFR-S-05`
  - tests: crates/engine/tests/fr_nfr_s_05.rs:1, crates/engine/tests/fr_nfr_s_05.rs:6, crates/engine/tests/fr_nfr_s_05.rs:10
- `FR-NFR-S-06`
  - tests: crates/engine/tests/fr_nfr_s_06.rs:1, crates/engine/tests/fr_nfr_s_06.rs:6, crates/engine/tests/fr_nfr_s_06.rs:10
- `FR-NFR-SCALE-02`
  - tests: crates/engine/tests/fr_nfr_scale_02.rs:1, crates/engine/tests/fr_nfr_scale_02.rs:6, crates/engine/tests/fr_nfr_scale_02.rs:10
- `FR-VIEWPORT-001`
  - code: crates/civis-cli/src/bin/three_d_quality.rs:54
- `NFR-CIV-SCALE-PERF-900`
  - code: crates/voxel/src/scale_stream.rs:1, crates/voxel/src/scale_stream.rs:16, crates/voxel/src/scale_stream.rs:58
  - tests: crates/voxel/src/scale_stream.rs:427, crates/voxel/src/scale_stream.rs:479, crates/voxel/src/scale_stream.rs:509

## Placeholder-only coverage (weakest evidence) (0)

These IDs are counted `COVERED` on tests whose file matches the auto-generated placeholder pattern above. Their tests assert properties of shared types, not the requirement, so treat the coverage as unverified until a real oracle exists.

_None._

