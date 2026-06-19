# Emergent Polities & Markets

**Status:** Design (planner stance — specs/AC/pseudocode only; no implementation).
**Scope:** How political organization and market *types* emerge from Layer-0 substrate rather than being hardcoded.
**Charter anchors:** [emergence-charter.md](../guides/emergence-charter.md) §"Markets of varying types" and §"Polities / states — decentralized, not necessarily explicit mutual collectives."
**Code substrate (read-only inputs):** `crates/agents/src/cluster.rs` (co-location clustering + `MembershipPayoff`), `crates/agents/src/diplomacy.rs` (drifting pairwise relations), `crates/agents/src/culture.rs` (trait/language drift), `crates/economy/src/{stocks,market,institution,allocation}.rs`.
**Algorithm references:** [game-rnd.md](../research/game-rnd.md) §1.3 (tâtonnement now, CDA next).

---

## 0. Governing principle

A **polity** is not a `faction: u32` and a **market** is not one fixed clearing model. Both are *measured patterns* over the existing substrate:

- A polity is an emergent **organizing structure** — a weighting over agents/clusters by how much their behavior is co-coordinated through co-location, kinship, shared culture, economic payoff, and coercion. Its *shape* (anarchic → networked → tributary → collective) is read off the substrate, never selected from an enum.
- A market is the **exchange regime** that arises in a locale. Its *type* (gift → barter → commodity → mercantile → credit → planned) is read off local scarcity, trust, surplus, and the presence of a coercive coordinator. The price-discovery *mechanism* (tâtonnement vs CDA) is a Layer-0-adjacent law per game-rnd §1.3; which one runs is selected by local conditions, not authored per region.

**Design rule restated:** model the *rule that produces* organization/exchange; the *category labels in this doc are read-out projections for the UI and the history feed*, not stored authoritative types.

---

## 1. Polity emergence

### 1.1 Inputs (all already present or derivable from substrate)

| Signal | Source | Meaning |
|---|---|---|
| Co-location | `cluster::cluster_by_colocation` | spatial connected components (single-link, union-find) |
| Kinship insulation | `culture::CultureProfile.kinship` | how strongly a population resists outside drift |
| Cultural similarity | `culture::TraitVector` distance | shared norms/identity |
| Economic payoff | `cluster::MembershipPayoff::payoff` | net benefit of belonging |
| Inter-cluster relations | `diplomacy::DiplomacyMatrix` | alliance/trade/rivalry/war score |
| Coercion | **new derived signal** (§1.3) | asymmetric ability to compel behavior |

### 1.2 The polity as a cohesion field, not a label

**FR-CIV-POLITY-001 — Cohesion graph.** A polity surface is computed as a **weighted graph over clusters** (nodes = `ClusterId`, edges = directed coordination weight). Edge weight from cluster `i` to cluster `j` is a monotone blend:

```
coord(i→j) = w_colo · colocation(i,j)
           + w_kin  · kinship_overlap(i,j)
           + w_cult · (1 − culture_distance(i,j))
           + w_econ · max(0, payoff_if_coordinated(i,j))
           + w_coer · coercion(i→j)          // directed, can be asymmetric
```

- `colocation`, `culture_distance` derive from existing modules; `coercion` is new (§1.3).
- Weights `w_*` are tuning constants, NOT switches that hardcode a regime.

**FR-CIV-POLITY-002 — Polity = community in the cohesion graph.** A polity is a **community-detection cluster** over the symmetric part of `coord` (e.g. label-propagation or modularity on `(coord(i→j)+coord(j→i))/2`). No fixed membership integer; membership is **cluster overlap**, exactly as the charter demands. An agent/cluster may sit in the fuzzy boundary of two polities (overlap weight on each).

### 1.3 Coercion — the missing Layer-0-adjacent signal

**FR-CIV-POLITY-003 — Coercion derives, never declared.** `coercion(i→j)` measures `i`'s capacity to compel `j`, computed from substrate only:

```
coercion(i→j) = clamp01(
      power(i) / (power(i) + power(j) + ε)        // relative capability
    · proximity(i,j)                              // must be able to reach j
    · (1 − relation_score(i,j).max(0))            // cooperation reduces need to coerce
)
```

where `power(c) = f(population(c), surplus(c), tool_stock(c), terrain_defensibility(c))` — all from `stocks::Stocks` (esp. `Good::Tools`, `Good::Metal`), cluster size, and planet/terrain. **Asymmetry of `coord` (coord(i→j) ≫ coord(j→i)) driven by `coercion` is what makes a *tributary* shape; symmetry makes a *collective* shape.** (See §1.4 read-out.)

