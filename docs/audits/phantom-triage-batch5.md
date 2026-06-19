# Phantom-ID Triage — Batch 5 (2026-06-11)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs, generated 2026-06-10)
Sorted by reference count (`len(code_refs) + len(test_refs)`), filtered to **CODE-ONLY-no-spec**, taking the next **75** after skipping IDs already in batch 1–4 (**~400 rows total**).

## Verdict taxonomy
- **REAL +stub-to-civ-021** — real implementation-backed capability with no existing FR spec artifact discovered in this pass.
- **REAL** — real implementation-backed capability covered by an existing requirement spec/artifact.
- **STALE** — trace/runner artifact with no dedicated requirement implementation.
- **RENAME** — implementation maps to an existing FR ID under naming drift.

| # | FR ID | Verdict | Evidence (file:line) |
|---|-------|---------|----------------------|
| 401 | FR-CIV-DIFFUSION-014 | **REAL +stub-to-civ-021** | crates/diffusion/src/lib.rs:306 |
| 402 | FR-CIV-DIFFUSION-015 | **REAL +stub-to-civ-021** | crates/diffusion/src/lib.rs:320 |
| 403 | FR-CIV-EMERGENCE-005 | **REAL** | docs/guides/voxel-emergent-vision-and-migration.md:141 |
| 404 | FR-CIV-EMERGENCE-006 | **REAL** | docs/guides/voxel-emergent-vision-and-migration.md:142 |
| 405 | FR-CIV-ENGINE-INT-001 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2397 |
| 406 | FR-CIV-ENGINE-INT-002 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2580 |
| 407 | FR-CIV-ENGINE-INT-003 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2613 |
| 408 | FR-CIV-ENGINE-INT-005 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2755 |
| 409 | FR-CIV-ENGINE-INT-010 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2189 |
| 410 | FR-CIV-ENGINE-INT-011 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2629 |
| 411 | FR-CIV-ENGINE-INT-012 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2642 |
| 412 | FR-CIV-ENGINE-INT-013 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2707 |
| 413 | FR-CIV-ENGINE-INT-014 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2745 |
| 414 | FR-CIV-ENGINE-INT-015 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2666 |
| 415 | FR-CIV-ENGINE-REPLAY-001 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2871 |
| 416 | FR-CIV-ENGINE-REPLAY-002 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:2896 |
| 417 | FR-CIV-ENGINE-REPLAY-003 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:3042 |
| 418 | FR-CIV-ENGINE-REPLAY-004 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:3058 |
| 419 | FR-CIV-ENGINE-REPLAY-005 | **REAL +stub-to-civ-021** | crates/engine/src/engine.rs:3108 |
| 420 | FR-CIV-GENETICS-011 | **REAL +stub-to-civ-021** | crates/genetics/src/lib.rs:234 |
| 421 | FR-CIV-GENETICS-012 | **REAL +stub-to-civ-021** | crates/genetics/src/lib.rs:242 |
| 422 | FR-CIV-GEO-002 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2025 |
| 423 | FR-CIV-GEO-003 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2026 |
| 424 | FR-CIV-GEO-005 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2028 |
| 425 | FR-CIV-GEO-006 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2029 |
| 426 | FR-CIV-GEO-007 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2030 |
| 427 | FR-CIV-GEO-008 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2031 |
| 428 | FR-CIV-GEO-009 | **REAL** | docs/specs/CIV-0300-rts-ui-ux-spec.md:2032 |
| 429 | FR-CIV-GODOT-ATTACH-001 | **REAL** | docs/development-guide/fr-godot-attach.md:9 |
| 430 | FR-CIV-GODOT-ATTACH-002 | **REAL** | docs/development-guide/fr-godot-attach.md:10 |
| 431 | FR-CIV-GODOT-ATTACH-003 | **REAL** | docs/development-guide/fr-godot-attach.md:11 |
| 432 | FR-CIV-GODOT-ATTACH-004 | **REAL** | docs/development-guide/fr-godot-attach.md:12 |
| 433 | FR-CIV-GODOT-F3D0 | **REAL +stub-to-civ-021** | clients/godot-ref/rust/src/ws_frame.rs:174 |
| 434 | FR-CIV-GODOT-UX-000 | **REAL** | docs/development-guide/fr-godot-attach.md:13 |
| 435 | FR-CIV-GODTOOL-901 | **REAL** | docs/specs/requirements/FR-CIV-GODTOOL.md:12 |
| 436 | FR-CIV-INFOVIEW-902 | **REAL** | docs/design/info-views.md:218 |
| 437 | FR-CIV-INFOVIEW-903 | **REAL** | docs/design/info-views.md:219 |
| 438 | FR-CIV-INFOVIEW-904 | **REAL** | docs/design/info-views.md:220 |
| 439 | FR-CIV-INFOVIEW-905 | **REAL** | docs/design/info-views.md:221 |
| 440 | FR-CIV-INFOVIEW-906 | **REAL** | docs/design/info-views.md:222 |
| 441 | FR-CIV-INFOVIEW-915 | **REAL** | docs/design/info-views.md:110 |
| 442 | FR-CIV-INFOVIEW-916 | **REAL** | docs/design/info-views.md:111 |
| 443 | FR-CIV-INFOVIEW-917 | **REAL** | docs/design/info-views.md:112 |
| 444 | FR-CIV-INFOVIEW-918 | **REAL** | docs/design/info-views.md:113 |
| 445 | FR-CIV-INFOVIEW-919 | **REAL** | docs/design/info-views.md:114 |
| 446 | FR-CIV-INFOVIEW-921 | **REAL** | docs/design/info-views.md:116 |
| 447 | FR-CIV-INFOVIEW-930 | **REAL** | docs/design/info-views.md:224 |
| 448 | FR-CIV-INFRA-001 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:353 |
| 449 | FR-CIV-INFRA-010 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:365 |
| 450 | FR-CIV-INFRA-011 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:376 |
| 451 | FR-CIV-INFRA-020 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:385 |
| 452 | FR-CIV-INFRA-021 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:395 |
| 453 | FR-CIV-INFRA-022 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:411 |
| 454 | FR-CIV-INFRA-050 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:447 |
| 455 | FR-CIV-INFRA-060 | **REAL +stub-to-civ-021** | crates/civ-traffic/src/lib.rs:457 |
| 456 | FR-CIV-LAWS-003 | **REAL +stub-to-civ-021** | crates/laws/src/lib.rs:240 |
| 457 | FR-CIV-LAWS-004 | **REAL +stub-to-civ-021** | crates/laws/src/lib.rs:261 |
| 458 | FR-CIV-LAWS-005 | **REAL +stub-to-civ-021** | crates/laws/src/lib.rs:283 |
| 459 | FR-CIV-LEGENDS-BROWSER-09 | **REAL** | docs/design/legends-engine.md:444 |
| 460 | FR-CIV-LEGENDS-CAUSAL-06 | **REAL** | docs/design/legends-engine.md:441 |
| 461 | FR-CIV-LEGENDS-GAP-12 | **REAL** | docs/design/legends-engine.md:447 |
| 462 | FR-CIV-LEGENDS-INSPECT-08 | **REAL** | docs/design/legends-engine.md:443 |
| 463 | FR-CIV-LEGENDS-NARRATOR-13 | **REAL** | docs/design/legends-engine.md:448 |
| 464 | FR-CIV-LEGENDS-PERSIST-11 | **REAL** | docs/design/legends-engine.md:446 |
| 465 | FR-CIV-LEGENDS-PRESIM-10 | **REAL** | docs/design/legends-engine.md:445 |
| 466 | FR-CIV-LEGENDS-PRODUCER-03 | **REAL** | docs/design/legends-engine.md:438 |
| 467 | FR-CIV-LEGENDS-RESOLVE-04 | **REAL** | docs/design/legends-engine.md:439 |
| 468 | FR-CIV-LEGENDS-SIG-05 | **REAL** | docs/design/legends-engine.md:440 |
| 469 | FR-CIV-LIFE-000 | **REAL +stub-to-civ-021** | crates/needs/src/lib.rs:328 |
| 470 | FR-CIV-LIFE-011 | **REAL +stub-to-civ-021** | crates/agents/src/daily_path.rs:326 |
| 471 | FR-CIV-LIFE-012 | **REAL +stub-to-civ-021** | crates/agents/src/daily_path.rs:362 |
| 472 | FR-CIV-LIFE-013 | **REAL +stub-to-civ-021** | crates/agents/src/daily_path.rs:374 |
| 473 | FR-CIV-LIFE-014 | **REAL +stub-to-civ-021** | crates/agents/src/daily_path.rs:398 |
| 474 | FR-CIV-LIFE-015 | **REAL +stub-to-civ-021** | crates/agents/src/daily_path.rs:406 |
| 475 | FR-CIV-LIFE-016 | **REAL +stub-to-civ-021** | crates/agents/src/daily_path.rs:414 |

## Summary

- **REAL**: 37
- **REAL +stub-to-civ-021**: 38
- **STALE**: 0
- **RENAME**: 0