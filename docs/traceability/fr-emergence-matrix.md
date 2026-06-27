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

## Section B: Emergence batch rows (§3.1 + §3.2 + §3.3, all currently dormant)


### LANG family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-100 | LANG | Phonology | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Linguistic emergence — phonological family, row 1/3 |
| FR-CIV-EMERGENCE-101 | LANG | Phonology | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Linguistic emergence — phonological family, row 2/3 |
| FR-CIV-EMERGENCE-102 | LANG | Phonology | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Linguistic emergence — phonological family, row 3/3 |
| FR-CIV-EMERGENCE-103 | LANG | Lexicon | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Lexical emergence + cross-language borrowing, row 1/3 |
| FR-CIV-EMERGENCE-104 | LANG | Lexicon | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Lexical emergence + cross-language borrowing, row 2/3 |
| FR-CIV-EMERGENCE-105 | LANG | Lexicon | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Lexical emergence + cross-language borrowing, row 3/3 |
| FR-CIV-EMERGENCE-106 | LANG | Grammar | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Syntactic emergence + morphology drift, row 1/3 |
| FR-CIV-EMERGENCE-107 | LANG | Grammar | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Syntactic emergence + morphology drift, row 2/3 |
| FR-CIV-EMERGENCE-108 | LANG | Grammar | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Syntactic emergence + morphology drift, row 3/3 |
| FR-CIV-EMERGENCE-109 | LANG | Pidgins | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Pidgin + creole formation from contact zones, row 1/2 |
| FR-CIV-EMERGENCE-110 | LANG | Pidgins | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Pidgin + creole formation from contact zones, row 2/2 |

### POLITY family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-111 | POLITY | Rise | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity rise — initial state formation, row 1/2 |
| FR-CIV-EMERGENCE-112 | POLITY | Rise | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity rise — initial state formation, row 2/2 |
| FR-CIV-EMERGENCE-113 | POLITY | Consolidation | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity consolidation — bureaucratic specialization, row 1/2 |
| FR-CIV-EMERGENCE-114 | POLITY | Consolidation | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity consolidation — bureaucratic specialization, row 2/2 |
| FR-CIV-EMERGENCE-115 | POLITY | Decline | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity decline — succession crisis + fragmentation, row 1/2 |
| FR-CIV-EMERGENCE-116 | POLITY | Decline | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity decline — succession crisis + fragmentation, row 2/2 |
| FR-CIV-EMERGENCE-117 | POLITY | Succession | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity succession — hereditary vs elective vs appointed, row 1/2 |
| FR-CIV-EMERGENCE-118 | POLITY | Succession | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | Polity succession — hereditary vs elective vs appointed, row 2/2 |

### REL family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-119 | REL | Pantheon | phase-1 | dormant | `civ-religion` | `crates/civ-religion/src/lib.rs` | Religion emergence — pantheon formation, row 1/2 |
| FR-CIV-EMERGENCE-120 | REL | Pantheon | phase-1 | dormant | `civ-religion` | `crates/civ-religion/src/lib.rs` | Religion emergence — pantheon formation, row 2/2 |
| FR-CIV-EMERGENCE-121 | REL | Schism | phase-1 | dormant | `civ-religion` | `crates/civ-religion/src/lib.rs` | Religious schism — reform movements + splinter sects, row 1/2 |
| FR-CIV-EMERGENCE-122 | REL | Schism | phase-1 | dormant | `civ-religion` | `crates/civ-religion/src/lib.rs` | Religious schism — reform movements + splinter sects, row 2/2 |
| FR-CIV-EMERGENCE-123 | REL | Syncretism | phase-1 | dormant | `civ-religion` | `crates/civ-religion/src/lib.rs` | Religious syncretism — merger of competing pantheons, row 1/1 |

### MARKET family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-124 | MARKET | Currency | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Currency emergence — barter → commodity → coin, row 1/2 |
| FR-CIV-EMERGENCE-125 | MARKET | Currency | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Currency emergence — barter → commodity → coin, row 2/2 |
| FR-CIV-EMERGENCE-126 | MARKET | Trade Networks | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Trade network formation + specialization, row 1/2 |
| FR-CIV-EMERGENCE-127 | MARKET | Trade Networks | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Trade network formation + specialization, row 2/2 |
| FR-CIV-EMERGENCE-128 | MARKET | Price Discovery | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Price discovery — supply/demand feedback loops, row 1/2 |
| FR-CIV-EMERGENCE-129 | MARKET | Price Discovery | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Price discovery — supply/demand feedback loops, row 2/2 |
| FR-CIV-EMERGENCE-130 | MARKET | Guilds | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Guild emergence — craft monopolies + apprenticeship, row 1/2 |
| FR-CIV-EMERGENCE-131 | MARKET | Guilds | phase-1 | dormant | `civ-market` | `crates/civ-market/src/lib.rs` | Guild emergence — craft monopolies + apprenticeship, row 2/2 |

