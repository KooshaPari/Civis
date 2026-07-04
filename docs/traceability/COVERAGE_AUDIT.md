# Requirements Coverage Audit

**Date:** 2026-06-23
**Branch:** `research/traceability-audit`
**Scope:** All `FR-*` and `NFR-*` tags found in `agileplus-specs/`, `crates/`, and `docs/`.
**Read-only:** under `C:/Users/koosh/Dev/Civis/.worktrees/wt-trace` only; no other-branch checkout; no `cargo` invoked.

---

## Method

1. Enumerated every `FR-` and `NFR-` token in `agileplus-specs/`, `crates/`, and `docs/`
   with `grep -rohE 'FR-[A-Z]+(-[A-Z]+)*-[0-9]+'` and the analogous NFR regex.
2. Deduplicated (1 201 unique `FR-` IDs; 34 unique `NFR-` IDs).
3. **Traced** = appears in any file under `docs/traceability/`
   (`fr-3d-matrix.md`, `fr-web-matrix.md`, `full-traceability-matrix.md`,
   `TRACEABILITY_MATRIX.md`, `index.md`, `civis-tracelinks.md`,
   `emergent-systems-tracelinks.md`).
4. **Untraced** = has an `FR-` ID in code or spec comments but no row in any
   official traceability matrix in `docs/traceability/`.

| Bucket | Count |
|--------|------:|
| Total unique `FR-` IDs in code + specs | **1 201** |
| Traced (in `docs/traceability/`) | **255** |
| Untraced (in code/specs but not in any trace matrix) | **946** |
| Total unique `NFR-` IDs in code + specs | 34 |
| NFRs traced | 0 (no NFR rows in any matrix today) |

The 946 untraced IDs are dominated by families that are implemented in
`crates/` with full code but were never carried into a matrix
(`FR-CIV-SPECIES` 51, `FR-SOC` 39, `FR-SESSION` 33, `FR-CIV-PERF` 30,
`FR-CIV-PSYCHE` 29, `FR-CIV-VEHICLE` 26, `FR-CIV-LEGENDS` 25,
`FR-CIV-ASSET` 23, `FR-CIV-CORE` 23, `FR-CIV-RTS` 23, …).

The brief's emphasis is **emergence systems** (language, faction, religion,
trade, architecture, climate, economy, demographics) and **dormant phases**
(FR-CIV-PSYCHE, FR-CIV-LEGENDS, FR-CIV-CULT, FR-CIV-SOCIAL, FR-CIV-DIPLO,
FR-CIV-LAWS, FR-CIV-AI).  Those are the focus of the proposed-FR-ID
section below; the rest of the 946 are enumerated by family-prefix in
**§4 Other untraced families**.

---

## Traced features

The following official matrices in `docs/traceability/` carry rows for
`FR-` IDs.  All 255 traced IDs are the union of rows in these files.

| File | Style | Coverage |
|------|-------|----------|
| [`TRACEABILITY_MATRIX.md`](TRACEABILITY_MATRIX.md) | Strategic — `FR-CORE-*`, `FR-ECON-*`, etc. | legacy core+economy |
| [`fr-3d-matrix.md`](fr-3d-matrix.md) | 3D extension — `FR-CIV-{VOXEL,BUILD,GENETICS,SPECIES,AGENTS,DIFFUSION,LAWS,RESEARCH,TACTICS,BEVY,PLANET,PROTO3D,UX}*` | 3D workspace |
| [`fr-web-matrix.md`](fr-web-matrix.md) | Web spectator — `FR-CIV-WEB-*` | `web/dashboard` |
| [`full-traceability-matrix.md`](full-traceability-matrix.md) | Roll-up snapshot | consolidated |
| [`civis-tracelinks.md`](civis-tracelinks.md) | Per-commit `FR-CIV-LIFE-*` style IDs by commit | by-commit history |
| [`emergent-systems-tracelinks.md`](emergent-systems-tracelinks.md) | Emergence DAG — `FR-CIV-0100` §3 + `phase_*` | emergence-only |
| [`index.md`](index.md) | Hub index, no rows of its own | n/a |

The 255 traced IDs are split roughly:

