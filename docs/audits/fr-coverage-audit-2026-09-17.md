# FR Coverage Audit

**Generated:** 2026-09-17  
**Source inventory:** `docs/audits/_id_inventory_v3.json`  
**Total IDs scanned:** 1506

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
| `COVERED` | 754 | 50.1 |
| `IMPL-NO-TEST` | 269 | 17.9 |
| `SPEC-ONLY` | 262 | 17.4 |
| `CODE-ONLY-no-spec` | 221 | 14.7 |
| **Total** | **1506** | **100.0** |

## Coverage by epic

| Epic | Total | COVERED | IMPL-NO-TEST | SPEC-ONLY | CODE-ONLY-no-spec |
|------|------:|--------:|-------------:|----------:|------------------:|
| FR-AI | 7 | 7 | 0 | 0 | 0 |
| FR-API | 4 | 4 | 0 | 0 | 0 |
| FR-ASSET | 4 | 4 | 0 | 0 | 0 |
| FR-ASSET-PIPELINE | 2 | 0 | 0 | 0 | 2 |
| FR-AUD | 3 | 3 | 0 | 0 | 0 |
| FR-CIV | 16 | 12 | 0 | 1 | 3 |
| FR-CIV-0001-TICK | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-3D | 16 | 15 | 0 | 0 | 1 |
| FR-CIV-ACCESS | 2 | 0 | 0 | 0 | 2 |
| FR-CIV-ACT | 4 | 4 | 0 | 0 | 0 |
| FR-CIV-ACTOR | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-ACTOR-001-LIFECYCLE | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-AGENTS | 17 | 3 | 0 | 14 | 0 |
| FR-CIV-AGGRESSION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-AI | 15 | 15 | 0 | 0 | 0 |
| FR-CIV-ARCH | 8 | 4 | 0 | 4 | 0 |
| FR-CIV-ARCH-A | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-ARCH-B | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-ARCH-C | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-ARCH-D | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-ARCH-NOSVG | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-ASSET | 20 | 0 | 20 | 0 | 0 |
| FR-CIV-ASSET-MANI | 2 | 0 | 2 | 0 | 0 |
| FR-CIV-ASSET-QUAL | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-AUDIO | 12 | 8 | 4 | 0 | 0 |
| FR-CIV-BELIEF | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-BEVY | 21 | 8 | 8 | 1 | 4 |
| FR-CIV-BIO | 3 | 3 | 0 | 0 | 0 |
| FR-CIV-BRUSH | 13 | 13 | 0 | 0 | 0 |
| FR-CIV-BUILD | 15 | 7 | 0 | 8 | 0 |
| FR-CIV-CA | 11 | 7 | 0 | 3 | 1 |
| FR-CIV-CARAVAN | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CLIENT | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-CLIENT-GODOT | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-CLIMATE | 7 | 3 | 0 | 0 | 4 |
| FR-CIV-COHESION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CONSTRUCTION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CONTENT | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-CORE | 21 | 20 | 0 | 0 | 1 |
| FR-CIV-CORE-DET | 3 | 3 | 0 | 0 | 0 |
| FR-CIV-CULT | 3 | 3 | 0 | 0 | 0 |
| FR-CIV-CULTURE | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-DET | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-DIFFUSION | 16 | 3 | 0 | 13 | 0 |
| FR-CIV-DIPLO | 16 | 7 | 0 | 1 | 8 |
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
| FR-CIV-EMERGENCE-RELIGION | 2 | 0 | 0 | 0 | 2 |
| FR-CIV-EMERGENT-MIGRATION | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-ENGINE-INT | 10 | 0 | 10 | 0 | 0 |
| FR-CIV-ENGINE-REPLAY | 5 | 0 | 5 | 0 | 0 |
| FR-CIV-ERA | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-FAMINE | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-FEST | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-FOG | 5 | 0 | 0 | 5 | 0 |
| FR-CIV-GAME | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-GENETICS | 6 | 4 | 0 | 2 | 0 |
| FR-CIV-GENETICS-SEED | 3 | 0 | 0 | 0 | 3 |
| FR-CIV-GEO | 10 | 0 | 10 | 0 | 0 |
| FR-CIV-GODOT-ATTACH | 5 | 1 | 4 | 0 | 0 |
| FR-CIV-GODOT-F3D0 | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-GODOT-UX | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-GODTOOL | 8 | 7 | 0 | 0 | 1 |
| FR-CIV-GOV | 8 | 2 | 0 | 0 | 6 |
| FR-CIV-HUD | 5 | 5 | 0 | 0 | 0 |
| FR-CIV-IDEOLOGY | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-INFOVIEW | 20 | 20 | 0 | 0 | 0 |
| FR-CIV-INFRA | 13 | 5 | 0 | 8 | 0 |
| FR-CIV-INSPECT | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-INSTITUTIONS | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-INT | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-L10N | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-L5 | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LANG | 10 | 5 | 0 | 5 | 0 |
| FR-CIV-LAWS | 10 | 3 | 0 | 7 | 0 |
| FR-CIV-LEGENDS | 9 | 8 | 0 | 0 | 1 |
| FR-CIV-LEGENDS-BROWSER | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-CAUSAL | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-CONFIG | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-GAP | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-GRAPH | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-INGEST | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-INSPECT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-NARRATOR | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PERF | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-PERSIST | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PRESIM | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-PRODUCER | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-QUERY | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-RESOLVE | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LEGENDS-SCALE | 1 | 0 | 0 | 1 | 0 |
| FR-CIV-LEGENDS-SIG | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-LIFE | 20 | 6 | 1 | 12 | 1 |
| FR-CIV-LLM | 6 | 0 | 0 | 6 | 0 |
| FR-CIV-MARKET | 8 | 8 | 0 | 0 | 0 |
| FR-CIV-MCP | 6 | 2 | 0 | 4 | 0 |
| FR-CIV-METRICS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-METRICS-001-TIMESERIES | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-MIGRATION | 5 | 0 | 0 | 5 | 0 |
| FR-CIV-MOD | 21 | 21 | 0 | 0 | 0 |
| FR-CIV-NEEDS-DECAY | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-NOTIFY | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-PBR | 11 | 8 | 0 | 0 | 3 |
| FR-CIV-PERF | 20 | 20 | 0 | 0 | 0 |
| FR-CIV-PERF-BUILD | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-PERF-RT | 3 | 3 | 0 | 0 | 0 |
| FR-CIV-PERF-WEB | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-PLANET | 12 | 5 | 3 | 3 | 1 |
| FR-CIV-POLITY | 8 | 8 | 0 | 0 | 0 |
| FR-CIV-PROTO | 15 | 15 | 0 | 0 | 0 |
| FR-CIV-PROTO3D | 20 | 5 | 1 | 14 | 0 |
| FR-CIV-PSYCHE | 29 | 10 | 16 | 3 | 0 |
| FR-CIV-PSYCHE-N11 | 1 | 0 | 0 | 0 | 1 |
| FR-CIV-QOL | 14 | 14 | 0 | 0 | 0 |
| FR-CIV-REL | 5 | 0 | 3 | 1 | 1 |
| FR-CIV-RELIGION | 2 | 0 | 2 | 0 | 0 |
| FR-CIV-RENDER | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-RES | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH | 13 | 5 | 0 | 8 | 0 |
| FR-CIV-RESEARCH-001-SCENARIO | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-002-SNAPSHOT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-003-EXPORT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-RESEARCH-004-REPLAY | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-ROAD | 6 | 6 | 0 | 0 | 0 |
| FR-CIV-RTS | 15 | 15 | 0 | 0 | 0 |
| FR-CIV-RTS-NATION | 2 | 2 | 0 | 0 | 0 |
| FR-CIV-RTS-RENDER | 5 | 5 | 0 | 0 | 0 |
| FR-CIV-RTS-ZOOM | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-SAVE | 4 | 2 | 0 | 2 | 0 |
| FR-CIV-SCALE | 8 | 8 | 0 | 0 | 0 |
| FR-CIV-SERVER | 3 | 2 | 0 | 0 | 1 |
| FR-CIV-SERVER-001-WS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-SERVER-002-PROTO | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-SOCIAL | 2 | 0 | 2 | 0 | 0 |
| FR-CIV-SOCIAL-001-INSTITUTIONS | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-SOCIAL-002-IDEOLOGY | 1 | 0 | 1 | 0 | 0 |
| FR-CIV-SPECIES | 48 | 2 | 30 | 16 | 0 |
| FR-CIV-TACTICS | 65 | 53 | 5 | 7 | 0 |
| FR-CIV-TECH | 21 | 0 | 21 | 0 | 0 |
| FR-CIV-TERRAIN | 6 | 0 | 0 | 6 | 0 |
| FR-CIV-TEST | 7 | 0 | 0 | 0 | 7 |
| FR-CIV-TRAFFIC-LANE | 4 | 3 | 0 | 1 | 0 |
| FR-CIV-UI | 3 | 0 | 3 | 0 | 0 |
| FR-CIV-UNREST | 2 | 0 | 0 | 0 | 2 |
| FR-CIV-UX | 7 | 5 | 2 | 0 | 0 |
| FR-CIV-VEHICLE | 26 | 26 | 0 | 0 | 0 |
| FR-CIV-VERIFY | 10 | 0 | 0 | 10 | 0 |
| FR-CIV-VOXEL | 18 | 16 | 2 | 0 | 0 |
| FR-CIV-VOXEL-DIRTY | 2 | 0 | 0 | 2 | 0 |
| FR-CIV-WAR | 15 | 14 | 1 | 0 | 0 |
| FR-CIV-WAR-001-UNITS | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-WAR-002-COMBAT | 1 | 1 | 0 | 0 | 0 |
| FR-CIV-WARFARE | 4 | 0 | 0 | 0 | 4 |
| FR-CIV-WEB | 9 | 1 | 8 | 0 | 0 |
| FR-CLIENT | 3 | 3 | 0 | 0 | 0 |
| FR-CLIM | 6 | 6 | 0 | 0 | 0 |
| FR-CORE | 10 | 10 | 0 | 0 | 0 |
| FR-DET | 7 | 7 | 0 | 0 | 0 |
| FR-DIP | 1 | 0 | 0 | 0 | 1 |
| FR-DIPL | 7 | 6 | 0 | 1 | 0 |
| FR-DOC | 1 | 0 | 0 | 1 | 0 |
| FR-ECO | 10 | 0 | 0 | 10 | 0 |
| FR-ECON | 10 | 10 | 0 | 0 | 0 |
| FR-ECON-EMERGE | 3 | 0 | 0 | 0 | 3 |
| FR-EMG | 24 | 0 | 0 | 0 | 24 |
| FR-GUARD | 2 | 2 | 0 | 0 | 0 |
| FR-INST | 6 | 6 | 0 | 0 | 0 |
| FR-INT | 1 | 1 | 0 | 0 | 0 |
| FR-LANGUAGE | 1 | 0 | 0 | 0 | 1 |
| FR-LOD | 4 | 3 | 0 | 1 | 0 |
| FR-MET | 1 | 1 | 0 | 0 | 0 |
| FR-METRICS | 5 | 3 | 0 | 2 | 0 |
| FR-MOD | 5 | 2 | 0 | 3 | 0 |
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
| FR-PROT | 6 | 1 | 0 | 5 | 0 |
| FR-PROTO | 5 | 5 | 0 | 0 | 0 |
| FR-REP | 1 | 1 | 0 | 0 | 0 |
| FR-REPLAY | 2 | 2 | 0 | 0 | 0 |
| FR-SAVE | 25 | 6 | 19 | 0 | 0 |
| FR-SESS | 6 | 6 | 0 | 0 | 0 |
| FR-SESSION | 33 | 33 | 0 | 0 | 0 |
| FR-SOC-CIV | 2 | 2 | 0 | 0 | 0 |
| FR-SOC-COH | 4 | 4 | 0 | 0 | 0 |
| FR-SOC-DET | 2 | 2 | 0 | 0 | 0 |
| FR-SOC-FAC | 2 | 2 | 0 | 0 | 0 |
| FR-SOC-HLT | 5 | 5 | 0 | 0 | 0 |
| FR-SOC-IDE | 6 | 6 | 0 | 0 | 0 |
| FR-SOC-INS | 7 | 7 | 0 | 0 | 0 |
| FR-SOC-INT | 4 | 4 | 0 | 0 | 0 |
| FR-SOC-INTG | 7 | 7 | 0 | 0 | 0 |
| FR-SOCI | 6 | 6 | 0 | 0 | 0 |
| FR-STOR | 1 | 1 | 0 | 0 | 0 |
| FR-TEST | 1 | 0 | 0 | 1 | 0 |
| FR-THRY | 4 | 0 | 0 | 4 | 0 |
| FR-UX | 27 | 5 | 22 | 0 | 0 |
| FR-VAL | 1 | 1 | 0 | 0 | 0 |
| FR-VIEWPORT | 1 | 0 | 0 | 0 | 1 |
| NFR-C | 7 | 0 | 7 | 0 | 0 |
| NFR-CIV | 13 | 0 | 0 | 13 | 0 |
| NFR-CIV-ACC | 4 | 0 | 3 | 1 | 0 |
| NFR-CIV-AI | 3 | 0 | 3 | 0 | 0 |
| NFR-CIV-DET | 4 | 3 | 0 | 1 | 0 |
| NFR-CIV-DEV-HYGIENE | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-CONFIG | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-LOUD | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-PERF | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-LEGENDS-SCALE | 1 | 0 | 1 | 0 | 0 |
| NFR-CIV-MAINT | 6 | 0 | 0 | 6 | 0 |
| NFR-CIV-PERF | 11 | 1 | 8 | 2 | 0 |
| NFR-CIV-PORT | 3 | 0 | 0 | 3 | 0 |
| NFR-CIV-REL | 4 | 0 | 0 | 4 | 0 |
| NFR-CIV-SCALE | 9 | 1 | 6 | 2 | 0 |
| NFR-CIV-SCALE-PERF | 1 | 0 | 0 | 0 | 1 |
| NFR-CIV-SEC | 4 | 0 | 0 | 4 | 0 |
| NFR-O | 6 | 0 | 6 | 0 | 0 |
| NFR-P | 8 | 0 | 8 | 0 | 0 |
| NFR-R | 6 | 0 | 6 | 0 | 0 |
| NFR-S | 6 | 0 | 6 | 0 | 0 |
| NFR-SCALE | 1 | 0 | 1 | 0 | 0 |

## Spec-only IDs (need implementation) (262)

- `FR-CIV-0100-`
  - spec: docs/traceability/emergent-systems-tracelinks.md:144, docs/traceability/fr-emergence-matrix.md:20, docs/traceability/fr-emergence-matrix.md:21
- `FR-CIV-AGENTS-002`
  - spec: docs/traceability/fr-civ-agents-002/fr-civ-agents-002-adr.md:1, docs/traceability/fr-civ-agents-002/fr-civ-agents-002-adr.md:6, docs/traceability/fr-civ-agents-002/fr-civ-agents-002-adr.md:11
  - tests: crates/agents/src/lib.rs:922
- `FR-CIV-AGENTS-003`
  - spec: docs/traceability/fr-civ-agents-003/fr-civ-agents-003-adr.md:1, docs/traceability/fr-civ-agents-003/fr-civ-agents-003-adr.md:6, docs/traceability/fr-civ-agents-003/fr-civ-agents-003-adr.md:11
  - tests: crates/agents/src/lib.rs:938
- `FR-CIV-AGENTS-011`
  - spec: docs/traceability/fr-civ-agents-011/fr-civ-agents-011-adr.md:1, docs/traceability/fr-civ-agents-011/fr-civ-agents-011-adr.md:6, docs/traceability/fr-civ-agents-011/fr-civ-agents-011-adr.md:11
  - tests: crates/agents/src/lib.rs:997
- `FR-CIV-AGENTS-020`
  - spec: docs/traceability/fr-civ-agents-020/fr-civ-agents-020-adr.md:1, docs/traceability/fr-civ-agents-020/fr-civ-agents-020-adr.md:6, docs/traceability/fr-civ-agents-020/fr-civ-agents-020-adr.md:11
  - tests: crates/agents/src/lib.rs:1013
- `FR-CIV-AGENTS-021`
  - spec: docs/traceability/fr-civ-agents-021/fr-civ-agents-021-adr.md:1, docs/traceability/fr-civ-agents-021/fr-civ-agents-021-adr.md:6, docs/traceability/fr-civ-agents-021/fr-civ-agents-021-adr.md:11
  - tests: crates/agents/src/lib.rs:1076
- `FR-CIV-AGENTS-022`
  - spec: docs/traceability/fr-civ-agents-022/fr-civ-agents-022-adr.md:1, docs/traceability/fr-civ-agents-022/fr-civ-agents-022-adr.md:6, docs/traceability/fr-civ-agents-022/fr-civ-agents-022-adr.md:11
  - tests: crates/agents/src/lib.rs:1090