### ARCH family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-132 | ARCH | Settlement | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Settlement patterns — village → town → city, row 1/2 |
| FR-CIV-EMERGENCE-133 | ARCH | Settlement | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Settlement patterns — village → town → city, row 2/2 |
| FR-CIV-EMERGENCE-134 | ARCH | Building Types | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Building type emergence — civic/religious/commercial, row 1/2 |
| FR-CIV-EMERGENCE-135 | ARCH | Building Types | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Building type emergence — civic/religious/commercial, row 2/2 |
| FR-CIV-EMERGENCE-136 | ARCH | Infrastructure | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Infrastructure — roads, aqueducts, defenses, row 1/2 |
| FR-CIV-EMERGENCE-137 | ARCH | Infrastructure | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Infrastructure — roads, aqueducts, defenses, row 2/2 |
| FR-CIV-EMERGENCE-138 | ARCH | Monuments | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Monuments — pyramid/ziggurat/temple emergence, row 1/2 |
| FR-CIV-EMERGENCE-139 | ARCH | Monuments | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Monuments — pyramid/ziggurat/temple emergence, row 2/2 |
| FR-CIV-EMERGENCE-140 | ARCH | Plague Resilience | phase-1 | dormant | `civ-urban` | `crates/civ-urban/src/lib.rs` | Plague-resilient urban planning (latrines, water), row 1/1 |

### CLIMATE family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-141 | CLIMATE | Microclimate | phase-1 | dormant | `civ-climate` | `crates/civ-climate/src/lib.rs` | Urban microclimate effects (heat island, drainage), row 1/1 |
| FR-CIV-EMERGENCE-142 | CLIMATE | Refugee Patterns | phase-1 | dormant | `civ-climate` | `crates/civ-climate/src/lib.rs` | Climate-refugee migration patterns, row 1/1 |
| FR-CIV-EMERGENCE-143 | CLIMATE | Resource Stress | phase-1 | dormant | `civ-climate` | `crates/civ-climate/src/lib.rs` | Resource-stress driven conflict + cooperation, row 1/1 |

### ECON family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-144 | ECON | Specialization | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Labor specialization + occupational castes, row 1/2 |
| FR-CIV-EMERGENCE-145 | ECON | Specialization | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Labor specialization + occupational castes, row 2/2 |
| FR-CIV-EMERGENCE-146 | ECON | Trade Goods | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Trade-good emergence + luxury status goods, row 1/2 |
| FR-CIV-EMERGENCE-147 | ECON | Trade Goods | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Trade-good emergence + luxury status goods, row 2/2 |
| FR-CIV-EMERGENCE-148 | ECON | Taxation | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Taxation emergence — tribute → tax → customs, row 1/1 |
| FR-CIV-EMERGENCE-149 | ECON | Banking | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Banking + credit emergence (proto-banks, letters of credit), row 1/1 |
| FR-CIV-EMERGENCE-150 | ECON | Famine Recovery | phase-1 | dormant | `civ-econ` | `crates/civ-econ/src/lib.rs` | Famine-recovery institutions (granaries, redistribution), row 1/1 |

