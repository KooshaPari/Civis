# Campaign Phase 2 — Recommendation (2026-06-11)

**Source:** rerun-4 audit (`docs/audits/fr-matrix.json`, `docs/audits/fr-matrix-2026-06-11.md`).
**Context:** BUILD-NEXT drained (15 slices shipped, COVERED 26 → 135, rerun-3 2026-06-10).
**Residual matrix:** 130 IMPL-NO-TEST, 323 SPEC-ONLY, 633 CODE-ONLY-no-spec (1221 IDs).
**Goal:** rank the next 3 multi-slice efforts (not single PRs) by **player-visible value**
using the `docs/audits/parity-benchmark-2026-06-10.md` top-20 impact scoring
(audience + session-blocker + word-of-mouth, ≤15) and the matrix gap buckets.

---

## 1. Ship the merged slice-10..15 (cheap COVERED harvest) — 1 multi-slice push

- **What:** Land `feat/build-next-10..15` (ECON-015 chain, EMERG 5-tile dashboard, AUDIO
  triggers/ui_sound/ducking, PBR policy substrate, DIPLO-008 counter + SAVE browser,
  LEGENDS-001/002/005/006 HistoricalEvent/Chronicle/query). One PR per slice, no rewrite.
- **Why top:** No new code needed — it's all already written on side branches; a merge
  campaign moves SPEC-ONLY → COVERED for ~30 IDs in 1–2 days and ends the "audit is
  ahead of main" skew (the rerun-3 commit is currently unreachable from origin/main).
- **Player impact:** Indirect but high — unlocks the EMERG dashboard in the watch/web UI,
  makes audio audible in-game, exposes diplomacy counter + save browser, and ships the
  Legends chronicle view. Lowest recoverable cost, highest delta-per-PR.

## 2. Close the IMPL-NO-TEST top-3 epics (WAR / BUILD / BEVY) — test-only PRs

- **What:** Add `crates/tactics/tests/` coverage for `FR-CIV-WAR-001..004/010/020` (8 IDs),
  `crates/build/tests/` for `FR-CIV-BUILD-001/002/003/010/020/030` (7 IDs), and
  `clients/bevy-ref/tests/` for `FR-CIV-BEVY-016/022/023/024/025/026` (6 IDs). Spec + code
  already exist; only `/// Covers: FR-…` doc-comments and a few assert/bench calls needed.
- **Why second:** 21 of the 130 IMPL-NO-TESTs (16%) collapse with 3 test PRs. Per the
  `coverage-baseline-2026-06-10.md` pattern, these are the fastest COVERED gains and
  zero design risk (everything is already implemented and committed).
- **Player impact:** Medium — these are simulation/UI plumbing (combat resolution,
  building-tier validation, Bevy attach surface), not headline features, but they
  harden the loop that runs every tick.

## 3. Promote parity top-3 epics (PBR, INFRA/econ-chain, AUDIO adaptive) — multi-slice

- **What:** Three multi-slice epics, ranked by `parity-benchmark-2026-06-10.md` top-20
  player-impact score and matrix readiness:
  1. **Modern GFX (score 14)** — `FR-CIV-PBR-001..008` SPEC-ONLY → impl + tests +
     `FR-CIV-POSTFX-001..004` new (GI/SSR/volumetric/DoF). 3 slices minimum, each a
     standalone PR. Closes the "behind Manor Lords on visuals" gap.
  2. **City-scale traffic & economy chain (score 15)** — promote `crates/civ-traffic/`
     lane code to a multi-hop production-chain UI; new `FR-CIV-ECON-CHAIN-001..010`.
     4 slices (data model → routing → UI panel → benchmark). Closes the "behind CS2
     on city-sim" gap, the single highest-impact gap in the parity benchmark.
  3. **Adaptive music + dynamic mix (score 12)** — extend `crates/audio/` (just shipped
     triggers/ui_sound/ducking in slice 11) with mood-keyed music bus. 2 slices.
     Closes the "behind CS2/Manor Lords on audio" gap and is the cheapest of the top-10
     because the audio substrate is already in tree.
- **Why third:** PBR/INFRA/AUDIO are the only top-20 parity rows where the matrix is
  already at SPEC-ONLY or IMPL-NO-TEST *with the substrate shipped* — they need
  *feature work on top of existing crates*, not greenfield. Other top-20 rows (god-
  powers #4, storyteller #9, MP #19) need net-new design + spec first, so they
  belong in Phase 3.

---

**Top single recommendation:** **#1 — land the slice-10..15 merges.** It is the only
item that simultaneously (a) updates the committed audit, (b) moves the COVERED count
into the 160–170 band without writing code, and (c) unblocks the slice-11..15
features that the parity top-3 (#3 above) all depend on. It is the highest-leverage
single action available in the residual matrix.