### 1.4 Shape read-out (UI/history projection, derived per tick — not stored as type)

**FR-CIV-POLITY-004 — Regime read-out from graph topology.** A polity's *displayed* shape is a pure function of its internal edge structure:

| Read-out label | Topological signature (within the polity community) |
|---|---|
| **Anarchic** | low mean coord, high reciprocity, no dominant node (degree ~ uniform, low coercion) |
| **Networked** | high reciprocity, multiple high-degree hubs, low coercion asymmetry |
| **Collective** | high coord, near-symmetric edges, payoff-driven, low coercion |
| **Tributary** | one node with high out-coercion to many low-power nodes (star, asymmetric) |
| **Hegemonic/centralized** | single dominant hub, high coord, high coercion to periphery |

These are computed by a classifier over `{mean_coord, reciprocity, coercion_asymmetry, degree_gini, hub_count}`. **The label is advisory text; the simulation only ever acts on the continuous graph.** Two polities can be the same label and behave differently.

### 1.5 Lifecycle: secession / merge / collapse

**FR-CIV-POLITY-005 — Secession.** A sub-community whose **internal** mean coord stays high while its coord to the rest of the polity decays below a hysteresis low-water mark splits into its own polity. Trigger sources: payoff falling (`MembershipPayoff` net-negative for the sub-group — reuse `cluster::should_leave`), relation souring (`DiplomacyMatrix` → Rivalry/War internally), or culture distance crossing a divergence threshold (`culture` drift). Hysteresis (separate join/leave thresholds) prevents flicker — mirror the existing `should_join`/`should_leave` two-threshold pattern.

**FR-CIV-POLITY-006 — Merge.** Two adjacent polities whose cross-coord exceeds their respective internal mean coord (for a sustained window) merge into one community. Driven by sustained trade (`DiplomacyMatrix` → Alliance), cultural convergence, or one absorbing the other via coercion (asymmetric merge = conquest read-out).

**FR-CIV-POLITY-007 — Collapse.** A polity dissolves when its internal mean coord falls below the anarchic floor for a sustained window (population loss, surplus collapse starving payoff, or coercion capacity evaporating). Collapse emits its member clusters back as independent nodes; downstream re-clustering may immediately re-form smaller polities. No "game over" — collapse is just the graph thinning out.

### 1.6 State ↔ market interaction

**FR-CIV-POLITY-008 — Polities act on markets only through existing institutions.** A polity influences exchange exclusively by parameterizing the market layer it overlaps — it does NOT get its own bespoke economy. Concretely it can: fund/drain an `institution::InstitutionLedger` treasury, bias `allocation::AllocationEngine` selection, or impose a coordination constraint that nudges the market *type* read-out (§2). A high-coercion polity over a locale pushes that locale's market toward **planned**; an anarchic polity leaves it at **gift/barter**. The polity never bypasses `verify_ledger_conservation` / `verify_conservation`.

---

## 2. Market type emergence

### 2.1 Local conditions vector

**FR-CIV-MARKET-001 — Per-locale condition probe.** For each market locale (a settlement catchment / spatial cell with ≥2 trading actors), compute a condition vector from substrate:

| Condition | Source | Drives toward |
|---|---|---|
| `scarcity` | `stocks::deficit` summed over goods vs `surplus` | barter/commodity when high, gift when abundant |
| `trust` | `diplomacy::DiplomacyMatrix` mean relation among participants | credit (high), barter (low) |
| `surplus` | `stocks::surplus` total | commodity/mercantile (tradable excess) |
| `specialization` | `stocks::comparative_advantage` divergence across actors | mercantile (long-range comparative advantage) |
| `coordinator` | overlapping polity coercion (§1.6) | planned (a coercive coordinator exists) |
| `liquidity_need` | volume × time-mismatch of trades | credit (deferred settlement) |

### 2.2 Type read-out (projection, not stored)

**FR-CIV-MARKET-002 — Market type is a classified read-out of conditions.** The displayed market type is a pure function of the condition vector — the *same* charter spectrum:

| Type | Emergence condition (dominant signals) |
|---|---|
| **Gift** | abundance (low scarcity) + high kinship/trust + small group → transfers with no return-leg requirement |
| **Barter** | scarcity + low trust + dual coincidence of wants present (`propose_trade` finds a 2-good match) |
| **Commodity** | surplus + a good emerges as numeraire (most-traded, most-liquid good becomes unit of account) |
| **Mercantile** | high specialization + comparative advantage across distant locales → routed long-range trade |
| **Credit** | high trust + liquidity need + repeated counterparties → deferred settlement / debt postings |
| **Planned** | a coercive coordinator (polity, §1.6) sets allocations → `AllocationEngine` overrides price discovery |

