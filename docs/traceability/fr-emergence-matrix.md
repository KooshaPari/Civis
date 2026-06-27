# FR Emergence Matrix — Civis Phase 3

**Purpose**: trace the 171 emergence + dormant-phase + charter-umbrella
`FR-` IDs identified in `docs/traceability/COVERAGE_AUDIT.md` §3.1, §3.2,
§3.3, and the `FR-CIV-0100` family.  This is the canonical row-level
source for the §3 IDs; `civis-tracelinks.md` and
`emergent-systems-tracelinks.md` link here.

**Schema** (mirrors `fr-3d-matrix.md`):

| Column | Meaning |
|---|---|
| FR ID | the canonical functional-requirement ID |
| Family | one of `emergence`, `dormant`, `charter` |
| Sub-cluster | the audit's sub-grouping (e.g. `civ-019`, `civ-021-batchA`) |
| Phase | `emergence` for §3.1, `dormant` for §3.2, `recovered` for §3.3, `charter` for `FR-CIV-0100` |
| Status | one of `planned`, `dormant`, `recovered`, `implemented`, `in-progress` |
| Owner | the FR's responsible subsystem or family |
| Trace link | pointer to spec/code that satisfies the FR (or `tbd` if not yet found) |
| Notes | free-form |

**Status legend** (from `COVERAGE_AUDIT.md` §3 follow-up #4):
- `planned` — code is sketched but no tests/coverage.
- `dormant` — phase is currently a no-op stub (see
  `crates/engine/src/emergence.rs:1-2`); the FRs are intentionally
  de-scoped until a re-emergence signal arrives.
- `recovered` — re-discovered in `civ-021` (and similar) and given a
  concrete traceability row here.
- `implemented` — code + tests + coverage present.
- `in-progress` — partial implementation; row tracks remaining work.

---

## Charter umbrella (1 row)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-0100` | charter | charter umbrella | charter | planned | engine | tbd | Top-level umbrella for all `civ-0XX` IDs; covers §3 emergence, §3 dormant, §3 recovered.  See `civis-tracelinks.md` and `emergent-systems-tracelinks.md`. |

---

## FR-CIV-EMERG-001..005 (5 rows — civ-019 emergence-metrics dashboard)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERG-001` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 1 |
| `FR-CIV-EMERG-002` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 2 |
| `FR-CIV-EMERG-003` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 3 |
| `FR-CIV-EMERG-004` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 4 |
| `FR-CIV-EMERG-005` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 5 |

---

## FR-CIV-EMERGENCE-001..006 (6 rows — civ-021 recovered-requirements batch A)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-001` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 1 (batch A) |
| `FR-CIV-EMERGENCE-002` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 2 (batch A) |
| `FR-CIV-EMERGENCE-003` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 3 (batch A) |
| `FR-CIV-EMERGENCE-004` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 4 (batch A) |
| `FR-CIV-EMERGENCE-005` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 5 (batch A) |
| `FR-CIV-EMERGENCE-006` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 6 (batch A) |

---

## FR-CIV-EMERGENCE-010..013 (4 rows — civ-021 recovered-requirements batch B)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-010` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 10 (batch B) |
| `FR-CIV-EMERGENCE-011` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 11 (batch B) |
| `FR-CIV-EMERGENCE-012` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 12 (batch B) |
| `FR-CIV-EMERGENCE-013` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 13 (batch B) |

---

## Aggregate counts (this matrix)

- Charter umbrella: 1 row (`FR-CIV-0100`)
- Emergence (`civ-019`): 5 rows
- Recovered (`civ-021` batch A): 6 rows
- Recovered (`civ-021` batch B): 4 rows
- **Total: 16 rows**

The remaining 155 IDs (171 − 16) are listed in
`docs/traceability/COVERAGE_AUDIT.md` §3.1, §3.2, §3.3 but are not yet
given individual rows here.  They are tracked in bulk via the
audit summary and will be added in subsequent PRs.

---

## See also

- `docs/traceability/COVERAGE_AUDIT.md` §3.1–§3.3 (171-ID source list)
- `docs/traceability/fr-3d-matrix.md` (existing 3D rendering matrix)
- `docs/traceability/index.md` (hub — links here)
- `civis-tracelinks.md` (civic-domain tracelinks, references §3)
- `emergent-systems-tracelinks.md` (emergence-family tracelinks, references §3)
- `tools/audit-fr-coverage/audit.sh` (regen script — see follow-up #5)

---

## Section B: Emergence batch rows (§3.1 + §3.2, all currently dormant)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-100` | emergence | civ-001 LANG family (1/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 1 |
| `FR-CIV-EMERGENCE-101` | emergence | civ-001 LANG family (2/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 2 |
| `FR-CIV-EMERGENCE-102` | emergence | civ-001 LANG family (3/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 3 |
| `FR-CIV-EMERGENCE-103` | emergence | civ-001 LANG family (4/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 4 |
| `FR-CIV-EMERGENCE-104` | emergence | civ-001 LANG family (5/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 5 |
| `FR-CIV-EMERGENCE-105` | emergence | civ-001 LANG family (6/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 6 |
| `FR-CIV-EMERGENCE-106` | emergence | civ-001 LANG family (7/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 7 |
| `FR-CIV-EMERGENCE-107` | emergence | civ-001 LANG family (8/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 8 |
| `FR-CIV-EMERGENCE-108` | emergence | civ-001 LANG family (9/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 9 |
| `FR-CIV-EMERGENCE-109` | emergence | civ-001 LANG family (10/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 10 |
| `FR-CIV-EMERGENCE-110` | emergence | civ-001 LANG family (11/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 11 |


## FR-CIV-EMERGENCE batch POLITY (8 rows — civ-002 POLITY family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-111` | emergence | civ-002 POLITY family (1/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 1 |
| `FR-CIV-EMERGENCE-112` | emergence | civ-002 POLITY family (2/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 2 |
| `FR-CIV-EMERGENCE-113` | emergence | civ-002 POLITY family (3/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 3 |
| `FR-CIV-EMERGENCE-114` | emergence | civ-002 POLITY family (4/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 4 |
| `FR-CIV-EMERGENCE-115` | emergence | civ-002 POLITY family (5/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 5 |
| `FR-CIV-EMERGENCE-116` | emergence | civ-002 POLITY family (6/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 6 |
| `FR-CIV-EMERGENCE-117` | emergence | civ-002 POLITY family (7/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 7 |
| `FR-CIV-EMERGENCE-118` | emergence | civ-002 POLITY family (8/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 8 |


## FR-CIV-EMERGENCE batch REL (5 rows — civ-003 REL family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-119` | emergence | civ-003 REL family (1/5) | dormant | dormant | engine | tbd | REL sub-cluster row 1 |
| `FR-CIV-EMERGENCE-120` | emergence | civ-003 REL family (2/5) | dormant | dormant | engine | tbd | REL sub-cluster row 2 |
| `FR-CIV-EMERGENCE-121` | emergence | civ-003 REL family (3/5) | dormant | dormant | engine | tbd | REL sub-cluster row 3 |
| `FR-CIV-EMERGENCE-122` | emergence | civ-003 REL family (4/5) | dormant | dormant | engine | tbd | REL sub-cluster row 4 |
| `FR-CIV-EMERGENCE-123` | emergence | civ-003 REL family (5/5) | dormant | dormant | engine | tbd | REL sub-cluster row 5 |


## FR-CIV-EMERGENCE batch MARKET (8 rows — civ-004 MARKET family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-124` | emergence | civ-004 MARKET family (1/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 1 |
| `FR-CIV-EMERGENCE-125` | emergence | civ-004 MARKET family (2/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 2 |
| `FR-CIV-EMERGENCE-126` | emergence | civ-004 MARKET family (3/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 3 |
| `FR-CIV-EMERGENCE-127` | emergence | civ-004 MARKET family (4/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 4 |
| `FR-CIV-EMERGENCE-128` | emergence | civ-004 MARKET family (5/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 5 |
| `FR-CIV-EMERGENCE-129` | emergence | civ-004 MARKET family (6/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 6 |
| `FR-CIV-EMERGENCE-130` | emergence | civ-004 MARKET family (7/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 7 |
| `FR-CIV-EMERGENCE-131` | emergence | civ-004 MARKET family (8/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 8 |


## FR-CIV-EMERGENCE batch ARCH (9 rows — civ-005 ARCH family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-132` | emergence | civ-005 ARCH family (1/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 1 |
| `FR-CIV-EMERGENCE-133` | emergence | civ-005 ARCH family (2/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 2 |
| `FR-CIV-EMERGENCE-134` | emergence | civ-005 ARCH family (3/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 3 |
| `FR-CIV-EMERGENCE-135` | emergence | civ-005 ARCH family (4/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 4 |
| `FR-CIV-EMERGENCE-136` | emergence | civ-005 ARCH family (5/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 5 |
| `FR-CIV-EMERGENCE-137` | emergence | civ-005 ARCH family (6/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 6 |
| `FR-CIV-EMERGENCE-138` | emergence | civ-005 ARCH family (7/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 7 |
| `FR-CIV-EMERGENCE-139` | emergence | civ-005 ARCH family (8/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 8 |
| `FR-CIV-EMERGENCE-140` | emergence | civ-005 ARCH family (9/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 9 |


## FR-CIV-EMERGENCE batch CLIMATE (3 rows — civ-006 CLIMATE family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-141` | emergence | civ-006 CLIMATE family (1/3) | dormant | partial | engine | tbd | CLIMATE sub-cluster row 1 |
| `FR-CIV-EMERGENCE-142` | emergence | civ-006 CLIMATE family (2/3) | dormant | partial | engine | tbd | CLIMATE sub-cluster row 2 |
| `FR-CIV-EMERGENCE-143` | emergence | civ-006 CLIMATE family (3/3) | dormant | partial | engine | tbd | CLIMATE sub-cluster row 3 |


## FR-CIV-EMERGENCE batch ECON (7 rows — civ-007 ECON family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-144` | emergence | civ-007 ECON family (1/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 1 |
| `FR-CIV-EMERGENCE-145` | emergence | civ-007 ECON family (2/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 2 |
| `FR-CIV-EMERGENCE-146` | emergence | civ-007 ECON family (3/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 3 |
| `FR-CIV-EMERGENCE-147` | emergence | civ-007 ECON family (4/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 4 |
| `FR-CIV-EMERGENCE-148` | emergence | civ-007 ECON family (5/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 5 |
| `FR-CIV-EMERGENCE-149` | emergence | civ-007 ECON family (6/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 6 |
| `FR-CIV-EMERGENCE-150` | emergence | civ-007 ECON family (7/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 7 |


## FR-CIV-EMERGENCE batch DEMO (17 rows — civ-008 DEMO family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-151` | emergence | civ-008 DEMO family (1/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 1 |
| `FR-CIV-EMERGENCE-152` | emergence | civ-008 DEMO family (2/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 2 |
| `FR-CIV-EMERGENCE-153` | emergence | civ-008 DEMO family (3/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 3 |
| `FR-CIV-EMERGENCE-154` | emergence | civ-008 DEMO family (4/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 4 |
| `FR-CIV-EMERGENCE-155` | emergence | civ-008 DEMO family (5/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 5 |
| `FR-CIV-EMERGENCE-156` | emergence | civ-008 DEMO family (6/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 6 |
| `FR-CIV-EMERGENCE-157` | emergence | civ-008 DEMO family (7/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 7 |
| `FR-CIV-EMERGENCE-158` | emergence | civ-008 DEMO family (8/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 8 |
| `FR-CIV-EMERGENCE-159` | emergence | civ-008 DEMO family (9/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 9 |
| `FR-CIV-EMERGENCE-160` | emergence | civ-008 DEMO family (10/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 10 |
| `FR-CIV-EMERGENCE-161` | emergence | civ-008 DEMO family (11/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 11 |
| `FR-CIV-EMERGENCE-162` | emergence | civ-008 DEMO family (12/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 12 |
| `FR-CIV-EMERGENCE-163` | emergence | civ-008 DEMO family (13/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 13 |
| `FR-CIV-EMERGENCE-164` | emergence | civ-008 DEMO family (14/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 14 |
| `FR-CIV-EMERGENCE-165` | emergence | civ-008 DEMO family (15/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 15 |
| `FR-CIV-EMERGENCE-166` | emergence | civ-008 DEMO family (16/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 16 |
| `FR-CIV-EMERGENCE-167` | emergence | civ-008 DEMO family (17/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 17 |


## FR-CIV-EMERGENCE batch PSYCHE (25 rows — civ-009 PSYCHE family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-168` | emergence | civ-009 PSYCHE family (1/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 1 |
| `FR-CIV-EMERGENCE-169` | emergence | civ-009 PSYCHE family (2/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 2 |
| `FR-CIV-EMERGENCE-170` | emergence | civ-009 PSYCHE family (3/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 3 |
| `FR-CIV-EMERGENCE-171` | emergence | civ-009 PSYCHE family (4/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 4 |
| `FR-CIV-EMERGENCE-172` | emergence | civ-009 PSYCHE family (5/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 5 |
| `FR-CIV-EMERGENCE-173` | emergence | civ-009 PSYCHE family (6/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 6 |
| `FR-CIV-EMERGENCE-174` | emergence | civ-009 PSYCHE family (7/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 7 |
| `FR-CIV-EMERGENCE-175` | emergence | civ-009 PSYCHE family (8/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 8 |
| `FR-CIV-EMERGENCE-176` | emergence | civ-009 PSYCHE family (9/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 9 |
| `FR-CIV-EMERGENCE-177` | emergence | civ-009 PSYCHE family (10/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 10 |
| `FR-CIV-EMERGENCE-178` | emergence | civ-009 PSYCHE family (11/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 11 |
| `FR-CIV-EMERGENCE-179` | emergence | civ-009 PSYCHE family (12/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 12 |
| `FR-CIV-EMERGENCE-180` | emergence | civ-009 PSYCHE family (13/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 13 |
| `FR-CIV-EMERGENCE-181` | emergence | civ-009 PSYCHE family (14/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 14 |
| `FR-CIV-EMERGENCE-182` | emergence | civ-009 PSYCHE family (15/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 15 |
| `FR-CIV-EMERGENCE-183` | emergence | civ-009 PSYCHE family (16/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 16 |
| `FR-CIV-EMERGENCE-184` | emergence | civ-009 PSYCHE family (17/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 17 |
| `FR-CIV-EMERGENCE-185` | emergence | civ-009 PSYCHE family (18/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 18 |
| `FR-CIV-EMERGENCE-186` | emergence | civ-009 PSYCHE family (19/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 19 |
| `FR-CIV-EMERGENCE-187` | emergence | civ-009 PSYCHE family (20/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 20 |
| `FR-CIV-EMERGENCE-188` | emergence | civ-009 PSYCHE family (21/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 21 |
| `FR-CIV-EMERGENCE-189` | emergence | civ-009 PSYCHE family (22/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 22 |
| `FR-CIV-EMERGENCE-190` | emergence | civ-009 PSYCHE family (23/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 23 |
| `FR-CIV-EMERGENCE-191` | emergence | civ-009 PSYCHE family (24/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 24 |
| `FR-CIV-EMERGENCE-192` | emergence | civ-009 PSYCHE family (25/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 25 |


## FR-CIV-EMERGENCE batch LEGENDS (15 rows — civ-010 LEGENDS family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-193` | emergence | civ-010 LEGENDS family (1/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 1 |
| `FR-CIV-EMERGENCE-194` | emergence | civ-010 LEGENDS family (2/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 2 |
| `FR-CIV-EMERGENCE-195` | emergence | civ-010 LEGENDS family (3/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 3 |
| `FR-CIV-EMERGENCE-196` | emergence | civ-010 LEGENDS family (4/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 4 |
| `FR-CIV-EMERGENCE-197` | emergence | civ-010 LEGENDS family (5/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 5 |
| `FR-CIV-EMERGENCE-198` | emergence | civ-010 LEGENDS family (6/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 6 |
| `FR-CIV-EMERGENCE-199` | emergence | civ-010 LEGENDS family (7/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 7 |
| `FR-CIV-EMERGENCE-200` | emergence | civ-010 LEGENDS family (8/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 8 |
| `FR-CIV-EMERGENCE-201` | emergence | civ-010 LEGENDS family (9/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 9 |
| `FR-CIV-EMERGENCE-202` | emergence | civ-010 LEGENDS family (10/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 10 |
| `FR-CIV-EMERGENCE-203` | emergence | civ-010 LEGENDS family (11/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 11 |
| `FR-CIV-EMERGENCE-204` | emergence | civ-010 LEGENDS family (12/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 12 |
| `FR-CIV-EMERGENCE-205` | emergence | civ-010 LEGENDS family (13/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 13 |
| `FR-CIV-EMERGENCE-206` | emergence | civ-010 LEGENDS family (14/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 14 |
| `FR-CIV-EMERGENCE-207` | emergence | civ-010 LEGENDS family (15/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 15 |


## FR-CIV-EMERGENCE batch AI (15 rows — civ-011 AI family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-208` | emergence | civ-011 AI family (1/15) | dormant | implemented | engine | tbd | AI sub-cluster row 1 |
| `FR-CIV-EMERGENCE-209` | emergence | civ-011 AI family (2/15) | dormant | implemented | engine | tbd | AI sub-cluster row 2 |
| `FR-CIV-EMERGENCE-210` | emergence | civ-011 AI family (3/15) | dormant | implemented | engine | tbd | AI sub-cluster row 3 |
| `FR-CIV-EMERGENCE-211` | emergence | civ-011 AI family (4/15) | dormant | implemented | engine | tbd | AI sub-cluster row 4 |
| `FR-CIV-EMERGENCE-212` | emergence | civ-011 AI family (5/15) | dormant | implemented | engine | tbd | AI sub-cluster row 5 |
| `FR-CIV-EMERGENCE-213` | emergence | civ-011 AI family (6/15) | dormant | implemented | engine | tbd | AI sub-cluster row 6 |
| `FR-CIV-EMERGENCE-214` | emergence | civ-011 AI family (7/15) | dormant | implemented | engine | tbd | AI sub-cluster row 7 |
| `FR-CIV-EMERGENCE-215` | emergence | civ-011 AI family (8/15) | dormant | implemented | engine | tbd | AI sub-cluster row 8 |
| `FR-CIV-EMERGENCE-216` | emergence | civ-011 AI family (9/15) | dormant | implemented | engine | tbd | AI sub-cluster row 9 |
| `FR-CIV-EMERGENCE-217` | emergence | civ-011 AI family (10/15) | dormant | implemented | engine | tbd | AI sub-cluster row 10 |
| `FR-CIV-EMERGENCE-218` | emergence | civ-011 AI family (11/15) | dormant | implemented | engine | tbd | AI sub-cluster row 11 |
| `FR-CIV-EMERGENCE-219` | emergence | civ-011 AI family (12/15) | dormant | implemented | engine | tbd | AI sub-cluster row 12 |
| `FR-CIV-EMERGENCE-220` | emergence | civ-011 AI family (13/15) | dormant | implemented | engine | tbd | AI sub-cluster row 13 |
| `FR-CIV-EMERGENCE-221` | emergence | civ-011 AI family (14/15) | dormant | implemented | engine | tbd | AI sub-cluster row 14 |
| `FR-CIV-EMERGENCE-222` | emergence | civ-011 AI family (15/15) | dormant | implemented | engine | tbd | AI sub-cluster row 15 |


## FR-CIV-EMERGENCE batch CULT (3 rows — civ-012 CULT family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-223` | emergence | civ-012 CULT family (1/3) | dormant | dormant | engine | tbd | CULT sub-cluster row 1 |
| `FR-CIV-EMERGENCE-224` | emergence | civ-012 CULT family (2/3) | dormant | dormant | engine | tbd | CULT sub-cluster row 2 |
| `FR-CIV-EMERGENCE-225` | emergence | civ-012 CULT family (3/3) | dormant | dormant | engine | tbd | CULT sub-cluster row 3 |


## FR-CIV-EMERGENCE batch SOCIAL (2 rows — civ-013 SOCIAL family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-226` | emergence | civ-013 SOCIAL family (1/2) | dormant | dormant | engine | tbd | SOCIAL sub-cluster row 1 |
| `FR-CIV-EMERGENCE-227` | emergence | civ-013 SOCIAL family (2/2) | dormant | dormant | engine | tbd | SOCIAL sub-cluster row 2 |


## FR-CIV-EMERGENCE batch DIPLO (8 rows — civ-014 DIPLO family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-228` | emergence | civ-014 DIPLO family (1/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 1 |
| `FR-CIV-EMERGENCE-229` | emergence | civ-014 DIPLO family (2/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 2 |
| `FR-CIV-EMERGENCE-230` | emergence | civ-014 DIPLO family (3/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 3 |
| `FR-CIV-EMERGENCE-231` | emergence | civ-014 DIPLO family (4/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 4 |
| `FR-CIV-EMERGENCE-232` | emergence | civ-014 DIPLO family (5/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 5 |
| `FR-CIV-EMERGENCE-233` | emergence | civ-014 DIPLO family (6/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 6 |
| `FR-CIV-EMERGENCE-234` | emergence | civ-014 DIPLO family (7/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 7 |
| `FR-CIV-EMERGENCE-235` | emergence | civ-014 DIPLO family (8/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 8 |


## FR-CIV-EMERGENCE batch LAWS (6 rows — civ-015 LAWS family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-236` | emergence | civ-015 LAWS family (1/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 1 |
| `FR-CIV-EMERGENCE-237` | emergence | civ-015 LAWS family (2/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 2 |
| `FR-CIV-EMERGENCE-238` | emergence | civ-015 LAWS family (3/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 3 |
| `FR-CIV-EMERGENCE-239` | emergence | civ-015 LAWS family (4/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 4 |
| `FR-CIV-EMERGENCE-240` | emergence | civ-015 LAWS family (5/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 5 |
| `FR-CIV-EMERGENCE-241` | emergence | civ-015 LAWS family (6/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 6 |


## Charter integration rows (4 rows — §3.3 INT-1..INT-4)

These are the concrete charter-umbrella integration rows that FR-CIV-0100
hooks into. They live in the same matrix as the per-system FRs so that a
single grep for `FR-CIV-0100-int` returns the full integration graph.

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-0100-int1` | charter | civ-016 INT-1 charter integration | charter | planned | engine | `crates/engine/src/charter_int1.rs` (tbd) | Hook 1: emergence-dashboard surfaces charter summary metrics |
| `FR-CIV-0100-int2` | charter | civ-017 INT-2 charter integration | charter | planned | engine | `crates/engine/src/charter_int2.rs` (tbd) | Hook 2: recovered-requirement batches A/B roll up into charter lineage |
| `FR-CIV-0100-int3` | charter | civ-018 INT-3 charter integration | charter | planned | engine | `crates/engine/src/charter_int3.rs` (tbd) | Hook 3: dormant-phase families are gated by charter re-emergence signal |
| `FR-CIV-0100-int4` | charter | civ-022 INT-4 charter integration | charter | planned | engine | `crates/engine/src/charter_int4.rs` (tbd) | Hook 4: charter propagates activation thresholds to all sub-cluster families |

---

## FR-CIV-EMERGENCE batch INT-1 (1 rows — civ-016 INT-1 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-242` | emergence | civ-016 INT-1 charter integration (1/1) | dormant | dormant | engine | tbd | INT-1 sub-cluster row 1 |


## FR-CIV-EMERGENCE batch INT-2 (1 rows — civ-017 INT-2 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-243` | emergence | civ-017 INT-2 charter integration (1/1) | dormant | dormant | engine | tbd | INT-2 sub-cluster row 1 |


## FR-CIV-EMERGENCE batch INT-3 (1 rows — civ-018 INT-3 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-244` | emergence | civ-018 INT-3 charter integration (1/1) | dormant | dormant | engine | tbd | INT-3 sub-cluster row 1 |


## FR-CIV-EMERGENCE batch INT-4 (1 rows — civ-022 INT-4 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-245` | emergence | civ-022 INT-4 charter integration (1/1) | dormant | dormant | engine | tbd | INT-4 sub-cluster row 1 |

---

## Section B: Emergence batch rows (§3.1 + §3.2 + §3.3, all currently dormant)

146 rows generated from COVERAGE_AUDIT.md §3.1 + §3.2 sub-clusters, plus 4 charter integration rows from §3.3. All marked `dormant` until each acquires spec + code + test triple. Promote to `planned` once a spec exists; to `implemented` once the code is in a crate; to `covered` once tests pin the contract.


| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|-------|--------|-------------|-------|--------|-------|------------|-------|
| FR-CIV-0100-int1 | Integration | charter-umbrella → emergence | dormant | dormant | @kooshapari | docs/traceability/fr-emergence-matrix.md#section-a | Charter row promotes a single emerg spec to COVERED via spec + code + test triple. Promoted by §3.4 first-priority set (FR-CIV-EMERG-001..005). |
| FR-CIV-0100-int2 | Integration | charter-umbrella → recovery | dormant | dormant | @kooshapari | docs/traceability/fr-emergence-matrix.md#section-a | Charter row promoting a recovered-requirement batch (FR-CIV-EMERGENCE-001..006) to COVERED. First spec to satisfy this gets green. |
| FR-CIV-0100-int3 | Integration | charter-umbrella → coupling | dormant | dormant | @kooshapari | docs/traceability/emergent-systems-tracelinks.md | Charter row promoting a 30-coupling point (one of the systems × APIs listed in tracelinks §2) to COVERED via a typed bridge + 5 doc tests. |
| FR-CIV-0100-int4 | Integration | charter-umbrella → audit-script | dormant | dormant | @kooshapari | Tools/audit-fr-coverage/audit.sh | Charter row closed by landing the audit.sh + .github/workflows/audit-fr-coverage.yml CI integration (currently in-flight per pending items). |

---

### B.1 Sub-cluster batches

## FR-CIV-EMERGENCE batch LANG (11 rows — civ-001 LANG family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-100` | emergence | civ-001 LANG family (1/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 1 |
| `FR-CIV-EMERGENCE-101` | emergence | civ-001 LANG family (2/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 2 |
| `FR-CIV-EMERGENCE-102` | emergence | civ-001 LANG family (3/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 3 |
| `FR-CIV-EMERGENCE-103` | emergence | civ-001 LANG family (4/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 4 |
| `FR-CIV-EMERGENCE-104` | emergence | civ-001 LANG family (5/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 5 |
| `FR-CIV-EMERGENCE-105` | emergence | civ-001 LANG family (6/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 6 |
| `FR-CIV-EMERGENCE-106` | emergence | civ-001 LANG family (7/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 7 |
| `FR-CIV-EMERGENCE-107` | emergence | civ-001 LANG family (8/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 8 |
| `FR-CIV-EMERGENCE-108` | emergence | civ-001 LANG family (9/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 9 |
| `FR-CIV-EMERGENCE-109` | emergence | civ-001 LANG family (10/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 10 |
| `FR-CIV-EMERGENCE-110` | emergence | civ-001 LANG family (11/11) | dormant | dormant | engine | tbd | LANG sub-cluster row 11 |


## FR-CIV-EMERGENCE batch POLITY (8 rows — civ-002 POLITY family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-111` | emergence | civ-002 POLITY family (1/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 1 |
| `FR-CIV-EMERGENCE-112` | emergence | civ-002 POLITY family (2/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 2 |
| `FR-CIV-EMERGENCE-113` | emergence | civ-002 POLITY family (3/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 3 |
| `FR-CIV-EMERGENCE-114` | emergence | civ-002 POLITY family (4/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 4 |
| `FR-CIV-EMERGENCE-115` | emergence | civ-002 POLITY family (5/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 5 |
| `FR-CIV-EMERGENCE-116` | emergence | civ-002 POLITY family (6/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 6 |
| `FR-CIV-EMERGENCE-117` | emergence | civ-002 POLITY family (7/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 7 |
| `FR-CIV-EMERGENCE-118` | emergence | civ-002 POLITY family (8/8) | dormant | dormant | engine | tbd | POLITY sub-cluster row 8 |


## FR-CIV-EMERGENCE batch REL (5 rows — civ-003 REL family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-119` | emergence | civ-003 REL family (1/5) | dormant | dormant | engine | tbd | REL sub-cluster row 1 |
| `FR-CIV-EMERGENCE-120` | emergence | civ-003 REL family (2/5) | dormant | dormant | engine | tbd | REL sub-cluster row 2 |
| `FR-CIV-EMERGENCE-121` | emergence | civ-003 REL family (3/5) | dormant | dormant | engine | tbd | REL sub-cluster row 3 |
| `FR-CIV-EMERGENCE-122` | emergence | civ-003 REL family (4/5) | dormant | dormant | engine | tbd | REL sub-cluster row 4 |
| `FR-CIV-EMERGENCE-123` | emergence | civ-003 REL family (5/5) | dormant | dormant | engine | tbd | REL sub-cluster row 5 |


## FR-CIV-EMERGENCE batch MARKET (8 rows — civ-004 MARKET family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-124` | emergence | civ-004 MARKET family (1/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 1 |
| `FR-CIV-EMERGENCE-125` | emergence | civ-004 MARKET family (2/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 2 |
| `FR-CIV-EMERGENCE-126` | emergence | civ-004 MARKET family (3/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 3 |
| `FR-CIV-EMERGENCE-127` | emergence | civ-004 MARKET family (4/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 4 |
| `FR-CIV-EMERGENCE-128` | emergence | civ-004 MARKET family (5/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 5 |
| `FR-CIV-EMERGENCE-129` | emergence | civ-004 MARKET family (6/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 6 |
| `FR-CIV-EMERGENCE-130` | emergence | civ-004 MARKET family (7/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 7 |
| `FR-CIV-EMERGENCE-131` | emergence | civ-004 MARKET family (8/8) | dormant | dormant | engine | tbd | MARKET sub-cluster row 8 |


## FR-CIV-EMERGENCE batch ARCH (9 rows — civ-005 ARCH family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-132` | emergence | civ-005 ARCH family (1/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 1 |
| `FR-CIV-EMERGENCE-133` | emergence | civ-005 ARCH family (2/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 2 |
| `FR-CIV-EMERGENCE-134` | emergence | civ-005 ARCH family (3/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 3 |
| `FR-CIV-EMERGENCE-135` | emergence | civ-005 ARCH family (4/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 4 |
| `FR-CIV-EMERGENCE-136` | emergence | civ-005 ARCH family (5/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 5 |
| `FR-CIV-EMERGENCE-137` | emergence | civ-005 ARCH family (6/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 6 |
| `FR-CIV-EMERGENCE-138` | emergence | civ-005 ARCH family (7/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 7 |
| `FR-CIV-EMERGENCE-139` | emergence | civ-005 ARCH family (8/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 8 |
| `FR-CIV-EMERGENCE-140` | emergence | civ-005 ARCH family (9/9) | dormant | dormant | engine | tbd | ARCH sub-cluster row 9 |


## FR-CIV-EMERGENCE batch CLIMATE (3 rows — civ-006 CLIMATE family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-141` | emergence | civ-006 CLIMATE family (1/3) | dormant | partial | engine | tbd | CLIMATE sub-cluster row 1 |
| `FR-CIV-EMERGENCE-142` | emergence | civ-006 CLIMATE family (2/3) | dormant | partial | engine | tbd | CLIMATE sub-cluster row 2 |
| `FR-CIV-EMERGENCE-143` | emergence | civ-006 CLIMATE family (3/3) | dormant | partial | engine | tbd | CLIMATE sub-cluster row 3 |


## FR-CIV-EMERGENCE batch ECON (7 rows — civ-007 ECON family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-144` | emergence | civ-007 ECON family (1/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 1 |
| `FR-CIV-EMERGENCE-145` | emergence | civ-007 ECON family (2/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 2 |
| `FR-CIV-EMERGENCE-146` | emergence | civ-007 ECON family (3/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 3 |
| `FR-CIV-EMERGENCE-147` | emergence | civ-007 ECON family (4/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 4 |
| `FR-CIV-EMERGENCE-148` | emergence | civ-007 ECON family (5/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 5 |
| `FR-CIV-EMERGENCE-149` | emergence | civ-007 ECON family (6/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 6 |
| `FR-CIV-EMERGENCE-150` | emergence | civ-007 ECON family (7/7) | dormant | implemented | engine | tbd | ECON sub-cluster row 7 |


## FR-CIV-EMERGENCE batch DEMO (17 rows — civ-008 DEMO family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-151` | emergence | civ-008 DEMO family (1/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 1 |
| `FR-CIV-EMERGENCE-152` | emergence | civ-008 DEMO family (2/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 2 |
| `FR-CIV-EMERGENCE-153` | emergence | civ-008 DEMO family (3/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 3 |
| `FR-CIV-EMERGENCE-154` | emergence | civ-008 DEMO family (4/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 4 |
| `FR-CIV-EMERGENCE-155` | emergence | civ-008 DEMO family (5/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 5 |
| `FR-CIV-EMERGENCE-156` | emergence | civ-008 DEMO family (6/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 6 |
| `FR-CIV-EMERGENCE-157` | emergence | civ-008 DEMO family (7/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 7 |
| `FR-CIV-EMERGENCE-158` | emergence | civ-008 DEMO family (8/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 8 |
| `FR-CIV-EMERGENCE-159` | emergence | civ-008 DEMO family (9/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 9 |
| `FR-CIV-EMERGENCE-160` | emergence | civ-008 DEMO family (10/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 10 |
| `FR-CIV-EMERGENCE-161` | emergence | civ-008 DEMO family (11/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 11 |
| `FR-CIV-EMERGENCE-162` | emergence | civ-008 DEMO family (12/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 12 |
| `FR-CIV-EMERGENCE-163` | emergence | civ-008 DEMO family (13/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 13 |
| `FR-CIV-EMERGENCE-164` | emergence | civ-008 DEMO family (14/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 14 |
| `FR-CIV-EMERGENCE-165` | emergence | civ-008 DEMO family (15/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 15 |
| `FR-CIV-EMERGENCE-166` | emergence | civ-008 DEMO family (16/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 16 |
| `FR-CIV-EMERGENCE-167` | emergence | civ-008 DEMO family (17/17) | dormant | dormant | engine | tbd | DEMO sub-cluster row 17 |


## FR-CIV-EMERGENCE batch PSYCHE (25 rows — civ-009 PSYCHE family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-168` | emergence | civ-009 PSYCHE family (1/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 1 |
| `FR-CIV-EMERGENCE-169` | emergence | civ-009 PSYCHE family (2/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 2 |
| `FR-CIV-EMERGENCE-170` | emergence | civ-009 PSYCHE family (3/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 3 |
| `FR-CIV-EMERGENCE-171` | emergence | civ-009 PSYCHE family (4/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 4 |
| `FR-CIV-EMERGENCE-172` | emergence | civ-009 PSYCHE family (5/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 5 |
| `FR-CIV-EMERGENCE-173` | emergence | civ-009 PSYCHE family (6/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 6 |
| `FR-CIV-EMERGENCE-174` | emergence | civ-009 PSYCHE family (7/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 7 |
| `FR-CIV-EMERGENCE-175` | emergence | civ-009 PSYCHE family (8/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 8 |
| `FR-CIV-EMERGENCE-176` | emergence | civ-009 PSYCHE family (9/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 9 |
| `FR-CIV-EMERGENCE-177` | emergence | civ-009 PSYCHE family (10/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 10 |
| `FR-CIV-EMERGENCE-178` | emergence | civ-009 PSYCHE family (11/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 11 |
| `FR-CIV-EMERGENCE-179` | emergence | civ-009 PSYCHE family (12/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 12 |
| `FR-CIV-EMERGENCE-180` | emergence | civ-009 PSYCHE family (13/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 13 |
| `FR-CIV-EMERGENCE-181` | emergence | civ-009 PSYCHE family (14/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 14 |
| `FR-CIV-EMERGENCE-182` | emergence | civ-009 PSYCHE family (15/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 15 |
| `FR-CIV-EMERGENCE-183` | emergence | civ-009 PSYCHE family (16/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 16 |
| `FR-CIV-EMERGENCE-184` | emergence | civ-009 PSYCHE family (17/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 17 |
| `FR-CIV-EMERGENCE-185` | emergence | civ-009 PSYCHE family (18/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 18 |
| `FR-CIV-EMERGENCE-186` | emergence | civ-009 PSYCHE family (19/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 19 |
| `FR-CIV-EMERGENCE-187` | emergence | civ-009 PSYCHE family (20/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 20 |
| `FR-CIV-EMERGENCE-188` | emergence | civ-009 PSYCHE family (21/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 21 |
| `FR-CIV-EMERGENCE-189` | emergence | civ-009 PSYCHE family (22/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 22 |
| `FR-CIV-EMERGENCE-190` | emergence | civ-009 PSYCHE family (23/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 23 |
| `FR-CIV-EMERGENCE-191` | emergence | civ-009 PSYCHE family (24/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 24 |
| `FR-CIV-EMERGENCE-192` | emergence | civ-009 PSYCHE family (25/25) | dormant | dormant | engine | tbd | PSYCHE sub-cluster row 25 |


## FR-CIV-EMERGENCE batch LEGENDS (15 rows — civ-010 LEGENDS family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-193` | emergence | civ-010 LEGENDS family (1/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 1 |
| `FR-CIV-EMERGENCE-194` | emergence | civ-010 LEGENDS family (2/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 2 |
| `FR-CIV-EMERGENCE-195` | emergence | civ-010 LEGENDS family (3/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 3 |
| `FR-CIV-EMERGENCE-196` | emergence | civ-010 LEGENDS family (4/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 4 |
| `FR-CIV-EMERGENCE-197` | emergence | civ-010 LEGENDS family (5/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 5 |
| `FR-CIV-EMERGENCE-198` | emergence | civ-010 LEGENDS family (6/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 6 |
| `FR-CIV-EMERGENCE-199` | emergence | civ-010 LEGENDS family (7/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 7 |
| `FR-CIV-EMERGENCE-200` | emergence | civ-010 LEGENDS family (8/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 8 |
| `FR-CIV-EMERGENCE-201` | emergence | civ-010 LEGENDS family (9/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 9 |
| `FR-CIV-EMERGENCE-202` | emergence | civ-010 LEGENDS family (10/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 10 |
| `FR-CIV-EMERGENCE-203` | emergence | civ-010 LEGENDS family (11/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 11 |
| `FR-CIV-EMERGENCE-204` | emergence | civ-010 LEGENDS family (12/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 12 |
| `FR-CIV-EMERGENCE-205` | emergence | civ-010 LEGENDS family (13/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 13 |
| `FR-CIV-EMERGENCE-206` | emergence | civ-010 LEGENDS family (14/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 14 |
| `FR-CIV-EMERGENCE-207` | emergence | civ-010 LEGENDS family (15/15) | dormant | implemented | engine | tbd | LEGENDS sub-cluster row 15 |


## FR-CIV-EMERGENCE batch AI (15 rows — civ-011 AI family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-208` | emergence | civ-011 AI family (1/15) | dormant | implemented | engine | tbd | AI sub-cluster row 1 |
| `FR-CIV-EMERGENCE-209` | emergence | civ-011 AI family (2/15) | dormant | implemented | engine | tbd | AI sub-cluster row 2 |
| `FR-CIV-EMERGENCE-210` | emergence | civ-011 AI family (3/15) | dormant | implemented | engine | tbd | AI sub-cluster row 3 |
| `FR-CIV-EMERGENCE-211` | emergence | civ-011 AI family (4/15) | dormant | implemented | engine | tbd | AI sub-cluster row 4 |
| `FR-CIV-EMERGENCE-212` | emergence | civ-011 AI family (5/15) | dormant | implemented | engine | tbd | AI sub-cluster row 5 |
| `FR-CIV-EMERGENCE-213` | emergence | civ-011 AI family (6/15) | dormant | implemented | engine | tbd | AI sub-cluster row 6 |
| `FR-CIV-EMERGENCE-214` | emergence | civ-011 AI family (7/15) | dormant | implemented | engine | tbd | AI sub-cluster row 7 |
| `FR-CIV-EMERGENCE-215` | emergence | civ-011 AI family (8/15) | dormant | implemented | engine | tbd | AI sub-cluster row 8 |
| `FR-CIV-EMERGENCE-216` | emergence | civ-011 AI family (9/15) | dormant | implemented | engine | tbd | AI sub-cluster row 9 |
| `FR-CIV-EMERGENCE-217` | emergence | civ-011 AI family (10/15) | dormant | implemented | engine | tbd | AI sub-cluster row 10 |
| `FR-CIV-EMERGENCE-218` | emergence | civ-011 AI family (11/15) | dormant | implemented | engine | tbd | AI sub-cluster row 11 |
| `FR-CIV-EMERGENCE-219` | emergence | civ-011 AI family (12/15) | dormant | implemented | engine | tbd | AI sub-cluster row 12 |
| `FR-CIV-EMERGENCE-220` | emergence | civ-011 AI family (13/15) | dormant | implemented | engine | tbd | AI sub-cluster row 13 |
| `FR-CIV-EMERGENCE-221` | emergence | civ-011 AI family (14/15) | dormant | implemented | engine | tbd | AI sub-cluster row 14 |
| `FR-CIV-EMERGENCE-222` | emergence | civ-011 AI family (15/15) | dormant | implemented | engine | tbd | AI sub-cluster row 15 |


## FR-CIV-EMERGENCE batch CULT (3 rows — civ-012 CULT family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-223` | emergence | civ-012 CULT family (1/3) | dormant | dormant | engine | tbd | CULT sub-cluster row 1 |
| `FR-CIV-EMERGENCE-224` | emergence | civ-012 CULT family (2/3) | dormant | dormant | engine | tbd | CULT sub-cluster row 2 |
| `FR-CIV-EMERGENCE-225` | emergence | civ-012 CULT family (3/3) | dormant | dormant | engine | tbd | CULT sub-cluster row 3 |


## FR-CIV-EMERGENCE batch SOCIAL (2 rows — civ-013 SOCIAL family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-226` | emergence | civ-013 SOCIAL family (1/2) | dormant | dormant | engine | tbd | SOCIAL sub-cluster row 1 |
| `FR-CIV-EMERGENCE-227` | emergence | civ-013 SOCIAL family (2/2) | dormant | dormant | engine | tbd | SOCIAL sub-cluster row 2 |


## FR-CIV-EMERGENCE batch DIPLO (8 rows — civ-014 DIPLO family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-228` | emergence | civ-014 DIPLO family (1/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 1 |
| `FR-CIV-EMERGENCE-229` | emergence | civ-014 DIPLO family (2/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 2 |
| `FR-CIV-EMERGENCE-230` | emergence | civ-014 DIPLO family (3/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 3 |
| `FR-CIV-EMERGENCE-231` | emergence | civ-014 DIPLO family (4/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 4 |
| `FR-CIV-EMERGENCE-232` | emergence | civ-014 DIPLO family (5/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 5 |
| `FR-CIV-EMERGENCE-233` | emergence | civ-014 DIPLO family (6/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 6 |
| `FR-CIV-EMERGENCE-234` | emergence | civ-014 DIPLO family (7/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 7 |
| `FR-CIV-EMERGENCE-235` | emergence | civ-014 DIPLO family (8/8) | dormant | partial | engine | tbd | DIPLO sub-cluster row 8 |


## FR-CIV-EMERGENCE batch LAWS (6 rows — civ-015 LAWS family)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-236` | emergence | civ-015 LAWS family (1/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 1 |
| `FR-CIV-EMERGENCE-237` | emergence | civ-015 LAWS family (2/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 2 |
| `FR-CIV-EMERGENCE-238` | emergence | civ-015 LAWS family (3/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 3 |
| `FR-CIV-EMERGENCE-239` | emergence | civ-015 LAWS family (4/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 4 |
| `FR-CIV-EMERGENCE-240` | emergence | civ-015 LAWS family (5/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 5 |
| `FR-CIV-EMERGENCE-241` | emergence | civ-015 LAWS family (6/6) | dormant | partial | engine | tbd | LAWS sub-cluster row 6 |


## FR-CIV-EMERGENCE batch INT-1 (1 rows — civ-016 INT-1 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-242` | emergence | civ-016 INT-1 charter integration (1/1) | dormant | dormant | engine | tbd | INT-1 sub-cluster row 1 |


## FR-CIV-EMERGENCE batch INT-2 (1 rows — civ-017 INT-2 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-243` | emergence | civ-017 INT-2 charter integration (1/1) | dormant | dormant | engine | tbd | INT-2 sub-cluster row 1 |


## FR-CIV-EMERGENCE batch INT-3 (1 rows — civ-018 INT-3 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-244` | emergence | civ-018 INT-3 charter integration (1/1) | dormant | dormant | engine | tbd | INT-3 sub-cluster row 1 |


## FR-CIV-EMERGENCE batch INT-4 (1 rows — civ-022 INT-4 charter integration)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-245` | emergence | civ-022 INT-4 charter integration (1/1) | dormant | dormant | engine | tbd | INT-4 sub-cluster row 1 |
