# Stub-test fill work plan

Total stub IDs to convert: **328** across **59** epics.

Each section is one epic. Within an epic, each stub gets a checklist the agent follows:

  1. Read the FR spec + intent + ADR
  2. Locate the implementing crate (or write the impl)
  3. Replace the placeholder test body with real FR assertions
  4. Remove the `Stub: TDD-red` marker line
  5. Run `cargo test -p <crate>` and confirm green
  6. Commit `test(<crate>): real assertions for FR-XYZ-NNN`

## Epic summary

| Epic | Stubs |
|------|------:|
| FR-CIV | 10 |
| FR-CIV-3D | 15 |
| FR-CIV-AI | 5 |
| FR-CIV-ARCH | 1 |
| FR-CIV-BRUSH | 13 |
| FR-CIV-CORE | 12 |
| FR-CIV-CORE-DET | 3 |
| FR-CIV-DET | 1 |
| FR-CIV-INFOVIEW | 11 |
| FR-CIV-LANG | 5 |
| FR-CIV-LEGENDS-BROWSER | 1 |
| FR-CIV-LEGENDS-CAUSAL | 1 |
| FR-CIV-LEGENDS-GAP | 1 |
| FR-CIV-LEGENDS-INSPECT | 1 |
| FR-CIV-LEGENDS-NARRATOR | 1 |
| FR-CIV-LEGENDS-PERSIST | 1 |
| FR-CIV-LEGENDS-PRESIM | 1 |
| FR-CIV-LEGENDS-PRODUCER | 1 |
| FR-CIV-LEGENDS-RESOLVE | 1 |
| FR-CIV-LEGENDS-SIG | 1 |
| FR-CIV-LLM | 6 |
| FR-CIV-MARKET | 8 |
| FR-CIV-MCP | 4 |
| FR-CIV-MOD | 20 |
| FR-CIV-PERF | 18 |
| FR-CIV-PERF-BUILD | 1 |
| FR-CIV-PERF-RT | 3 |
| FR-CIV-PERF-WEB | 1 |
| FR-CIV-POLITY | 8 |
| FR-CIV-QOL | 14 |
| FR-CIV-RTS | 13 |
| FR-CIV-RTS-NATION | 2 |
| FR-CIV-RTS-RENDER | 5 |
| FR-CIV-RTS-ZOOM | 1 |
| FR-CIV-TERRAIN | 6 |
| FR-CIV-VEHICLE | 26 |
| FR-CIV-VERIFY | 10 |
| FR-DET | 7 |
| FR-DOC | 1 |
| FR-GUARD | 2 |
| FR-INT | 1 |
| FR-MET | 1 |
| FR-METRICS | 2 |
| FR-NET | 3 |
| FR-PROT | 4 |
| FR-REP | 1 |
| FR-SESSION | 32 |
| FR-SOC-CIV | 2 |
| FR-SOC-COH | 4 |
| FR-SOC-DET | 2 |
| FR-SOC-FAC | 2 |
| FR-SOC-HLT | 5 |
| FR-SOC-IDE | 6 |
| FR-SOC-INS | 7 |
| FR-SOC-INT | 4 |
| FR-SOC-INTG | 7 |
| FR-STOR | 1 |
| FR-TEST | 1 |
| FR-VAL | 1 |

## FR-CIV (10)

### FR-CIV-0104-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_001.rs`

### FR-CIV-0104-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_002.rs`

### FR-CIV-0104-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_003.rs`

### FR-CIV-0104-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_004.rs`

### FR-CIV-0104-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_005.rs`

### FR-CIV-0104-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_006.rs`

### FR-CIV-0104-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_007.rs`

### FR-CIV-0104-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_008.rs`

### FR-CIV-0104-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-009/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_009.rs`

### FR-CIV-0104-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-0104-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_0104_010.rs`

## FR-CIV-3D (15)

### FR-CIV-3D-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_001.rs`

### FR-CIV-3D-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_002.rs`

### FR-CIV-3D-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_003.rs`

