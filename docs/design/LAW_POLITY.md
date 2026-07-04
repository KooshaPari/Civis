# Emergent Law & Polity

**Status:** Design (planner stance — specs/AC/pseudocode only; no implementation).
**Scope:** How *law codes* and *governance forms* (band → chiefdom → state) emerge from the same substrate as polities themselves, and how *enforcement* (norm policing, sanctions, courts) is itself a derived pattern over the substrate. Law is **not** a hardcoded `Code::Hammurabi`/`Code::CommonLaw` enum; governance is **not** a `GovType::Monarchy`/`Republic` enum. Both are **read-out projections** over a continuous substrate.
**Charter anchors:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) §"Polities / states — decentralized, not necessarily explicit mutual collectives" and §"Ideology & culture & language" (norms drift; not authored).
**Sibling docs:** [`polities-markets.md`](./polities-markets.md) (polity shape read-out — FR-CIV-POLITY-001..008), [`CULTURE_IDEOLOGY.md`](./CULTURE_IDEOLOGY.md) (norm/ideology drift substrate), [`DIPLOMACY_EMERGENCE.md`](./DIPLOMACY_EMERGENCE.md) (relation substrate feeding sanctions), [`crates/engine/src/religion.rs`](../../crates/engine/src/religion.rs) (`BeliefConcept::Taboo` is the existing proto-norm primitive).
**Code substrate (read-only inputs):** `crates/agents/src/cluster.rs` (co-location clusters), `crates/agents/src/culture.rs` (`CultureProfile.traits`, `kinship`), `crates/agents/src/diplomacy.rs` (`DiplomacyMatrix`), `crates/economy/src/institution.rs` (`InstitutionLedger` for treasury/postings), `crates/engine/src/faction_emergence.rs` (k-means ideology clusters), `crates/engine/src/religion.rs` (`Belief`/`BeliefConcept` — natural proto-norms), `crates/engine/src/emergence.rs` (`emergence_legends`, `emergence_social`).

---

## 0. Governing principle

The simulation hardcodes *what agents prefer, what they can do, and what they can remember*. It does **not** hardcode **what they ought to do**, **who decides that**, or **what happens if they don't**.

A **law** is not a stored rule. It is a **statistical regularity** in the (norm, enforcer, response) triple that arises in a population. A **code** (e.g. "Hammurabi," "Twelve Tables," "common law," "sharia," "halakha") is a **read-out label** projected onto the regularity for historians and UI. A **governance form** is a **read-out label** projected onto the *coordination topology* of how rules are decided and applied, not an enum the sim authors.

**Design rule (restated):** model the *behavioral mechanics* — norm formation, norm transmission, norm violation detection, norm-violation response, norm codification. The category labels in this doc are **read-out projections** for the UI and the history feed, not stored authoritative types. This mirrors `polities-markets.md` §0 and `CULTURE_IDEOLOGY.md` §0 exactly.

**Consequence — same as polity:** the engine never has a `GovType::Monarchy` field. The dynasty / chief / magistrate label is computed by a classifier over the continuous substrate. Two "monarchies" with different substrate statistics are different political realities that the UI happens to call the same thing.

---

## 1. Layering: polity shape vs governance form vs law code

These three read-outs are **distinct projections** of overlapping substrate. Confusing them is the most common design error.

| Concept | Substrate it reads out | Re-reads as the substrate changes | Reused in this doc |
|---|---|---|---|
| **Polity shape** (anarchic, networked, collective, tributary, hegemonic) | Coordination topology over clusters (cohesion graph) | yes — secession/merge/collapse (POLITY-005/006/007) | §3.1 (the polity container the law lives inside) |
| **Governance form** (band, tribe, chiefdom, state, federation, …) | *Decision procedure* + *enforcement apparatus* inside a polity | yes — can drift while polity shape stays put (e.g. a chiefdom installs a council) | §3 (the central topic) |
| **Law code** (customary, codified, common-law, revealed, …) | *Codification status* + *transmission mode* of the operative norm set | yes — codification is itself a slow drift | §2 (the law content) |
| **Norm** (the actual behavioral rule, e.g. "don't take another clan's waterhole") | a `CultureProfile` dimension, or a `BeliefConcept::Taboo`, or a posture in a `Psyche` | yes — drifts like any cultural trait | §1.1, §2.1 |