- 116 in `fr-3d-matrix.md` (3D extension rows, all `implemented`)
- the rest in `TRACEABILITY_MATRIX.md`, `full-traceability-matrix.md`,
  and the emergence ledger.

---

## Untraced features — focus on emergence systems and dormant phases

Per the brief, the following are the **emergence systems** and **dormant
phases** that have full source-code support (compile-time present in
`crates/`) but no row in any official trace matrix.  For each row I
propose a FR-ID scheme that **re-uses the existing in-code family prefix**
so the proposal is non-breaking and grep-friendly.  I also flag the
emergence-charter umbrella `FR-CIV-0100` (which is in fact itself
untraced today — `FR-CIV-0100` §3 emergence should be the first row of
the emergence ledger) as the binding requirement for dormant phases.

### §3.1 Emergence systems called out in the brief

| System | Existing in-code IDs (untraced) | Implementing code (non-exhaustive) | Proposed FR-ID | Notes |
|--------|--------------------------------|------------------------------------|----------------|-------|
| **Language** | `FR-CIV-LANG-001..010` (+ `LANG-9`) | `crates/agents/src/social.rs` (phoneme drift) | **Promote as-is → `FR-CIV-LANG-*` rows in new `fr-emergence-matrix.md`** | dormant phase: phoneme-drift functions exist in `crates/agents/src/social.rs` but no row in any matrix; charter `FR-CIV-0100` §3 emergence is umbrella |
| **Faction / Polity** | `FR-CIV-POLITY-001..008` | `crates/agents/src/cluster.rs` (`phase_emergence` → polity clustering) | **Promote `FR-CIV-POLITY-*`** | faction emergence is computed but no trace row; umbrella `FR-CIV-0100` |
| **Religion** | `FR-CIV-REL-001..004`, `FR-CIV-RELIGION-002` | `crates/agents/src/cluster.rs` (belief) | **Promote `FR-CIV-REL-*` and `FR-CIV-RELIGION-*`** | belief phase is wired; religion phase dormant |
| **Trade** | `FR-CIV-MARKET-001..008` | `crates/economy/src/market.rs` (`apply_pressure`) | **Promote `FR-CIV-MARKET-*`** | market pricing wired (`emergent-systems-tracelinks.md` §3d); eight trade-mix IDs in code untraced |
| **Architecture** | `FR-CIV-ARCH-001..008` (+ `ARCH-NOSVG-001`) | `crates/build/` (`BuildingGraph` era-grammar, freehand) | **Promote `FR-CIV-ARCH-*`** | era-grammar histograms exist; no trace row |
| **Climate** | `FR-CIV-CLIMATE-001..003` (+ partial `FR-CIV-PLANET-003..060`) | `crates/planet/` (`phase_disasters`, `wildfire_ignition_temp_fp`) | **Promote `FR-CIV-CLIMATE-*`; new `FR-CIV-PLANET-3..60` rows** | climate-disasters wired; only `FR-CIV-PLANET-000..002` traced today |
| **Economy** | `FR-CIV-ECON-001..004,015`, `FR-CIV-ECON-CHAIN-001`, `FR-CIV-ECON-VIZ-001`, `FR-ECO-001..010` | `crates/economy/` (`phase_economy`, `market`, `joule`) | **Promote `FR-CIV-ECON-*` and `FR-ECO-*`** | strategic `FR-ECON-*` is in `TRACEABILITY_MATRIX.md`; the `FR-CIV-ECON-*` and `FR-ECO-*` shadow-IDs are untraced |
| **Demographics** | `FR-CIV-LIFE-004,011..016,021..025,035` (13) + `FR-CIV-ACT-001,003,004,005` (4) | `crates/agents/src/cluster.rs` (life/demography) | **Promote `FR-CIV-LIFE-*` and `FR-CIV-ACT-*`** | demography functions present in `crates/agents/src/`; only the `FR-CIV-LIFE-*` rows in `civis-tracelinks.md` are referenced by-commit, not in a matrix |

### §3.2 Dormant-phase families (have code, no `phase_*` wiring, no trace row)