### FR-CIV-3D-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_004.rs`

### FR-CIV-3D-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_005.rs`

### FR-CIV-3D-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_006.rs`

### FR-CIV-3D-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_007.rs`

### FR-CIV-3D-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_008.rs`

### FR-CIV-3D-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-009/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_009.rs`

### FR-CIV-3D-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_010.rs`

### FR-CIV-3D-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-011/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_011.rs`

### FR-CIV-3D-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-012/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_012.rs`

### FR-CIV-3D-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-013/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_013.rs`

### FR-CIV-3D-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-014/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_014.rs`

### FR-CIV-3D-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-3d-015/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_3d_015.rs`

## FR-CIV-AI (5)

### FR-CIV-AI-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-ai-011/`
- stub file(s):
  - `crates/ai/tests/fr_fr_civ_ai_011.rs`

### FR-CIV-AI-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-ai-012/`
- stub file(s):
  - `crates/ai/tests/fr_fr_civ_ai_012.rs`

### FR-CIV-AI-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-ai-013/`
- stub file(s):
  - `crates/ai/tests/fr_fr_civ_ai_013.rs`

### FR-CIV-AI-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-ai-014/`
- stub file(s):
  - `crates/ai/tests/fr_fr_civ_ai_014.rs`

### FR-CIV-AI-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-ai-015/`
- stub file(s):
  - `crates/ai/tests/fr_fr_civ_ai_015.rs`

## FR-CIV-ARCH (1)

### FR-CIV-ARCH-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-arch-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_arch_006.rs`

## FR-CIV-BRUSH (13)

### FR-CIV-BRUSH-01

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-01/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_01.rs`

### FR-CIV-BRUSH-02

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-02/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_02.rs`

### FR-CIV-BRUSH-03

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-03/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_03.rs`

### FR-CIV-BRUSH-04

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-04/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_04.rs`

### FR-CIV-BRUSH-05

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-05/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_05.rs`

### FR-CIV-BRUSH-06

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-06/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_06.rs`

### FR-CIV-BRUSH-07

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-07/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_07.rs`

### FR-CIV-BRUSH-08

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-08/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_08.rs`

### FR-CIV-BRUSH-09

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-09/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_09.rs`

### FR-CIV-BRUSH-10

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-10/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_10.rs`

### FR-CIV-BRUSH-11

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-11/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_11.rs`

### FR-CIV-BRUSH-12

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-12/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_12.rs`

### FR-CIV-BRUSH-13

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-brush-13/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_brush_13.rs`

## FR-CIV-CORE (12)

### FR-CIV-CORE-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_006.rs`

### FR-CIV-CORE-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_007.rs`

### FR-CIV-CORE-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_008.rs`

### FR-CIV-CORE-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-009/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_009.rs`

### FR-CIV-CORE-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_010.rs`

### FR-CIV-CORE-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-011/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_011.rs`

### FR-CIV-CORE-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-012/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_012.rs`

### FR-CIV-CORE-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-014/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_014.rs`

### FR-CIV-CORE-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-015/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_015.rs`

### FR-CIV-CORE-016

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-016/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_016.rs`

### FR-CIV-CORE-017

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-017/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_017.rs`

### FR-CIV-CORE-018

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-018/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_018.rs`

## FR-CIV-CORE-DET (3)

### FR-CIV-CORE-DET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-det-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_det_001.rs`

### FR-CIV-CORE-DET-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-det-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_det_002.rs`

### FR-CIV-CORE-DET-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-core-det-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_core_det_003.rs`

## FR-CIV-DET (1)

### FR-CIV-DET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-det-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_det_001.rs`

## FR-CIV-INFOVIEW (11)

### FR-CIV-INFOVIEW-902

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-902/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_902.rs`

### FR-CIV-INFOVIEW-903

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-903/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_903.rs`

### FR-CIV-INFOVIEW-904

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-904/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_904.rs`

### FR-CIV-INFOVIEW-906

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-906/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_906.rs`

