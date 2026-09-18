# FR Coverage Audit — methodology notes (2026-09-18)

These notes record how the automated audit in `docs/audits/fr-matrix.json` and
`docs/audits/fr-coverage-audit-<date>.md` should be read. They exist because a
day of work found several ways the raw numbers were misleading, and the
headline count is only meaningful alongside these caveats.

## How a status is derived

`docs/audits/_gather_ids.py` scans the repo and records, per requirement ID, the
file:line references where the ID appears, bucketed into:

- **spec** — a document that states the requirement
- **trace** — `docs/traceability/**.md`
- **code** — implementing source
- **test** — a test, or a `/// Covers <ID>.` line in a file containing `#[cfg(test)]`

`scripts/traceability/gen-fr-audit.py` then classifies:

| Status | Meaning |
|---|---|
| `COVERED` | spec/trace + code + test |
| `IMPL-NO-TEST` | spec/trace + code, no test |
| `SPEC-ONLY` | spec/trace, no implementing code |
| `CODE-ONLY-no-spec` | code, no spec/trace |

## Corrections applied on 2026-09-18

1. **Requirement-stating docs were counted as implementing code.** Every
   markdown file outside a small allowlist counted as `code`, so an ID was
   reported `COVERED` when only a design/spec document mentioned it.
   707 IDs had their *only* code reference in such a document.
   `docs/models/**`, `docs/specs/**`, and `docs/design/**` are now `spec`
   sources. COVERED fell 754 → 434.

2. **In-source tests were reported as having no implementation.** Inside a
   `#[cfg(test)]` module in a crate's own `src/lib.rs`, a reference was recorded
   as a test ref only, so an ID implemented and tested in one file read as
   `SPEC-ONLY`. A non-`tests/` `.rs` source file now records a code ref too.
   COVERED rose 450 → 573; SPEC-ONLY fell 768 → 645.

3. **Tests that name an ID without verifying it were treated as coverage.**
   552 test files under `crates/**/tests/` are auto-generated placeholders with
   the doc comment "Verify `<ID>` type existence and basic behavior." and a body
   that only touches a shared type. Several are byte-identical across unrelated
   IDs:

   - `crates/ai/tests/fr_fr_civ_ai_001.rs` … `_005.rs` all assert
     `assert_eq!(SCHEMA_VERSION, 0); let _ = AiConfig::default();`
   - `crates/voxel/tests/fr_fr_civ_voxel_023.rs` and `_024.rs` are identical
   - `crates/civ-traffic/tests/fr_fr_civ_road_901.rs` and `_902.rs` are identical

   **70 IDs are `COVERED` on placeholder tests alone.** The generator now lists
   them under "Placeholder-only coverage" on every run. Read those 70 as
   unverified, not as done.

4. **An overstated matrix row.** `fr-emergence-matrix.md` listed
   `FR-CIV-ARCH-006` as `traced` against `build::graph_ron_roundtrip`, but that
   oracle labels itself `FR-CIV-BUILD-001`. Corrected to `code-only`.

## How to read the current numbers

- A `COVERED` row means the ID is mentioned by a spec, some code, and some test.
  It does **not** by itself mean the test verifies the requirement. Prefer
  checking the placeholder list first.
- `SPEC-ONLY` is not automatically "unimplemented". It means no reference to the
  ID was found in non-test source. 391 SPEC-ONLY rows still carry a test ref,
  mostly placeholder files.
- `docs/audits/fr-3d-matrix.md` agreed with the audit on all 122 of its rows.
  `fr-emergence-matrix.md` and `fr-nfr-matrix.md` use their own vocabulary
  (`traced` / `code-only` / `stub` / `dormant` / `alias`) and are hand-maintained
  documents that predate this audit, so row-level disagreements between them and
  this file are expected and are not automatically errors.

## Highest-value next work

1. **Replace placeholder tests with real oracles.** 70 IDs are reported covered
   on tests that verify nothing, and ~320 more carry a placeholder alongside
   real tests. Each placeholder is a specific, small task: assert the behaviour
   the requirement describes.
2. **Triage the SPEC-ONLY set for genuinely unimplemented requirements.** At
   present these are the honest backlog of stated-but-unbuilt work.