- `FR-CIV-AGENTS-023`
  - spec: docs/traceability/fr-civ-agents-023/fr-civ-agents-023-adr.md:1, docs/traceability/fr-civ-agents-023/fr-civ-agents-023-adr.md:6, docs/traceability/fr-civ-agents-023/fr-civ-agents-023-adr.md:11
  - tests: crates/agents/src/lib.rs:1337
- `FR-CIV-AGENTS-024`
  - spec: docs/traceability/fr-civ-agents-024/fr-civ-agents-024-adr.md:1, docs/traceability/fr-civ-agents-024/fr-civ-agents-024-adr.md:6, docs/traceability/fr-civ-agents-024/fr-civ-agents-024-adr.md:11
  - tests: crates/agents/src/lib.rs:1360
- `FR-CIV-AGENTS-025`
  - spec: docs/traceability/fr-civ-agents-025/fr-civ-agents-025-adr.md:1, docs/traceability/fr-civ-agents-025/fr-civ-agents-025-adr.md:6, docs/traceability/fr-civ-agents-025/fr-civ-agents-025-adr.md:11
  - tests: crates/agents/src/lib.rs:1654
- `FR-CIV-AGENTS-030`
  - spec: docs/traceability/fr-civ-agents-030/fr-civ-agents-030-adr.md:1, docs/traceability/fr-civ-agents-030/fr-civ-agents-030-adr.md:6, docs/traceability/fr-civ-agents-030/fr-civ-agents-030-adr.md:11
  - tests: crates/agents/src/lib.rs:1241
- `FR-CIV-AGENTS-031`
  - spec: docs/traceability/fr-civ-agents-031/fr-civ-agents-031-adr.md:1, docs/traceability/fr-civ-agents-031/fr-civ-agents-031-adr.md:6, docs/traceability/fr-civ-agents-031/fr-civ-agents-031-adr.md:11
  - tests: crates/agents/src/lib.rs:1262
- `FR-CIV-AGENTS-032`
  - spec: docs/traceability/fr-civ-agents-032/fr-civ-agents-032-adr.md:1, docs/traceability/fr-civ-agents-032/fr-civ-agents-032-adr.md:6, docs/traceability/fr-civ-agents-032/fr-civ-agents-032-adr.md:11
  - tests: crates/agents/src/lib.rs:1273
- `FR-CIV-AGENTS-033`
  - spec: docs/traceability/fr-civ-agents-033/fr-civ-agents-033-adr.md:1, docs/traceability/fr-civ-agents-033/fr-civ-agents-033-adr.md:6, docs/traceability/fr-civ-agents-033/fr-civ-agents-033-adr.md:11
  - tests: crates/agents/src/lib.rs:1289
- `FR-CIV-AGENTS-034`
  - spec: docs/traceability/fr-civ-agents-034/fr-civ-agents-034-adr.md:1, docs/traceability/fr-civ-agents-034/fr-civ-agents-034-adr.md:6, docs/traceability/fr-civ-agents-034/fr-civ-agents-034-adr.md:11
  - tests: crates/agents/src/lib.rs:1317
- `FR-CIV-ARCH-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-arch-004/fr-civ-arch-004-adr.md:1, docs/traceability/fr-civ-arch-004/fr-civ-arch-004-adr.md:6
  - tests: crates/build/src/lib.rs:1047
- `FR-CIV-ARCH-005`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-arch-005/fr-civ-arch-005-adr.md:1, docs/traceability/fr-civ-arch-005/fr-civ-arch-005-adr.md:6
  - tests: crates/build/src/lib.rs:1069, crates/engine/tests/fr_fr_civ_arch_005.rs:1, crates/engine/tests/fr_fr_civ_arch_005.rs:6
- `FR-CIV-ARCH-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-arch-006/fr-civ-arch-006-adr.md:1, docs/traceability/fr-civ-arch-006/fr-civ-arch-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_civ_arch_006.rs:1, crates/engine/tests/fr_fr_civ_arch_006.rs:6, crates/engine/tests/fr_fr_civ_arch_006.rs:10
- `FR-CIV-ARCH-007`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-arch-007/fr-civ-arch-007-adr.md:1, docs/traceability/fr-civ-arch-007/fr-civ-arch-007-adr.md:6
  - tests: crates/build/src/lib.rs:1098
- `FR-CIV-BEVY-003`
  - spec: docs/traceability/fr-3d-matrix.md:177, docs/traceability/full-traceability-matrix.md:306, docs/traceability/fr-civ-bevy-003/fr-civ-bevy-003-adr.md:1
  - tests: clients/bevy-ref/src/lib.rs:2061
- `FR-CIV-BUILD-004`
  - spec: docs/traceability/fr-civ-build-004/fr-civ-build-004-adr.md:1, docs/traceability/fr-civ-build-004/fr-civ-build-004-adr.md:6, docs/traceability/fr-civ-build-004/fr-civ-build-004-adr.md:11
  - tests: crates/build/src/lib.rs:641
- `FR-CIV-BUILD-005`
  - spec: docs/traceability/fr-civ-build-005/fr-civ-build-005-adr.md:1, docs/traceability/fr-civ-build-005/fr-civ-build-005-adr.md:6, docs/traceability/fr-civ-build-005/fr-civ-build-005-adr.md:11
  - tests: crates/build/src/lib.rs:657
- `FR-CIV-BUILD-010-`
  - spec: docs/traceability/fr-3d-matrix.md:230
- `FR-CIV-BUILD-011`
  - spec: docs/traceability/fr-civ-build-011/fr-civ-build-011-adr.md:1, docs/traceability/fr-civ-build-011/fr-civ-build-011-adr.md:6, docs/traceability/fr-civ-build-011/fr-civ-build-011-adr.md:11
  - tests: crates/build/src/lib.rs:696
- `FR-CIV-BUILD-012`
  - spec: docs/traceability/fr-civ-build-012/fr-civ-build-012-adr.md:1, docs/traceability/fr-civ-build-012/fr-civ-build-012-adr.md:6, docs/traceability/fr-civ-build-012/fr-civ-build-012-adr.md:11
  - tests: crates/build/src/lib.rs:727
- `FR-CIV-BUILD-013`
  - spec: docs/traceability/fr-civ-build-013/fr-civ-build-013-adr.md:1, docs/traceability/fr-civ-build-013/fr-civ-build-013-adr.md:6, docs/traceability/fr-civ-build-013/fr-civ-build-013-adr.md:11
  - tests: crates/build/src/lib.rs:750
- `FR-CIV-BUILD-014`
  - spec: docs/traceability/fr-civ-build-014/fr-civ-build-014-adr.md:1, docs/traceability/fr-civ-build-014/fr-civ-build-014-adr.md:6, docs/traceability/fr-civ-build-014/fr-civ-build-014-adr.md:11
  - tests: crates/build/src/lib.rs:780
- `FR-CIV-BUILD-015`
  - spec: docs/traceability/fr-civ-build-015/fr-civ-build-015-adr.md:1, docs/traceability/fr-civ-build-015/fr-civ-build-015-adr.md:6, docs/traceability/fr-civ-build-015/fr-civ-build-015-adr.md:11
  - tests: crates/build/src/lib.rs:916
- `FR-CIV-CA-006`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-ca-006/fr-civ-ca-006-adr.md:1, docs/traceability/fr-civ-ca-006/fr-civ-ca-006-adr.md:6
  - tests: crates/voxel/src/fluid_ca.rs:2468, crates/voxel/src/fluid_ca.rs:2495
- `FR-CIV-CA-007`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-ca-007/fr-civ-ca-007-adr.md:1, docs/traceability/fr-civ-ca-007/fr-civ-ca-007-adr.md:6
  - tests: crates/voxel/src/fluid_ca.rs:2054, crates/voxel/src/fluid_ca.rs:2074, crates/voxel/src/fluid_ca.rs:2135
- `FR-CIV-CA-010`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-ca-010/fr-civ-ca-010-adr.md:1, docs/traceability/fr-civ-ca-010/fr-civ-ca-010-adr.md:6
  - tests: crates/voxel/src/fluid_ca.rs:2949, crates/voxel/src/fluid_ca.rs:2953, crates/voxel/src/fluid_ca.rs:2996
- `FR-CIV-DET-001`
  - spec: agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:36, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:53, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:73
  - tests: crates/engine/tests/fr_fr_civ_det_001.rs:1, crates/engine/tests/fr_fr_civ_det_001.rs:6, crates/engine/tests/fr_fr_civ_det_001.rs:10
- `FR-CIV-DIFFUSION-002`
  - spec: docs/traceability/fr-civ-diffusion-002/fr-civ-diffusion-002-adr.md:1, docs/traceability/fr-civ-diffusion-002/fr-civ-diffusion-002-adr.md:6, docs/traceability/fr-civ-diffusion-002/fr-civ-diffusion-002-adr.md:11
  - tests: crates/diffusion/src/lib.rs:112
- `FR-CIV-DIFFUSION-004`
  - spec: docs/traceability/fr-civ-diffusion-004/fr-civ-diffusion-004-adr.md:1, docs/traceability/fr-civ-diffusion-004/fr-civ-diffusion-004-adr.md:6, docs/traceability/fr-civ-diffusion-004/fr-civ-diffusion-004-adr.md:11
  - tests: crates/diffusion/src/lib.rs:136
- `FR-CIV-DIFFUSION-005`
  - spec: docs/traceability/fr-civ-diffusion-005/fr-civ-diffusion-005-adr.md:1, docs/traceability/fr-civ-diffusion-005/fr-civ-diffusion-005-adr.md:6, docs/traceability/fr-civ-diffusion-005/fr-civ-diffusion-005-adr.md:11
  - tests: crates/diffusion/src/lib.rs:144
- `FR-CIV-DIFFUSION-006`
  - spec: docs/traceability/fr-civ-diffusion-006/fr-civ-diffusion-006-adr.md:1, docs/traceability/fr-civ-diffusion-006/fr-civ-diffusion-006-adr.md:6, docs/traceability/fr-civ-diffusion-006/fr-civ-diffusion-006-adr.md:11
  - tests: crates/diffusion/src/lib.rs:158
- `FR-CIV-DIFFUSION-007`
  - spec: docs/traceability/fr-civ-diffusion-007/fr-civ-diffusion-007-adr.md:1, docs/traceability/fr-civ-diffusion-007/fr-civ-diffusion-007-adr.md:6, docs/traceability/fr-civ-diffusion-007/fr-civ-diffusion-007-adr.md:11
  - tests: crates/diffusion/src/lib.rs:175
- `FR-CIV-DIFFUSION-008`
  - spec: docs/traceability/fr-civ-diffusion-008/fr-civ-diffusion-008-adr.md:1, docs/traceability/fr-civ-diffusion-008/fr-civ-diffusion-008-adr.md:6, docs/traceability/fr-civ-diffusion-008/fr-civ-diffusion-008-adr.md:11
  - tests: crates/diffusion/src/lib.rs:190
- `FR-CIV-DIFFUSION-009`
  - spec: docs/traceability/fr-civ-diffusion-009/fr-civ-diffusion-009-adr.md:1, docs/traceability/fr-civ-diffusion-009/fr-civ-diffusion-009-adr.md:6, docs/traceability/fr-civ-diffusion-009/fr-civ-diffusion-009-adr.md:11
  - tests: crates/diffusion/src/lib.rs:212
- `FR-CIV-DIFFUSION-010`
  - spec: docs/traceability/fr-civ-diffusion-010/fr-civ-diffusion-010-adr.md:1, docs/traceability/fr-civ-diffusion-010/fr-civ-diffusion-010-adr.md:6, docs/traceability/fr-civ-diffusion-010/fr-civ-diffusion-010-adr.md:11
  - tests: crates/diffusion/src/lib.rs:228
- `FR-CIV-DIFFUSION-011`
  - spec: docs/traceability/fr-civ-diffusion-011/fr-civ-diffusion-011-adr.md:1, docs/traceability/fr-civ-diffusion-011/fr-civ-diffusion-011-adr.md:6, docs/traceability/fr-civ-diffusion-011/fr-civ-diffusion-011-adr.md:11
  - tests: crates/diffusion/src/lib.rs:258
- `FR-CIV-DIFFUSION-012`
  - spec: docs/traceability/fr-civ-diffusion-012/fr-civ-diffusion-012-adr.md:1, docs/traceability/fr-civ-diffusion-012/fr-civ-diffusion-012-adr.md:6, docs/traceability/fr-civ-diffusion-012/fr-civ-diffusion-012-adr.md:11
  - tests: crates/diffusion/src/lib.rs:286
- `FR-CIV-DIFFUSION-013`
  - spec: docs/traceability/fr-civ-diffusion-013/fr-civ-diffusion-013-adr.md:1, docs/traceability/fr-civ-diffusion-013/fr-civ-diffusion-013-adr.md:6, docs/traceability/fr-civ-diffusion-013/fr-civ-diffusion-013-adr.md:11
  - tests: crates/diffusion/src/lib.rs:296
- `FR-CIV-DIFFUSION-014`
  - spec: docs/traceability/fr-civ-diffusion-014/fr-civ-diffusion-014-adr.md:1, docs/traceability/fr-civ-diffusion-014/fr-civ-diffusion-014-adr.md:6, docs/traceability/fr-civ-diffusion-014/fr-civ-diffusion-014-adr.md:11
  - tests: crates/diffusion/src/lib.rs:311
- `FR-CIV-DIFFUSION-015`
  - spec: docs/traceability/fr-civ-diffusion-015/fr-civ-diffusion-015-adr.md:1, docs/traceability/fr-civ-diffusion-015/fr-civ-diffusion-015-adr.md:6, docs/traceability/fr-civ-diffusion-015/fr-civ-diffusion-015-adr.md:11
  - tests: crates/diffusion/src/lib.rs:325
- `FR-CIV-DIPLO-007`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-diplo-007/fr-civ-diplo-007-adr.md:1, docs/traceability/fr-civ-diplo-007/fr-civ-diplo-007-adr.md:6
  - tests: crates/diplomacy/src/lib.rs:1653, crates/diplomacy/src/lib.rs:1655
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
- `FR-CIV-GENETICS-011`
  - spec: docs/traceability/fr-civ-genetics-011/fr-civ-genetics-011-adr.md:1, docs/traceability/fr-civ-genetics-011/fr-civ-genetics-011-adr.md:6, docs/traceability/fr-civ-genetics-011/fr-civ-genetics-011-adr.md:11
  - tests: crates/genetics/src/lib.rs:250
- `FR-CIV-GENETICS-012`
  - spec: docs/traceability/fr-civ-genetics-012/fr-civ-genetics-012-adr.md:1, docs/traceability/fr-civ-genetics-012/fr-civ-genetics-012-adr.md:6, docs/traceability/fr-civ-genetics-012/fr-civ-genetics-012-adr.md:11
  - tests: crates/genetics/src/lib.rs:258
- `FR-CIV-GODOT-F3D0`
  - spec: docs/traceability/fr-civ-godot-f3d0/fr-civ-godot-f3d0-adr.md:1, docs/traceability/fr-civ-godot-f3d0/fr-civ-godot-f3d0-adr.md:6, docs/traceability/fr-civ-godot-f3d0/fr-civ-godot-f3d0-adr.md:11
  - tests: clients/godot-ref/rust/src/ws_frame.rs:174
- `FR-CIV-INFRA-001`
  - spec: docs/traceability/civis-tracelinks.md:25, docs/traceability/fr-civ-infra-001/fr-civ-infra-001-adr.md:1, docs/traceability/fr-civ-infra-001/fr-civ-infra-001-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:364
- `FR-CIV-INFRA-010`
  - spec: docs/traceability/civis-tracelinks.md:26, docs/traceability/fr-civ-infra-010/fr-civ-infra-010-adr.md:1, docs/traceability/fr-civ-infra-010/fr-civ-infra-010-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:376
- `FR-CIV-INFRA-011`
  - spec: docs/traceability/civis-tracelinks.md:27, docs/traceability/fr-civ-infra-011/fr-civ-infra-011-adr.md:1, docs/traceability/fr-civ-infra-011/fr-civ-infra-011-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:387
- `FR-CIV-INFRA-020`
  - spec: docs/traceability/civis-tracelinks.md:28, docs/traceability/fr-civ-infra-020/fr-civ-infra-020-adr.md:1, docs/traceability/fr-civ-infra-020/fr-civ-infra-020-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:396
- `FR-CIV-INFRA-021`
  - spec: docs/traceability/civis-tracelinks.md:29, docs/traceability/fr-civ-infra-021/fr-civ-infra-021-adr.md:1, docs/traceability/fr-civ-infra-021/fr-civ-infra-021-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:406
- `FR-CIV-INFRA-022`
  - spec: docs/traceability/civis-tracelinks.md:30, docs/traceability/fr-civ-infra-022/fr-civ-infra-022-adr.md:1, docs/traceability/fr-civ-infra-022/fr-civ-infra-022-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:422
- `FR-CIV-INFRA-050`
  - spec: docs/traceability/civis-tracelinks.md:33, docs/traceability/fr-civ-infra-050/fr-civ-infra-050-adr.md:1, docs/traceability/fr-civ-infra-050/fr-civ-infra-050-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:458