### FR-CIV-INFOVIEW-915

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-915/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_915.rs`

### FR-CIV-INFOVIEW-916

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-916/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_916.rs`

### FR-CIV-INFOVIEW-917

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-917/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_917.rs`

### FR-CIV-INFOVIEW-918

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-918/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_918.rs`

### FR-CIV-INFOVIEW-919

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-919/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_919.rs`

### FR-CIV-INFOVIEW-921

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-921/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_921.rs`

### FR-CIV-INFOVIEW-930

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-infoview-930/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_infoview_930.rs`

## FR-CIV-LANG (5)

### FR-CIV-LANG-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-lang-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_lang_004.rs`

### FR-CIV-LANG-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-lang-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_lang_006.rs`

### FR-CIV-LANG-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-lang-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_lang_007.rs`

### FR-CIV-LANG-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-lang-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_lang_008.rs`

### FR-CIV-LANG-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-lang-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_lang_010.rs`

## FR-CIV-LEGENDS-BROWSER (1)

### FR-CIV-LEGENDS-BROWSER-09

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-browser-09/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_browser_09.rs`

## FR-CIV-LEGENDS-CAUSAL (1)

### FR-CIV-LEGENDS-CAUSAL-06

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-causal-06/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_causal_06.rs`

## FR-CIV-LEGENDS-GAP (1)

### FR-CIV-LEGENDS-GAP-12

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-gap-12/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_gap_12.rs`

## FR-CIV-LEGENDS-INSPECT (1)

### FR-CIV-LEGENDS-INSPECT-08

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-inspect-08/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_inspect_08.rs`

## FR-CIV-LEGENDS-NARRATOR (1)

### FR-CIV-LEGENDS-NARRATOR-13

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-narrator-13/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_narrator_13.rs`

## FR-CIV-LEGENDS-PERSIST (1)

### FR-CIV-LEGENDS-PERSIST-11

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-persist-11/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_persist_11.rs`

## FR-CIV-LEGENDS-PRESIM (1)

### FR-CIV-LEGENDS-PRESIM-10

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-presim-10/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_presim_10.rs`

## FR-CIV-LEGENDS-PRODUCER (1)

### FR-CIV-LEGENDS-PRODUCER-03

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-producer-03/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_producer_03.rs`

## FR-CIV-LEGENDS-RESOLVE (1)

### FR-CIV-LEGENDS-RESOLVE-04

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-resolve-04/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_resolve_04.rs`

## FR-CIV-LEGENDS-SIG (1)

### FR-CIV-LEGENDS-SIG-05

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-legends-sig-05/`
- stub file(s):
  - `crates/legends/tests/fr_fr_civ_legends_sig_05.rs`

## FR-CIV-LLM (6)

### FR-CIV-LLM-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-llm-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_llm_001.rs`

### FR-CIV-LLM-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-llm-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_llm_002.rs`

### FR-CIV-LLM-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-llm-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_llm_003.rs`

### FR-CIV-LLM-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-llm-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_llm_004.rs`

### FR-CIV-LLM-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-llm-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_llm_005.rs`

### FR-CIV-LLM-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-llm-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_llm_006.rs`

## FR-CIV-MARKET (8)

### FR-CIV-MARKET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-001/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_001.rs`

### FR-CIV-MARKET-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-002/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_002.rs`

### FR-CIV-MARKET-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-003/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_003.rs`

### FR-CIV-MARKET-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-004/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_004.rs`

### FR-CIV-MARKET-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-005/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_005.rs`

### FR-CIV-MARKET-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-006/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_006.rs`

### FR-CIV-MARKET-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-007/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_007.rs`

### FR-CIV-MARKET-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-market-008/`
- stub file(s):
  - `crates/economy/tests/fr_fr_civ_market_008.rs`

## FR-CIV-MCP (4)