Per `crates/engine/src/emergence.rs:1-2` ("gap-audit §1, master-roadmap S2")
these are the systems that have **full source** in `crates/` but are
**not currently invoked from `Simulation::tick`**.

| Family | Untraced count | Implementing code (non-exhaustive) | Proposed FR-ID | Notes |
|--------|---------------:|------------------------------------|----------------|-------|
| `FR-CIV-PSYCHE-*` | 29 (incl. `PSYCHE-900..921`) | `crates/agents/src/psyche.rs` (OCEAN traits) | **Promote as-is → rows in new `fr-emergence-matrix.md` under emergence-charter `FR-CIV-0100` §3** | `phase_emergence` is a no-op stub; OCEAN trait computation exists but never ticks |
| `FR-CIV-LEGENDS-*` | 25 (incl. `LEGENDS-BROWSER-09`, `CAUSAL-06`, `CONFIG-04`, `GAP-12`, `GRAPH-01`, `INGEST-02`, `INSPECT-08`, `LOUD-03`, `NARRATOR-13`, `PERF-01`, `PERSIST-11`, `PRESIM-10`, `PRODUCER-03`, `QUERY-07`, `RESOLVE-04`, `SCALE-02`, `SIG-05`) | `crates/legends/` (browser, causal-graph, narrator) | **Promote as-is** | legends crate exists; subsystem IDs untraced |
| `FR-CIV-AI-*` | 15 | `crates/agents/src/ai.rs` (decision-policy) | **Promote `FR-CIV-AI-*`** | AI decision-policy code present; no `phase_ai` in tick |
| `FR-CIV-CULT-*` | 3 | `crates/diffusion/` (Bass/Rogers S-curve) | **Promote `FR-CIV-CULT-*`** | culture diffusion wired at adoption-curve layer; no cult-phase tick |
| `FR-CIV-SOCIAL-*` | 2 | `crates/agents/src/social.rs` (graph kernels) | **Promote `FR-CIV-SOCIAL-*`** | social-graph adjacency code exists; no `phase_social` in tick |
| `FR-CIV-DIPLO-*` | 8 | `crates/diplomacy/` and `crates/agents/src/diplomacy.rs` | **Promote `FR-CIV-DIPLO-*`** | diplomacy phase wired (`emergent-systems-tracelinks.md` row 5) but the eight FR-CIV-DIPLO IDs are not in any matrix |
| `FR-CIV-LAWS-*` | 6 (`LAWS-003..005,007..009`) | `crates/laws/` (LawDb) | **Promote `FR-CIV-LAWS-003..005,007..009`** | `FR-CIV-LAWS-000..002,006` are traced; the six newer IDs untraced |
| `FR-CIV-EMERG-*` | 5 (`EMERG-001..005`) | `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md` | **Promote `FR-CIV-EMERG-*`** | defined in spec `civ-019`; not in any matrix |
| `FR-CIV-EMERGENCE-*` | 10 (`EMERGENCE-001..006,010..013`) | `agileplus-specs/civ-021-recovered-requirements/spec.md` | **Promote `FR-CIV-EMERGENCE-*`** | recovered-requirements spec; not in any matrix |

### §3.3 Emergence-charter umbrella (highest priority)

`FR-CIV-0100` itself (§3 emergence) is the umbrella requirement for all
the dormant phases above.  **It is itself untraced** — it appears only in
[`emergent-systems-tracelinks.md`](emergent-systems-tracelinks.md) as a
section reference, not as a matrix row.  Proposed:

| ID | Proposed row |
|----|--------------|
| `FR-CIV-0100` | New row in `fr-3d-matrix.md` (or new `fr-emergence-matrix.md`): "Emergence charter — life/society/economy/belief/diplomacy must emerge from state with bidirectional coupling; physical/environmental/genomic law is hard-coded only." Crate: `crates/engine/src/emergence.rs`. Tests: `phase_*` regression suite. Status: `in_progress`. |
| `FR-CIV-0100-int` | Integration row: "Emergence charter end-to-end — emergent-systems-tracelinks.md rows 1–11 all `COVERED` (spec+code+test)." |
| `FR-CIV-0100-int2` | Integration row: "Couplings ledger rows 1–22 are wired in `engine.rs:tick_with_emergence_source`." |
| `FR-CIV-0100-int3` | Integration row: "Dormant phases (PSYCHE, LEGENDS, AI, CULT, SOCIAL, DIPLO, LANG, POLITY, REL, MARKET, ARCH, LIFE, ACT) are explicitly tagged `dormant` in matrix until `phase_*` wiring lands." |