- `FR-CIV-INFRA-060`
  - spec: docs/traceability/civis-tracelinks.md:34, docs/traceability/fr-civ-infra-060/fr-civ-infra-060-adr.md:1, docs/traceability/fr-civ-infra-060/fr-civ-infra-060-adr.md:6
  - tests: crates/civ-traffic/src/lib.rs:468
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
- `FR-CIV-LAWS-003`
  - spec: docs/traceability/fr-civ-laws-003/fr-civ-laws-003-adr.md:1, docs/traceability/fr-civ-laws-003/fr-civ-laws-003-adr.md:6, docs/traceability/fr-civ-laws-003/fr-civ-laws-003-adr.md:11
  - tests: crates/laws/src/lib.rs:368
- `FR-CIV-LAWS-004`
  - spec: docs/traceability/fr-civ-laws-004/fr-civ-laws-004-adr.md:1, docs/traceability/fr-civ-laws-004/fr-civ-laws-004-adr.md:6, docs/traceability/fr-civ-laws-004/fr-civ-laws-004-adr.md:11
  - tests: crates/laws/src/lib.rs:389
- `FR-CIV-LAWS-005`
  - spec: docs/traceability/fr-civ-laws-005/fr-civ-laws-005-adr.md:1, docs/traceability/fr-civ-laws-005/fr-civ-laws-005-adr.md:6, docs/traceability/fr-civ-laws-005/fr-civ-laws-005-adr.md:11
  - tests: crates/laws/src/lib.rs:411
- `FR-CIV-LAWS-006`
  - spec: docs/traceability/fr-3d-matrix.md:87
  - tests: crates/laws/src/lib.rs:421, crates/laws/src/lib.rs:447, crates/laws/src/lib.rs:463
- `FR-CIV-LAWS-007`
  - spec: docs/traceability/fr-emergence-matrix.md:303
  - tests: crates/laws/src/lib.rs:530
- `FR-CIV-LAWS-008`
  - spec: docs/traceability/fr-emergence-matrix.md:304
  - tests: crates/laws/src/lib.rs:551
- `FR-CIV-LAWS-009`
  - spec: docs/traceability/fr-emergence-matrix.md:305
  - tests: crates/laws/src/lib.rs:559
- `FR-CIV-LEGENDS-CONFIG-04`
  - spec: docs/traceability/fr-emergence-matrix.md:252
- `FR-CIV-LEGENDS-PERF-01`
  - spec: docs/traceability/fr-emergence-matrix.md:253
- `FR-CIV-LEGENDS-SCALE-02`
  - spec: docs/traceability/fr-emergence-matrix.md:254
- `FR-CIV-LIFE-000`
  - spec: docs/traceability/civis-tracelinks.md:13, docs/traceability/fr-civ-life-000/fr-civ-life-000-adr.md:1, docs/traceability/fr-civ-life-000/fr-civ-life-000-adr.md:6
  - tests: crates/needs/src/lib.rs:374
- `FR-CIV-LIFE-011`
  - spec: docs/traceability/fr-civ-life-011/fr-civ-life-011-adr.md:1, docs/traceability/fr-civ-life-011/fr-civ-life-011-adr.md:6, docs/traceability/fr-civ-life-011/fr-civ-life-011-adr.md:11
  - tests: crates/agents/src/daily_path.rs:334
- `FR-CIV-LIFE-012`
  - spec: docs/traceability/fr-civ-life-012/fr-civ-life-012-adr.md:1, docs/traceability/fr-civ-life-012/fr-civ-life-012-adr.md:6, docs/traceability/fr-civ-life-012/fr-civ-life-012-adr.md:11
  - tests: crates/agents/src/daily_path.rs:370
- `FR-CIV-LIFE-013`
  - spec: docs/traceability/fr-civ-life-013/fr-civ-life-013-adr.md:1, docs/traceability/fr-civ-life-013/fr-civ-life-013-adr.md:6, docs/traceability/fr-civ-life-013/fr-civ-life-013-adr.md:11
  - tests: crates/agents/src/daily_path.rs:382
- `FR-CIV-LIFE-014`
  - spec: docs/traceability/fr-civ-life-014/fr-civ-life-014-adr.md:1, docs/traceability/fr-civ-life-014/fr-civ-life-014-adr.md:6, docs/traceability/fr-civ-life-014/fr-civ-life-014-adr.md:11
  - tests: crates/agents/src/daily_path.rs:406
- `FR-CIV-LIFE-015`
  - spec: docs/traceability/fr-civ-life-015/fr-civ-life-015-adr.md:1, docs/traceability/fr-civ-life-015/fr-civ-life-015-adr.md:6, docs/traceability/fr-civ-life-015/fr-civ-life-015-adr.md:11
  - tests: crates/agents/src/daily_path.rs:430
- `FR-CIV-LIFE-016`
  - spec: docs/traceability/fr-civ-life-016/fr-civ-life-016-adr.md:1, docs/traceability/fr-civ-life-016/fr-civ-life-016-adr.md:6, docs/traceability/fr-civ-life-016/fr-civ-life-016-adr.md:11
  - tests: crates/agents/src/daily_path.rs:438
- `FR-CIV-LIFE-021`
  - spec: docs/traceability/fr-civ-life-021/fr-civ-life-021-adr.md:1, docs/traceability/fr-civ-life-021/fr-civ-life-021-adr.md:6, docs/traceability/fr-civ-life-021/fr-civ-life-021-adr.md:11
  - tests: crates/economy/src/stocks.rs:341
- `FR-CIV-LIFE-022`
  - spec: docs/traceability/fr-civ-life-022/fr-civ-life-022-adr.md:1, docs/traceability/fr-civ-life-022/fr-civ-life-022-adr.md:6, docs/traceability/fr-civ-life-022/fr-civ-life-022-adr.md:11
  - tests: crates/economy/src/stocks.rs:353, crates/economy/src/stocks.rs:472
- `FR-CIV-LIFE-023`
  - spec: docs/traceability/fr-civ-life-023/fr-civ-life-023-adr.md:1, docs/traceability/fr-civ-life-023/fr-civ-life-023-adr.md:6, docs/traceability/fr-civ-life-023/fr-civ-life-023-adr.md:11
  - tests: crates/economy/src/stocks.rs:360
- `FR-CIV-LIFE-024`
  - spec: docs/traceability/fr-civ-life-024/fr-civ-life-024-adr.md:1, docs/traceability/fr-civ-life-024/fr-civ-life-024-adr.md:6, docs/traceability/fr-civ-life-024/fr-civ-life-024-adr.md:11
  - tests: crates/economy/src/stocks.rs:371, crates/economy/src/stocks.rs:412
- `FR-CIV-LIFE-025`
  - spec: docs/traceability/fr-civ-life-025/fr-civ-life-025-adr.md:1, docs/traceability/fr-civ-life-025/fr-civ-life-025-adr.md:6, docs/traceability/fr-civ-life-025/fr-civ-life-025-adr.md:11
  - tests: crates/economy/src/stocks.rs:397
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
  - spec: docs/traceability/fr-emergence-matrix.md:208
- `FR-CIV-MIGRATION-002`
  - spec: docs/traceability/fr-emergence-matrix.md:209
- `FR-CIV-MIGRATION-003`
  - spec: docs/traceability/fr-emergence-matrix.md:210
- `FR-CIV-MIGRATION-004`
  - spec: docs/traceability/fr-emergence-matrix.md:211
- `FR-CIV-MIGRATION-005`
  - spec: docs/traceability/fr-emergence-matrix.md:212
- `FR-CIV-PLANET-003`
  - spec: docs/traceability/fr-civ-planet-003/fr-civ-planet-003-adr.md:1, docs/traceability/fr-civ-planet-003/fr-civ-planet-003-adr.md:6, docs/traceability/fr-civ-planet-003/fr-civ-planet-003-adr.md:11
  - tests: crates/planet/src/lib.rs:160
- `FR-CIV-PLANET-004`
  - spec: docs/traceability/fr-civ-planet-004/fr-civ-planet-004-adr.md:1, docs/traceability/fr-civ-planet-004/fr-civ-planet-004-adr.md:6, docs/traceability/fr-civ-planet-004/fr-civ-planet-004-adr.md:11
  - tests: crates/planet/src/lib.rs:182
- `FR-CIV-PLANET-005`
  - spec: docs/traceability/fr-civ-planet-005/fr-civ-planet-005-adr.md:1, docs/traceability/fr-civ-planet-005/fr-civ-planet-005-adr.md:6, docs/traceability/fr-civ-planet-005/fr-civ-planet-005-adr.md:11
  - tests: crates/planet/src/lib.rs:195
- `FR-CIV-PROTO3D-003`
  - spec: docs/traceability/fr-civ-proto3d-003/fr-civ-proto3d-003-adr.md:1, docs/traceability/fr-civ-proto3d-003/fr-civ-proto3d-003-adr.md:6, docs/traceability/fr-civ-proto3d-003/fr-civ-proto3d-003-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1204
- `FR-CIV-PROTO3D-004`
  - spec: docs/traceability/fr-civ-proto3d-004/fr-civ-proto3d-004-adr.md:1, docs/traceability/fr-civ-proto3d-004/fr-civ-proto3d-004-adr.md:6, docs/traceability/fr-civ-proto3d-004/fr-civ-proto3d-004-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1221
- `FR-CIV-PROTO3D-005`
  - spec: docs/traceability/fr-civ-proto3d-005/fr-civ-proto3d-005-adr.md:1, docs/traceability/fr-civ-proto3d-005/fr-civ-proto3d-005-adr.md:6, docs/traceability/fr-civ-proto3d-005/fr-civ-proto3d-005-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1276
- `FR-CIV-PROTO3D-006`
  - spec: docs/traceability/fr-civ-proto3d-006/fr-civ-proto3d-006-adr.md:1, docs/traceability/fr-civ-proto3d-006/fr-civ-proto3d-006-adr.md:6, docs/traceability/fr-civ-proto3d-006/fr-civ-proto3d-006-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1237
- `FR-CIV-PROTO3D-007`
  - spec: docs/traceability/fr-civ-proto3d-007/fr-civ-proto3d-007-adr.md:1, docs/traceability/fr-civ-proto3d-007/fr-civ-proto3d-007-adr.md:6, docs/traceability/fr-civ-proto3d-007/fr-civ-proto3d-007-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1265
- `FR-CIV-PROTO3D-008`
  - spec: docs/traceability/fr-civ-proto3d-008/fr-civ-proto3d-008-adr.md:1, docs/traceability/fr-civ-proto3d-008/fr-civ-proto3d-008-adr.md:6, docs/traceability/fr-civ-proto3d-008/fr-civ-proto3d-008-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1069
- `FR-CIV-PROTO3D-009`
  - spec: docs/traceability/fr-civ-proto3d-009/fr-civ-proto3d-009-adr.md:1, docs/traceability/fr-civ-proto3d-009/fr-civ-proto3d-009-adr.md:6, docs/traceability/fr-civ-proto3d-009/fr-civ-proto3d-009-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1039
- `FR-CIV-PROTO3D-010`
  - spec: docs/traceability/fr-civ-proto3d-010/fr-civ-proto3d-010-adr.md:1, docs/traceability/fr-civ-proto3d-010/fr-civ-proto3d-010-adr.md:6, docs/traceability/fr-civ-proto3d-010/fr-civ-proto3d-010-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1020, crates/server/src/voxel_frame_builder.rs:233
- `FR-CIV-PROTO3D-011`
  - spec: docs/traceability/fr-civ-proto3d-011/fr-civ-proto3d-011-adr.md:1, docs/traceability/fr-civ-proto3d-011/fr-civ-proto3d-011-adr.md:6, docs/traceability/fr-civ-proto3d-011/fr-civ-proto3d-011-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1103, crates/server/src/voxel_frame_builder.rs:242
- `FR-CIV-PROTO3D-012`
  - spec: docs/traceability/fr-civ-proto3d-012/fr-civ-proto3d-012-adr.md:1, docs/traceability/fr-civ-proto3d-012/fr-civ-proto3d-012-adr.md:6, docs/traceability/fr-civ-proto3d-012/fr-civ-proto3d-012-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1139, crates/server/src/voxel_frame_builder.rs:274
- `FR-CIV-PROTO3D-013`
  - spec: docs/traceability/fr-civ-proto3d-013/fr-civ-proto3d-013-adr.md:1, docs/traceability/fr-civ-proto3d-013/fr-civ-proto3d-013-adr.md:6, docs/traceability/fr-civ-proto3d-013/fr-civ-proto3d-013-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1163
- `FR-CIV-PROTO3D-015`
  - spec: docs/traceability/fr-civ-proto3d-015/fr-civ-proto3d-015-adr.md:1, docs/traceability/fr-civ-proto3d-015/fr-civ-proto3d-015-adr.md:6, docs/traceability/fr-civ-proto3d-015/fr-civ-proto3d-015-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1443, crates/protocol-3d/src/lib.rs:1464
- `FR-CIV-PROTO3D-016`
  - spec: docs/traceability/fr-civ-proto3d-016/fr-civ-proto3d-016-adr.md:1, docs/traceability/fr-civ-proto3d-016/fr-civ-proto3d-016-adr.md:6, docs/traceability/fr-civ-proto3d-016/fr-civ-proto3d-016-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1483, crates/protocol-3d/src/lib.rs:1501
- `FR-CIV-PROTO3D-017`
  - spec: docs/traceability/fr-civ-proto3d-017/fr-civ-proto3d-017-adr.md:1, docs/traceability/fr-civ-proto3d-017/fr-civ-proto3d-017-adr.md:6, docs/traceability/fr-civ-proto3d-017/fr-civ-proto3d-017-adr.md:11
  - tests: crates/protocol-3d/src/lib.rs:1520
- `FR-CIV-PSYCHE-004`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:1, docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:6
- `FR-CIV-PSYCHE-007`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-007/fr-civ-psyche-007-adr.md:1, docs/traceability/fr-civ-psyche-007/fr-civ-psyche-007-adr.md:6
- `FR-CIV-PSYCHE-008`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-psyche-008/fr-civ-psyche-008-adr.md:1, docs/traceability/fr-civ-psyche-008/fr-civ-psyche-008-adr.md:6
- `FR-CIV-REL-004`
  - spec: docs/traceability/fr-emergence-matrix.md:79
- `FR-CIV-RESEARCH-010`
  - spec: docs/traceability/fr-civ-research-010/fr-civ-research-010-adr.md:1, docs/traceability/fr-civ-research-010/fr-civ-research-010-adr.md:6, docs/traceability/fr-civ-research-010/fr-civ-research-010-adr.md:11
  - tests: crates/research/src/lib.rs:426
- `FR-CIV-RESEARCH-011`
  - spec: docs/traceability/fr-civ-research-011/fr-civ-research-011-adr.md:1, docs/traceability/fr-civ-research-011/fr-civ-research-011-adr.md:6, docs/traceability/fr-civ-research-011/fr-civ-research-011-adr.md:11
  - tests: crates/research/src/lib.rs:444
- `FR-CIV-RESEARCH-012`
  - spec: docs/traceability/fr-civ-research-012/fr-civ-research-012-adr.md:1, docs/traceability/fr-civ-research-012/fr-civ-research-012-adr.md:6, docs/traceability/fr-civ-research-012/fr-civ-research-012-adr.md:11
  - tests: crates/research/src/lib.rs:462
- `FR-CIV-RESEARCH-020`
  - spec: docs/traceability/fr-civ-research-020/fr-civ-research-020-adr.md:1, docs/traceability/fr-civ-research-020/fr-civ-research-020-adr.md:6, docs/traceability/fr-civ-research-020/fr-civ-research-020-adr.md:11
  - tests: crates/research/src/lib.rs:480
- `FR-CIV-RESEARCH-030`
  - spec: docs/traceability/fr-civ-research-030/fr-civ-research-030-adr.md:1, docs/traceability/fr-civ-research-030/fr-civ-research-030-adr.md:6, docs/traceability/fr-civ-research-030/fr-civ-research-030-adr.md:11
  - tests: crates/research/src/lib.rs:512
- `FR-CIV-RESEARCH-031`
  - spec: docs/traceability/fr-civ-research-031/fr-civ-research-031-adr.md:1, docs/traceability/fr-civ-research-031/fr-civ-research-031-adr.md:6, docs/traceability/fr-civ-research-031/fr-civ-research-031-adr.md:11
  - tests: crates/research/src/lib.rs:542
- `FR-CIV-RESEARCH-032`
  - spec: docs/traceability/fr-civ-research-032/fr-civ-research-032-adr.md:1, docs/traceability/fr-civ-research-032/fr-civ-research-032-adr.md:6, docs/traceability/fr-civ-research-032/fr-civ-research-032-adr.md:11
  - tests: crates/research/src/lib.rs:565
- `FR-CIV-RESEARCH-033`
  - spec: docs/traceability/fr-civ-research-033/fr-civ-research-033-adr.md:1, docs/traceability/fr-civ-research-033/fr-civ-research-033-adr.md:6, docs/traceability/fr-civ-research-033/fr-civ-research-033-adr.md:11
  - tests: crates/research/src/lib.rs:635