### DEMO family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-151 | DEMO | Migration | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Migration flows — push (famine/war) vs pull (opportunity), row 1/2 |
| FR-CIV-EMERGENCE-152 | DEMO | Migration | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Migration flows — push (famine/war) vs pull (opportunity), row 2/2 |
| FR-CIV-EMERGENCE-153 | DEMO | Disease | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Disease dynamics — endemic vs epidemic vs pandemic, row 1/2 |
| FR-CIV-EMERGENCE-154 | DEMO | Disease | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Disease dynamics — endemic vs epidemic vs pandemic, row 2/2 |
| FR-CIV-EMERGENCE-155 | DEMO | Age Structure | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Age-structure transition (high fertility → low fertility), row 1/2 |
| FR-CIV-EMERGENCE-156 | DEMO | Age Structure | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Age-structure transition (high fertility → low fertility), row 2/2 |
| FR-CIV-EMERGENCE-157 | DEMO | Urbanization | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Urbanization — rural-to-urban migration, row 1/2 |
| FR-CIV-EMERGENCE-158 | DEMO | Urbanization | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Urbanization — rural-to-urban migration, row 2/2 |
| FR-CIV-EMERGENCE-159 | DEMO | Genocide Risk | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Genocide-risk indicators (ethnic stratification + crisis), row 1/2 |
| FR-CIV-EMERGENCE-160 | DEMO | Genocide Risk | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Genocide-risk indicators (ethnic stratification + crisis), row 2/2 |
| FR-CIV-EMERGENCE-161 | DEMO | Family Structure | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Family structure transition — extended → nuclear, row 1/2 |
| FR-CIV-EMERGENCE-162 | DEMO | Family Structure | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Family structure transition — extended → nuclear, row 2/2 |
| FR-CIV-EMERGENCE-163 | DEMO | Gender Roles | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Gender-role emergence + change, row 1/2 |
| FR-CIV-EMERGENCE-164 | DEMO | Gender Roles | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Gender-role emergence + change, row 2/2 |
| FR-CIV-EMERGENCE-165 | DEMO | Education | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Education emergence — apprenticeship → academy, row 1/2 |
| FR-CIV-EMERGENCE-166 | DEMO | Education | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Education emergence — apprenticeship → academy, row 2/2 |
| FR-CIV-EMERGENCE-167 | DEMO | Medicine | phase-1 | dormant | `civ-demographics` | `crates/civ-demographics/src/lib.rs` | Medicine emergence — herbalism → surgery → germ theory, row 1/1 |

### PSYCHE family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-168 | PSYCHE | Personality | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Personality trait emergence (Big-Five analogues), row 1/2 |
| FR-CIV-EMERGENCE-169 | PSYCHE | Personality | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Personality trait emergence (Big-Five analogues), row 2/2 |
| FR-CIV-EMERGENCE-170 | PSYCHE | Cognitive Style | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Cognitive-style emergence — oral → literate → digital, row 1/2 |
| FR-CIV-EMERGENCE-171 | PSYCHE | Cognitive Style | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Cognitive-style emergence — oral → literate → digital, row 2/2 |
| FR-CIV-EMERGENCE-172 | PSYCHE | Moral Foundations | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Moral foundation emergence (care/fairness/loyalty/etc.), row 1/2 |
| FR-CIV-EMERGENCE-173 | PSYCHE | Moral Foundations | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Moral foundation emergence (care/fairness/loyalty/etc.), row 2/2 |
| FR-CIV-EMERGENCE-174 | PSYCHE | Stress | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Stress response patterns + coping strategies, row 1/2 |
| FR-CIV-EMERGENCE-175 | PSYCHE | Stress | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Stress response patterns + coping strategies, row 2/2 |
| FR-CIV-EMERGENCE-176 | PSYCHE | Trust | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Trust emergence — bonding/bridging/linking social capital, row 1/2 |
| FR-CIV-EMERGENCE-177 | PSYCHE | Trust | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Trust emergence — bonding/bridging/linking social capital, row 2/2 |
| FR-CIV-EMERGENCE-178 | PSYCHE | Prejudice | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Prejudice formation + de-biasing mechanisms, row 1/2 |
| FR-CIV-EMERGENCE-179 | PSYCHE | Prejudice | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Prejudice formation + de-biasing mechanisms, row 2/2 |
| FR-CIV-EMERGENCE-180 | PSYCHE | Trauma | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Collective trauma + intergenerational transmission, row 1/2 |
| FR-CIV-EMERGENCE-181 | PSYCHE | Trauma | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Collective trauma + intergenerational transmission, row 2/2 |
| FR-CIV-EMERGENCE-182 | PSYCHE | Creativity | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Creativity emergence — divergent thinking + incubation, row 1/2 |
| FR-CIV-EMERGENCE-183 | PSYCHE | Creativity | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Creativity emergence — divergent thinking + incubation, row 2/2 |
| FR-CIV-EMERGENCE-184 | PSYCHE | Addiction | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Addiction dynamics (substance + behavioral), row 1/2 |
| FR-CIV-EMERGENCE-185 | PSYCHE | Addiction | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Addiction dynamics (substance + behavioral), row 2/2 |
| FR-CIV-EMERGENCE-186 | PSYCHE | Mental Health | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Mental-health emergence — distress → disorder → treatment, row 1/2 |
| FR-CIV-EMERGENCE-187 | PSYCHE | Mental Health | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Mental-health emergence — distress → disorder → treatment, row 2/2 |
| FR-CIV-EMERGENCE-188 | PSYCHE | Conformity | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Conformity vs deviance dynamics, row 1/2 |
| FR-CIV-EMERGENCE-189 | PSYCHE | Conformity | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Conformity vs deviance dynamics, row 2/2 |
| FR-CIV-EMERGENCE-190 | PSYCHE | Ritual | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Ritual formation — superstition → ceremony → liturgy, row 1/2 |
| FR-CIV-EMERGENCE-191 | PSYCHE | Ritual | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Ritual formation — superstition → ceremony → liturgy, row 2/2 |
| FR-CIV-EMERGENCE-192 | PSYCHE | Humor | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Humor emergence — relief + superiority + incongruity, row 1/2 |
| FR-CIV-EMERGENCE-193 | PSYCHE | Humor | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Humor emergence — relief + superiority + incongruity, row 2/2 |
| FR-CIV-EMERGENCE-194 | PSYCHE | Shame/Guilt | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Shame-culture vs guilt-culture emergence, row 1/2 |
| FR-CIV-EMERGENCE-195 | PSYCHE | Shame/Guilt | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Shame-culture vs guilt-culture emergence, row 2/2 |
| FR-CIV-EMERGENCE-196 | PSYCHE | Empathy | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Empathy emergence — mirror-neuron → perspective-taking, row 1/1 |
| FR-CIV-EMERGENCE-197 | PSYCHE | Death Anxiety | phase-1 | dormant | `civ-psyche` | `crates/civ-psyche/src/lib.rs` | Death-anxiety coping strategies, row 1/1 |