### FR-CIV-MCP-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mcp-002/`
- stub file(s):
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs`

### FR-CIV-MCP-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mcp-004/`
- stub file(s):
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs`

### FR-CIV-MCP-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mcp-005/`
- stub file(s):
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs`

### FR-CIV-MCP-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mcp-006/`
- stub file(s):
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs`

## FR-CIV-MOD (20)

### FR-CIV-MOD-000

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-000/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_000.rs`

### FR-CIV-MOD-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-002/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_002.rs`

### FR-CIV-MOD-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-003/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_003.rs`

### FR-CIV-MOD-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-004/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_004.rs`

### FR-CIV-MOD-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-005/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_005.rs`

### FR-CIV-MOD-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-006/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_006.rs`

### FR-CIV-MOD-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-007/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_007.rs`

### FR-CIV-MOD-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-008/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_008.rs`

### FR-CIV-MOD-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-009/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_009.rs`

### FR-CIV-MOD-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-010/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_010.rs`

### FR-CIV-MOD-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-011/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_011.rs`

### FR-CIV-MOD-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-012/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_012.rs`

### FR-CIV-MOD-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-013/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_013.rs`

### FR-CIV-MOD-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-014/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_014.rs`

### FR-CIV-MOD-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-015/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_015.rs`

### FR-CIV-MOD-016

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-016/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_016.rs`

### FR-CIV-MOD-017

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-017/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_017.rs`

### FR-CIV-MOD-018

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-018/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_018.rs`

### FR-CIV-MOD-019

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-019/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_019.rs`

### FR-CIV-MOD-020

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-mod-020/`
- stub file(s):
  - `crates/mod-host/tests/fr_fr_civ_mod_020.rs`

## FR-CIV-PERF (18)

### FR-CIV-PERF-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_002.rs`

### FR-CIV-PERF-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_003.rs`

### FR-CIV-PERF-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_004.rs`

### FR-CIV-PERF-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_005.rs`

### FR-CIV-PERF-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_006.rs`

### FR-CIV-PERF-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_007.rs`

### FR-CIV-PERF-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_008.rs`

### FR-CIV-PERF-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-009/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_009.rs`

### FR-CIV-PERF-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-011/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_011.rs`

### FR-CIV-PERF-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-012/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_012.rs`

### FR-CIV-PERF-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-013/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_013.rs`

### FR-CIV-PERF-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-014/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_014.rs`

### FR-CIV-PERF-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-015/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_015.rs`

### FR-CIV-PERF-016

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-016/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_016.rs`

### FR-CIV-PERF-017

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-017/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_017.rs`

### FR-CIV-PERF-018

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-018/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_018.rs`

### FR-CIV-PERF-019

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-019/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_019.rs`

### FR-CIV-PERF-020

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-020/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_020.rs`

## FR-CIV-PERF-BUILD (1)

### FR-CIV-PERF-BUILD-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-build-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_build_001.rs`

## FR-CIV-PERF-RT (3)

### FR-CIV-PERF-RT-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-rt-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_rt_001.rs`

### FR-CIV-PERF-RT-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-rt-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_rt_002.rs`

### FR-CIV-PERF-RT-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-rt-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_rt_003.rs`

## FR-CIV-PERF-WEB (1)

### FR-CIV-PERF-WEB-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-perf-web-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_perf_web_001.rs`

## FR-CIV-POLITY (8)

### FR-CIV-POLITY-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_001.rs`

### FR-CIV-POLITY-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_002.rs`

### FR-CIV-POLITY-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_003.rs`

### FR-CIV-POLITY-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_004.rs`

### FR-CIV-POLITY-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_005.rs`

### FR-CIV-POLITY-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_006.rs`

### FR-CIV-POLITY-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_007.rs`

### FR-CIV-POLITY-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-polity-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_polity_008.rs`

## FR-CIV-QOL (14)

### FR-CIV-QOL-100

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-100/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_100.rs`

### FR-CIV-QOL-110

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-110/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_110.rs`

### FR-CIV-QOL-120

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-120/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_120.rs`

### FR-CIV-QOL-130

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-130/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_130.rs`

### FR-CIV-QOL-140

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-140/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_140.rs`

### FR-CIV-QOL-150

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-150/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_150.rs`