- `FR-CIV-SAVE-003`
  - spec: docs/traceability/civis-tracelinks.md:66, docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:1, docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:6
- `FR-CIV-SAVE-004`
  - spec: docs/traceability/civis-tracelinks.md:67, docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:1, docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:6
- `FR-CIV-SPECIES-002`
  - spec: docs/traceability/fr-civ-species-002/fr-civ-species-002-adr.md:1, docs/traceability/fr-civ-species-002/fr-civ-species-002-adr.md:6, docs/traceability/fr-civ-species-002/fr-civ-species-002-adr.md:11
  - tests: crates/species/src/lib.rs:169
- `FR-CIV-SPECIES-003`
  - spec: docs/traceability/fr-civ-species-003/fr-civ-species-003-adr.md:1, docs/traceability/fr-civ-species-003/fr-civ-species-003-adr.md:6, docs/traceability/fr-civ-species-003/fr-civ-species-003-adr.md:11
  - tests: crates/species/src/lib.rs:179
- `FR-CIV-SPECIES-004`
  - spec: docs/traceability/fr-civ-species-004/fr-civ-species-004-adr.md:1, docs/traceability/fr-civ-species-004/fr-civ-species-004-adr.md:6, docs/traceability/fr-civ-species-004/fr-civ-species-004-adr.md:11
  - tests: crates/species/src/lib.rs:208
- `FR-CIV-SPECIES-005`
  - spec: docs/traceability/fr-civ-species-005/fr-civ-species-005-adr.md:1, docs/traceability/fr-civ-species-005/fr-civ-species-005-adr.md:6, docs/traceability/fr-civ-species-005/fr-civ-species-005-adr.md:11
  - tests: crates/species/src/lib.rs:225
- `FR-CIV-SPECIES-006`
  - spec: docs/traceability/fr-civ-species-006/fr-civ-species-006-adr.md:1, docs/traceability/fr-civ-species-006/fr-civ-species-006-adr.md:6, docs/traceability/fr-civ-species-006/fr-civ-species-006-adr.md:11
  - tests: crates/species/src/lib.rs:248
- `FR-CIV-SPECIES-007`
  - spec: docs/traceability/fr-civ-species-007/fr-civ-species-007-adr.md:1, docs/traceability/fr-civ-species-007/fr-civ-species-007-adr.md:6, docs/traceability/fr-civ-species-007/fr-civ-species-007-adr.md:11
  - tests: crates/species/src/lib.rs:268
- `FR-CIV-SPECIES-008`
  - spec: docs/traceability/fr-civ-species-008/fr-civ-species-008-adr.md:1, docs/traceability/fr-civ-species-008/fr-civ-species-008-adr.md:6, docs/traceability/fr-civ-species-008/fr-civ-species-008-adr.md:11
  - tests: crates/species/src/lib.rs:286
- `FR-CIV-SPECIES-009`
  - spec: docs/traceability/fr-civ-species-009/fr-civ-species-009-adr.md:1, docs/traceability/fr-civ-species-009/fr-civ-species-009-adr.md:6, docs/traceability/fr-civ-species-009/fr-civ-species-009-adr.md:11
  - tests: crates/species/src/lib.rs:330
- `FR-CIV-SPECIES-010`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-010/fr-civ-species-010-adr.md:1, docs/traceability/fr-civ-species-010/fr-civ-species-010-adr.md:6
  - tests: crates/species/src/lib.rs:341
- `FR-CIV-SPECIES-011`
  - spec: FUNCTIONAL_REQUIREMENTS.md, docs/traceability/fr-civ-species-011/fr-civ-species-011-adr.md:1, docs/traceability/fr-civ-species-011/fr-civ-species-011-adr.md:6
  - tests: crates/species/src/lib.rs:359
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
- `FR-CIV-TACTICS-001-`
  - spec: docs/traceability/fr-3d-matrix.md:109, docs/traceability/full-traceability-matrix.md:214
- `FR-CIV-TACTICS-002`
  - spec: docs/traceability/fr-civ-tactics-002/fr-civ-tactics-002-adr.md:1, docs/traceability/fr-civ-tactics-002/fr-civ-tactics-002-adr.md:6, docs/traceability/fr-civ-tactics-002/fr-civ-tactics-002-adr.md:11
  - tests: crates/tactics/src/lib.rs:336
- `FR-CIV-TACTICS-003`
  - spec: docs/traceability/fr-civ-tactics-003/fr-civ-tactics-003-adr.md:1, docs/traceability/fr-civ-tactics-003/fr-civ-tactics-003-adr.md:6, docs/traceability/fr-civ-tactics-003/fr-civ-tactics-003-adr.md:11
  - tests: crates/tactics/src/lib.rs:350
- `FR-CIV-TACTICS-011`
  - spec: docs/traceability/fr-civ-tactics-011/fr-civ-tactics-011-adr.md:1, docs/traceability/fr-civ-tactics-011/fr-civ-tactics-011-adr.md:6, docs/traceability/fr-civ-tactics-011/fr-civ-tactics-011-adr.md:11
  - tests: crates/tactics/src/lib.rs:564
- `FR-CIV-TACTICS-100`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:36, docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:1, docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_100.rs:1, crates/tactics/tests/fr_fr_civ_tactics_100.rs:6, crates/tactics/tests/fr_fr_civ_tactics_100.rs:14
- `FR-CIV-TACTICS-101`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:40, docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:1, docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_101.rs:1, crates/tactics/tests/fr_fr_civ_tactics_101.rs:6, crates/tactics/tests/fr_fr_civ_tactics_101.rs:14
- `FR-CIV-TACTICS-102`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:43, docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:1, docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:6
  - tests: crates/tactics/tests/fr_fr_civ_tactics_102.rs:1, crates/tactics/tests/fr_fr_civ_tactics_102.rs:6, crates/tactics/tests/fr_fr_civ_tactics_102.rs:14
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
- `FR-CIV-TRAFFIC-LANE-004`
  - spec: docs/traceability/civis-tracelinks.md:43, docs/traceability/fr-civ-traffic-lane-004/fr-civ-traffic-lane-004-adr.md:1, docs/traceability/fr-civ-traffic-lane-004/fr-civ-traffic-lane-004-adr.md:6
  - tests: crates/civ-traffic/src/lane.rs:371
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
- `FR-CIV-VOXEL-DIRTY-001`
  - spec: docs/traceability/fr-civ-voxel-dirty-001/fr-civ-voxel-dirty-001-adr.md:1, docs/traceability/fr-civ-voxel-dirty-001/fr-civ-voxel-dirty-001-adr.md:6, docs/traceability/fr-civ-voxel-dirty-001/fr-civ-voxel-dirty-001-adr.md:11
  - tests: crates/voxel/src/fluid_ca.rs:2701
- `FR-CIV-VOXEL-DIRTY-002`
  - spec: docs/traceability/fr-civ-voxel-dirty-002/fr-civ-voxel-dirty-002-adr.md:1, docs/traceability/fr-civ-voxel-dirty-002/fr-civ-voxel-dirty-002-adr.md:6, docs/traceability/fr-civ-voxel-dirty-002/fr-civ-voxel-dirty-002-adr.md:11
  - tests: crates/voxel/src/fluid_ca.rs:2746
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
- `FR-LOD-001`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:75, docs/traceability/fr-lod-001/fr-lod-001-adr.md:1, docs/traceability/fr-lod-001/fr-lod-001-adr.md:6
  - tests: crates/engine/src/lod.rs:99, crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:3, crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:138
- `FR-METRICS-004`
  - spec: docs/FR.md:36, docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:1, docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:6
  - tests: crates/engine/tests/fr_fr_metrics_004.rs:1, crates/engine/tests/fr_fr_metrics_004.rs:6, crates/engine/tests/fr_fr_metrics_004.rs:10
- `FR-METRICS-005`
  - spec: docs/FR.md:37, docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:1, docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:6
  - tests: crates/engine/tests/fr_fr_metrics_005.rs:1, crates/engine/tests/fr_fr_metrics_005.rs:6, crates/engine/tests/fr_fr_metrics_005.rs:10
- `FR-MOD-002`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:233, docs/traceability/fr-mod-002/fr-mod-002-adr.md:1, docs/traceability/fr-mod-002/fr-mod-002-adr.md:6
  - tests: crates/mod-host/src/lib.rs:2180, crates/mod-host/src/lib.rs:2183, crates/mod-host/src/lib.rs:2193
- `FR-MOD-003`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:234, docs/traceability/fr-mod-003/fr-mod-003-adr.md:1, docs/traceability/fr-mod-003/fr-mod-003-adr.md:6
  - tests: crates/mod-host/src/lib.rs:2339, crates/mod-host/src/lib.rs:2342, crates/mod-host/src/lib.rs:2366
- `FR-MOD-005`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:236, docs/traceability/fr-mod-005/fr-mod-005-adr.md:1, docs/traceability/fr-mod-005/fr-mod-005-adr.md:6
  - tests: crates/mod-host/src/lib.rs:2449, crates/mod-host/src/lib.rs:2452, crates/mod-host/src/lib.rs:2490
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
- `FR-PROT-006`
  - spec: docs/traceability/TRACEABILITY_MATRIX.md:183, docs/traceability/fr-prot-006/fr-prot-006-adr.md:1, docs/traceability/fr-prot-006/fr-prot-006-adr.md:6
  - tests: crates/engine/tests/fr_fr_prot_006.rs:1, crates/engine/tests/fr_fr_prot_006.rs:6, crates/engine/tests/fr_fr_prot_006.rs:10
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
- `NFR-CIV-DET-004`
  - spec: docs/reference/non-functional-requirements.md:172, docs/reference/non-functional-requirements.md:561, docs/reference/non-functional-requirements.md:598
  - tests: crates/engine/tests/fr_nfr_civ_det_004.rs:1
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

## Implemented but untested IDs (269)

- `FR-CIV-ASSET-001`
  - spec: docs/traceability/fr-civ-asset-001/fr-civ-asset-001-adr.md:1, docs/traceability/fr-civ-asset-001/fr-civ-asset-001-adr.md:6, docs/traceability/fr-civ-asset-001/fr-civ-asset-001-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2425, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2427
- `FR-CIV-ASSET-002`
  - spec: docs/traceability/fr-civ-asset-002/fr-civ-asset-002-adr.md:1, docs/traceability/fr-civ-asset-002/fr-civ-asset-002-adr.md:6, docs/traceability/fr-civ-asset-002/fr-civ-asset-002-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206
- `FR-CIV-ASSET-003`
  - spec: docs/traceability/fr-civ-asset-003/fr-civ-asset-003-adr.md:1, docs/traceability/fr-civ-asset-003/fr-civ-asset-003-adr.md:6, docs/traceability/fr-civ-asset-003/fr-civ-asset-003-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2447, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3207
- `FR-CIV-ASSET-004`
  - spec: docs/traceability/fr-civ-asset-004/fr-civ-asset-004-adr.md:1, docs/traceability/fr-civ-asset-004/fr-civ-asset-004-adr.md:6, docs/traceability/fr-civ-asset-004/fr-civ-asset-004-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2457, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3208
- `FR-CIV-ASSET-005`
  - spec: docs/traceability/fr-civ-asset-005/fr-civ-asset-005-adr.md:1, docs/traceability/fr-civ-asset-005/fr-civ-asset-005-adr.md:6, docs/traceability/fr-civ-asset-005/fr-civ-asset-005-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2467, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3209
- `FR-CIV-ASSET-006`
  - spec: docs/traceability/fr-civ-asset-006/fr-civ-asset-006-adr.md:1, docs/traceability/fr-civ-asset-006/fr-civ-asset-006-adr.md:6, docs/traceability/fr-civ-asset-006/fr-civ-asset-006-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2477, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3210
- `FR-CIV-ASSET-007`
  - spec: docs/traceability/fr-civ-asset-007/fr-civ-asset-007-adr.md:1, docs/traceability/fr-civ-asset-007/fr-civ-asset-007-adr.md:6, docs/traceability/fr-civ-asset-007/fr-civ-asset-007-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2487, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3211
- `FR-CIV-ASSET-008`
  - spec: docs/traceability/fr-civ-asset-008/fr-civ-asset-008-adr.md:1, docs/traceability/fr-civ-asset-008/fr-civ-asset-008-adr.md:6, docs/traceability/fr-civ-asset-008/fr-civ-asset-008-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2497, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212
- `FR-CIV-ASSET-009`
  - spec: docs/traceability/fr-civ-asset-009/fr-civ-asset-009-adr.md:1, docs/traceability/fr-civ-asset-009/fr-civ-asset-009-adr.md:6, docs/traceability/fr-civ-asset-009/fr-civ-asset-009-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2507, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213
- `FR-CIV-ASSET-010`
  - spec: docs/traceability/fr-civ-asset-010/fr-civ-asset-010-adr.md:1, docs/traceability/fr-civ-asset-010/fr-civ-asset-010-adr.md:6, docs/traceability/fr-civ-asset-010/fr-civ-asset-010-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2425, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2517
- `FR-CIV-ASSET-011`
  - spec: docs/traceability/fr-civ-asset-011/fr-civ-asset-011-adr.md:1, docs/traceability/fr-civ-asset-011/fr-civ-asset-011-adr.md:6, docs/traceability/fr-civ-asset-011/fr-civ-asset-011-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2529
- `FR-CIV-ASSET-012`
  - spec: docs/traceability/fr-civ-asset-012/fr-civ-asset-012-adr.md:1, docs/traceability/fr-civ-asset-012/fr-civ-asset-012-adr.md:6, docs/traceability/fr-civ-asset-012/fr-civ-asset-012-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2539, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2917, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3216
- `FR-CIV-ASSET-013`
  - spec: docs/traceability/fr-civ-asset-013/fr-civ-asset-013-adr.md:1, docs/traceability/fr-civ-asset-013/fr-civ-asset-013-adr.md:6, docs/traceability/fr-civ-asset-013/fr-civ-asset-013-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2549, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3217
- `FR-CIV-ASSET-014`
  - spec: docs/traceability/fr-civ-asset-014/fr-civ-asset-014-adr.md:1, docs/traceability/fr-civ-asset-014/fr-civ-asset-014-adr.md:6, docs/traceability/fr-civ-asset-014/fr-civ-asset-014-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2559, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3218
- `FR-CIV-ASSET-015`
  - spec: docs/traceability/fr-civ-asset-015/fr-civ-asset-015-adr.md:1, docs/traceability/fr-civ-asset-015/fr-civ-asset-015-adr.md:6, docs/traceability/fr-civ-asset-015/fr-civ-asset-015-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:1714, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2569, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3219
- `FR-CIV-ASSET-016`
  - spec: docs/traceability/fr-civ-asset-016/fr-civ-asset-016-adr.md:1, docs/traceability/fr-civ-asset-016/fr-civ-asset-016-adr.md:6, docs/traceability/fr-civ-asset-016/fr-civ-asset-016-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2579, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3220
- `FR-CIV-ASSET-017`
  - spec: docs/traceability/fr-civ-asset-017/fr-civ-asset-017-adr.md:1, docs/traceability/fr-civ-asset-017/fr-civ-asset-017-adr.md:6, docs/traceability/fr-civ-asset-017/fr-civ-asset-017-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2589, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3221
- `FR-CIV-ASSET-018`
  - spec: docs/traceability/fr-civ-asset-018/fr-civ-asset-018-adr.md:1, docs/traceability/fr-civ-asset-018/fr-civ-asset-018-adr.md:6, docs/traceability/fr-civ-asset-018/fr-civ-asset-018-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2599, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3222
- `FR-CIV-ASSET-019`
  - spec: docs/traceability/fr-civ-asset-019/fr-civ-asset-019-adr.md:1, docs/traceability/fr-civ-asset-019/fr-civ-asset-019-adr.md:6, docs/traceability/fr-civ-asset-019/fr-civ-asset-019-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2609, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223
- `FR-CIV-ASSET-020`
  - spec: docs/traceability/fr-civ-asset-020/fr-civ-asset-020-adr.md:1, docs/traceability/fr-civ-asset-020/fr-civ-asset-020-adr.md:6, docs/traceability/fr-civ-asset-020/fr-civ-asset-020-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527, docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2619
- `FR-CIV-ASSET-MANI-001`
  - spec: docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:1, docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:6, docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212
- `FR-CIV-ASSET-MANI-002`
  - spec: docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:1, docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:6, docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213
- `FR-CIV-ASSET-QUAL-001`
  - spec: docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:1, docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:6, docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:11
  - code: docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3224
- `FR-CIV-AUDIO-009`
  - spec: docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:1, docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:6, docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:11
  - code: docs/design/audio-direction.md:302
- `FR-CIV-AUDIO-010`
  - spec: docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:1, docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:6, docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:11
  - code: docs/design/audio-direction.md:303
- `FR-CIV-AUDIO-011`
  - spec: docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:1, docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:6, docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:11
  - code: docs/design/audio-direction.md:304