### §3.4 Recovery for the "Phase 1" emergence batch

The 13 highest-priority IDs to **immediately** add to a new
`docs/traceability/fr-emergence-matrix.md` (and link from
`emergent-systems-tracelinks.md`):

```
FR-CIV-0100          (charter, see §3.3)
FR-CIV-EMERG-001..005        (civ-019 emergence-metrics dashboard)
FR-CIV-EMERGENCE-001..006    (civ-021 recovered-requirements batch A)
FR-CIV-EMERGENCE-010..013    (civ-021 recovered-requirements batch B)
```

Plus all families in §3.1 and §3.2, totalling 11 (language) + 8
(faction) + 5 (religion) + 8 (trade) + 9 (architecture) + 3 (climate)
+ 7 (economy) + 17 (demographics) + 29 (psyche) + 25 (legends) + 15
(ai) + 3 (cult) + 2 (social) + 8 (diplo) + 6 (laws) + 5 (emerg) + 10
(emergence-charter) = **171 IDs** that map 1-to-1 onto the brief's
"emergence systems + dormant phases" list.

---

## Other untraced families (count by family prefix)

The remaining 946 − 171 = 775 untraced IDs are not in the brief's
emergence/dormant focus.  They are listed by family prefix so a future
sweep can promote them.  Counts are the number of untraced IDs sharing
that prefix (≥ 1).

### Top-50 non-emergence untraced families (775 IDs total)