### FR-CIV-QOL-160

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-160/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_160.rs`

### FR-CIV-QOL-170

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-170/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_170.rs`

### FR-CIV-QOL-180

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-180/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_180.rs`

### FR-CIV-QOL-190

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-190/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_190.rs`

### FR-CIV-QOL-200

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-200/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_200.rs`

### FR-CIV-QOL-210

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-210/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_210.rs`

### FR-CIV-QOL-220

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-220/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_220.rs`

### FR-CIV-QOL-230

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-qol-230/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_qol_230.rs`

## FR-CIV-RTS (13)

### FR-CIV-RTS-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_003.rs`

### FR-CIV-RTS-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_004.rs`

### FR-CIV-RTS-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_005.rs`

### FR-CIV-RTS-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_006.rs`

### FR-CIV-RTS-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_007.rs`

### FR-CIV-RTS-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_008.rs`

### FR-CIV-RTS-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-009/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_009.rs`

### FR-CIV-RTS-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_010.rs`

### FR-CIV-RTS-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-011/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_011.rs`

### FR-CIV-RTS-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-012/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_012.rs`

### FR-CIV-RTS-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-013/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_013.rs`

### FR-CIV-RTS-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-014/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_014.rs`

### FR-CIV-RTS-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-015/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_015.rs`

## FR-CIV-RTS-NATION (2)

### FR-CIV-RTS-NATION-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-nation-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_nation_001.rs`

### FR-CIV-RTS-NATION-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-nation-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_nation_002.rs`

## FR-CIV-RTS-RENDER (5)

### FR-CIV-RTS-RENDER-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-render-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_render_001.rs`

### FR-CIV-RTS-RENDER-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-render-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_render_002.rs`

### FR-CIV-RTS-RENDER-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-render-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_render_003.rs`

### FR-CIV-RTS-RENDER-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-render-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_render_004.rs`

### FR-CIV-RTS-RENDER-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-render-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_render_005.rs`

## FR-CIV-RTS-ZOOM (1)

### FR-CIV-RTS-ZOOM-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-rts-zoom-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_rts_zoom_001.rs`

## FR-CIV-TERRAIN (6)

### FR-CIV-TERRAIN-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-terrain-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_terrain_001.rs`

### FR-CIV-TERRAIN-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-terrain-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_terrain_002.rs`

### FR-CIV-TERRAIN-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-terrain-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_terrain_003.rs`

### FR-CIV-TERRAIN-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-terrain-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_terrain_004.rs`

### FR-CIV-TERRAIN-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-terrain-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_terrain_005.rs`

### FR-CIV-TERRAIN-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-terrain-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_terrain_006.rs`

## FR-CIV-VEHICLE (26)

### FR-CIV-VEHICLE-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_001.rs`

### FR-CIV-VEHICLE-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_002.rs`

### FR-CIV-VEHICLE-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_003.rs`

### FR-CIV-VEHICLE-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_004.rs`

### FR-CIV-VEHICLE-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_005.rs`

### FR-CIV-VEHICLE-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_010.rs`

### FR-CIV-VEHICLE-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-011/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_011.rs`

### FR-CIV-VEHICLE-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-012/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_012.rs`

### FR-CIV-VEHICLE-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-013/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_013.rs`

### FR-CIV-VEHICLE-014

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-014/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_014.rs`

### FR-CIV-VEHICLE-020

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-020/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_020.rs`

### FR-CIV-VEHICLE-021

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-021/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_021.rs`

### FR-CIV-VEHICLE-022

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-022/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_022.rs`

### FR-CIV-VEHICLE-023

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-023/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_023.rs`

### FR-CIV-VEHICLE-024

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-024/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_024.rs`

### FR-CIV-VEHICLE-030

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-030/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_030.rs`

### FR-CIV-VEHICLE-040

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-040/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_040.rs`

### FR-CIV-VEHICLE-041

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-041/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_041.rs`

### FR-CIV-VEHICLE-042

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-042/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_042.rs`