- `FR-CIV-AUDIO-012`
  - spec: docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:1, docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:6, docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:11
  - code: docs/design/audio-direction.md:305
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
- `FR-CIV-ENGINE-INT-001`
  - spec: docs/traceability/fr-civ-engine-int-001/fr-civ-engine-int-001-adr.md:1, docs/traceability/fr-civ-engine-int-001/fr-civ-engine-int-001-adr.md:6, docs/traceability/fr-civ-engine-int-001/fr-civ-engine-int-001-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:949
- `FR-CIV-ENGINE-INT-002`
  - spec: docs/traceability/fr-civ-engine-int-002/fr-civ-engine-int-002-adr.md:1, docs/traceability/fr-civ-engine-int-002/fr-civ-engine-int-002-adr.md:6, docs/traceability/fr-civ-engine-int-002/fr-civ-engine-int-002-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1136
- `FR-CIV-ENGINE-INT-003`
  - spec: docs/traceability/fr-civ-engine-int-003/fr-civ-engine-int-003-adr.md:1, docs/traceability/fr-civ-engine-int-003/fr-civ-engine-int-003-adr.md:6, docs/traceability/fr-civ-engine-int-003/fr-civ-engine-int-003-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1169
- `FR-CIV-ENGINE-INT-005`
  - spec: docs/traceability/fr-civ-engine-int-005/fr-civ-engine-int-005-adr.md:1, docs/traceability/fr-civ-engine-int-005/fr-civ-engine-int-005-adr.md:6, docs/traceability/fr-civ-engine-int-005/fr-civ-engine-int-005-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1351
- `FR-CIV-ENGINE-INT-010`
  - spec: docs/traceability/fr-civ-engine-int-010/fr-civ-engine-int-010-adr.md:1, docs/traceability/fr-civ-engine-int-010/fr-civ-engine-int-010-adr.md:6, docs/traceability/fr-civ-engine-int-010/fr-civ-engine-int-010-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:77
- `FR-CIV-ENGINE-INT-011`
  - spec: docs/traceability/fr-civ-engine-int-011/fr-civ-engine-int-011-adr.md:1, docs/traceability/fr-civ-engine-int-011/fr-civ-engine-int-011-adr.md:6, docs/traceability/fr-civ-engine-int-011/fr-civ-engine-int-011-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1185
- `FR-CIV-ENGINE-INT-012`
  - spec: docs/traceability/fr-civ-engine-int-012/fr-civ-engine-int-012-adr.md:1, docs/traceability/fr-civ-engine-int-012/fr-civ-engine-int-012-adr.md:6, docs/traceability/fr-civ-engine-int-012/fr-civ-engine-int-012-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1223
- `FR-CIV-ENGINE-INT-013`
  - spec: docs/traceability/fr-civ-engine-int-013/fr-civ-engine-int-013-adr.md:1, docs/traceability/fr-civ-engine-int-013/fr-civ-engine-int-013-adr.md:6, docs/traceability/fr-civ-engine-int-013/fr-civ-engine-int-013-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1303
- `FR-CIV-ENGINE-INT-014`
  - spec: docs/traceability/fr-civ-engine-int-014/fr-civ-engine-int-014-adr.md:1, docs/traceability/fr-civ-engine-int-014/fr-civ-engine-int-014-adr.md:6, docs/traceability/fr-civ-engine-int-014/fr-civ-engine-int-014-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1341
- `FR-CIV-ENGINE-INT-015`
  - spec: docs/traceability/fr-civ-engine-int-015/fr-civ-engine-int-015-adr.md:1, docs/traceability/fr-civ-engine-int-015/fr-civ-engine-int-015-adr.md:6, docs/traceability/fr-civ-engine-int-015/fr-civ-engine-int-015-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1248
- `FR-CIV-ENGINE-REPLAY-001`
  - spec: docs/traceability/fr-civ-engine-replay-001/fr-civ-engine-replay-001-adr.md:1, docs/traceability/fr-civ-engine-replay-001/fr-civ-engine-replay-001-adr.md:6, docs/traceability/fr-civ-engine-replay-001/fr-civ-engine-replay-001-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1520
- `FR-CIV-ENGINE-REPLAY-002`
  - spec: docs/traceability/fr-civ-engine-replay-002/fr-civ-engine-replay-002-adr.md:1, docs/traceability/fr-civ-engine-replay-002/fr-civ-engine-replay-002-adr.md:6, docs/traceability/fr-civ-engine-replay-002/fr-civ-engine-replay-002-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1545
- `FR-CIV-ENGINE-REPLAY-003`
  - spec: docs/traceability/fr-civ-engine-replay-003/fr-civ-engine-replay-003-adr.md:1, docs/traceability/fr-civ-engine-replay-003/fr-civ-engine-replay-003-adr.md:6, docs/traceability/fr-civ-engine-replay-003/fr-civ-engine-replay-003-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1680
- `FR-CIV-ENGINE-REPLAY-004`
  - spec: docs/traceability/fr-civ-engine-replay-004/fr-civ-engine-replay-004-adr.md:1, docs/traceability/fr-civ-engine-replay-004/fr-civ-engine-replay-004-adr.md:6, docs/traceability/fr-civ-engine-replay-004/fr-civ-engine-replay-004-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1696
- `FR-CIV-ENGINE-REPLAY-005`
  - spec: docs/traceability/fr-civ-engine-replay-005/fr-civ-engine-replay-005-adr.md:1, docs/traceability/fr-civ-engine-replay-005/fr-civ-engine-replay-005-adr.md:6, docs/traceability/fr-civ-engine-replay-005/fr-civ-engine-replay-005-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1746
- `FR-CIV-GEO-001`
  - spec: docs/traceability/fr-civ-geo-001/fr-civ-geo-001-adr.md:1, docs/traceability/fr-civ-geo-001/fr-civ-geo-001-adr.md:6, docs/traceability/fr-civ-geo-001/fr-civ-geo-001-adr.md:11
  - code: docs/reference/FR_TRACKER.md:22, docs/reports/STATUS_REPORT.md:95, docs/specs/CIV-0300-rts-ui-ux-spec.md:2024
- `FR-CIV-GEO-002`
  - spec: docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:1, docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:6, docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2025
- `FR-CIV-GEO-003`
  - spec: docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:1, docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:6, docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2026
- `FR-CIV-GEO-004`
  - spec: docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:1, docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:6, docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:11
  - code: docs/reports/STATUS_REPORT.md:96, docs/specs/CIV-0300-rts-ui-ux-spec.md:2027
- `FR-CIV-GEO-005`
  - spec: docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:1, docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:6, docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2028
- `FR-CIV-GEO-006`
  - spec: docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:1, docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:6, docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2029
- `FR-CIV-GEO-007`
  - spec: docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:1, docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:6, docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2030
- `FR-CIV-GEO-008`
  - spec: docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:1, docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:6, docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2031
- `FR-CIV-GEO-009`
  - spec: docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:1, docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:6, docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:11
  - code: docs/specs/CIV-0300-rts-ui-ux-spec.md:2032
- `FR-CIV-GEO-010`
  - spec: docs/traceability/fr-civ-geo-010/fr-civ-geo-010-adr.md:1, docs/traceability/fr-civ-geo-010/fr-civ-geo-010-adr.md:6, docs/traceability/fr-civ-geo-010/fr-civ-geo-010-adr.md:11
  - code: docs/specs/CIV-0101-two-zoom-lod-v1.md:1580, docs/specs/CIV-0101-two-zoom-lod-v1.md:1582, docs/specs/CIV-0101-two-zoom-lod-v1.md:1584
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
- `FR-CIV-PLANET-010`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:160, agileplus-specs/civ-021-recovered-requirements/spec.md:166, docs/traceability/fr-civ-planet-010/fr-civ-planet-010-adr.md:1
  - code: crates/engine/src/engine/engine_tests.rs:966, crates/engine/src/engine.rs:3381, crates/server/src/jsonrpc.rs:515
- `FR-CIV-PLANET-020`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:85, docs/traceability/fr-civ-planet-020/fr-civ-planet-020-adr.md:1, docs/traceability/fr-civ-planet-020/fr-civ-planet-020-adr.md:6
  - code: crates/engine/src/climate.rs:16, crates/engine/src/climate.rs:35, crates/engine/src/climate.rs:48
- `FR-CIV-PLANET-030`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:91, docs/traceability/fr-civ-planet-030/fr-civ-planet-030-adr.md:1, docs/traceability/fr-civ-planet-030/fr-civ-planet-030-adr.md:6
  - code: crates/engine/src/climate.rs:35, crates/engine/src/engine/engine_tests.rs:1816, crates/engine/src/engine.rs:813
- `FR-CIV-PROTO3D-009-`
  - spec: docs/traceability/fr-3d-matrix.md:229
  - code: docs/development-guide/p-w1-kickoff.md:116, docs/specs/gap-audit.md:306
- `FR-CIV-PSYCHE-010`
  - spec: docs/traceability/fr-civ-psyche-010/fr-civ-psyche-010-adr.md:1, docs/traceability/fr-civ-psyche-010/fr-civ-psyche-010-adr.md:6, docs/traceability/fr-civ-psyche-010/fr-civ-psyche-010-adr.md:11
  - code: docs/design/psyche-social.md:142, docs/design/psyche-social.md:255, docs/design/psyche-social.md:277
- `FR-CIV-PSYCHE-011`
  - spec: docs/traceability/fr-civ-psyche-011/fr-civ-psyche-011-adr.md:1, docs/traceability/fr-civ-psyche-011/fr-civ-psyche-011-adr.md:6, docs/traceability/fr-civ-psyche-011/fr-civ-psyche-011-adr.md:11
  - code: docs/design/psyche-social.md:191, docs/design/psyche-social.md:259, docs/design/psyche-social.md:278
- `FR-CIV-PSYCHE-020`
  - spec: docs/traceability/fr-civ-psyche-020/fr-civ-psyche-020-adr.md:1, docs/traceability/fr-civ-psyche-020/fr-civ-psyche-020-adr.md:6, docs/traceability/fr-civ-psyche-020/fr-civ-psyche-020-adr.md:11
  - code: docs/design/civ-culture-emergent.md:7, docs/design/psyche-social.md:153, docs/design/psyche-social.md:256
- `FR-CIV-PSYCHE-021`
  - spec: docs/traceability/fr-civ-psyche-021/fr-civ-psyche-021-adr.md:1, docs/traceability/fr-civ-psyche-021/fr-civ-psyche-021-adr.md:6, docs/traceability/fr-civ-psyche-021/fr-civ-psyche-021-adr.md:11
  - code: docs/design/psyche-social.md:209, docs/design/psyche-social.md:280
- `FR-CIV-PSYCHE-024`
  - spec: docs/traceability/fr-civ-psyche-024/fr-civ-psyche-024-adr.md:1, docs/traceability/fr-civ-psyche-024/fr-civ-psyche-024-adr.md:6, docs/traceability/fr-civ-psyche-024/fr-civ-psyche-024-adr.md:11
  - code: docs/design/psyche-social.md:233, docs/design/psyche-social.md:264, docs/design/psyche-social.md:281
- `FR-CIV-PSYCHE-030`
  - spec: docs/traceability/fr-civ-psyche-030/fr-civ-psyche-030-adr.md:1, docs/traceability/fr-civ-psyche-030/fr-civ-psyche-030-adr.md:6, docs/traceability/fr-civ-psyche-030/fr-civ-psyche-030-adr.md:11
  - code: docs/design/psyche-social.md:162, docs/design/psyche-social.md:257, docs/design/psyche-social.md:258
- `FR-CIV-PSYCHE-031`
  - spec: docs/traceability/fr-civ-psyche-031/fr-civ-psyche-031-adr.md:1, docs/traceability/fr-civ-psyche-031/fr-civ-psyche-031-adr.md:6, docs/traceability/fr-civ-psyche-031/fr-civ-psyche-031-adr.md:11
  - code: docs/design/psyche-social.md:174, docs/design/psyche-social.md:283
- `FR-CIV-PSYCHE-032`
  - spec: docs/traceability/fr-civ-psyche-032/fr-civ-psyche-032-adr.md:1, docs/traceability/fr-civ-psyche-032/fr-civ-psyche-032-adr.md:6, docs/traceability/fr-civ-psyche-032/fr-civ-psyche-032-adr.md:11
  - code: docs/design/psyche-social.md:203, docs/design/psyche-social.md:260, docs/design/psyche-social.md:284
- `FR-CIV-PSYCHE-033`
  - spec: docs/traceability/fr-civ-psyche-033/fr-civ-psyche-033-adr.md:1, docs/traceability/fr-civ-psyche-033/fr-civ-psyche-033-adr.md:6, docs/traceability/fr-civ-psyche-033/fr-civ-psyche-033-adr.md:11
  - code: docs/design/psyche-social.md:206, docs/design/psyche-social.md:261, docs/design/psyche-social.md:285
- `FR-CIV-PSYCHE-034`
  - spec: docs/traceability/fr-civ-psyche-034/fr-civ-psyche-034-adr.md:1, docs/traceability/fr-civ-psyche-034/fr-civ-psyche-034-adr.md:6, docs/traceability/fr-civ-psyche-034/fr-civ-psyche-034-adr.md:11
  - code: docs/design/psyche-social.md:241, docs/design/psyche-social.md:286
- `FR-CIV-PSYCHE-035`
  - spec: docs/traceability/fr-civ-psyche-035/fr-civ-psyche-035-adr.md:1, docs/traceability/fr-civ-psyche-035/fr-civ-psyche-035-adr.md:6, docs/traceability/fr-civ-psyche-035/fr-civ-psyche-035-adr.md:11
  - code: docs/design/psyche-social.md:245, docs/design/psyche-social.md:287
- `FR-CIV-PSYCHE-036`
  - spec: docs/traceability/fr-civ-psyche-036/fr-civ-psyche-036-adr.md:1, docs/traceability/fr-civ-psyche-036/fr-civ-psyche-036-adr.md:6, docs/traceability/fr-civ-psyche-036/fr-civ-psyche-036-adr.md:11
  - code: docs/design/psyche-social.md:246, docs/design/psyche-social.md:288
- `FR-CIV-PSYCHE-037`
  - spec: docs/traceability/fr-civ-psyche-037/fr-civ-psyche-037-adr.md:1, docs/traceability/fr-civ-psyche-037/fr-civ-psyche-037-adr.md:6, docs/traceability/fr-civ-psyche-037/fr-civ-psyche-037-adr.md:11
  - code: docs/design/psyche-social.md:247, docs/design/psyche-social.md:289
- `FR-CIV-PSYCHE-040`
  - spec: docs/traceability/fr-civ-psyche-040/fr-civ-psyche-040-adr.md:1, docs/traceability/fr-civ-psyche-040/fr-civ-psyche-040-adr.md:6, docs/traceability/fr-civ-psyche-040/fr-civ-psyche-040-adr.md:11
  - code: docs/design/psyche-social.md:6, docs/design/psyche-social.md:290
- `FR-CIV-PSYCHE-900`
  - spec: docs/traceability/civis-tracelinks.md:142, docs/traceability/fr-civ-psyche-900/fr-civ-psyche-900-adr.md:1, docs/traceability/fr-civ-psyche-900/fr-civ-psyche-900-adr.md:6
  - code: crates/agents/src/psyche.rs:312, crates/agents/src/psyche.rs:424, crates/engine/src/dormant_phases.rs:3
- `FR-CIV-PSYCHE-911`
  - spec: docs/traceability/fr-civ-psyche-911/fr-civ-psyche-911-adr.md:1, docs/traceability/fr-civ-psyche-911/fr-civ-psyche-911-adr.md:6, docs/traceability/fr-civ-psyche-911/fr-civ-psyche-911-adr.md:11
  - code: crates/agents/src/psyche.rs:491, crates/engine/src/engine/ai_decision.rs:57, crates/engine/src/engine/engine_tests.rs:451
- `FR-CIV-REL-001`
  - spec: docs/traceability/fr-emergence-matrix.md:76
  - code: crates/engine/src/dormant_phases.rs:2, crates/engine/src/dormant_phases.rs:25, crates/engine/src/dormant_phases.rs:29
- `FR-CIV-REL-002`
  - spec: docs/traceability/fr-emergence-matrix.md:77
  - code: crates/engine/src/dormant_phases.rs:36
- `FR-CIV-REL-003`
  - spec: docs/traceability/fr-emergence-matrix.md:78
  - code: crates/engine/src/dormant_phases.rs:21, crates/engine/src/dormant_phases.rs:65, crates/engine/src/gameplay.rs:159
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
- `FR-CIV-SPECIES-100`
  - spec: docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:1, docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:6, docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:11
  - code: docs/design/species-sentience.md:73
- `FR-CIV-SPECIES-101`
  - spec: docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:1, docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:6, docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:11
  - code: docs/design/species-sentience.md:74
- `FR-CIV-SPECIES-102`
  - spec: docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:1, docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:6, docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:11
  - code: docs/design/species-sentience.md:75