| Count | Family prefix | Notes |
|------:|---------------|-------|
| 51 | `FR-CIV-SPECIES` | species crate has full DNA→phenotype; only `SPECIES-000..001` traced |
| 39 | `FR-SOC` | social sub-prefixes (CIV/COH/DET/FAC/HLT/IDE/INS/INT/INTG) — separate social-internal naming |
| 33 | `FR-SESSION` | session-scoped server IDs (civ-server) |
| 30 | `FR-CIV-PERF` | performance NFRs in code (incl. `PERF-9,900..902, BUILD-001, FRAME-256, RT-001..003, WEB-001`) |
| 26 | `FR-CIV-VEHICLE` | vehicle simulation IDs |
| 25 | `FR-CIV-LEGENDS` | (dormant — see §3.2) |
| 23 | `FR-CIV-ASSET` | asset pipeline IDs |
| 23 | `FR-CIV-CORE` | core-engine IDs that overlap with strategic `FR-CORE-*` |
| 23 | `FR-CIV-RTS` | RTS-style client (incl. `RTS-NATION-001..002, RENDER-001..005, ZOOM-001`) |
| 22 | `FR-UX` | UX-level IDs (001..027) |
| 21 | `FR-CIV-MOD` | mod-host IDs (000..020) — overlap with `FR-MOD-*` |
| 21 | `FR-CIV-TECH` | technology-tree IDs (001..021) |
| 20 | `FR-SAVE` | save system (006..025) |
| 18 | `FR-CIV-INFOVIEW` | inspector (9, 902..906, 911..921, 930) |
| 16 | `FR-CIV-VOXEL` | voxel additions (`VOXEL-006..007,020..025,030..032, BRIDGE-001..003, DIRTY-001..002`) |
| 15 | `FR-CIV-AI` | (dormant — see §3.2) |
| 15 | `FR-CIV-BEVY` | Bevy reference client (001..003, 013..022, 025, 028) |
| 15 | `FR-CIV-ENGINE` | engine-internal (`ENGINE-INT-001..003,005,010..015, REPLAY-001..005`) |
| 15 | `FR-CIV-PROTO` | protocol-3d IDs (001..015) |
| 15 | `FR-CIV-WAR` | war/combat IDs (001..004, 010..013, 020..022, 030, 040..042) |
| 14 | `FR-CIV-AGENTS` | agent-IDs (002..003, 011, 020..025, 030..034) |
| 14 | `FR-CIV-DIFFUSION` | diffusion IDs (002..015) |
| 14 | `FR-CIV-QOL` | quality-of-life (100..230) |
| 14 | `FR-CIV-SCALE` | scale-multiplier (001..008, 9, 900..902, 910, 920) |
| 13 | `FR-CIV-BRUSH` | brush tools (01..13) |
| 13 | `FR-CIV-LIFE` | (emergence demography — see §3.1) |
| 12 | `FR-CIV-AUDIO` | audio pipeline (001..012) |
| 10 | `FR-CIV-CA` | cellular-automata (001..010) |
| 10 | `FR-CIV-EMERGENCE` | (civ-021 — see §3.2) |
| 10 | `FR-CIV-GEO` | geography (001..010) |
| 10 | `FR-CIV-VERIFY` | verification harness (001..010) |
| 9 | `FR-CIV-ARCH` | (emergence architecture — see §3.1) |
| 9 | `FR-CIV-GODTOOL` | god-tool authoring (100, 9, 900..901, 910..912, 920..921) |
| 9 | `FR-CIV-PLANET` | planet (003..005, 010, 020, 030, 040, 050, 060) |
| 9 | `FR-CIV-RESEARCH` | research additions (004, 010..012, 020, 030..033) |
| 8 | `FR-CIV-DIPLO` | (dormant — see §3.2) |
| 8 | `FR-CIV-MARKET` | (emergence trade — see §3.1) |
| 8 | `FR-CIV-PBR` | PBR material (001..008) |
| 8 | `FR-CIV-POLITY` | (emergence faction — see §3.1) |
| 8 | `FR-ECO` | economic NFRs (001..010) |
| 7 | `FR-C` | `FR-C-01..07` (orphans) |
| 7 | `FR-CIV-ECON` | (emergence economy — see §3.1) |
| 7 | `FR-DET` | determinism NFRs (001..007) |
| 6 | `FR-CIV` | `FR-CIV-{0001,0104,014,016,0700,3}` (orphans) |
| 6 | `FR-CIV-LAWS` | (dormant — see §3.2) |
| 6 | `FR-CIV-LLM` | LLM hook (001..006) |
| 6 | `FR-CIV-MAINT` | maintenance (001..006) |
| 6 | `FR-CIV-MCP` | MCP server (001..006) |
| 6 | `FR-CIV-NOTIFY` | notifications (900..901, 910..911, 920..921) |
| 6 | `FR-CIV-ROAD` | road building (900..902, 910, 920..921) |
| 6 | `FR-CIV-TACTICS` | tactics additions (002..003, 011, 100..102) |
| 6 | `FR-CIV-TERRAIN` | terrain (001..006) |
| 6 | `FR-O` / `FR-R` / `FR-S` | `FR-O-01..06`, `FR-R-01..06`, `FR-S-01..06` (orphans) |
| 5 | `FR-CIV-EMERG` | (civ-019 — see §3.2) |
| 5 | `FR-CIV-FOG` | fog-of-war (001..005) |
| 5 | `FR-CIV-GENETICS` | genetics additions (011..012, SEED-001..003) |
| 5 | `FR-CIV-GODOT` | Godot additions (001..005) |
| 5 | `FR-CIV-HUD` | HUD (001..005) |
| 4 | `FR-CIV-ACC` | accessibility (001..004) |
| 4 | `FR-CIV-ACT` | (emergence actor — see §3.1) |
| 4 | `FR-CIV-DET` | determinism IDs (001..004) |
| 4 | `FR-CIV-INSPECT` | inspector (901..903, 920) |
| 4 | `FR-CIV-REL` | (emergence religion — see §3.1) |
| 4 | `FR-CIV-SEC` | security (001..004) |
| 3 | `FR-CIV-BIO` | biology (001..003) |
| 3 | `FR-CIV-CLIENT` | client (006, GODOT-001..002) |
| 3 | `FR-CIV-CLIMATE` | (emergence climate — see §3.1) |
| 3 | `FR-CIV-CULT` | (dormant — see §3.2) |
| 3 | `FR-CIV-INFRA` | infrastructure (070..072) |
| 3 | `FR-CIV-PORT` | port (001..003) |
| 3 | `FR-CIV-RENDER` | render (001..002, CROWD-001) |
| 3 | `FR-CIV-SERVER` | server (001..003) |
| 3 | `FR-CIV-TEST` | test harness (001..003) |
| 3 | `FR-CIV-UI` | UI (001..003) |
| 3 | `FR-CIV-UX` | UX additions (002..003, 005) |
| 3 | `FR-NET` | network NFRs (001..003) |
| 2 | `FR-CIV-ACTOR` | actor (001..002) |
| 2 | `FR-CIV-GAME` | game (001, 003) |
| 2 | `FR-CIV-GOV` | government (001..002) |
| 2 | `FR-CIV-SOCIAL` | (dormant — see §3.2) |
| 2 | `FR-GUARD` | guard NFRs (001..002) |
| 2 | `FR-METRICS` | metrics (004..005) |
| 2 | `FR-PHENO` | `PHENO-VOXEL-CUBIC-001`, `PHENO-VOXEL-WORLD-001` |
| 1 | `FR-AUTH-001` / `FR-CIV-CITY-PLAN-001` / `FR-CIV-CONTENT-001` / `FR-CIV-DEV-HYGIENE-001` / `FR-CIV-METRICS-001` / `FR-CIV-MP-001` / `FR-CIV-POSTFX-001` / `FR-CIV-RELIGION-002` / `FR-CIV-RES-001` / `FR-CIV-STORY-001` / `FR-DOC-001` / `FR-INT-001` / `FR-MET-001` / `FR-REP-001` / `FR-SCALE-02` / `FR-STOR-001` / `FR-TEST-001` / `FR-UX-006` / `FR-VAL-001` | one-offs |

