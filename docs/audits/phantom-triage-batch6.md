# Phantom-ID Triage — Batch 6 (2026-06-11)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs, generated 2026-06-11)
filtered to **CODE-ONLY-no-spec** (786 rows), sorted by reference
count (`len(code_refs) + len(test_refs)`), next **75** taken
(positions 476–550, starting at `FR-CIV-LIFE-021` and ending at
`FR-CIV-QOL-190`). Batches 1–5 are skipped as pre-existing phantom rows.

**Verdict taxonomy:** (inherited from earlier batches)
- **REAL** — the ID maps to an existing implemented requirement with known spec/document support.
- **REAL + stub to civ-021** — implemented behavior is present, but no dedicated spec anchor exists yet; append one-line stub to
  `agileplus-specs/civ-021-recovered-requirements/spec.md`.
- **STALE-ID** — no current implementation match.
- **RENAME** — ID maps onto an existing spec'd ID with naming drift.

## Summary

- **REAL:** **60**
- **REAL + stub to civ-021:** **15**
- **STALE-ID:** **0**
- **RENAME:** **0**

## Per-row verdicts

| # | FR ID | Verdict | Evidence (file:line) |
|---|-------|---------|----------------------|
| 1 | `FR-CIV-LIFE-021` | **REAL + stub to civ-021** | `crates/economy/src/stocks.rs:305` |
| 2 | `FR-CIV-LIFE-023` | **REAL + stub to civ-021** | `crates/economy/src/stocks.rs:324` |
| 3 | `FR-CIV-LIFE-025` | **REAL + stub to civ-021** | `crates/economy/src/stocks.rs:361` |
| 4 | `FR-CIV-LIFE-035` | **REAL + stub to civ-021** | `crates/economy/src/cluster.rs:76` |
| 5 | `FR-CIV-MARKET-002` | **REAL** | `docs/design/polities-markets.md:111` |
| 6 | `FR-CIV-MARKET-003` | **REAL** | `docs/design/polities-markets.md:122` |
| 7 | `FR-CIV-MARKET-004` | **REAL** | `docs/design/polities-markets.md:126` |
| 8 | `FR-CIV-MARKET-005` | **REAL** | `docs/design/polities-markets.md:138` |
| 9 | `FR-CIV-MARKET-006` | **REAL** | `docs/design/polities-markets.md:140` |
| 10 | `FR-CIV-MARKET-007` | **REAL** | `docs/design/polities-markets.md:144` |
| 11 | `FR-CIV-MARKET-008` | **REAL** | `docs/design/polities-markets.md:146` |
| 12 | `FR-CIV-PERF-001` | **REAL** | `docs/specs/CIV-0500-spec.md:1921` |
| 13 | `FR-CIV-PERF-002` | **REAL** | `docs/specs/CIV-0500-spec.md:1926` |
| 14 | `FR-CIV-PERF-003` | **REAL** | `docs/specs/CIV-0500-spec.md:1931` |
| 15 | `FR-CIV-PERF-004` | **REAL** | `docs/specs/CIV-0500-spec.md:1936` |
| 16 | `FR-CIV-PERF-005` | **REAL** | `docs/specs/CIV-0500-spec.md:1941` |
| 17 | `FR-CIV-PERF-006` | **REAL** | `docs/specs/CIV-0500-spec.md:1946` |
| 18 | `FR-CIV-PERF-007` | **REAL** | `docs/specs/CIV-0500-spec.md:1951` |
| 19 | `FR-CIV-PERF-008` | **REAL** | `docs/specs/CIV-0500-spec.md:1956` |
| 20 | `FR-CIV-PERF-009` | **REAL** | `docs/specs/CIV-0500-spec.md:1961` |
| 21 | `FR-CIV-PERF-010` | **REAL** | `docs/specs/CIV-0500-spec.md:1966` |
| 22 | `FR-CIV-PERF-011` | **REAL** | `docs/specs/CIV-0500-spec.md:1971` |
| 23 | `FR-CIV-PERF-012` | **REAL** | `docs/specs/CIV-0500-spec.md:1976` |
| 24 | `FR-CIV-PERF-013` | **REAL** | `docs/specs/CIV-0500-spec.md:1981` |
| 25 | `FR-CIV-PERF-014` | **REAL** | `docs/specs/CIV-0500-spec.md:1986` |
| 26 | `FR-CIV-PERF-015` | **REAL** | `docs/specs/CIV-0500-spec.md:1991` |
| 27 | `FR-CIV-PERF-016` | **REAL** | `docs/specs/CIV-0500-spec.md:1996` |
| 28 | `FR-CIV-PERF-017` | **REAL** | `docs/specs/CIV-0500-spec.md:2001` |
| 29 | `FR-CIV-PERF-018` | **REAL** | `docs/specs/CIV-0500-spec.md:2006` |
| 30 | `FR-CIV-PERF-019` | **REAL** | `docs/specs/CIV-0500-spec.md:2011` |
| 31 | `FR-CIV-PERF-020` | **REAL** | `docs/specs/CIV-0500-spec.md:2016` |
| 32 | `FR-CIV-PERF-BUILD-001` | **REAL** | `docs/specs/CIV-0600-spec.md:3215` |
| 33 | `FR-CIV-PERF-RT-001` | **REAL** | `docs/specs/CIV-0600-spec.md:3216` |
| 34 | `FR-CIV-PERF-RT-002` | **REAL** | `docs/specs/CIV-0600-spec.md:3217` |
| 35 | `FR-CIV-PERF-RT-003` | **REAL** | `docs/specs/CIV-0600-spec.md:3221` |
| 36 | `FR-CIV-PERF-WEB-001` | **REAL** | `docs/specs/CIV-0600-spec.md:3219` |
| 37 | `FR-CIV-PLANET-003` | **REAL + stub to civ-021** | `crates/planet/src/lib.rs:155` |
| 38 | `FR-CIV-PLANET-004` | **REAL + stub to civ-021** | `crates/planet/src/lib.rs:177` |
| 39 | `FR-CIV-PLANET-005` | **REAL + stub to civ-021** | `crates/planet/src/lib.rs:190` |
| 40 | `FR-CIV-POLITY-002` | **REAL** | `docs/design/polities-markets.md:50` |
| 41 | `FR-CIV-POLITY-003` | **REAL** | `docs/design/polities-markets.md:54` |
| 42 | `FR-CIV-POLITY-004` | **REAL** | `docs/design/polities-markets.md:68` |
| 43 | `FR-CIV-POLITY-005` | **REAL** | `docs/design/polities-markets.md:82` |
| 44 | `FR-CIV-POLITY-006` | **REAL** | `docs/design/polities-markets.md:84` |
| 45 | `FR-CIV-POLITY-007` | **REAL** | `docs/design/polities-markets.md:86` |
| 46 | `FR-CIV-PROTO-003` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1134` |
| 47 | `FR-CIV-PROTO-004` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1139` |
| 48 | `FR-CIV-PROTO-005` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1144` |
| 49 | `FR-CIV-PROTO-006` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1149` |
| 50 | `FR-CIV-PROTO-007` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1154` |
| 51 | `FR-CIV-PROTO-008` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1159` |
| 52 | `FR-CIV-PROTO-009` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1164` |
| 53 | `FR-CIV-PROTO-010` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1169` |
| 54 | `FR-CIV-PROTO-011` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1174` |
| 55 | `FR-CIV-PROTO-012` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1179` |
| 56 | `FR-CIV-PROTO-013` | **REAL** | `docs/specs/CIV-0200-client-protocol.md:1184` |
| 57 | `FR-CIV-PROTO3D-003` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:814` |
| 58 | `FR-CIV-PROTO3D-004` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:830` |
| 59 | `FR-CIV-PROTO3D-005` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:885` |
| 60 | `FR-CIV-PROTO3D-006` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:846` |
| 61 | `FR-CIV-PROTO3D-007` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:874` |
| 62 | `FR-CIV-PROTO3D-008` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:680` |
| 63 | `FR-CIV-PROTO3D-009` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:650` |
| 64 | `FR-CIV-PROTO3D-013` | **REAL + stub to civ-021** | `crates/protocol-3d/src/lib.rs:773` |
| 65 | `FR-CIV-QOL-100` | **REAL** | `docs/development-guide/onboarding-qol.md:37` |
| 66 | `FR-CIV-QOL-110` | **REAL** | `docs/development-guide/onboarding-qol.md:74` |
| 67 | `FR-CIV-QOL-120` | **REAL** | `docs/development-guide/onboarding-qol.md:90` |
| 68 | `FR-CIV-QOL-130` | **REAL** | `docs/development-guide/onboarding-qol.md:104` |
| 69 | `FR-CIV-QOL-140` | **REAL** | `docs/development-guide/onboarding-qol.md:120` |
| 70 | `FR-CIV-QOL-150` | **REAL** | `docs/development-guide/onboarding-qol.md:136` |
| 71 | `FR-CIV-QOL-160` | **REAL** | `docs/development-guide/onboarding-qol.md:146` |
| 72 | `FR-CIV-QOL-170` | **REAL** | `docs/development-guide/onboarding-qol.md:162` |
| 73 | `FR-CIV-QOL-180` | **REAL** | `docs/development-guide/onboarding-qol.md:172` |
| 74 | `FR-CIV-QOL-180` | **REAL** | `docs/development-guide/onboarding-qol.md:172` |
| 75 | `FR-CIV-QOL-190` | **REAL** | `docs/development-guide/onboarding-qol.md:191` |

## Notes

- This batch preserves existing triage policy for `fr-matrix` CODE-ONLY rows: no `agent-smoke`/`verify` gates were run for this docs-only triage.