- `FR-CIV-SPECIES-103`
  - spec: docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:1, docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:6, docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:11
  - code: docs/design/species-sentience.md:76
- `FR-CIV-SPECIES-104`
  - spec: docs/traceability/fr-civ-species-104/fr-civ-species-104-adr.md:1, docs/traceability/fr-civ-species-104/fr-civ-species-104-adr.md:6, docs/traceability/fr-civ-species-104/fr-civ-species-104-adr.md:11
  - code: docs/design/species-sentience.md:77, docs/design/species-sentience.md:101
- `FR-CIV-SPECIES-105`
  - spec: docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:1, docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:6, docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:11
  - code: docs/design/species-sentience.md:78
- `FR-CIV-SPECIES-200`
  - spec: docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:1, docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:6, docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:11
  - code: docs/design/species-sentience.md:98
- `FR-CIV-SPECIES-201`
  - spec: docs/traceability/fr-civ-species-201/fr-civ-species-201-adr.md:1, docs/traceability/fr-civ-species-201/fr-civ-species-201-adr.md:6, docs/traceability/fr-civ-species-201/fr-civ-species-201-adr.md:11
  - code: docs/design/species-sentience.md:99, docs/design/species-sentience.md:206
- `FR-CIV-SPECIES-202`
  - spec: docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:1, docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:6, docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:11
  - code: docs/design/species-sentience.md:100
- `FR-CIV-SPECIES-203`
  - spec: docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:1, docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:6, docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:11
  - code: docs/design/species-sentience.md:101
- `FR-CIV-SPECIES-204`
  - spec: docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:1, docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:6, docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:11
  - code: docs/design/species-sentience.md:102
- `FR-CIV-SPECIES-205`
  - spec: docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:1, docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:6, docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:11
  - code: docs/design/species-sentience.md:103
- `FR-CIV-SPECIES-300`
  - spec: docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:1, docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:6, docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:11
  - code: docs/design/species-sentience.md:120
- `FR-CIV-SPECIES-301`
  - spec: docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:1, docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:6, docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:11
  - code: docs/design/species-sentience.md:121
- `FR-CIV-SPECIES-302`
  - spec: docs/traceability/fr-civ-species-302/fr-civ-species-302-adr.md:1, docs/traceability/fr-civ-species-302/fr-civ-species-302-adr.md:6, docs/traceability/fr-civ-species-302/fr-civ-species-302-adr.md:11
  - code: docs/design/species-sentience.md:122, docs/design/species-sentience.md:192
- `FR-CIV-SPECIES-303`
  - spec: docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:1, docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:6, docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:11
  - code: docs/design/species-sentience.md:123
- `FR-CIV-SPECIES-304`
  - spec: docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:1, docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:6, docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:11
  - code: docs/design/species-sentience.md:124
- `FR-CIV-SPECIES-400`
  - spec: docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:1, docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:6, docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:11
  - code: docs/design/species-sentience.md:167
- `FR-CIV-SPECIES-401`
  - spec: docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:1, docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:6, docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:11
  - code: docs/design/species-sentience.md:168
- `FR-CIV-SPECIES-402`
  - spec: docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:1, docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:6, docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:11
  - code: docs/design/species-sentience.md:169
- `FR-CIV-SPECIES-403`
  - spec: docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:1, docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:6, docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:11
  - code: docs/design/species-sentience.md:170
- `FR-CIV-SPECIES-404`
  - spec: docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:1, docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:6, docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:11
  - code: docs/design/species-sentience.md:171
- `FR-CIV-SPECIES-405`
  - spec: docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:1, docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:6, docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:11
  - code: docs/design/species-sentience.md:172
- `FR-CIV-SPECIES-406`
  - spec: docs/traceability/fr-civ-species-406/fr-civ-species-406-adr.md:1, docs/traceability/fr-civ-species-406/fr-civ-species-406-adr.md:6, docs/traceability/fr-civ-species-406/fr-civ-species-406-adr.md:11
  - code: docs/design/species-sentience.md:173, docs/design/species-sentience.md:216
- `FR-CIV-SPECIES-500`
  - spec: docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:1, docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:6, docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:11
  - code: docs/design/species-sentience.md:191
- `FR-CIV-SPECIES-501`
  - spec: docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:1, docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:6, docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:11
  - code: docs/design/species-sentience.md:192
- `FR-CIV-SPECIES-502`
  - spec: docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:1, docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:6, docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:11
  - code: docs/design/species-sentience.md:193
- `FR-CIV-SPECIES-503`
  - spec: docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:1, docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:6, docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:11
  - code: docs/design/species-sentience.md:194
- `FR-CIV-SPECIES-504`
  - spec: docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:1, docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:6, docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:11
  - code: docs/design/species-sentience.md:195
- `FR-CIV-SPECIES-505`
  - spec: docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:1, docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:6, docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:11
  - code: docs/design/species-sentience.md:196
- `FR-CIV-TACTICS-025-`
  - spec: docs/traceability/fr-3d-matrix.md:123, docs/traceability/fr-3d-matrix.md:124, docs/traceability/fr-3d-matrix.md:125
  - code: crates/engine/src/engine/engine_tests.rs:1572, crates/engine/src/engine/engine_tests.rs:1594, crates/engine/src/engine/engine_tests.rs:1618
- `FR-CIV-TACTICS-032`
  - spec: docs/traceability/fr-3d-matrix.md:119, docs/traceability/full-traceability-matrix.md:227, docs/traceability/fr-civ-tactics-032/fr-civ-tactics-032-adr.md:1
  - code: docs/development-guide/p-w1-kickoff.md:31
- `FR-CIV-TACTICS-035`
  - spec: docs/traceability/fr-3d-matrix.md:122, docs/traceability/full-traceability-matrix.md:230, docs/traceability/fr-civ-tactics-035/fr-civ-tactics-035-adr.md:1
  - code: crates/engine/src/engine.rs:805, crates/tactics/src/military_phase.rs:1, docs/development-guide/p-w1-kickoff.md:34
- `FR-CIV-TACTICS-045`
  - spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:56, docs/traceability/fr-3d-matrix.md:135, docs/traceability/full-traceability-matrix.md:240
  - code: crates/engine/src/scenario.rs:99, docs/development-guide/p-w1-kickoff.md:47
- `FR-CIV-TACTICS-050`
  - spec: docs/traceability/fr-3d-matrix.md:140, docs/traceability/full-traceability-matrix.md:245, docs/traceability/fr-civ-tactics-050/fr-civ-tactics-050-adr.md:1
  - code: crates/engine/src/engine/military_phases.rs:234, crates/engine/src/scenario.rs:105, crates/engine/src/scenario.rs:189
- `FR-CIV-TECH-001`
  - spec: docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:1, docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:6, docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:11
  - code: docs/design/tech-engineering.md:225
- `FR-CIV-TECH-002`
  - spec: docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:1, docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:6, docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:11
  - code: docs/design/tech-engineering.md:226
- `FR-CIV-TECH-003`
  - spec: docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:1, docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:6, docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:11
  - code: docs/design/tech-engineering.md:227
- `FR-CIV-TECH-004`
  - spec: docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:1, docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:6, docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:11
  - code: docs/design/tech-engineering.md:228
- `FR-CIV-TECH-005`
  - spec: docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:1, docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:6, docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:11
  - code: docs/design/tech-engineering.md:229
- `FR-CIV-TECH-006`
  - spec: docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:1, docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:6, docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:11
  - code: docs/design/tech-engineering.md:230
- `FR-CIV-TECH-007`
  - spec: docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:1, docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:6, docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:11
  - code: docs/design/tech-engineering.md:231
- `FR-CIV-TECH-008`
  - spec: docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:1, docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:6, docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:11
  - code: docs/design/tech-engineering.md:232
- `FR-CIV-TECH-009`
  - spec: docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:1, docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:6, docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:11
  - code: docs/design/tech-engineering.md:233
- `FR-CIV-TECH-010`
  - spec: docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:1, docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:6, docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:11
  - code: docs/design/tech-engineering.md:234
- `FR-CIV-TECH-011`
  - spec: docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:1, docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:6, docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:11
  - code: docs/design/tech-engineering.md:235
- `FR-CIV-TECH-012`
  - spec: docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:1, docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:6, docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:11
  - code: docs/design/tech-engineering.md:236
- `FR-CIV-TECH-013`
  - spec: docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:1, docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:6, docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:11
  - code: docs/design/tech-engineering.md:237
- `FR-CIV-TECH-014`
  - spec: docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:1, docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:6, docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:11
  - code: docs/design/tech-engineering.md:238
- `FR-CIV-TECH-015`
  - spec: docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:1, docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:6, docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:11
  - code: docs/design/tech-engineering.md:239
- `FR-CIV-TECH-016`
  - spec: docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:1, docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:6, docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:11
  - code: docs/design/tech-engineering.md:240
- `FR-CIV-TECH-017`
  - spec: docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:1, docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:6, docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:11
  - code: docs/design/tech-engineering.md:241
- `FR-CIV-TECH-018`
  - spec: docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:1, docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:6, docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:11
  - code: docs/design/tech-engineering.md:242
- `FR-CIV-TECH-019`
  - spec: docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:1, docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:6, docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:11
  - code: docs/design/tech-engineering.md:243
- `FR-CIV-TECH-020`
  - spec: docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:1, docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:6, docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:11
  - code: docs/design/tech-engineering.md:244
- `FR-CIV-TECH-021`
  - spec: docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:1, docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:6, docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:11
  - code: docs/design/tech-engineering.md:245
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
  - code: crates/engine/src/engine/engine_tests.rs:1375
- `FR-CIV-VOXEL-007`
  - spec: docs/traceability/fr-civ-voxel-007/fr-civ-voxel-007-adr.md:1, docs/traceability/fr-civ-voxel-007/fr-civ-voxel-007-adr.md:6, docs/traceability/fr-civ-voxel-007/fr-civ-voxel-007-adr.md:11
  - code: crates/engine/src/engine/engine_tests.rs:1425
- `FR-CIV-WAR-020`
  - spec: agileplus-specs/civ-021-recovered-requirements/spec.md:228, docs/traceability/fr-civ-war-020/fr-civ-war-020-adr.md:1, docs/traceability/fr-civ-war-020/fr-civ-war-020-adr.md:6
  - code: docs/design/warfare.md:106, docs/design/warfare.md:195
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
- `FR-SAVE-006`
  - spec: docs/traceability/fr-save-006/fr-save-006-adr.md:1, docs/traceability/fr-save-006/fr-save-006-adr.md:6, docs/traceability/fr-save-006/fr-save-006-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2805, docs/specs/CIV-1000-save-load-persistence-spec.md:2943
- `FR-SAVE-007`
  - spec: docs/traceability/fr-save-007/fr-save-007-adr.md:1, docs/traceability/fr-save-007/fr-save-007-adr.md:6, docs/traceability/fr-save-007/fr-save-007-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2806, docs/specs/CIV-1000-save-load-persistence-spec.md:2943
- `FR-SAVE-008`
  - spec: docs/traceability/fr-save-008/fr-save-008-adr.md:1, docs/traceability/fr-save-008/fr-save-008-adr.md:6, docs/traceability/fr-save-008/fr-save-008-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2807, docs/specs/CIV-1000-save-load-persistence-spec.md:2949
- `FR-SAVE-009`
  - spec: docs/traceability/fr-save-009/fr-save-009-adr.md:1, docs/traceability/fr-save-009/fr-save-009-adr.md:6, docs/traceability/fr-save-009/fr-save-009-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2808
- `FR-SAVE-011`
  - spec: docs/traceability/fr-save-011/fr-save-011-adr.md:1, docs/traceability/fr-save-011/fr-save-011-adr.md:6, docs/traceability/fr-save-011/fr-save-011-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2810, docs/specs/CIV-1000-save-load-persistence-spec.md:2963
- `FR-SAVE-012`
  - spec: docs/traceability/fr-save-012/fr-save-012-adr.md:1, docs/traceability/fr-save-012/fr-save-012-adr.md:6, docs/traceability/fr-save-012/fr-save-012-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2811, docs/specs/CIV-1000-save-load-persistence-spec.md:2963
- `FR-SAVE-013`
  - spec: docs/traceability/fr-save-013/fr-save-013-adr.md:1, docs/traceability/fr-save-013/fr-save-013-adr.md:6, docs/traceability/fr-save-013/fr-save-013-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2812, docs/specs/CIV-1000-save-load-persistence-spec.md:2969
- `FR-SAVE-014`
  - spec: docs/traceability/fr-save-014/fr-save-014-adr.md:1, docs/traceability/fr-save-014/fr-save-014-adr.md:6, docs/traceability/fr-save-014/fr-save-014-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2813, docs/specs/CIV-1000-save-load-persistence-spec.md:2969
- `FR-SAVE-015`
  - spec: docs/traceability/fr-save-015/fr-save-015-adr.md:1, docs/traceability/fr-save-015/fr-save-015-adr.md:6, docs/traceability/fr-save-015/fr-save-015-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2814, docs/specs/CIV-1000-save-load-persistence-spec.md:2969
- `FR-SAVE-016`
  - spec: docs/traceability/fr-save-016/fr-save-016-adr.md:1, docs/traceability/fr-save-016/fr-save-016-adr.md:6, docs/traceability/fr-save-016/fr-save-016-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2815, docs/specs/CIV-1000-save-load-persistence-spec.md:2977
- `FR-SAVE-017`
  - spec: docs/traceability/fr-save-017/fr-save-017-adr.md:1, docs/traceability/fr-save-017/fr-save-017-adr.md:6, docs/traceability/fr-save-017/fr-save-017-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2816, docs/specs/CIV-1000-save-load-persistence-spec.md:2977
- `FR-SAVE-018`
  - spec: docs/traceability/fr-save-018/fr-save-018-adr.md:1, docs/traceability/fr-save-018/fr-save-018-adr.md:6, docs/traceability/fr-save-018/fr-save-018-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2817, docs/specs/CIV-1000-save-load-persistence-spec.md:2977
- `FR-SAVE-019`
  - spec: docs/traceability/fr-save-019/fr-save-019-adr.md:1, docs/traceability/fr-save-019/fr-save-019-adr.md:6, docs/traceability/fr-save-019/fr-save-019-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2818, docs/specs/CIV-1000-save-load-persistence-spec.md:2977
- `FR-SAVE-020`
  - spec: docs/traceability/fr-save-020/fr-save-020-adr.md:1, docs/traceability/fr-save-020/fr-save-020-adr.md:6, docs/traceability/fr-save-020/fr-save-020-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2819
- `FR-SAVE-021`
  - spec: docs/traceability/fr-save-021/fr-save-021-adr.md:1, docs/traceability/fr-save-021/fr-save-021-adr.md:6, docs/traceability/fr-save-021/fr-save-021-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2820, docs/specs/CIV-1000-save-load-persistence-spec.md:2987
- `FR-SAVE-022`
  - spec: docs/traceability/fr-save-022/fr-save-022-adr.md:1, docs/traceability/fr-save-022/fr-save-022-adr.md:6, docs/traceability/fr-save-022/fr-save-022-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2821, docs/specs/CIV-1000-save-load-persistence-spec.md:2993
- `FR-SAVE-023`
  - spec: docs/traceability/fr-save-023/fr-save-023-adr.md:1, docs/traceability/fr-save-023/fr-save-023-adr.md:6, docs/traceability/fr-save-023/fr-save-023-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2822, docs/specs/CIV-1000-save-load-persistence-spec.md:3001
- `FR-SAVE-024`
  - spec: docs/traceability/fr-save-024/fr-save-024-adr.md:1, docs/traceability/fr-save-024/fr-save-024-adr.md:6, docs/traceability/fr-save-024/fr-save-024-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2823
- `FR-SAVE-025`
  - spec: docs/traceability/fr-save-025/fr-save-025-adr.md:1, docs/traceability/fr-save-025/fr-save-025-adr.md:6, docs/traceability/fr-save-025/fr-save-025-adr.md:11
  - code: docs/specs/CIV-1000-save-load-persistence-spec.md:2824
- `FR-UX-006`
  - spec: docs/traceability/fr-ux-006/fr-ux-006-adr.md:1, docs/traceability/fr-ux-006/fr-ux-006-adr.md:6, docs/traceability/fr-ux-006/fr-ux-006-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:928
- `FR-UX-007`
  - spec: docs/traceability/fr-ux-007/fr-ux-007-adr.md:1, docs/traceability/fr-ux-007/fr-ux-007-adr.md:6, docs/traceability/fr-ux-007/fr-ux-007-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:931
- `FR-UX-008`
  - spec: docs/traceability/fr-ux-008/fr-ux-008-adr.md:1, docs/traceability/fr-ux-008/fr-ux-008-adr.md:6, docs/traceability/fr-ux-008/fr-ux-008-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:934
