# CODE-ONLY orphans deleted — agent-D (P3 cleanup pass)

These orphan test stubs were auto-generated, contained no real source
code, and pointed at FR-IDs that have no substantive implementation. They
were deleted under slice-D Option B per the instructions ("Default to
Option B for FR-NFR-P/NFR-C/NFR-R/NFR-S etc. that look like generic
categories without substance").

## Deleted files

### FR-NFR-C-01..07 (7 stubs) — generic compliance category shells

| File | Test body content |
|------|-------------------|
| `crates/engine/tests/fr_nfr_c_01.rs` | `civ_engine::WorldState::default(); assert!(ws.tick == 0);` |
| `crates/engine/tests/fr_nfr_c_02.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_03.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_04.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_05.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_06.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_07.rs` | same stub body, different ID |

### FR-NFR-CIV-SEC-001..004 (4 stubs) — security-compliance shells

| File | Test body content |
|------|-------------------|
| `crates/engine/tests/fr_nfr_civ_sec_001.rs` | `WorldState::default(); tick==0` |
| `crates/engine/tests/fr_nfr_civ_sec_002.rs` | same stub body |
| `crates/engine/tests/fr_nfr_civ_sec_003.rs` | same stub body |
| `crates/engine/tests/fr_nfr_civ_sec_004.rs` | same stub body |

### FR-NFR-CIV-LEGENDS-PERF-01 (1 stub)

| File | Test body content |
|------|-------------------|
| `crates/engine/tests/fr_nfr_civ_legends_perf_01.rs` | `WorldState::default(); tick==0` |

## Rationale

Each deleted file contained only the boilerplate `verify_*_basic()` test
that asserts `WorldState::default().tick == 0` — a "test" that proves
nothing about the actual FR. No `spec_refs`, no real source. Continuing
to maintain them just adds noise to `cargo test --workspace` output.

## Sister IDs to investigate later (NOT in scope for this slice)

- `NFR-C-*` compliance style IDs in `docs/traceability/nfr-c-*` already
  have ADR/Plan/Research/Spec files; the `FR-NFR-C-0X` IDs were unrelated
  generic shells and not the same lineage.
- `FR-NFR-CIV-SEC-*` exist as audit table rows only. If a real compliance
  surface lands, write the spec under `docs/traceability/fr-nfr-civ-sec-*/`
  rather than recreating the stub tests.

## Reference

- Slice file: `docs/audits/P3-cleanup-slice/P3-agent-D.md`
- Build/test verification: ran `cargo build --workspace --tests` —
  see final summary commit message for outcome.