### Emergence-focused roll-up

**Total emergence + dormant-phase IDs (§3.1 + §3.2 + §3.3 + `FR-CIV-0100` family):** 171.

**Total untraced IDs covered by this audit:** 946.

**Coverage gap summary:**

- 21.1% of unique `FR-` IDs are traced (255 / 1 201).
- 78.9% are untraced (946 / 1 201) — the 171 emergence/dormant IDs in
  §3 are the highest-priority subset to recover.

---

## NFR coverage

The 34 unique `NFR-` IDs found in code/specs are **0%** traced — no
matrix in `docs/traceability/` has a row with an `NFR-` ID.  The
candidates are spread across:

- `NFR-PERF-*` (perf NFRs in code)
- `NFR-NET-*` (network NFRs)
- `NFR-SEC-*` (security NFRs)
- `NFR-AVAIL-*` (availability)
- `NFR-*` (others)

**Proposal:** add a new `docs/traceability/nfr-matrix.md` mirroring the
`fr-3d-matrix.md` style, with one row per NFR- ID grouped by family.
This is **out of scope** for the current brief (which only asked for FR)
but is flagged here for follow-up.

---

## Follow-up work (proposed, not done in this audit)

1. **Create `docs/traceability/fr-emergence-matrix.md`** with one row per
   ID listed in §3.1, §3.2, and §3.3 — totalling 171 rows.  Use
   `fr-3d-matrix.md` table format.
2. **Create `docs/traceability/nfr-matrix.md`** for the 34 NFRs.
3. **Wire `civis-tracelinks.md` and `emergent-systems-tracelinks.md`**
   to the new matrix: replace bare `FR-CIV-0100 §3 emergence` references
   with concrete row IDs from `fr-emergence-matrix.md`.
4. **Mark dormant-phase families `dormant` (not `planned`)** in the new
   matrix so the gap is explicit; the existing `phase_emergence` is
   currently a no-op stub, see `crates/engine/src/emergence.rs:1-2`.
5. **Re-run this audit** (committing the script in `tools/audit-fr-coverage/`
   so it's a 1-line `just audit-fr-coverage` re-run).

---

*Generated 2026-06-23 by the requirements-coverage audit on
`research/traceability-audit`.  Re-runnable with
`tools/audit-fr-coverage/audit.sh` (added in follow-up #5).*