- `FR-UX-009`
  - spec: docs/traceability/fr-ux-009/fr-ux-009-adr.md:1, docs/traceability/fr-ux-009/fr-ux-009-adr.md:6, docs/traceability/fr-ux-009/fr-ux-009-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:939
- `FR-UX-010`
  - spec: docs/traceability/fr-ux-010/fr-ux-010-adr.md:1, docs/traceability/fr-ux-010/fr-ux-010-adr.md:6, docs/traceability/fr-ux-010/fr-ux-010-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:942
- `FR-UX-011`
  - spec: docs/traceability/fr-ux-011/fr-ux-011-adr.md:1, docs/traceability/fr-ux-011/fr-ux-011-adr.md:6, docs/traceability/fr-ux-011/fr-ux-011-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:945
- `FR-UX-012`
  - spec: docs/traceability/fr-ux-012/fr-ux-012-adr.md:1, docs/traceability/fr-ux-012/fr-ux-012-adr.md:6, docs/traceability/fr-ux-012/fr-ux-012-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:948
- `FR-UX-013`
  - spec: docs/traceability/fr-ux-013/fr-ux-013-adr.md:1, docs/traceability/fr-ux-013/fr-ux-013-adr.md:6, docs/traceability/fr-ux-013/fr-ux-013-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:951
- `FR-UX-014`
  - spec: docs/traceability/fr-ux-014/fr-ux-014-adr.md:1, docs/traceability/fr-ux-014/fr-ux-014-adr.md:6, docs/traceability/fr-ux-014/fr-ux-014-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:956
- `FR-UX-015`
  - spec: docs/traceability/fr-ux-015/fr-ux-015-adr.md:1, docs/traceability/fr-ux-015/fr-ux-015-adr.md:6, docs/traceability/fr-ux-015/fr-ux-015-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:959
- `FR-UX-016`
  - spec: docs/traceability/fr-ux-016/fr-ux-016-adr.md:1, docs/traceability/fr-ux-016/fr-ux-016-adr.md:6, docs/traceability/fr-ux-016/fr-ux-016-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:962
- `FR-UX-017`
  - spec: docs/traceability/fr-ux-017/fr-ux-017-adr.md:1, docs/traceability/fr-ux-017/fr-ux-017-adr.md:6, docs/traceability/fr-ux-017/fr-ux-017-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:965
- `FR-UX-018`
  - spec: docs/traceability/fr-ux-018/fr-ux-018-adr.md:1, docs/traceability/fr-ux-018/fr-ux-018-adr.md:6, docs/traceability/fr-ux-018/fr-ux-018-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:970
- `FR-UX-019`
  - spec: docs/traceability/fr-ux-019/fr-ux-019-adr.md:1, docs/traceability/fr-ux-019/fr-ux-019-adr.md:6, docs/traceability/fr-ux-019/fr-ux-019-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:973
- `FR-UX-020`
  - spec: docs/traceability/fr-ux-020/fr-ux-020-adr.md:1, docs/traceability/fr-ux-020/fr-ux-020-adr.md:6, docs/traceability/fr-ux-020/fr-ux-020-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:976
- `FR-UX-021`
  - spec: docs/traceability/fr-ux-021/fr-ux-021-adr.md:1, docs/traceability/fr-ux-021/fr-ux-021-adr.md:6, docs/traceability/fr-ux-021/fr-ux-021-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:979
- `FR-UX-022`
  - spec: docs/traceability/fr-ux-022/fr-ux-022-adr.md:1, docs/traceability/fr-ux-022/fr-ux-022-adr.md:6, docs/traceability/fr-ux-022/fr-ux-022-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:982
- `FR-UX-023`
  - spec: docs/traceability/fr-ux-023/fr-ux-023-adr.md:1, docs/traceability/fr-ux-023/fr-ux-023-adr.md:6, docs/traceability/fr-ux-023/fr-ux-023-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:987
- `FR-UX-024`
  - spec: docs/traceability/fr-ux-024/fr-ux-024-adr.md:1, docs/traceability/fr-ux-024/fr-ux-024-adr.md:6, docs/traceability/fr-ux-024/fr-ux-024-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:990
- `FR-UX-025`
  - spec: docs/traceability/fr-ux-025/fr-ux-025-adr.md:1, docs/traceability/fr-ux-025/fr-ux-025-adr.md:6, docs/traceability/fr-ux-025/fr-ux-025-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:993
- `FR-UX-026`
  - spec: docs/traceability/fr-ux-026/fr-ux-026-adr.md:1, docs/traceability/fr-ux-026/fr-ux-026-adr.md:6, docs/traceability/fr-ux-026/fr-ux-026-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:996
- `FR-UX-027`
  - spec: docs/traceability/fr-ux-027/fr-ux-027-adr.md:1, docs/traceability/fr-ux-027/fr-ux-027-adr.md:6, docs/traceability/fr-ux-027/fr-ux-027-adr.md:11
  - code: docs/models/civ-sim/USER_SPEC.md:999
- `NFR-C-01`
  - spec: docs/traceability/index.md:1151, docs/traceability/nfr-c-01/nfr-c-01-spec.md:1, docs/traceability/nfr-c-01/nfr-c-01-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2041
- `NFR-C-02`
  - spec: docs/traceability/index.md:1152, docs/traceability/nfr-c-02/nfr-c-02-spec.md:1, docs/traceability/nfr-c-02/nfr-c-02-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2042
- `NFR-C-03`
  - spec: docs/traceability/index.md:1153, docs/traceability/nfr-c-03/nfr-c-03-spec.md:1, docs/traceability/nfr-c-03/nfr-c-03-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2043
- `NFR-C-04`
  - spec: docs/traceability/index.md:1154, docs/traceability/nfr-c-04/nfr-c-04-spec.md:1, docs/traceability/nfr-c-04/nfr-c-04-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2044
- `NFR-C-05`
  - spec: docs/traceability/index.md:1155, docs/traceability/nfr-c-05/nfr-c-05-spec.md:1, docs/traceability/nfr-c-05/nfr-c-05-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2045
- `NFR-C-06`
  - spec: docs/traceability/index.md:1156, docs/traceability/nfr-c-06/nfr-c-06-spec.md:1, docs/traceability/nfr-c-06/nfr-c-06-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2046
- `NFR-C-07`
  - spec: docs/traceability/index.md:1157, docs/traceability/nfr-c-07/nfr-c-07-spec.md:1, docs/traceability/nfr-c-07/nfr-c-07-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2047
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
  - spec: docs/traceability/index.md:1162, docs/traceability/nfr-civ-ai-001/nfr-civ-ai-001-research.md:1, docs/traceability/nfr-civ-ai-001/nfr-civ-ai-001-research.md:4
  - code: crates/ai/src/pool.rs:1, docs/design/civ-ai-crate.md:48, docs/design/civ-ai-crate.md:180
- `NFR-CIV-AI-002`
  - spec: docs/traceability/index.md:1163, docs/traceability/nfr-civ-ai-002/nfr-civ-ai-002-research.md:1, docs/traceability/nfr-civ-ai-002/nfr-civ-ai-002-research.md:4
  - code: docs/design/civ-ai-crate.md:49
- `NFR-CIV-AI-003`
  - spec: docs/traceability/index.md:1164, docs/traceability/nfr-civ-ai-003/nfr-civ-ai-003-research.md:1, docs/traceability/nfr-civ-ai-003/nfr-civ-ai-003-research.md:4
  - code: crates/ai/src/lib.rs:13, crates/ai/src/lib.rs:229, docs/design/civ-ai-crate.md:50
- `NFR-CIV-DEV-HYGIENE-001`
  - spec: docs/traceability/index.md:1169, docs/traceability/nfr-civ-dev-hygiene-001/nfr-civ-dev-hygiene-001-spec.md:1, docs/traceability/nfr-civ-dev-hygiene-001/nfr-civ-dev-hygiene-001-spec.md:5
  - code: docs/ops/history-purge-plan.md:4
- `NFR-CIV-LEGENDS-CONFIG-04`
  - spec: docs/traceability/index.md:1170, docs/traceability/nfr-civ-legends-config-04/nfr-civ-legends-config-04-research.md:1, docs/traceability/nfr-civ-legends-config-04/nfr-civ-legends-config-04-research.md:4
  - code: crates/legends/src/config.rs:1, docs/design/legends-engine.md:454
- `NFR-CIV-LEGENDS-LOUD-03`
  - spec: docs/traceability/index.md:1171, docs/traceability/nfr-civ-legends-loud-03/nfr-civ-legends-loud-03-research.md:1, docs/traceability/nfr-civ-legends-loud-03/nfr-civ-legends-loud-03-research.md:4
  - code: docs/design/legends-engine.md:453
- `NFR-CIV-LEGENDS-PERF-01`
  - spec: docs/traceability/index.md:1172, docs/traceability/nfr-civ-legends-perf-01/nfr-civ-legends-perf-01-research.md:1, docs/traceability/nfr-civ-legends-perf-01/nfr-civ-legends-perf-01-research.md:4
  - code: docs/design/legends-engine.md:451
- `NFR-CIV-LEGENDS-SCALE-02`
  - spec: docs/traceability/index.md:1173, docs/traceability/nfr-civ-legends-scale-02/nfr-civ-legends-scale-02-adr.md:1, docs/traceability/nfr-civ-legends-scale-02/nfr-civ-legends-scale-02-adr.md:6
  - code: crates/legends/src/lib.rs:17, docs/design/legends-engine.md:452
- `NFR-CIV-PERF-003`
  - spec: agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:69, agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:25, docs/reference/non-functional-requirements.md:55
  - code: docs/design/civ-perf-dirty-incremental.md:9, docs/design/civ-perf-dirty-incremental.md:502, docs/guides/voxel-emergent-vision-and-migration.md:170
- `NFR-CIV-PERF-004`
  - spec: docs/reference/non-functional-requirements.md:69, docs/reference/non-functional-requirements.md:193, docs/reference/non-functional-requirements.md:194
  - code: docs/guides/voxel-emergent-vision-and-migration.md:189
- `NFR-CIV-PERF-005`
  - spec: agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:24, docs/reference/non-functional-requirements.md:86, docs/reference/non-functional-requirements.md:216
  - code: docs/design/civ-perf-dirty-incremental.md:10, docs/design/civ-perf-dirty-incremental.md:502, docs/guides/voxel-emergent-vision-and-migration.md:96
- `NFR-CIV-PERF-006`
  - spec: docs/reference/non-functional-requirements.md:100, docs/reference/non-functional-requirements.md:556, docs/reference/non-functional-requirements.md:596
  - code: docs/guides/voxel-emergent-vision-and-migration.md:172
- `NFR-CIV-PERF-008`
  - spec: docs/traceability/index.md:1187, docs/traceability/nfr-civ-perf-008/nfr-civ-perf-008-research.md:1, docs/traceability/nfr-civ-perf-008/nfr-civ-perf-008-research.md:4
  - code: docs/guides/voxel-emergent-vision-and-migration.md:171, docs/guides/voxel-emergent-vision-and-migration.md:191
- `NFR-CIV-PERF-900`
  - spec: docs/traceability/fr-nfr-matrix.md:24, docs/traceability/fr-nfr-matrix.md:148, docs/traceability/index.md:1188
  - code: docs/agileplus/epics/civ-w5-scale.md:14, docs/agileplus/epics/civ-w5-scale.md:27, docs/agileplus/README.md:24
- `NFR-CIV-PERF-901`
  - spec: docs/traceability/fr-nfr-matrix.md:25, docs/traceability/index.md:1189, docs/traceability/nfr-civ-perf-901/nfr-civ-perf-901-research.md:1
  - code: docs/agileplus/epics/civ-w5-scale.md:15, docs/agileplus/epics/civ-w5-scale.md:27, docs/agileplus/README.md:24