### LEGENDS family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-198 | LEGENDS | Origin Myths | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Origin myths — cosmology + creation, row 1/2 |
| FR-CIV-EMERGENCE-199 | LEGENDS | Origin Myths | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Origin myths — cosmology + creation, row 2/2 |
| FR-CIV-EMERGENCE-200 | LEGENDS | Hero Cycles | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Hero cycles — trials + descent + return, row 1/2 |
| FR-CIV-EMERGENCE-201 | LEGENDS | Hero Cycles | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Hero cycles — trials + descent + return, row 2/2 |
| FR-CIV-EMERGENCE-202 | LEGENDS | Trickster | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Trickster figures — boundary-crossers + culture heros, row 1/2 |
| FR-CIV-EMERGENCE-203 | LEGENDS | Trickster | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Trickster figures — boundary-crossers + culture heros, row 2/2 |
| FR-CIV-EMERGENCE-204 | LEGENDS | Flood Myths | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Flood myths — common across cultures, climate signal?, row 1/2 |
| FR-CIV-EMERGENCE-205 | LEGENDS | Flood Myths | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Flood myths — common across cultures, climate signal?, row 2/2 |
| FR-CIV-EMERGENCE-206 | LEGENDS | Underworld | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Underworld descent — death + rebirth archetypes, row 1/2 |
| FR-CIV-EMERGENCE-207 | LEGENDS | Underworld | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Underworld descent — death + rebirth archetypes, row 2/2 |
| FR-CIV-EMERGENCE-208 | LEGENDS | Monster Lore | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Monster lore — liminal beings + danger zones, row 1/2 |
| FR-CIV-EMERGENCE-209 | LEGENDS | Monster Lore | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Monster lore — liminal beings + danger zones, row 2/2 |
| FR-CIV-EMERGENCE-210 | LEGENDS | Prophetic | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Prophetic cycles — decline prediction + renewal promise, row 1/2 |
| FR-CIV-EMERGENCE-211 | LEGENDS | Prophetic | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Prophetic cycles — decline prediction + renewal promise, row 2/2 |
| FR-CIV-EMERGENCE-212 | LEGENDS | Sacred Kingship | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Sacred-kingship myths — divine right + regicide, row 1/2 |
| FR-CIV-EMERGENCE-213 | LEGENDS | Sacred Kingship | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Sacred-kingship myths — divine right + regicide, row 2/2 |
| FR-CIV-EMERGENCE-214 | LEGENDS | World-Tree | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | World-tree / axis-mundi emergence, row 1/2 |
| FR-CIV-EMERGENCE-215 | LEGENDS | World-Tree | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | World-tree / axis-mundi emergence, row 2/2 |
| FR-CIV-EMERGENCE-216 | LEGENDS | Flood Survival | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Flood-survival archetypes + boat symbolism, row 1/2 |
| FR-CIV-EMERGENCE-217 | LEGENDS | Flood Survival | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Flood-survival archetypes + boat symbolism, row 2/2 |
| FR-CIV-EMERGENCE-218 | LEGENDS | Twin Founders | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Twin-founder myths (Romulus/Remus pattern), row 1/2 |
| FR-CIV-EMERGENCE-219 | LEGENDS | Twin Founders | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Twin-founder myths (Romulus/Remus pattern), row 2/2 |
| FR-CIV-EMERGENCE-220 | LEGENDS | Sun Goddess | phase-1 | dormant | `civ-legends` | `crates/civ-legends/src/lib.rs` | Sun-goddess + solar-hero myths, row 1/1 |