### FR-CIV-VEHICLE-043

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-043/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_043.rs`

### FR-CIV-VEHICLE-044

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-044/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_044.rs`

### FR-CIV-VEHICLE-045

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-045/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_045.rs`

### FR-CIV-VEHICLE-046

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-046/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_046.rs`

### FR-CIV-VEHICLE-047

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-047/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_047.rs`

### FR-CIV-VEHICLE-050

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-050/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_050.rs`

### FR-CIV-VEHICLE-060

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-vehicle-060/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_vehicle_060.rs`

## FR-CIV-VERIFY (10)

### FR-CIV-VERIFY-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_001.rs`

### FR-CIV-VERIFY-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_002.rs`

### FR-CIV-VERIFY-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_003.rs`

### FR-CIV-VERIFY-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_004.rs`

### FR-CIV-VERIFY-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_005.rs`

### FR-CIV-VERIFY-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_006.rs`

### FR-CIV-VERIFY-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_007.rs`

### FR-CIV-VERIFY-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-008/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_008.rs`

### FR-CIV-VERIFY-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-009/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_009.rs`

### FR-CIV-VERIFY-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-civ-verify-010/`
- stub file(s):
  - `crates/engine/tests/fr_fr_civ_verify_010.rs`

## FR-DET (7)

### FR-DET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_001.rs`

### FR-DET-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_002.rs`

### FR-DET-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_003.rs`

### FR-DET-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_004.rs`

### FR-DET-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_005.rs`

### FR-DET-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_006.rs`

### FR-DET-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-det-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_det_007.rs`

## FR-DOC (1)

### FR-DOC-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-doc-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_doc_001.rs`

## FR-GUARD (2)

### FR-GUARD-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-guard-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_guard_001.rs`

### FR-GUARD-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-guard-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_guard_002.rs`

## FR-INT (1)

### FR-INT-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-int-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_int_001.rs`

## FR-MET (1)

### FR-MET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-met-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_met_001.rs`

## FR-METRICS (2)

### FR-METRICS-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-metrics-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_metrics_004.rs`

### FR-METRICS-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-metrics-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_metrics_005.rs`

## FR-NET (3)

### FR-NET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-net-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_net_001.rs`

### FR-NET-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-net-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_net_002.rs`

### FR-NET-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-net-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_net_003.rs`

## FR-PROT (4)

### FR-PROT-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-prot-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_prot_001.rs`

### FR-PROT-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-prot-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_prot_002.rs`

### FR-PROT-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-prot-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_prot_003.rs`

### FR-PROT-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-prot-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_prot_005.rs`

## FR-REP (1)

### FR-REP-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-rep-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_rep_001.rs`

## FR-SESSION (32)

### FR-SESSION-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-001/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_001.rs`

### FR-SESSION-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-002/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_002.rs`

### FR-SESSION-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-003/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_003.rs`

### FR-SESSION-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-004/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_004.rs`

### FR-SESSION-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-005/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_005.rs`

### FR-SESSION-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-006/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_006.rs`

### FR-SESSION-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-007/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_007.rs`

### FR-SESSION-008

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-008/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_008.rs`

### FR-SESSION-009

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-009/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_009.rs`

### FR-SESSION-010

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-010/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_010.rs`

### FR-SESSION-011

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-011/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_011.rs`

### FR-SESSION-012

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-012/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_012.rs`

### FR-SESSION-013

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-013/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_013.rs`

### FR-SESSION-015

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-015/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_015.rs`

### FR-SESSION-016

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-016/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_016.rs`

### FR-SESSION-017

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-017/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_017.rs`

### FR-SESSION-018

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-018/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_018.rs`

### FR-SESSION-019

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-019/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_019.rs`

### FR-SESSION-020

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-020/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_020.rs`

### FR-SESSION-021

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-021/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_021.rs`

### FR-SESSION-022

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-022/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_022.rs`

### FR-SESSION-023

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-023/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_023.rs`

### FR-SESSION-024

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-024/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_024.rs`