- `NFR-CIV-PERF-902`
  - spec: docs/traceability/fr-nfr-matrix.md:26, docs/traceability/index.md:1190, docs/traceability/nfr-civ-perf-902/nfr-civ-perf-902-research.md:1
  - code: docs/agileplus/epics/civ-w5-scale.md:16, docs/agileplus/epics/civ-w5-scale.md:28, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-002`
  - spec: docs/reference/non-functional-requirements.md:110, docs/reference/non-functional-requirements.md:206, docs/reference/non-functional-requirements.md:563
  - code: docs/guides/voxel-emergent-vision-and-migration.md:96, docs/guides/voxel-emergent-vision-and-migration.md:152
- `NFR-CIV-SCALE-004`
  - spec: docs/traceability/index.md:1201, docs/traceability/nfr-civ-scale-004/nfr-civ-scale-004-adr.md:1, docs/traceability/nfr-civ-scale-004/nfr-civ-scale-004-adr.md:6
  - code: docs/guides/voxel-emergent-vision-and-migration.md:172
- `NFR-CIV-SCALE-900`
  - spec: docs/traceability/fr-nfr-matrix.md:48, docs/traceability/index.md:1202, docs/traceability/nfr-civ-scale-900/nfr-civ-scale-900-adr.md:1
  - code: docs/agileplus/epics/civ-w5-scale.md:9, docs/agileplus/epics/civ-w5-scale.md:22, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-902`
  - spec: docs/traceability/fr-nfr-matrix.md:50, docs/traceability/fr-nfr-matrix.md:159, docs/traceability/index.md:1204
  - code: docs/agileplus/epics/civ-w5-scale.md:11, docs/agileplus/epics/civ-w5-scale.md:24, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-910`
  - spec: docs/traceability/fr-nfr-matrix.md:51, docs/traceability/index.md:1205, docs/traceability/nfr-civ-scale-910/nfr-civ-scale-910-adr.md:1
  - code: docs/agileplus/epics/civ-w5-scale.md:12, docs/agileplus/epics/civ-w5-scale.md:25, docs/agileplus/README.md:24
- `NFR-CIV-SCALE-920`
  - spec: docs/traceability/fr-nfr-matrix.md:52, docs/traceability/fr-nfr-matrix.md:159, docs/traceability/index.md:1206
  - code: docs/agileplus/epics/civ-w5-scale.md:13, docs/agileplus/epics/civ-w5-scale.md:26, docs/agileplus/README.md:24
- `NFR-O-01`
  - spec: docs/traceability/index.md:1211, docs/traceability/nfr-o-01/nfr-o-01-spec.md:1, docs/traceability/nfr-o-01/nfr-o-01-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2088
- `NFR-O-02`
  - spec: docs/traceability/index.md:1212, docs/traceability/nfr-o-02/nfr-o-02-spec.md:1, docs/traceability/nfr-o-02/nfr-o-02-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2089
- `NFR-O-03`
  - spec: docs/traceability/index.md:1213, docs/traceability/nfr-o-03/nfr-o-03-spec.md:1, docs/traceability/nfr-o-03/nfr-o-03-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2090
- `NFR-O-04`
  - spec: docs/traceability/index.md:1214, docs/traceability/nfr-o-04/nfr-o-04-spec.md:1, docs/traceability/nfr-o-04/nfr-o-04-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2091
- `NFR-O-05`
  - spec: docs/traceability/index.md:1215, docs/traceability/nfr-o-05/nfr-o-05-spec.md:1, docs/traceability/nfr-o-05/nfr-o-05-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2092
- `NFR-O-06`
  - spec: docs/traceability/index.md:1216, docs/traceability/nfr-o-06/nfr-o-06-spec.md:1, docs/traceability/nfr-o-06/nfr-o-06-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2093
- `NFR-P-01`
  - spec: docs/traceability/index.md:1217, docs/traceability/nfr-p-01/nfr-p-01-spec.md:1, docs/traceability/nfr-p-01/nfr-p-01-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2053
- `NFR-P-02`
  - spec: docs/traceability/index.md:1218, docs/traceability/nfr-p-02/nfr-p-02-spec.md:1, docs/traceability/nfr-p-02/nfr-p-02-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2054
- `NFR-P-03`
  - spec: docs/traceability/index.md:1219, docs/traceability/nfr-p-03/nfr-p-03-spec.md:1, docs/traceability/nfr-p-03/nfr-p-03-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2055
- `NFR-P-04`
  - spec: docs/traceability/index.md:1220, docs/traceability/nfr-p-04/nfr-p-04-spec.md:1, docs/traceability/nfr-p-04/nfr-p-04-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2056
- `NFR-P-05`
  - spec: docs/traceability/index.md:1221, docs/traceability/nfr-p-05/nfr-p-05-spec.md:1, docs/traceability/nfr-p-05/nfr-p-05-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2057
- `NFR-P-06`
  - spec: docs/traceability/index.md:1222, docs/traceability/nfr-p-06/nfr-p-06-spec.md:1, docs/traceability/nfr-p-06/nfr-p-06-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2058
- `NFR-P-07`
  - spec: docs/traceability/index.md:1223, docs/traceability/nfr-p-07/nfr-p-07-spec.md:1, docs/traceability/nfr-p-07/nfr-p-07-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2059
- `NFR-P-08`
  - spec: docs/traceability/index.md:1224, docs/traceability/nfr-p-08/nfr-p-08-spec.md:1, docs/traceability/nfr-p-08/nfr-p-08-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2060
- `NFR-R-01`
  - spec: docs/traceability/index.md:1225, docs/traceability/nfr-r-01/nfr-r-01-spec.md:1, docs/traceability/nfr-r-01/nfr-r-01-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2077
- `NFR-R-02`
  - spec: docs/traceability/index.md:1226, docs/traceability/nfr-r-02/nfr-r-02-spec.md:1, docs/traceability/nfr-r-02/nfr-r-02-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2078
- `NFR-R-03`
  - spec: docs/traceability/index.md:1227, docs/traceability/nfr-r-03/nfr-r-03-spec.md:1, docs/traceability/nfr-r-03/nfr-r-03-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2079
- `NFR-R-04`
  - spec: docs/traceability/index.md:1228, docs/traceability/nfr-r-04/nfr-r-04-spec.md:1, docs/traceability/nfr-r-04/nfr-r-04-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2080
- `NFR-R-05`
  - spec: docs/traceability/index.md:1229, docs/traceability/nfr-r-05/nfr-r-05-spec.md:1, docs/traceability/nfr-r-05/nfr-r-05-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2081
- `NFR-R-06`
  - spec: docs/traceability/index.md:1230, docs/traceability/nfr-r-06/nfr-r-06-spec.md:1, docs/traceability/nfr-r-06/nfr-r-06-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2082
- `NFR-S-01`
  - spec: docs/traceability/index.md:1231, docs/traceability/nfr-s-01/nfr-s-01-spec.md:1, docs/traceability/nfr-s-01/nfr-s-01-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2066
- `NFR-S-02`
  - spec: docs/traceability/index.md:1232, docs/traceability/nfr-s-02/nfr-s-02-spec.md:1, docs/traceability/nfr-s-02/nfr-s-02-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2067
- `NFR-S-03`
  - spec: docs/traceability/index.md:1233, docs/traceability/nfr-s-03/nfr-s-03-spec.md:1, docs/traceability/nfr-s-03/nfr-s-03-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2068
- `NFR-S-04`
  - spec: docs/traceability/index.md:1234, docs/traceability/nfr-s-04/nfr-s-04-spec.md:1, docs/traceability/nfr-s-04/nfr-s-04-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2069
- `NFR-S-05`
  - spec: docs/traceability/index.md:1235, docs/traceability/nfr-s-05/nfr-s-05-spec.md:1, docs/traceability/nfr-s-05/nfr-s-05-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2070
- `NFR-S-06`
  - spec: docs/traceability/index.md:1236, docs/traceability/nfr-s-06/nfr-s-06-spec.md:1, docs/traceability/nfr-s-06/nfr-s-06-spec.md:5
  - code: docs/models/civ-sim/TECHNICAL_SPEC.md:2071
- `NFR-SCALE-02`
  - spec: docs/traceability/fr-emergence-matrix.md:254, docs/traceability/index.md:1237, docs/traceability/nfr-scale-02/nfr-scale-02-adr.md:1
  - code: crates/legends/src/config.rs:18

## Code-only IDs (missing spec/traceability) (221)

- `FR-ASSET-PIPELINE-001`
  - code: crates/asset-pipeline/src/lib.rs:3, crates/asset-pipeline/src/lib.rs:18, crates/asset-pipeline/src/lib.rs:57
- `FR-ASSET-PIPELINE-002`
  - code: crates/asset-pipeline/Cargo.toml:8, crates/asset-pipeline/src/bin/svg_export.rs:20, crates/asset-pipeline/src/error.rs:9
- `FR-CIV-0104-011`
  - code: docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1504
- `FR-CIV-014`
  - code: crates/engine/src/engine/engine_tests.rs:3123
  - tests: crates/engine/src/emergence.rs:1625, crates/engine/src/save.rs:335, crates/engine/src/save.rs:375
- `FR-CIV-0700`
  - code: docs/design/civ-actor-assets-fix.md:322
- `FR-CIV-3D`
  - code: docs/design/civ-actor-assets-fix.md:251
- `FR-CIV-ACCESS-010`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:47
- `FR-CIV-ACCESS-020`
  - code: docs/adr/ADR-021-accessibility-and-l10n-strategy.md:48
- `FR-CIV-AGGRESSION-001`
  - code: crates/engine/src/engine.rs:485, crates/engine/src/engine.rs:2184
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:3
- `FR-CIV-ARCH-A-001`
  - tests: crates/build/src/tiers.rs:405
- `FR-CIV-ARCH-A-002`
  - tests: crates/build/src/tiers.rs:426
- `FR-CIV-ARCH-A-003`
  - tests: crates/build/src/tiers.rs:448
- `FR-CIV-ARCH-B-001`
  - tests: crates/build/src/tiers.rs:472
- `FR-CIV-ARCH-B-002`
  - tests: crates/build/src/tiers.rs:491
- `FR-CIV-ARCH-B-003`
  - tests: crates/build/src/tiers.rs:505
- `FR-CIV-ARCH-B-004`
  - tests: crates/build/src/tiers.rs:514
- `FR-CIV-ARCH-C-001`
  - tests: crates/build/src/tiers.rs:530
- `FR-CIV-ARCH-C-002`
  - tests: crates/build/src/tiers.rs:558
- `FR-CIV-ARCH-C-003`
  - tests: crates/build/src/tiers.rs:576
- `FR-CIV-ARCH-C-004`
  - tests: crates/build/src/tiers.rs:586
- `FR-CIV-ARCH-D-001`
  - tests: crates/build/src/tiers.rs:602
- `FR-CIV-ARCH-D-002`
  - tests: crates/build/src/tiers.rs:618
- `FR-CIV-ARCH-D-003`
  - tests: crates/build/src/tiers.rs:633
- `FR-CIV-ARCH-D-004`
  - tests: crates/build/src/tiers.rs:643
- `FR-CIV-BELIEF-001`
  - code: crates/engine/src/religion.rs:40, crates/engine/src/religion.rs:55, crates/engine/src/religion.rs:88
- `FR-CIV-BEVY-028`
  - code: crates/server/src/ws_bridge.rs:57
  - tests: crates/server/tests/ws_smoke.rs:1513, crates/server/tests/ws_smoke.rs:1522
- `FR-CIV-BEVY-034`
  - tests: clients/bevy-ref/src/diplomacy_ui.rs:625
- `FR-CIV-BEVY-035`
  - tests: clients/bevy-ref/src/lib.rs:1973
- `FR-CIV-BEVY-036`
  - code: clients/bevy-ref/src/menus.rs:4, clients/bevy-ref/src/menus.rs:1348
- `FR-CIV-CA-011`
  - code: crates/voxel/src/fluid_ca.rs:1328, crates/voxel/src/fluid_ca.rs:1666
- `FR-CIV-CARAVAN-001`
  - code: crates/engine/src/caravan.rs:3
- `FR-CIV-CLIENT-006`
  - code: crates/civis-mcp/src/server.rs:346, crates/civis-mcp/src/server.rs:2264, crates/server/src/jsonrpc.rs:82
- `FR-CIV-CLIENT-011`
  - code: clients/bevy-ref/src/tutorial.rs:3
- `FR-CIV-CLIENT-013`
  - code: clients/bevy-ref/src/civ_history.rs:2
- `FR-CIV-CLIMATE-1`
  - tests: crates/planet/src/seasonal.rs:181
- `FR-CIV-CLIMATE-2`
  - tests: crates/planet/src/seasonal.rs:193
- `FR-CIV-CLIMATE-3`
  - tests: crates/planet/src/seasonal.rs:209
- `FR-CIV-CLIMATE-4`
  - tests: crates/planet/src/seasonal.rs:218
- `FR-CIV-COHESION-001`
  - code: crates/engine/src/engine/social_settlement_phases.rs:272
- `FR-CIV-CONSTRUCTION-001`
  - code: crates/engine/src/engine.rs:459, crates/engine/src/engine.rs:2175
  - tests: crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4, crates/engine/tests/persistence_replay_coverage.rs:13
- `FR-CIV-CONTENT-001`
  - code: crates/engine/src/emergence_coupling.rs:471
- `FR-CIV-CORE-021`
  - tests: crates/build/tests/fr_matrix_batch12.rs:765
- `FR-CIV-CULTURE-001`
  - code: crates/engine/src/engine.rs:471, crates/engine/src/engine.rs:2183
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:2
- `FR-CIV-DIPLO-003-006`
  - tests: crates/diplomacy/src/shadow_networks.rs:660
- `FR-CIV-DIPLO-003-01`
  - tests: crates/diplomacy/src/shadow_networks.rs:403, crates/diplomacy/src/shadow_networks.rs:405, crates/diplomacy/src/shadow_networks.rs:433
- `FR-CIV-DIPLO-003-02`
  - tests: crates/diplomacy/src/shadow_networks.rs:470, crates/diplomacy/src/shadow_networks.rs:472, crates/diplomacy/src/shadow_networks.rs:509
- `FR-CIV-DIPLO-003-03`
  - tests: crates/diplomacy/src/shadow_networks.rs:520, crates/diplomacy/src/shadow_networks.rs:522, crates/diplomacy/src/shadow_networks.rs:560
- `FR-CIV-DIPLO-003-04`
  - tests: crates/diplomacy/src/shadow_networks.rs:594, crates/diplomacy/src/shadow_networks.rs:596
- `FR-CIV-DIPLO-003-05`
  - tests: crates/diplomacy/src/shadow_networks.rs:620, crates/diplomacy/src/shadow_networks.rs:622
- `FR-CIV-DIPLO-003-06`
  - tests: crates/diplomacy/src/shadow_networks.rs:658
- `FR-CIV-DIPLO-003-07`
  - tests: crates/diplomacy/src/shadow_networks.rs:710, crates/diplomacy/src/shadow_networks.rs:712
- `FR-CIV-DIPLOMACY-001`
  - code: crates/engine/src/engine.rs:2161, crates/engine/src/engine.rs:2336
  - tests: crates/engine/tests/diplomacy_flow.rs:20, crates/engine/tests/persistence_replay_coverage.rs:15
- `FR-CIV-DIPLOMACY-004`
  - code: crates/diplomacy/src/stance.rs:1, crates/engine/src/engine.rs:433, crates/engine/src/engine.rs:996
- `FR-CIV-ECON-010`
  - code: crates/engine/src/engine.rs:525, crates/engine/src/engine.rs:716, crates/engine/src/engine.rs:2193
  - tests: crates/engine/tests/riot_migrant_taxation_persistence.rs:3
- `FR-CIV-ECON-FOCUS-001`
  - code: crates/engine/src/engine.rs:465, crates/engine/src/engine.rs:2176
  - tests: crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4, crates/engine/tests/persistence_replay_coverage.rs:14
- `FR-CIV-EMERGE-DASH-001`
  - code: clients/bevy-ref/src/emergence_dashboard.rs:3
- `FR-CIV-EMERGENCE-RELIGION-1`
  - code: docs/design/RELIGION_EMERGENCE.md:243
- `FR-CIV-EMERGENCE-RELIGION-2`
  - code: docs/design/RELIGION_EMERGENCE.md:456
- `FR-CIV-EMERGENT-MIGRATION-001`
  - code: crates/engine/src/emergent_migration.rs:1, crates/engine/src/emergent_migration.rs:25
- `FR-CIV-ERA-001`
  - code: crates/engine/src/engine.rs:531
  - tests: crates/engine/tests/era_emergence_significance_persistence.rs:2
- `FR-CIV-FAMINE-001`
  - code: crates/engine/src/famine.rs:1
- `FR-CIV-FEST-001`
  - code: crates/engine/src/festivals.rs:1
- `FR-CIV-GAME-001`
  - code: clients/bevy-ref/src/gameplay_hud.rs:3, clients/bevy-ref/src/lib.rs:541, clients/bevy-ref/src/outcome_overlay.rs:3
- `FR-CIV-GAME-002`
  - code: clients/bevy-ref/src/god_panel.rs:2, crates/engine/src/gameplay.rs:2, crates/engine/src/scenario.rs:140
  - tests: crates/engine/src/gameplay.rs:888
- `FR-CIV-GAME-003`
  - code: clients/bevy-ref/src/era_hud.rs:2, crates/engine/src/era.rs:1
- `FR-CIV-GENETICS-SEED-001`
  - code: crates/engine/src/engine/engine_tests.rs:2907
- `FR-CIV-GENETICS-SEED-002`
  - code: crates/engine/src/engine/engine_tests.rs:2957
- `FR-CIV-GENETICS-SEED-003`
  - code: crates/engine/src/engine/engine_tests.rs:3003
- `FR-CIV-GODTOOL-001`
  - code: crates/civis-mcp/src/server.rs:148
- `FR-CIV-GOV-003`
  - code: crates/civ-institutions/src/lib.rs:38, crates/engine/src/engine.rs:849, crates/engine/src/engine.rs:858
  - tests: crates/engine/tests/fr_civ_gov_institutions.rs:19, crates/engine/tests/fr_civ_gov_institutions.rs:132, crates/engine/tests/fr_civ_gov_institutions.rs:133
- `FR-CIV-GOV-010`
  - code: crates/engine/src/engine.rs:3424
  - tests: crates/engine/tests/fr_civ_gov_mood.rs:1
- `FR-CIV-GOV-020`
  - code: crates/engine/src/social_types.rs:5, crates/engine/src/social_types.rs:69
  - tests: crates/engine/tests/fr_civ_gov_stratification.rs:1, crates/engine/tests/fr_civ_gov_stratification.rs:28, crates/engine/tests/fr_civ_gov_stratification.rs:29
- `FR-CIV-GOV-030`
  - code: crates/engine/src/engine.rs:916, crates/engine/src/engine.rs:2929, crates/engine/src/lib.rs:197
  - tests: crates/engine/tests/fr_civ_gov_cohesion.rs:1, crates/engine/tests/fr_civ_gov_cohesion.rs:22, crates/engine/tests/fr_civ_gov_cohesion.rs:23
- `FR-CIV-GOV-100`
  - code: crates/engine/src/engine.rs:863, crates/engine/src/engine.rs:868, crates/engine/src/engine.rs:873
  - tests: crates/engine/tests/fr_emergence_quality.rs:347
- `FR-CIV-GOV-200`
  - code: crates/engine/src/engine/social_settlement_phases.rs:179
- `FR-CIV-IDEOLOGY-001`
  - code: crates/engine/src/engine.rs:479, crates/engine/src/engine.rs:2183
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:2
- `FR-CIV-INSTITUTIONS-001`
  - code: crates/engine/src/engine.rs:451, crates/engine/src/engine.rs:2175
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
  - code: crates/engine/src/engine.rs:991
- `FR-CIV-LIFE-004`
  - code: docs/design/civ-003-emergent-lifecycle.md:101
- `FR-CIV-NEEDS-DECAY-01`
  - code: crates/needs/src/decay.rs:32, crates/needs/src/decay.rs:219
  - tests: crates/needs/src/decay.rs:336, crates/needs/src/decay.rs:361, crates/needs/src/decay.rs:386
- `FR-CIV-PBR-009`
  - code: crates/voxel/src/material_pbr.rs:754
  - tests: crates/voxel/src/material_pbr.rs:1461
- `FR-CIV-PBR-010`
  - code: CHANGELOG.md:16, crates/voxel/src/material_pbr.rs:14
  - tests: crates/voxel/src/material_pbr.rs:1560, crates/voxel/src/material_pbr.rs:1562, crates/voxel/src/material_pbr.rs:1594
- `FR-CIV-PBR-011`
  - code: CHANGELOG.md:17, crates/voxel/src/atlas/gpu_atlas.rs:1, crates/voxel/src/atlas/gpu_atlas.rs:40
- `FR-CIV-PLANET-050`
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
- `FR-CIV-UNREST-001`
  - code: crates/engine/src/engine/social_settlement_phases.rs:367, crates/engine/src/engine.rs:491, crates/engine/src/engine.rs:953
  - tests: crates/engine/tests/culture_ideology_aggression_persistence.rs:3, crates/engine/tests/fr_civ_unrest_001.rs:1
- `FR-CIV-UNREST-002`
  - code: crates/engine/src/engine.rs:497, crates/engine/src/engine.rs:519, crates/engine/src/engine.rs:2193
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
  - code: crates/emergence-oracle/src/oracles/architecture.rs:1, crates/emergence-oracle/src/oracles/architecture.rs:22
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
  - code: crates/emergence-oracle/src/oracles/migration_pressure.rs:1, crates/emergence-oracle/src/oracles/migration_pressure.rs:16, crates/emergence-oracle/src/oracles/trade_flow.rs:1
  - tests: crates/emergence-oracle/src/oracles/migration_pressure.rs:54
- `FR-LANGUAGE-001`
  - code: crates/engine/src/engine/culture_phases.rs:230, crates/engine/src/engine.rs:169, crates/engine/src/engine.rs:828
- `FR-MUSIC-001`
  - tests: crates/engine/src/engine/engine_tests.rs:3698
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