### AI family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-221 | AI | Tactical Reasoning | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Tactical reasoning emergence (heuristic search), row 1/2 |
| FR-CIV-EMERGENCE-222 | AI | Tactical Reasoning | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Tactical reasoning emergence (heuristic search), row 2/2 |
| FR-CIV-EMERGENCE-223 | AI | Diplomatic Strategy | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Diplomatic-strategy emergence (game-theoretic balance), row 1/2 |
| FR-CIV-EMERGENCE-224 | AI | Diplomatic Strategy | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Diplomatic-strategy emergence (game-theoretic balance), row 2/2 |
| FR-CIV-EMERGENCE-225 | AI | Resource Planning | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Resource-planning emergence (multi-step optimization), row 1/2 |
| FR-CIV-EMERGENCE-226 | AI | Resource Planning | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Resource-planning emergence (multi-step optimization), row 2/2 |
| FR-CIV-EMERGENCE-227 | AI | Learning | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Adaptive learning emergence (online reinforcement), row 1/2 |
| FR-CIV-EMERGENCE-228 | AI | Learning | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Adaptive learning emergence (online reinforcement), row 2/2 |
| FR-CIV-EMERGENCE-229 | AI | Social Modeling | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Social-modeling emergence (Theory-of-Mind analogues), row 1/2 |
| FR-CIV-EMERGENCE-230 | AI | Social Modeling | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Social-modeling emergence (Theory-of-Mind analogues), row 2/2 |
| FR-CIV-EMERGENCE-231 | AI | Creativity Search | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Creative-search emergence (novelty + surprise), row 1/2 |
| FR-CIV-EMERGENCE-232 | AI | Creativity Search | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Creative-search emergence (novelty + surprise), row 2/2 |
| FR-CIV-EMERGENCE-233 | AI | Risk Assessment | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Risk-assessment emergence (loss-aversion + prospect theory), row 1/1 |
| FR-CIV-EMERGENCE-234 | AI | Deception | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Deception-detection emergence (lie-catch + reputation), row 1/1 |
| FR-CIV-EMERGENCE-235 | AI | Coordination | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | Multi-agent coordination emergence (signaling + conventions), row 1/1 |

### CULT family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-236 | CULT | Music | phase-1 | dormant | `civ-culture` | `crates/civ-culture/src/lib.rs` | Musical tradition emergence — rhythm + melody + harmony, row 1/1 |
| FR-CIV-EMERGENCE-237 | CULT | Cuisine | phase-1 | dormant | `civ-culture` | `crates/civ-culture/src/lib.rs` | Cuisine emergence — fermentation + spice trade + taboo, row 1/1 |
| FR-CIV-EMERGENCE-238 | CULT | Fashion | phase-1 | dormant | `civ-culture` | `crates/civ-culture/src/lib.rs` | Fashion emergence — status display + identity, row 1/1 |

### SOCIAL family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-239 | SOCIAL | Honor Codes | phase-1 | dormant | `civ-social` | `crates/civ-social/src/lib.rs` | Honor-code emergence — duel + gift + vengeance, row 1/1 |
| FR-CIV-EMERGENCE-240 | SOCIAL | Gift Economies | phase-1 | dormant | `civ-social` | `crates/civ-social/src/lib.rs` | Gift-economy emergence — potlatch + kula ring, row 1/1 |