### FR-SESSION-025

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-025/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_025.rs`

### FR-SESSION-026

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-026/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_026.rs`

### FR-SESSION-027

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-027/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_027.rs`

### FR-SESSION-028

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-028/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_028.rs`

### FR-SESSION-029

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-029/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_029.rs`

### FR-SESSION-030

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-030/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_030.rs`

### FR-SESSION-031

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-031/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_031.rs`

### FR-SESSION-032

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-032/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_032.rs`

### FR-SESSION-033

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-session-033/`
- stub file(s):
  - `crates/server/tests/fr_fr_session_033.rs`

## FR-SOC-CIV (2)

### FR-SOC-CIV-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-civ-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_civ_001.rs`

### FR-SOC-CIV-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-civ-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_civ_002.rs`

## FR-SOC-COH (4)

### FR-SOC-COH-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-coh-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_coh_001.rs`

### FR-SOC-COH-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-coh-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_coh_002.rs`

### FR-SOC-COH-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-coh-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_coh_003.rs`

### FR-SOC-COH-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-coh-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_coh_004.rs`

## FR-SOC-DET (2)

### FR-SOC-DET-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-det-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_det_001.rs`

### FR-SOC-DET-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-det-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_det_002.rs`

## FR-SOC-FAC (2)

### FR-SOC-FAC-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-fac-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_fac_001.rs`

### FR-SOC-FAC-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-fac-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_fac_002.rs`

## FR-SOC-HLT (5)

### FR-SOC-HLT-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-hlt-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_hlt_001.rs`

### FR-SOC-HLT-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-hlt-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_hlt_002.rs`

### FR-SOC-HLT-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-hlt-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_hlt_003.rs`

### FR-SOC-HLT-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-hlt-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_hlt_004.rs`

### FR-SOC-HLT-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-hlt-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_hlt_005.rs`

## FR-SOC-IDE (6)

### FR-SOC-IDE-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ide-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ide_001.rs`

### FR-SOC-IDE-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ide-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ide_002.rs`

### FR-SOC-IDE-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ide-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ide_003.rs`

### FR-SOC-IDE-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ide-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ide_004.rs`

### FR-SOC-IDE-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ide-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ide_005.rs`

### FR-SOC-IDE-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ide-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ide_006.rs`

## FR-SOC-INS (7)

### FR-SOC-INS-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_001.rs`

### FR-SOC-INS-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_002.rs`

### FR-SOC-INS-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_003.rs`

### FR-SOC-INS-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_004.rs`

### FR-SOC-INS-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_005.rs`

### FR-SOC-INS-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_006.rs`

### FR-SOC-INS-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-ins-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_ins_007.rs`

## FR-SOC-INT (4)

### FR-SOC-INT-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-int-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_int_001.rs`

### FR-SOC-INT-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-int-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_int_002.rs`

### FR-SOC-INT-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-int-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_int_003.rs`

### FR-SOC-INT-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-int-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_int_004.rs`

## FR-SOC-INTG (7)

### FR-SOC-INTG-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_001.rs`

### FR-SOC-INTG-002

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-002/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_002.rs`

### FR-SOC-INTG-003

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-003/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_003.rs`

### FR-SOC-INTG-004

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-004/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_004.rs`

### FR-SOC-INTG-005

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-005/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_005.rs`

### FR-SOC-INTG-006

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-006/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_006.rs`

### FR-SOC-INTG-007

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-soc-intg-007/`
- stub file(s):
  - `crates/engine/tests/fr_fr_soc_intg_007.rs`

## FR-STOR (1)

### FR-STOR-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-stor-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_stor_001.rs`

## FR-TEST (1)

### FR-TEST-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-test-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_test_001.rs`

## FR-VAL (1)

### FR-VAL-001

- crate: `UNKNOWN — locate from spec`
- spec dir: `docs/traceability/fr-val-001/`
- stub file(s):
  - `crates/engine/tests/fr_fr_val_001.rs`