All four ride on the same substrate. They re-classify at different rates and on different signals.

### 1.1 Norm substrate (the atoms)

A **norm** is *just* a behavioral disposition, **not** a special component. The substrate already has three places where norms live:

1. **`CultureProfile.traits : [f32; 4]`** — diffused cultural traits, generic axes (e.g. insularity, deference, redistribution, martial valor). These already drift via `mutate_traits` and diffuse via `mix_trait_vectors` along contact edges.
2. **`BeliefConcept::Taboo { action }`** — the existing proto-norm primitive in `crates/engine/src/religion.rs`. Strength + social spread already drive transmission.
3. **Posture in `Psyche`** — a per-agent `PAD`/OCEAN state that, when chronic, *is* a norm-following disposition (e.g. chronic deference → "subjects don't contradict chiefs").

**FR-CIV-LAW-001 — No new norm type.** Norms are computed read-outs of `(CultureProfile.traits, Religion.beliefs[Taboo|Ritual], Psyche, MembershipPayoff drift)`. The sim stores only the underlying substrate; the "norm" object is a *derived annotation* computed on demand for the history feed and UI. This is exactly the "measured, emergent pattern over the substrate" rule from the charter.

### 1.2 Why this matters (the rejection of an authored legal code)

A common mistake is to model a `LegalCode` struct with `Vec<Article>` populated by LLMs or hand-authored JSON. This would:

- inject authored content into a charter-bounded "Layer 1+ emerges" zone,
- collapse **what** a culture prescribes into a single ordered list (cultures don't have one list; they have overlapping, contested, contradictory dispositions),
- freeze the law in place until someone edits the JSON, breaking FR-CIV-DIPLO-007 (no fixed enums) by stealth.

The pattern in this doc reuses the polities-markets read-out pattern (FR-CIV-POLITY-004): **store continuous substrate, project labels on read**.

---

## 2. Law emergence (the *what* of governance)

### 2.1 The norm distribution

For a polity surface `P` (a community in the cohesion graph from POLITY-002), define the **norm distribution** `N_P` as a per-action histogram over the agents in `P`:

```
N_P(action) = {
  prescribed_share: f32,   // 0..1 share of agents with disposition to perform action
  proscribed_share: f32,   // 0..1 share of agents with disposition to refrain
  intensity: f32,          // mean |disposition| (how strongly held, not just how broadly)
  dispute: f32,            // 1 - (prescribed + proscribed) — the share of uncommitted
}
```

**FR-CIV-LAW-002 — Norm distribution derives from agent substrate.** `N_P(action)` is a per-tick aggregate over `(Psyche.deference, CultureProfile.traits, Religion.beliefs[Taboo action=…])` for the agents in `P`'s member cluster overlap. No stored `LegalCode`. The set of `action`s considered is itself a finite vocabulary derived from the action palette the agents can actually take (the `Psyche` action set, the diplomacy action set, the exchange action set) — never an authored list of "crimes."

**FR-CIV-LAW-003 — Codification is a derived state, not a switch.** A norm is *codified* (projectable as a written "code") when:

```
codified(P, action) := dispersion(N_P, action) < codify_threshold
                    ∧ proscribed_share(P, action) > proscribed_floor
                    ∧ persistence(N_P, action, window) > persistence_floor
```

— i.e. the population is highly converged, the proscription is broadly held, and the convergence has persisted for a window. **Codification is a slow, hysteresis-bounded drift** (mirror the polity's secession/merge hysteresis, FR-CIV-POLITY-005). No `bool is_codified` field. The *appearance* of a code (the readable projection) is a function of the substrate's convergence and persistence, not an authored state.

### 2.2 The law-code read-out

**FR-CIV-LAW-004 — Code read-out is a classifier over norm distribution + substrate.**

| Code label | Topological signature |
|---|---|
| **Customary** | Low codification, high dispute → live, contested practice, no single text |
| **Codified statutory** | High codification, low dispute, proscribed floor met → appears as a fixed list |
| **Common-law** | High codification via *precedent* — many small norm records with `precedent_strength` edges between them (a separate read-out over `legends::SagaGraph` rather than over the polity's `LegalCode`) |
| **Revealed / scriptural** | High codification, attached to a `Religion` cluster with a `BeliefConcept::MoralOverseer` (the existing religion.rs primitive) and a `ritual_load` ≥ ritual_floor |
| **Implicit / unstated** | Low codification, low dispute, *and* high kinship insulation — shared so thoroughly it's never articulated |

These labels are read-only. Two polities can both be labeled "common law" with different substrate (e.g. one has royal-prerogative precedents, another has mercantile precedents). The UI shows the label; the mechanics act on the continuous distribution.

### 2.3 Law stability and drift

**FR-CIV-LAW-005 — Laws drift on the same vectors as culture.** When a `CultureProfile.traits` axis moves enough to push `N_P(action)` across a hysteresis threshold, the operative norm shifts. Drift sources are the existing `culture::mutate_traits` and the existing `religion::spread_religion`. **No new drift engine** — law drift is a *consequence* of culture and belief drift, which is the charter-correct dependency direction.

**FR-CIV-LAW-006 — Major reform is a punctuated event.** Sometimes a norm distribution collapses (mass conversion, conquest, plague, new substrate that invalidates an old proscription). The system emits a `law.reform.v1` history event when the dispersion of `N_P(action)` across a polity exceeds a reform-fanout threshold; the new state is the substrate's *new* norm distribution. Reform is the substrate, observed. No "reform by decree" logic.

### 2.4 Inter-polity law interaction

**FR-CIV-LAW-007 — Law interaction flows through existing relations substrate, never direct.** Two adjacent polities with very different `N_P` distributions (e.g. one's `proscribed_share(action=adultery) ≈ 0`, the other's `proscribed_share ≈ 0.95`) don't *negotiate laws*. They:

- feed the divergence into `DiplomacyMatrix` (an `MoralMismatch` signal — already in the relation substrate; see `crates/agents/src/diplomacy.rs`),
- influence cross-polity `MembershipPayoff` for migrants/visitors (already in the cluster substrate),
- modify trade openness via the existing `MARKET-001` `trust` and `coordinator` vectors (already in polities-markets).

No "treaty of extradition" component. The diplomatic apparatus — diplomacy.rs's `apply_signal`, treaties, etc. — handles it. Law divergence is observed through the existing `DiplomacyMatrix`, never *managed* by an authored law component.

---

## 3. Governance emergence (the *who decides* and *who enforces*)

Governance form is about **decision procedure** and **enforcement apparatus**, not the law content. A band and a state can both have the law "don't steal"; the difference is *who decides what's theft* and *what happens if you do it*.

### 3.1 Governance as a 4-axis vector

**FR-CIV-LAW-008 — Governance state is a 4-axis vector, not an enum.** For polity surface `P` at evaluation tick `t`:

```
gov(P) = (
  decision_concentration : f32,   // 0 = diffuse consensus, 1 = single decision-point
  enforcement_concentration: f32, // 0 = diffuse, 1 = specialized enforcer cohort
  appointment_mode: f32,          // 0 = ascribed, 1 = selected on merit, with the
                                  //       in-between (elected, rotating, bought, etc.)
                                  //       living on the continuum
  accountability: f32,            // 0 = unaccountable, 1 = strong recall/impeachment
)
```

These four axes derive from substrate:

- `decision_concentration(P)` — Gini of decision-events in `legends::SagaGraph` over `P`'s member clusters (the existing `Legends` ingestion path in `crates/engine/src/emergence.rs::emergence_legends`). Many "X decided Y" events concentrated on a few agents → high concentration.
- `enforcement_concentration(P)` — ratio of agents in `P` who have performed an enforcement action in a window (sanctioning, mediating, punishing) to the polity's mean agent count, plus the Gini of that same distribution.
- `appointment_mode(P)` — derived from the *legends* of how a recent decision-point agent became a decision-point: birthright, coup, election, purchase, lottery, acclaim.
- `accountability(P)` — ratio of decision-point agents who lost their role in a window to total decision-point agents, weighted by how often the loss followed a sanctioned act.

No `GovType::Monarchy` field. The governance state *is* the 4-tuple; the readable label is a classifier output.

### 3.2 The governance-form read-out (band → state, the spectrum the user asked about)

**FR-CIV-LAW-009 — Governance label is a classifier output over the 4-axis vector + polity scale.**

The classical anthropological spectrum (Service / Fried / Carneiro) maps onto a 2D projection of `(decision_concentration, enforcement_concentration)` at varying scales. The "band → chiefdom → state" arc is the same vector climbing both axes as polity scale grows:

| Label | decision_conc | enforcer_conc | scale (members) | other signals |
|---|---|---|---|---|
| **Band** | < 0.10 | < 0.05 | < ~30 | `kinship` insulation high, no standing enforcer cohort |
| **Tribe** | 0.10–0.25 | 0.05–0.15 | ~30–300 | segmentary lineage; emergent council of elders visible in legends |
| **Chiefdom** | 0.25–0.55 | 0.15–0.40 | ~300–3,000 | hereditary / ascribed decision point + small retinue of enforcers; `accountability` low |
| **Early state** | 0.55–0.80 | 0.40–0.70 | ~3,000+ | decision point + dedicated enforcer cohort, possibly codified laws, possibly tax-funded (`InstitutionLedger`) |
| **Mature state** | > 0.80 | > 0.70 | large | bureaucratic specialisation; accountability apparatus present |
| **Federation / league** | mid, *and* `decision_concentration` measured per member-state is low | low per member | multiple polities linked by `Alliance`-level diplomacy | polity shape (POLITY-004) reads out as "Networked" *and* governance reads out as "Federation" — *both* labels refer to the same substrate from different angles |
| **Anarchic / acephalous** | < 0.10 | < 0.10 | any | no decision point survives across a window; enforcer action is peer-led, ad hoc |

This is **the same read-out pattern** as POLITY-004 (anarchic / networked / collective / tributary / hegemonic) and CULTURE_IDEOLOGY's read-outs. Three independent read-outs over partly overlapping substrate. They co-vary but are not redundant.

**AC-1 — No governance-form enum is the source of truth.** The governance form is a derived label recomputed from substrate each evaluation. The simulation core never *acts* on the label — only on the continuous 4-axis vector.

**AC-2 — All listed governance labels are reachable from substrate alone in scenario tests**, and none is the default. (Reachability test plan in §7.)

### 3.3 Decision procedure (the "who decides" half)

**FR-CIV-LAW-010 — Decision events are derived from `legends::SagaGraph`.** An agent's contribution of a *decision* — picking the clan's waterhole, declaring war, allocating the surplus, expelling a member — is already recordable in the `emergence_legends` pipeline. The decision-procedure classifier reads this graph and identifies the structural pattern: single recurring decision-point agent (chief / monarch / magistrate), rotating role (council seat), diffuse (assembly), formal hierarchy (bureaucracy).

The "decision" itself is **not** an authored action type. It's whatever the agents actually do that other agents then *treat* as a decision (a side-effect of cultural deference + scarcity of decision-relevant info). The system only observes the legend; the agent's `Psyche` and `MembershipPayoff` mechanics produced it.

### 3.4 Enforcement apparatus (the "what happens if you don't" half)

**FR-CIV-LAW-011 — Enforcement emerges from norm-violation response patterns.**

The substrate already models:

- `BeliefConcept::Taboo { action }` (religion.rs) — a proscribed action with a strength/spread
- `DiplomacyMatrix` sanctions (already in diplomacy.rs)
- `MembershipPayoff` — a clan can eject a norm-violator (already in cluster.rs via `should_leave`)

A polity's **enforcement apparatus** is the read-out of how these substrate responses cluster:

- *Informal / peer-led* — sanctions come from peers (diplomacy.rs), ejection comes from `MembershipPayoff.should_leave`, no specialized enforcer role appears in legends.
- *Specialized enforcer cohort* — a small number of agents perform a *disproportionate* share of sanctioning actions; a `Warrior`/`Guard`/`Sheriff` archetype emerges in the legend distribution (NOT authored; observed in `emergence_legends`).
- *Formal court system* — distinct sub-graph in legends with `evidence presentation → judgment → sanction` patterns, run by agents with `appointment_mode` near `selected`/`elected`.
- *Terror / mass punishment* — disproportionate enforcement events with low `accountability` of the enforcer, and high `dispute` collapse in the norms being enforced.

**AC-3 — Enforcement is a derived read-out, never stored as `EnforcementType::Court`/`EnforcementType::Mob`.** The simulation acts on the continuous distribution of enforcement events in legends; the UI shows the label.

**FR-CIV-LAW-012 — Tax-funded enforcement emerges from `InstitutionLedger` overlap.** When a polity's governance vector climbs `enforcement_concentration` high enough that the enforcer cohort is *full-time*, the only substrate-sustainable funding source is the `InstitutionLedger` treasury (already in `crates/economy/src/institution.rs`). Tax-funded enforcement is therefore a **derived** condition: the polity's `InstitutionLedger` shows regular inflows to an enforcement-relevant posting, and the enforcer cohort draws from it. There is no `Taxation::Levy` switch that "creates" a state; the levy is the substrate pattern that the system observes and labels.

### 3.5 Accountability and feedback (the loop that prevents runaway)

**FR-CIV-LAW-013 — Accountability is the ratio of decision-point losses to decision-point tenures, weighted by cause.** A decision-point agent who is *routinely* lost (deposed, killed, exiled) following a sanctioned act has a high-accountability profile. The substrate is `legends::SagaGraph` events of the form `(<agent>, lost_role, <cause>)` over a sliding window — the cause is itself derived from preceding events (a sanctioned act → a coup → a deposition is a *causally-chained* legend pattern).

**FR-CIV-LAW-014 — Governance is a coupled dynamical loop with no authored equilibrium.**

```
culture+kinship+belief ──► norm distribution N_P ──► governance vector gov(P)
        ▲                                                    │
        │                                                    ▼
   diplomacy relations ◄── enforcement outcomes ◄── decision events
        │                                                    │
        ▼                                                    ▼
   membership payoff, trade, faction split/merge ◄──── polity shape (POLITY-001..008)
```

No equilibrium is hand-tuned. The 4-axis vector drifts on the same forces that drift culture, diplomacy, and economy. Whether a polity stabilizes as a chiefdom, oscillates between federation and hegemon, or collapses back to anarchic is **a property of the substrate, not of authored targets**.

---

## 4. State, federation, empire (multi-polity governance)

**FR-CIV-LAW-015 — Multi-polity governance is a higher-level read-out over the polity graph.** A *federation* is what you call it when:

- the polity shape (POLITY-004) reads out as "Networked" (multiple hubs, high reciprocity), AND
- a *second-tier* coordination exists in the cohesion graph above the member polities (a meta-community), AND
- that meta-community has its own `gov(P_meta)` vector with mid decision_concentration.

An *empire* is a "Tributary" or "Hegemonic" polity shape (POLITY-004) where the dominant hub is geographically distant from the periphery and the periphery polities retain low `decision_concentration` (local autonomy) but high `decision_concentration *upward*` (recognition of the imperial center).

**AC-4 — Federation and empire labels are derived from the polity graph + governance vectors, not stored.** The polity IDs remain emergent `ClusterId` references (FR-CIV-DIPLO-007).

---

## 5. Enforcement failure modes (the "what can go wrong" reading)

A design doc that doesn't talk about failure modes is a happy-path toy. The substrate must surface failures legibly:

| Failure mode | Substrate signature | History event |
|---|---|---|
| **Lawless collapse** | `enforcement_concentration` decays to 0, norm distribution `dispute` rises | `law.collapse.v1` |
| **Tyranny** | `decision_concentration` high, `accountability` < 0.1, `dispute` rising in multiple `action`s simultaneously | `gov.tyranny.v1` |
| **Captured enforcers** | enforcer cohort actions correlate strongly with kinship/factional alignment, not with norm violations | `enforcer.captured.v1` |
| **Revolution** | sustained mass `MembershipPayoff.should_leave` events in a window, followed by decision-point loss | `gov.revolution.v1` |
| **Legal sclerosis** | codification rate rises but `dispute` doesn't drop — codified laws persist while the substrate has moved on | `law.sclerosis.v1` |
| **Law-shopping migration** | sustained agent flow from high-proscription polities to low-proscription adjacent polities | `law.migration.v1` |
| **Moral panic** | rapid `dispute` collapse in one `action` with high `social_spread` (a `BeliefConcept::Taboo` fires across the network) | `law.panic.v1` |

Each is a **read-out** over existing substrate (legends, culture, diplomacy, membership payoff, religion). No `FailureMode` enum. The history event is for the feed and the historian UI; the *mechanics* are the substrate's own dynamics.

---

## 6. What this design explicitly does NOT add

These would be charter violations (they hardcode emergent content):

- ❌ A `LegalCode { articles: Vec<Article> }` component.
- ❌ A `GovType { Monarchy, Republic, … }` enum in any state surface.
- ❌ A `LegalSystem { Common, Civil, Sharia, … }` enum.
- ❌ An LLM call that "drafts" laws.
- ❌ A "legislator" agent type with special-case decision logic.
- ❌ A `crime_severity` table or a `punishment_table` field.
- ❌ A `tax_rate` field on a polity component (taxes are `InstitutionLedger` postings, full stop).
- ❌ A "constitution" struct authored per polity.

What it DOES add (as derived projections and classifiers):

- ✅ A `gov_vector(P): (f32, f32, f32, f32)` read-out function.
- ✅ A `norm_distribution(P, action): (f32, f32, f32, f32)` aggregate.
- ✅ A `codification_status(P, action): CodifyState` read-out (no storage).
- ✅ A `governance_label(gov_vector) -> GovLabel` classifier.
- ✅ A `code_label(codification, religion_overlap, precedent_graph) -> CodeLabel` classifier.
- ✅ `law.*.v1` and `gov.*.v1` history events (read-out, not state change).
- ✅ Reuse of `legends::SagaGraph`, `DiplomacyMatrix`, `MembershipPayoff`, `BeliefConcept::Taboo`, `InstitutionLedger` — the existing substrate.

---

## 7. Acceptance criteria (behavioral, mechanism-agnostic)

These mirror the structure of `polities-markets.md` §4 and are the binding contract for any implementation downstream.

| AC | Criterion |
|---|---|
| **AC-1** | No stored `LegalCode`/`GovType`/`LawSystem` enum is the source of truth; all three are derived read-outs recomputed from substrate each evaluation. |
| **AC-2** | All governance forms in §3.2 are *reachable* from substrate alone in scenario tests (give inputs → observe read-out), and none is the default. |
| **AC-3** | All code labels in §2.2 are reachable from substrate alone; codification is a slow drift with hysteresis (no per-tick flicker under steady substrate). |
| **AC-4** | Codification hysteresis: a norm that has been highly dispersed for a window does not flip to codified on a single tick of low dispersion, and vice versa (separate enter/exit thresholds). |
| **AC-5** | `decision_concentration` and `enforcement_concentration` rise together with polity scale (no field forces them; they correlate via substrate dynamics). |
| **AC-6** | Tax-funded enforcement emerges from `InstitutionLedger` postings — no authored `Taxation` switch, no stored `levy` field. |
| **AC-7** | All seven failure modes in §5 are observable as read-outs over substrate; each has a history event with the substrate signatures in §5 as input. |
| **AC-8** | `gov_vector` and `norm_distribution` are pure functions of substrate + tick → read-out is deterministic *given* the substrate snapshot (the substrate itself may be non-deterministic per the charter's determinism-isn't-required rule). |
| **AC-9** | Law divergence between polities flows only through `DiplomacyMatrix` and `MembershipPayoff`; no direct "treaty of extradition" component is ever added. |
| **AC-10** | No LLM call participates in any law or governance read-out, classifier, or event (LLM is for lore garnish only, per FR-CIV-LLM-001..006). |
| **AC-11** | Read-out labels appear in `sim.snapshot` and the history feed but never in the `state` write-path that affects downstream mechanics — labels are *projections of* mechanics, not inputs to them. |

---

## 8. Phased WBS + dependency DAG

**Phase L1 — Substrate aggregation (no new regime types)**
| Task | Description | Depends On |
|---|---|---|
| T1 | `norm_distribution(P, action)` aggregate over `(Psyche, CultureProfile, Religion.beliefs)` for polity `P`'s members (LAW-002) | polities-markets T4/T5 |
| T2 | `action_vocabulary()` — the finite set of `action`s queried for norm distribution, derived from the union of action sets the agents can actually take | — |
| T3 | `decision_event_classifier` over `legends::SagaGraph` (LAW-010) | `legends-engine.md` ingest path |
| T4 | `enforcement_event_classifier` over `legends::SagaGraph` + `DiplomacyMatrix` sanctions (LAW-011) | T3 |

**Phase L2 — Read-outs**
| Task | Description | Depends On |
|---|---|---|
| T5 | `gov_vector(P): (f32, f32, f32, f32)` (LAW-008) | T3, T4 |
| T6 | `codification_status(P, action)` with hysteresis (LAW-003, AC-4) | T1 |
| T7 | `governance_label(gov_vector)` classifier + scale gate (LAW-009) | T5 |
| T8 | `code_label(codification, religion_overlap, precedent_graph)` classifier (LAW-004) | T6, T3 |

**Phase L3 — Coupling (read-out → existing mechanics)**
| Task | Description | Depends On |
|---|---|---|
| T9 | Feed `gov_vector` into `MARKET-006` `coordinator` (LAW-012, polities-markets T9) | T5, polities-markets T5 |
| T10 | Feed `norm_distribution` divergence into `DiplomacyMatrix.apply_signal` as `MoralMismatch` (LAW-007) | T1, diplomacy substrate |
| T11 | Failure-mode read-outs → `law.*.v1` / `gov.*.v1` history events (§5) | T5, T6 |
| T12 | Federation/empire higher-level read-out (LAW-015) | T5, T7, polities-markets T5 |

**Phase L4 — Validation**
| Task | Description | Depends On |
|---|---|---|
| T13 | Scenario tests for AC-1..AC-11: reachability of governance forms and code labels; hysteresis; substrate-only paths; failure-mode observability | T7–T12 |

DAG: {T1,T2,T3,T4} → {T5,T6} → {T7,T8,T9,T10} → {T11,T12} → T13.

The DAG **deliberately depends on** the polities-markets T4/T5 (cohesion graph + community detection) and the culture substrate T1/T2. Law and governance are a **second-order read-out** over polity + culture + belief. Adding law *before* the polity substrate is built is the most likely way to violate the charter.

---

## 9. Readable surface (API/UI projection contract)

Downstream code reads the same projection pattern as polities-markets §6 — all derived, all read-only:

- **Polity governance view:** `{ polity: ClusterId, gov_vector: (f32, f32, f32, f32), gov_label: GovLabel, scale: u32 }`
- **Norm view per action:** `{ polity: ClusterId, action, prescribed_share, proscribed_share, intensity, dispute, codification: CodifyState, code_label: CodeLabel }`
- **Failure feed:** `Vec<LawEvent>` with `law.collapse.v1`, `gov.tyranny.v1`, `enforcer.captured.v1`, `gov.revolution.v1`, `law.sclerosis.v1`, `law.migration.v1`, `law.panic.v1` — all read-out, all advisory text.
- **History narration:** the historian uses these labels in saga prose (FR-CIV-LEGENDS-005/008); the saga-graph node IDs reference `ClusterId` and `agent_id` only.

The simulation core stores only the **continuous substrate** (norm distribution aggregates, governance vector components, codification hysteresis counters, legend events, `InstitutionLedger` postings). Labels are computed on read. This is the charter's "measured, emergent pattern over the substrate," applied to law and polity governance.

---

## 10. Cross-references

- [`emergence-charter.md`](../guides/emergence-charter.md) — Layer-0 / Layer-1+ contract; "polities are decentralized, not necessarily explicit mutual collectives."
- [`polities-markets.md`](./polities-markets.md) §0–§6 — sibling design: cohesion graph, community detection, polity shape read-out, secession/merge/collapse.
- [`CULTURE_IDEOLOGY.md`](./CULTURE_IDEOLOGY.md) — culture/ideology drift substrate; norms are derived from `CultureProfile.traits`, not authored.
- [`DIPLOMACY_EMERGENCE.md`](./DIPLOMACY_EMERGENCE.md) — `DiplomacyMatrix` substrate feeding law divergence and sanctions.
- [`crates/engine/src/religion.rs`](../../crates/engine/src/religion.rs) — `BeliefConcept::Taboo` is the existing proto-norm primitive; this design reuses it.
- [`crates/agents/src/cluster.rs`](../../crates/agents/src/cluster.rs) — `MembershipPayoff` is the substrate that ejection (informal enforcement) and join/leave already live on.
- [`crates/economy/src/institution.rs`](../../crates/economy/src/institution.rs) — `InstitutionLedger` is the substrate for tax-funded enforcement and credit/debt.
- [`crates/engine/src/emergence.rs`](../../crates/engine/src/emergence.rs) — `emergence_legends` pipeline that produces the decision and enforcement event substrate.
- [`FUNCTIONAL_REQUIREMENTS.md`](../../FUNCTIONAL_REQUIREMENTS.md) FR-CIV-DIPLO-007 — Polity IDs reference emergent `ClusterId`, not `faction: u32`. (This design obeys; the law/governance IDs are the same `ClusterId` references.)