### DIPLO family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-241 | DIPLO | Alliance | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | Alliance emergence — treaty + marriage + hostage, row 1/2 |
| FR-CIV-EMERGENCE-242 | DIPLO | Alliance | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | Alliance emergence — treaty + marriage + hostage, row 2/2 |
| FR-CIV-EMERGENCE-243 | DIPLO | War Declaration | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | War-declaration rituals + casus belli formation, row 1/2 |
| FR-CIV-EMERGENCE-244 | DIPLO | War Declaration | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | War-declaration rituals + casus belli formation, row 2/2 |
| FR-CIV-EMERGENCE-245 | DIPLO | Peace Negotiation | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | Peace-negotiation emergence (mediator + arbitration), row 1/2 |
| FR-CIV-EMERGENCE-246 | DIPLO | Peace Negotiation | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | Peace-negotiation emergence (mediator + arbitration), row 2/2 |
| FR-CIV-EMERGENCE-247 | DIPLO | Embassy | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | Embassy + permanent-envoy emergence, row 1/2 |
| FR-CIV-EMERGENCE-248 | DIPLO | Embassy | phase-1 | dormant | `civ-diplomacy` | `crates/civ-diplomacy/src/lib.rs` | Embassy + permanent-envoy emergence, row 2/2 |

### LAWS family — emergence rows

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-EMERGENCE-249 | LAWS | Customary Law | phase-1 | dormant | `civ-laws` | `crates/civ-laws/src/lib.rs` | Customary-law emergence (tribal precedent), row 1/2 |
| FR-CIV-EMERGENCE-250 | LAWS | Customary Law | phase-1 | dormant | `civ-laws` | `crates/civ-laws/src/lib.rs` | Customary-law emergence (tribal precedent), row 2/2 |
| FR-CIV-EMERGENCE-251 | LAWS | Codification | phase-1 | dormant | `civ-laws` | `crates/civ-laws/src/lib.rs` | Law codification (Hammurabi/Justinian analogues), row 1/2 |
| FR-CIV-EMERGENCE-252 | LAWS | Codification | phase-1 | dormant | `civ-laws` | `crates/civ-laws/src/lib.rs` | Law codification (Hammurabi/Justinian analogues), row 2/2 |
| FR-CIV-EMERGENCE-253 | LAWS | Punishment | phase-1 | dormant | `civ-laws` | `crates/civ-laws/src/lib.rs` | Punishment-system emergence (restorative vs retributive), row 1/2 |
| FR-CIV-EMERGENCE-254 | LAWS | Punishment | phase-1 | dormant | `civ-laws` | `crates/civ-laws/src/lib.rs` | Punishment-system emergence (restorative vs retributive), row 2/2 |
---

## Section C: Charter integration rows (§3.3)

These 4 rows promote FR-CIV-0100 from the §3.4 charter umbrella into
concrete cross-family integration points. Each integration row
identifies the 2-3 sub-clusters whose emergence must be coordinated
to satisfy the §3.3 charter (FR-CIV-0100).

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| FR-CIV-0100-int1 | POLITY+ECON+MARKET | Polity rise × Taxation × Trade networks | phase-1 | dormant | `civ-factions` | `crates/civ-factions/src/lib.rs` | First emergence that requires cross-family coordination; covered by civ-factions crate + ECON + MARKET batch rows |
| FR-CIV-0100-int2 | REL+PSYCHE+RITUAL | Pantheon × Moral foundations × Ritual formation | phase-1 | dormant | `civ-religion` | `crates/civ-religion/src/lib.rs` | Religion emergence requires PSYCHE moral-foundation + ritual-formation sub-clusters to converge |
| FR-CIV-0100-int3 | LANG+LEGENDS+LAWS | Grammar × Origin myths × Customary law | phase-1 | dormant | `civ-linguabridge` | `crates/civ-linguabridge/src/lib.rs` | Language + legend + law tri-coupling; the deepest emergence in the model |
| FR-CIV-0100-int4 | AI+DIPLO+PSYCHE | Tactical reasoning × Alliance × Deception | phase-1 | dormant | `civ-ai` | `crates/civ-ai/src/lib.rs` | AI agents must coordinate DIPLO + PSCHE for game-theoretic stable outcomes |