**FR-CIV-MARKET-003 — One spectrum, smooth transitions.** Locales hold a soft membership over types (weights), not a hard switch, so a market can be "mostly barter, partly credit." Read-out picks the argmax for the label; mechanics blend.

### 2.3 Price discovery mechanism selection

**FR-CIV-MARKET-004 — Tâtonnement is the default law.** Every priced locale runs **damped tâtonnement** as the baseline price-discovery dynamic (game-rnd §1.3, "now"). Replace the current placeholder `MarketState::step` random-walk delta with excess-demand–driven adjustment:

```
// per good g, per tick:
excess(g)   = demand(g) − supply(g)          // from aggregated stocks/profiles in the locale
price(g)   += λ · price(g) · excess(g) / scale   // multiplicative, damped by λ
price(g)    = max(MIN_PRICE, price(g))           // stays strictly positive (keep current invariant)
```

- Integer/fixed-point cents preserved (existing `i64` cents); `λ` and `scale` are integer-friendly.
- **Determinism not required** (charter §"Determinism is NOT a requirement"): real noise on `excess` is welcome for livelier convergence. The existing replay-stable tests in `market.rs` should be relaxed/retargeted to *invariants* (prices positive, excess shrinks toward zero) rather than bit-identical sequences.

**FR-CIV-MARKET-005 — CDA upgrade near high-trust settlements.** Where `trust` and trade volume are high AND the locale is near-camera/active, the locale upgrades to a **continuous double auction** (order-book bid/ask matching — game-rnd §1.3 "NEXT", ~150-LOC matching engine). Tâtonnement remains the cheaper far-field / low-trust default. Selection is by condition, LOD tier, and trust — never authored per region. CDA clears at the matched price; that matched price seeds the locale's tâtonnement when it later LODs back out.

**FR-CIV-MARKET-006 — Planned override.** When a coercive coordinator overlaps the locale (FR-CIV-POLITY-008), price discovery is partially or fully replaced by `AllocationEngine` decisions; the read-out becomes **planned**. Mis-set planned prices produce visible shortages/surpluses (queues, spoilage) — the same `stocks` deficit/surplus signals — so planning is legible and can fail back toward barter on collapse.

### 2.4 Numeraire & credit emergence

**FR-CIV-MARKET-007 — Money emerges, is not declared.** No hardcoded currency. A **numeraire** emerges as the good with the highest liquidity (trade frequency × acceptability across counterparties) in a region; prices may re-denominate against it. Different regions may settle on different numeraires; cross-region trade implies an emergent exchange rate (ratio of local numeraire prices). `Good::Metal`/`Good::Tools` are *likely* numeraires by durability but never privileged in code.

**FR-CIV-MARKET-008 — Credit/debt via institution postings.** Deferred settlement reuses `institution::InstitutionLedger` double-entry postings: a credit market records a debt as a balanced posting (debtor liability ↔ creditor asset) that settles later. Conservation (`verify_conservation`) holds throughout; default/forgiveness is a posting that writes the debt off and souring `DiplomacyMatrix` relation between the parties.

---

## 3. State ↔ market feedback loop (closing the cycle)

```
                 stocks/surplus/scarcity ─┐
                                          ▼
   culture+kinship ──► cohesion graph ──► polity shape (POLITY-004)
        ▲                  │                   │
        │                  ▼                   ▼ (coercion overlap, POLITY-008)
   diplomacy relations  coercion ──────► market type (MARKET-002)
        ▲                                      │
        │                                      ▼
        └────────── trade volume / price ── price discovery (MARKET-004/005/006)
                          (feeds diplomacy + payoff, closing the loop)
```

- Prices and trade volume feed back into `DiplomacyMatrix.apply_signal` (`trade_volume`, `resource_competition`) and into `MembershipPayoff`, which re-shapes the cohesion graph next tick. The whole system is a coupled dynamical loop with **no authored equilibrium** — equilibria emerge or fail to.

---

## 4. Acceptance criteria (behavioral, mechanism-agnostic)

| AC | Criterion |
|---|---|
| AC-1 | No stored `faction`/`govt_type`/`market_type` enum is the source of truth; polity & market type are derived read-outs recomputed from substrate each evaluation. |
| AC-2 | All five polity shapes are *reachable* from substrate alone in scenario tests (give inputs → observe read-out), and none is the default. |
| AC-3 | All six market types are reachable from condition vectors alone; transitions are smooth (soft membership), not instantaneous switches. |
| AC-4 | Secession, merge, and collapse each have hysteresis (distinct enter/exit thresholds) — no per-tick flicker under steady inputs. |
| AC-5 | Tâtonnement drives every priced good's excess demand toward zero under stationary supply/demand (convergence test), with prices strictly positive (existing invariant preserved). |
| AC-6 | CDA upgrade is reversible: a locale that loses trust/volume/LOD falls back to tâtonnement seeded by the last CDA clearing price, with no conservation violation. |
| AC-7 | A planned override produces measurable shortage/surplus when allocations are mis-set, and collapses back to barter when the coordinator dissolves. |
| AC-8 | Every credit/debt cycle preserves `InstitutionLedger::verify_conservation` and `verify_ledger_conservation` end-to-end (no joule/stock creation). |
| AC-9 | Numeraire selection is data-driven: changing which good is most liquid changes the emergent numeraire with no code change. |
| AC-10 | Polity influence on markets flows only through existing institution/allocation APIs; polities never mutate stocks or prices directly. |

---

## 5. Phased WBS + dependency DAG

**Phase P1 — Substrate signals** (extend existing modules; no new regime types)
| Task | Description | Depends On |
|---|---|---|
| T1 | `coercion(i→j)` derived signal from stocks/proximity/relations (POLITY-003) | — |
| T2 | Per-locale condition-probe vector (MARKET-001) over existing stocks/diplomacy | — |
| T3 | Liquidity/numeraire metric over trade frequency (MARKET-007 input) | — |

**Phase P2 — Emergence cores**
| Task | Description | Depends On |
|---|---|---|
| T4 | Cohesion graph builder over clusters (POLITY-001) | T1 |
| T5 | Community detection → polity membership overlap (POLITY-002) | T4 |
| T6 | Replace `MarketState::step` random-walk with damped tâtonnement (MARKET-004) | T2 |
| T7 | Market-type classifier from condition vector, soft membership (MARKET-002/003) | T2, T3 |

**Phase P3 — Read-outs & coupling**
| Task | Description | Depends On |
|---|---|---|
| T8 | Polity shape classifier (POLITY-004) | T5 |
| T9 | State↔market coupling via institution/allocation only (POLITY-008, MARKET-006) | T5, T7 |
| T10 | Credit/debt as institution postings (MARKET-008) | T7 |
| T11 | Feed price/trade-volume back into diplomacy + payoff (§3 loop) | T6, T7 |

**Phase P4 — Lifecycle & richer discovery**
| Task | Description | Depends On |
|---|---|---|
| T12 | Secession/merge/collapse with hysteresis (POLITY-005/006/007) | T8 |
| T13 | CDA matching engine, LOD/trust-gated upgrade + fallback (MARKET-005) | T6, T7 |
| T14 | Emergent numeraire + cross-region exchange rate (MARKET-007) | T3, T6 |

**Phase P5 — Validation**
| Task | Description | Depends On |
|---|---|---|
| T15 | Scenario tests for AC-1..AC-10 (reachability, convergence, conservation, hysteresis) | T8–T14 |

DAG (linear-ish): {T1,T2,T3} → {T4,T6,T7} → {T5,T8,T9,T10,T11} → {T12,T13,T14} → T15.

---

## 6. Readable surface (the API/UI projection contract)

What downstream code (overlays, history feed, inspect tool) reads — **all derived, all read-only projections**:

- **Polity:** `{ members: Vec<(ClusterId, overlap_weight)>, shape_label: PolityShape, mean_coord, reciprocity, coercion_asymmetry }`. `PolityShape` is `{Anarchic, Networked, Collective, Tributary, Hegemonic}` *for display only*.
- **Market:** `{ locale, type_weights: [f32; 6], type_label: MarketType, prices: BTreeMap<Good, i64_cents>, numeraire: Option<Good>, mechanism: {Tatonnement|Cda|Planned} }`.
- **History events** (emergent-friendly, RON-defined achievements per game-rnd §"Achievements"): "polity seceded", "market hit hyperinflation", "credit market emerged", "tributary collapsed to barter".
- **Conservation guarantees surfaced:** any debt/credit read-out is backed by a balanced `InstitutionPosting`; any price by a tâtonnement/CDA step that kept prices positive.

The simulation core stores only the **continuous substrate** (cohesion graph weights, condition vectors, prices, postings). Labels are computed on read. This is the charter's "measured, emergent pattern over the substrate," made concrete.
