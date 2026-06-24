# CIV-ECONMARK: Emergent Markets + Emergent Numeraire — Design Spec

> **Status:** Design (planner-only, 2026-06-14). No implementation code in this document.
> **Spec ID:** `civ-economy-emergent-markets` | **Epic:** E2 (emergence), layered on E0/E1 (MVP substrate) | **Pattern ancestors:** [`civ-003-emergent-lifecycle.md`](civ-003-emergent-lifecycle.md) (charter constraint + classifier read-out + shared-gradient coupling + criticality knobs), [`polities-markets.md`](polities-markets.md) (price-discovery / numeraire / credit read-outs).
> **Governing canon:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md), [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md), [`docs/design/emergence-dashboard.md`](emergence-dashboard.md).
> **Code substrate (read-only inputs):** `crates/economy/src/allocator.rs` (`Allocator`, `Bid`, `Offer`, `ClearedTrade`, `Allocator::post_bid`, `Allocator::post_offer`, `Allocator::clear`), `crates/economy/src/market.rs` (`MarketState`, `MarketState::step`, `MarketState::prices`), `crates/economy/src/stocks.rs` (`Stocks`, `Good`, `GOODS`, `ProductionProfile`, `surplus`, `deficit`, `comparative_advantage`, `propose_trade`, `apply_trade`), `crates/economy/src/institution.rs` (`InstitutionLedger`, `InstitutionPosting`, `LedgerSide`, `INSTITUTION_MARKET`, `INSTITUTION_TREASURY`), `crates/economy/src/lib.rs` (`EconomyState`, `step`, `drain_energy_budget`).
> **Traceability:** FR-CIV-MARKET-001..008 (carry-over from `polities-markets.md` §2 — fully consumed by this spec), FR-CIV-ECON-003 (numeraire emergence), FR-CIV-EMERGENCE-001 (micro-driver → macro-pattern mapping), FR-CIV-EMERGENCE-002 (shared-gradient coupling — no API edges between layers).

---

## 0. Charter constraint

The Civis Emergence Charter forbids hardcoding life / society / psyche / polity / market. The current `crates/economy/src/market.rs` `MarketState::step` is exactly the kind of authored behavior the charter rules out: a `deterministic_price_delta(tick, good)` whose only inputs are `tick` and a byte-hash of the good name, producing a price that drifts in a *scripted* way unrelated to any agent need, any stock gradient, any scarcity, any trade. The current `MarketState::default()` is a hardcoded price table (`food = 1_000`, `energy = 1_000` cents) — also forbidden.

This document specifies how **prices** and **money** EMERGE from the existing substrate:

- **Prices** are a measurement of the cross-pressure between local scarcity (Stocks) and willingness-to-pay (Bid/Offer) at the allocator's order book. There is no `prices: BTreeMap<Good, i64>` that any system mutates independently of trades. Prices are a *read-out* of the last cleared-pair midpoints plus a stock-derived drift, written only as a derived field by `Allocator::clear` (not by `MarketState::step`).
- **Numeraire** is the good with the highest *trade frequency × acceptability* over a sliding window. It is a *measured* label that can change tick-to-tick. No good is privileged in code (`Good::Metal` is *likely* but never *declared*). A region's numeraire may differ from another region's, and a cross-region exchange rate is a ratio of local numeraire prices — also derived, never authored.
- **Money** is a *role*, not a token: the act of pricing goods against the numeraire is what makes the numeraire be money. There is no `currency_id: u32` field anywhere on any agent or institution.

The pre-existing `MarketState::step` random-walk and the hardcoded `Default for MarketState` are explicitly superseded by this spec (the `proptest!` invariants in `market.rs:138-146` — `prices_remain_positive_after_n_steps` — are preserved as properties of the new emergent path, not as bit-identical sequences).

The charter test applied to every field below: *can this emerge from Layer-0 rules?* Yes — prices from order-book cross-pressure on `Allocator`, numeraire from trade-frequency, money from the pricing role. Nothing here is a hardcoded price table, a fixed currency, or a numeraire enum.

---

## 1. Core emergence model

### 1.1 The market state is a measurement, not a state field

There is no `MarketState` whose `step` mutates prices independent of any actual exchange. Instead, the `Allocator` is the **floor**: it is the *only* place where price is set, and only as the clearing-price side-effect of a real `ClearedTrade`. Two read-only projector functions map the live `Allocator` order book + `Stocks` gradients into human- and dashboard-readable price signals:

| Continuous driver | Crate + field | What it does |
|---|---|---|
| Bid order book | `Allocator::bids: BTreeMap<OrderId, Bid>` (`allocator.rs:102`) | Live willingness-to-pay for each `(bidder, good)` pair; bidder's institution pays |
| Offer order book | `Allocator::offers: BTreeMap<OrderId, Offer>` (`allocator.rs:104`) | Live willingness-to-accept; offerer's institution receives |
| Cleared trades | `Vec<ClearedTrade>` returned by `Allocator::clear` (`allocator.rs:176-389`) | The *only* place a per-unit price is set (mid-point of crossing pair, or rationing midpoint) |
| Local stock gradient | `Stocks::get(Good)` (`stocks.rs:55-57`) | Per-actor surplus/deficit driver that pushes bid/offer pressure |
| Comparative advantage | `comparative_advantage(&ProductionProfile)` (`stocks.rs:185`) | Per-actor specialization signal; biases the offer side of the book toward the actor's strongest good |
| Trade-frequency ledger | `Allocator::trades_log: Vec<ClearedTrade>` (NEW field; see §6 P1-A) | Append-only history of every cleared trade; the substrate for numeraire selection |
| Institution ledger | `InstitutionLedger::postings` (`institution.rs:68`) | The conservation layer — every price-conveying transfer posts a balanced pair |
| Allocation engine | `AllocationEngine` (CapitalistAllocator / PlannedAllocator / JouleAllocator, `allocation.rs:5-58`) | Regime selector that determines whether a locale is price-clearing or priority-clearing |

### 1.2 The emergent price signal is a classified read-out, not a stored state

Two projector functions produce the price field that overlays, dashboards, and the polity market-type classifier (FR-CIV-MARKET-002) read. Both are *pure*, *read-only*, and re-evaluated every call — never stored on `MarketState`:

```text
fn emergent_clearing_price(
    allocator: &Allocator,
    trades_log: &[ClearedTrade],
    good: GoodId,
    lookback_ticks: u64,
) -> i64

fn scarcity_drift_price(
    local_stocks: &Stocks,
    profile: &ProductionProfile,
    good: Good,
    anchor_price: i64,
    scarcity_gain: f32,
) -> i64
```

| Read-out field | Definition | Why it is derived, not stored |
|---|---|---|
| **Crossing clearing price** | `emergent_clearing_price` — most recent `ClearedTrade::price` for `good` whose `!rationed` flag is true (crossing pair mid-point from `allocator.rs:255`) | The only price ever set in a real exchange. No fictional price exists. |
| **Rationing clearing price** | Most recent `ClearedTrade::price` for `good` with `rationed == true` (mid-point of best unmatched bid/ask from `allocator.rs:392-416`) | The "scarcity" price signal; converges toward the crossing price as rationing becomes rarer. |
| **Scarcity drift** | `scarcity_drift_price` — `anchor_price * (1 + scarcity_gain * deficit / max(stock, 1))`, integer-fixed-point | The supply-side pull: stocks near zero push price up; surplus pulls it down. No global `MarketState::step` needed. |
| **Windowed VWAP** | Volume-weighted average of `ClearedTrade::price * ClearedTrade::quantity` over the last `lookback_ticks` cleared trades per good | Smooths noise without storing a price book; the inputs are the *trades themselves*. |
| **Numeraire price** | Emergent unit of account — the good with the highest `(trade_count × acceptability_in_other_goods)` over `lookback_ticks` (see §3) | The good that *becomes* money; not selected by code, not declared by an enum. |

`MarketState` itself is *not* removed in this spec — the new `MarketState` is a **pure read-only struct** that holds `(last_emergent_clearing: BTreeMap<GoodId, i64>, last_rationing: BTreeMap<GoodId, i64>, last_scarcity_drift: BTreeMap<GoodId, i64>, numeraire: Option<GoodId>, vwap: BTreeMap<GoodId, i64>, recomputed_tick: u64)` — every field written only by the read-out projectors. The current `MarketState::step` (which mutates prices independent of trades) is replaced by `MarketState::recompute(allocator, stocks_view, lookback) -> &Self`, which is a pure function of the order book + stocks view. The `Default for MarketState` that hardcodes `food = 1_000`, `energy = 1_000` cents is replaced by a `Default` that returns an *empty* `MarketState` (no prices pre-baked — the very first trade in a new scenario sets the first price; until then, `emergent_clearing_price(g)` returns `None` and the UI shows "no trade yet").

### 1.3 What produces each macro phenomenon

**Clearing prices.** A crossing pair in `Allocator::clear` (`allocator.rs:251-270`) sets `clearing_price = (bid_price + offer_price) / 2`. The crossing-price series for good `g` is the *measurement* of how much one side will pay relative to the other at the moment of exchange. There is no `update_price()` callable that bypasses a real trade.

**Hyperinflation / deflation.** Sustained unilateral pressure on the book — bids rising while asks stay put, or vice versa — pushes the crossing mid-point each tick. The price is the integrated history of imbalance, not a stochastic delta. Convergence comes from a deeper cause: actors learn (via psyche temperament + utility AI, not via this spec) that posting too high a bid / too low an ask gets cleared instantly, while posting too low / too high sits on the book — so the *distribution* of posted prices self-organizes around the true scarcity.

**The numeraire.** A good becomes money when it is the most *liquid* — most-traded AND most-acceptable-as-counterparty. `Stocks::surplus` (`stocks.rs:172-175`) and `comparative_advantage` (`stocks.rs:185`) are the actors' supply-side pressures; the bid/offer book is the demand-side pressure. The good that is offered in the largest number of *cross-good* trades (i.e. is the side of a `ClearedTrade` that is not the "primary" good) is the candidate numeraire. This is *measured*, not declared.

**Cross-region exchange rates.** When region `A`'s numeraire is `g_A` and region `B`'s is `g_B`, the cross-rate is `emergent_clearing_price(g_A in A) / emergent_clearing_price(g_B in B)`. The rates are derived; arbitrage flows (§2.3) cause the two prices to converge when transport is cheap and diverge when it is expensive — the rates themselves are not authored.

**Credit / debt.** A `ClearedTrade` with `bidder == offerer` (self-trade, already supported at `allocator.rs:448-457`) is the seed pattern: a balanced `InstitutionPosting` of a future obligation. The full credit-market spec lives in `polities-markets.md` §FR-CIV-MARKET-008 and is reused here unchanged; this spec only guarantees that the *price field* on which credit is denominated is itself emergent.

---

## 2. Bidirectional coupling — the substrate gradient, not the API edge

The charter explicitly forbids one emergent layer calling another through an API boundary with no lag. The mechanism here is the same as `civ-003-emergent-lifecycle.md` §2: **shared conserved gradients with explicit lags**. Three gradients carry the coupling: `Stocks` (integer stock vector), the `Allocator` order book (per-tick live bids/offers), and the institution posting log (the audit trail). Nothing in this spec introduces a new edge between crates.

### 2.1 Market → Lifecycle / Diplomacy / Agents (downward causation, no API call)

| Market signal (read by what) | Effect on the layer | Mechanism (shared gradient, not call) |
|---|---|---|
| `ClearedTrade::price` rises for `Good::Food` (read by agents / needs) | `Needs::food` decay is unchanged, but the *cost* of acquiring food via trade rises; agents whose profile can't produce food switch consumption strategies | The next `Allocator::post_bid` call from a food-short agent posts a *higher* `Bid::price` (driven by psyche `drives[security]` + needs criticality, not by a market→agent API). The trade clears, the price is *read* by the agent's next decision step. |
| `ClearedTrade::price` falls for `Good::Tools` (read by agents / cluster) | Tool-bearing agents' relative bargaining power rises; they post more offers of food-for-tools (long-range mercantile pattern) | The substrate signal is the *price fall* observable on the order book. No `market.on_price_change(...)` callback exists. |
| Numeraire shifts from `Good::Food` to `Good::Metal` (read by diplomacy / polity) | Trade relationships measured in "metal-equivalent" start to make sense; old food-debt postings still exist as historical `ClearedTrade` records but are no longer the natural unit | The numeraire read-out is consumed by the diplomacy `apply_signal` (the existing `trade_volume` channel) and the polity cohesion graph weight `w_econ · payoff_if_coordinated` (`polities-markets.md` §1.2). Both consume the *measured* numeraire; neither mutates it. |
| Rationing trades fire (`ClearedTrade::rationed == true`, allocator.rs:357) | Agents downstream read "supply is short for `g`" | The rationing flag is observable on the trade log; the next `Allocator::post_offer` from a long-on-`g` agent *responds* to that observable scarcity by raising `Offer::price`. Lag = 1 economy tick. |

### 2.2 Lifecycle / Diplomacy / Agents → Market (upward causation with lag)

| Layer signal | Market effect | Lag mechanism |
|---|---|---|
| `Stocks::get(Good::Food)` falls toward zero on an actor | That actor's next `Allocator::post_bid` posts a *higher* `Bid::price` for food (driven by need decay) | 1 tick lag: stock change → needs tick → utility decision → next bid post |
| `comparative_advantage(profile) == Good::Metal` on actor X | X's next `Allocator::post_offer` posts a metal offer; the offer quantity tracks X's metal stock | 1 tick lag: profile read → offer post |
| High `DiplomacyMatrix.relation(actor, counterparty)` (alliance) | Counterparty is *more likely* to be on the same book as the actor (because both cluster co-locate + share culture); cross-cluster trade volume rises | Indirect: kinship/culture drift → cluster overlap → spatial co-location → order book co-presence. No direct market↔diplomacy edge. |
| Polity coercion overlap (FR-CIV-POLITY-008) | `AllocationRegime` for that locale flips to `Planned` (the regime selector is in `allocation.rs:111-120`); planned override is `polities-markets.md` §FR-CIV-MARKET-006 | 1 tick lag: polity overlap computed → regime selector re-evaluates → next order-book clear runs under the new regime |
| Mortality / immigration shock (lifecycle FR-CIV-LIFE-003) | Number of active bidders / offerers in the locale drops; book thins; price volatility rises | Structural lag = 1 generation (~20 in-game years for a human cohort) |

### 2.3 Arbitrage as a shared-gradient phenomenon, not a call

Arbitrage is the *visible* signature that two locale's price fields have diverged: actor in locale A with surplus food sees locale B's food price (the crossing clearing price on B's book) is higher than A's, so posts a long-range bid on B's book (via existing `propose_trade` (`stocks.rs:214-270`) and the `Allocator::post_bid` path). The two prices converge through these *actual bids*, not through an `arbitrage_actor.tick()` callback. The emergent cross-rate settles to the transport-cost band; if transport is forbidden (no path), the rates diverge permanently and the two regions develop independent numeraires — also emergent.

### 2.4 Coupling to credit / polity (shared institution ledger)

Credit and polity coercion both flow through `InstitutionLedger::post` (`institution.rs:146-168`); a credit posting is a balanced pair that sits in the posting log (`institution.rs:68`); a polity treasury posts a funding transfer to `INSTITUTION_MARKET` that biases the book on that institution's behalf. The price field on the credit settlement reads off the *clearing price at the time the credit is posted* (recorded in the `ClearedTrade` and copied into the institution posting as the unit-of-account anchor). The settlement price is never *re-priced* later — late settlement is a separate posting, not a price update.

---

## 3. Emergent numeraire

### 3.1 Numeraire is a measurement over the trade log

There is no `numeraire: Good` field stored on any agent, institution, or economy state. The numeraire for a locale at tick `t` is the *good* `g*` that maximises a liquidity score computed only from the trade log:

```text
fn select_numeraire(
    trades_log: &[ClearedTrade],
    lookback: u64,
    tick: u64,
) -> Option<GoodId>
```

| Component | Definition | Source |
|---|---|---|
| `trade_count(g, lookback)` | Number of `ClearedTrade` entries for good `g` in `trades_log` whose `tick > tick - lookback` | `Allocator::trades_log` (new, §6 P1-A) |
| `cross_good_acceptance(g, lookback)` | Number of distinct *other* goods `g'` such that at least one `ClearedTrade` exists in the window where `cleared_trade.good == g` and there is a separate `ClearedTrade` with `cleared_trade.good == g'` between overlapping institution pairs (counterparty graph) | Same |
| `liquidity_score(g)` | `trade_count(g) * sqrt(cross_good_acceptance(g))` | Derived |
| `numeraire(g*)` | `argmax_g liquidity_score(g)`; ties broken by FIFO `ClearedTrade::tick` | Derived |

The numeraire is recomputed every time the dashboard / overlay asks (cheap: a single pass over the windowed log). The numeraire can change tick-to-tick. No code path treats a particular good as privileged.

### 3.2 Money as a role, not a token

When the numeraire is `g*`, all *other* goods' emergent prices can be re-denominated by dividing by the numeraire's emergent clearing price (or by VWAP of `g*`). This re-denomination is the *act of pricing* — it is what makes `g*` function as money. The re-denomination is computed on read, never written to a stored price field. Two different locales may have two different numeraires; the cross-rate is the ratio of their `emergent_clearing_price(g_local)` series.

### 3.3 What the numeraire read-out does NOT do

- It does NOT privilege any good in code. `Good::Metal` and `Good::Tools` are *likely* numeraires by durability + acceptability, but a famine region with no tool production and a salt-flat (if `Good::Salt` were ever added) can have a different numeraire. Acceptance is measured, not declared.
- It does NOT create a `currency_id` field. There is no token; money is the act of pricing.
- It does NOT lock in. The numeraire can shift. The shift is itself a measurable event (a `numeraire_shift.v1` replay-bus event) consumed by legends engine for narrative.

### 3.4 What this spec inherits from `polities-markets.md` §FR-CIV-MARKET-007

The numeraire spec in `polities-markets.md` §2.4 already declares "no hardcoded currency, the most-liquid good wins." This document makes it *implementation-ready* by: (a) naming the substrate field (`Allocator::trades_log` — the only new data structure), (b) defining the `select_numeraire` signature, (c) defining the input window `lookback`, (d) tying the recompute cadence to the economy tick (`step` in `lib.rs:189-199`).

---

## 4. Criticality knobs — edge of chaos

All knobs concentrate in a new `EmergentMarketParams` struct on `EconomyState` (loaded from scenario RON, same pattern as `civ-003-emergent-lifecycle.md` §3 `LifecycleParams`). Defaults target weak emergence (Class 4): the system is in a price-formation band, not collapsing to barter-rationing and not exploding to hyper-deflation.

| Parameter | Type | Default | Effect on market dynamics | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `tatonnement_lambda` | `i64` (integer scale 1..1000) | `50` | Step size of the per-tick scarcity-drift adjustment on non-cleared goods | `> 500` (prices oscillate / overshoot) | `< 5` (prices never move; barter deadlock) |
| `tatonnement_scale` | `i64` | `1_000_000` | Denominator that keeps scarcity drift integer | depends on `lambda` | depends on `lambda` |
| `min_price_cents` | `i64` | `1` | Floor on every emergent clearing price (preserves the `market.rs:144` positive-price invariant) | `0` (prices go to zero, barter is free) | `> 100` (friction kills liquidity) |
| `rationing_lookback` | `u64` | `256` | Number of past ticks over which rationing frequency is averaged for the dashboard | `> 4096` (smoke alarm too slow) | `< 16` (noisy signal) |
| `vwap_lookback` | `u64` | `1024` | Sliding window for the VWAP read-out (used by overlays) | `> 16384` (stale prices) | `< 64` (jitter) |
| `numeraire_lookback` | `u64` | `4096` | Sliding window for `select_numeraire`; longer = more stable numeraire | `> 32768` (numeraire inertia) | `< 256` (numeraire flickers) |
| `numeraire_min_trade_count` | `u64` | `8` | Minimum trade count in the window for a good to be *eligible* to be numeraire | `> 64` (only well-traded goods can be money — biases toward durability) | `1` (any single trade can crown a numeraire) |
| `arbitrage_window` | `u64` | `512` | Sliding window for the cross-region arbitrage convergence metric (see §6 dashboard) | `> 8192` (stale arb signal) | `< 64` (noisy) |
| `emergent_overshoot_cap` | `f32` | `0.20` | Maximum fractional price change per tick in the scarcity-drift path (antitrust fuse) | `0.0` (no drift) | `1.0` (50%/tick moves are allowed) |
| `clearing_persistence_decay` | `f32` | `0.95` | EWMA weight when the crossing-price series is empty (smooth carry-over of last clearing) | `1.0` (carries forever; unresponsive) | `0.5` (flickers) |

All knobs are grouped in one `EmergentMarketParams` struct (not scattered across modules) and loaded from the scenario RON config. The emergence dashboard (§5) plots a real-time criticality indicator so the designer can see whether the system is heading toward heat-death or explosion before adjusting.

---

## 5. Observable emergence metrics for the dashboard

These metrics feed the **Emergence Dashboard** (`crates/engine/src/emergence.rs` expansion per `emergence-dashboard.md` §3.2 entropy on `market-good` layer and §3.5 coupling MI on `(market-good, building-type)` and `(market-good, civilian-faction)` pairs). They are aggregates computed cheaply from existing `Allocator` + `ClearedTrade` + `Stocks` views — no new agent-level state.

| Metric | How to compute | Target signature (healthy market) | Failure mode |
|---|---|---|---|
| **Market-good entropy** | Histogram of `ClearedTrade::good` over `vwap_lookback` window; Shannon entropy; normalised by `log2 GOODS.len()` | High (0.6..0.9 of normalised) — many goods actively traded | Low (one good dominates) = monopoly/clique |
| **Clearing-price variance per good** | Variance of `ClearedTrade::price` per good over the window; report top-3 goods | Moderate; oscillates with shocks then settles | Persistent near-zero = barter deadlock (no price discovery) |
| **Rationing frequency** | `count(cleared_trade.rationed == true) / count(cleared_trade.all)` over `rationing_lookback` | Low (0.0..0.15) — most clears are crossing | Sustained > 0.5 = chronic scarcity (charter violation: hardcoded prices wouldn't fix it; the substrate needs more supply) |
| **Book liquidity (depth)** | Sum of `Bid::quantity` over all live bids + sum of `Offer::quantity` over all live offers, per good | High and roughly balanced bid↔ask | One side always empty = single-sided market (e.g. everyone is a net consumer of `g`) |
| **Crossing ratio** | `count(cleared_trade.rationed == false) / count(cleared_trade.all)` | High (≥ 0.85) — book self-organizes | Low = price discovery is broken; agents post blind prices |
| **Numeraire persistence** | Number of consecutive ticks the same `GoodId` is the argmax in `select_numeraire`; reset to 0 on shift | Long runs (hundreds of ticks) | Frequent shifts (< 50 ticks) = currency chaos (legibility, but not charter violation) |
| **Numeraire share of trade volume** | `sum(cleared_trade.quantity for cleared_trade.good == numeraire) / sum(all quantity)` over `numeraire_lookback` | Moderate (0.05..0.30) — numeraire is one of many traded goods but not dominant | Zero (numeraire never traded as a side good) = misnamed; it is a preferred reserve, not a unit of account. > 0.5 = hoarding |
| **Cross-region price dispersion** | For each good `g` in two locales A,B: `|vwap_A(g) - vwap_B(g)| / max(vwap_A(g), vwap_B(g))`, averaged over goods | Small (≤ 0.2) when transport is cheap; large when isolated | Persistent near-1.0 in adjacent regions with road access = arbitrage failure (institution-level friction) |
| **Numeraire exchange-rate volatility** | Variance of the cross-rate over `arbitrage_window` | Small | Large and growing = speculation or transport breakdown |
| **MI: market-good ↔ civilian-faction** | Histogram-based MI per `emergence-dashboard.md` §3.5 on the canonical pair | Moderate (0.2..0.6 normalised) — factions specialise but aren't perfectly predicted by what they trade | Near 0 = no faction/economy coupling; near 1.0 = factions are just trade clubs |
| **MI: market-good ↔ building-type** | Same on `(good, building_type)` | Moderate | Near 0 = economy is decoupled from built form; near 1.0 = building type fully determines trade |
| **Conservation invariant** | `InstitutionLedger::verify_conservation` + `verify_ledger_conservation` per tick | Always Ok | Any failure = critical bug; halt and alarm |
| **Institution balance non-negativity** | All `InstitutionAccount::balance_joules >= 0` | Always true | Negative = a posting violated the floor; halt and alarm |

The `emergence.metrics.v1` replay-bus event (per `emergence-dashboard.md` §5) is extended with the `market_good` layer so the Godot / Unreal / web clients show the same entropy + MI series offline. The `emergence.alarm.v1` event is extended with three new alarm IDs derived from this spec: `MT-MKT-001` (chronic rationing frequency > 0.5 for 256 ticks), `MT-MKT-002` (numeraire shift > 4 times in 4096 ticks — currency chaos), `MT-MKT-003` (cross-region dispersion > 0.8 in adjacent regions with road access — arbitrage dead).

---

## 6. Phased implementation plan

This is a DAG-structured WBS. No code is written here; file paths, struct names, and function signatures are identified for the implementing agent. Every phase extends *existing* substrate — no new crate, no API edge to lifecycle / diplomacy.

### Phase 0 — Prerequisite audit (no new structs)

| Task | File | Depends on | Agent effort |
|---|---|---|---|
| P0-A: Verify `Allocator::clear` already returns `Vec<ClearedTrade>` with `price`, `rationed`, `quantity`, `tick` populated (it does; `allocator.rs:176-389` + `best_unmatched_price_for_good` at `allocator.rs:392-416`) | `crates/economy/src/allocator.rs` | — | 1 tool call |
| P0-B: Confirm `Stocks::get(Good)` + `surplus` / `deficit` are integer-only and clamp at zero (they do; `stocks.rs:55-75`, `stocks.rs:172-182`) | `crates/economy/src/stocks.rs` | — | 1 tool call |
| P0-C: Map which `MarketState::prices` consumers exist (search for `market_state.prices`, `MarketState::default()`, `MarketState::step`) and which depend on the hardcoded `food=1_000`, `energy=1_000` defaults | `crates/` (search) | — | 3 tool calls |
| P0-D: Identify the economy phase driver (where `step(state)` is called; per `lib.rs:189-199`) and confirm it's the right hook for `MarketState::recompute` (it is) | `crates/economy/src/lib.rs` + `crates/engine/src/engine.rs` | — | 2 tool calls |
| P0-E: Confirm `Allocator::trades_log` does NOT yet exist; confirm the `ClearedTrade` returned by `Allocator::clear` is currently discarded by all callers | `crates/economy/src/allocator.rs` + callers | — | 2 tool calls |

### Phase 1 — Extend `Allocator` with append-only trade log + new bid/offer pressure functions

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P1-A: Add `pub trades_log: Vec<ClearedTrade>` field to `Allocator` (`allocator.rs:99-107`) | `crates/economy/src/allocator.rs` | P0-E | Append every `ClearedTrade` returned by `Allocator::clear` into `trades_log` *inside* `clear` itself (line 268, 361). Add `trades_log.retain(\|t\| t.tick > current_tick.saturating_sub(self.max_log_ticks))` to bound memory. |
| P1-B: Add `EmergentMarketParams` struct (all §4 knobs) to a new `crates/economy/src/emergent.rs` module | new `crates/economy/src/emergent.rs` | P0-A | Re-exported via `lib.rs:32` alongside the other economy exports. `Default::default()` returns the §4 defaults. |
| P1-C: Add `pub fn emergent_clearing_price(allocator: &Allocator, good: &str, lookback_ticks: u64) -> Option<i64>` | `crates/economy/src/emergent.rs` | P1-A | Walks `allocator.trades_log` from the tail, returns the most recent `ClearedTrade::price` for `good` whose `!rationed`, or `None` if absent. |
| P1-D: Add `pub fn select_numeraire(allocator: &Allocator, params: &EmergentMarketParams, tick: u64) -> Option<GoodId>` | same | P1-A, P1-B | Computes `liquidity_score(g)` per §3.1, applies `numeraire_min_trade_count` floor, returns argmax. |
| P1-E: Add `pub fn scarcity_drift_price(stocks: &Stocks, profile: &ProductionProfile, good: Good, anchor: i64, params: &EmergentMarketParams) -> i64` | same | P1-B, P0-B | Integer arithmetic: `deficit = stocks::deficit(...)`, `drift = (anchor * scarcity_gain_bps * deficit) / (max(stock, 1) * scale)`, clamp by `emergent_overshoot_cap`. |
| P1-F: Tests: P1-C returns None for empty log; returns most recent non-rationed price when present; P1-D returns None when no good clears `numeraire_min_trade_count`; P1-E drift is monotonic in deficit when anchor > 0; P1-A retain window does not drop fresh entries | `crates/economy/src/emergent.rs` tests | P1-A..E | Property-based via `proptest` consistent with the rest of the crate |

### Phase 2 — Replace `MarketState` with the read-only projector

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P2-A: Change `MarketState` to the read-only struct described in §1.2 (fields: `last_emergent_clearing`, `last_rationing`, `last_scarcity_drift`, `numeraire`, `vwap`, `recomputed_tick`). Remove `MarketState::step`. | `crates/economy/src/market.rs` | P1-C, P1-D, P1-E | The new struct has *no* `step` method. |
| P2-B: Add `pub fn recompute(allocator: &Allocator, stocks: &Stocks, profile: &ProductionProfile, params: &EmergentMarketParams, tick: u64) -> MarketState` | same | P2-A, P1-B | Calls P1-C, P1-D, P1-E for each `Good` in `GOODS`; computes VWAP; populates the struct; returns it. Pure function. |
| P2-C: Change `Default for MarketState` to return an empty struct (no hardcoded `food=1_000`); add a helper `MarketState::is_empty()` | same | P2-A | Preserves the `prices_remain_positive_after_n_steps` invariant trivially (vacuously true on empty) — and adds the new invariant: *a price only exists if a trade existed* |
| P2-D: Wire `MarketState::recompute` into `EconomyState::step` (`lib.rs:189-199`) so the struct is refreshed once per economy tick from the current `Allocator` + stocks view | `crates/economy/src/lib.rs` | P2-B | The recompute writes to a *new* field `EconomyState::market: MarketState` (replacing the current re-export; see P2-E). |
| P2-E: Update `pub use market::MarketState;` (`lib.rs:31`) and add `pub use emergent::{EmergentMarketParams, emergent_clearing_price, select_numeraire, scarcity_drift_price};` | same | P1-B, P2-A | One new export block. |
| P2-F: Migrate every `MarketState::step` caller found in P0-C to either (a) read `EconomyState::market` once at the call site or (b) call `MarketState::recompute` explicitly if the caller needs a local snapshot | callers from P0-C | P2-D | The `proptest!` `prices_remain_positive_after_n_steps` test at `market.rs:138-146` is replaced by a test that asserts `MarketState::recompute` after a sequence of `Allocator::clear` calls yields a non-empty struct with the most recent crossing price for the most-recently-traded good. |
| P2-G: Tests: P2-C empty default; P2-B recompute is pure (same inputs → same output); P2-B respects `numeraire_lookback` window; P2-B never returns a negative price (preserves the existing positivity invariant); P2-F migration: each call site compiles and runs the same scenario without conservation violation | `crates/economy/src/market.rs` tests + call-site integration tests | P2-A..F | Property tests on P2-B purity are mandatory per charter (§"Determinism NOT required" still allows *function-purity* assertions on read-out projectors) |

### Phase 3 — Bidirectional coupling hooks (no new edges)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P3-A: Add `pub fn record_bid_for_need(stocks: &Stocks, profile: &ProductionProfile, good: Good, market: &MarketState, params: &EmergentMarketParams) -> Bid` helper to `crates/economy/src/emergent.rs` | `crates/economy/src/emergent.rs` | P2-B, P0-B | Pure: returns a `Bid` with `price = scarcity_drift_price(stocks, profile, good, market.last_emergent_clearing[good], params)` and `quantity = stocks::deficit(stocks, profile, good).max(0)`. Agents in `civ-agents` use this when posting; no direct API from `civ-economy` to `civ-agents`. |
| P3-B: Add `pub fn record_offer_from_surplus(stocks: &Stocks, profile: &ProductionProfile, market: &MarketState, params: &EmergentMarketParams) -> Vec<Offer>` | same | P3-A | One `Offer` per good where `stocks::surplus > 0`, priced at the crossing price minus a 1..5% buffer (psychology-style; the exact buffer is a sub-knob). |
| P3-C: Add `pub fn cross_locale_arbitrage_opportunity(local: &MarketState, remote: &MarketState, good: GoodId, transport_cost_cents: i64) -> Option<(Side, i64)>` | same | P2-B | Returns `Some((Buy, remote_price - local_price - transport))` if positive, `Some((Sell, local_price - remote_price - transport))` if positive, else `None`. The actor that sees this opportunity is the *existing* `propose_trade` path (`stocks.rs:214-270`) — it just reads `cross_locale_arbitrage_opportunity` to decide what to offer. |
| P3-D: Add `numeraire_shift.v1` replay-bus event to the engine phase; payload `{tick, old: Option<GoodId>, new: Option<GoodId>, liquidity_score_new: u64}` | `crates/engine/src/engine.rs` | P2-B | Emitted when `select_numeraire` returns a different `GoodId` than the previous tick. Consumed by `civ-watch` event feed + legends engine. |
| P3-E: Tests: P3-A bid price rises monotonically in deficit; P3-B offer count equals count of surplus goods; P3-C returns `None` when transport cost exceeds price gap; P3-D fires only on actual numeraire change; all P3 tests are pure (same inputs → same outputs) | `crates/economy/src/emergent.rs` tests + engine integration test | P3-A..D | Property-based |

### Phase 4 — Criticality knobs + dashboard metrics

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P4-A: Add `EmergentMarketParams` field to `EconomyState` (next to `last_step_budget_joules` at `lib.rs:78`) so the params travel with the snapshot | `crates/economy/src/lib.rs` | P1-B | Serialise with `#[serde(default)]`; load from scenario RON. |
| P4-B: Extend `crates/engine/src/emergence.rs` with `market_good` layer: `compute_market_metrics(world, economy, allocator, params) -> MarketMetrics` returning the 13 metrics from §5 | `crates/engine/src/emergence.rs` | P1-A, P2-B | All metrics aggregate from existing `Allocator::trades_log`, `MarketState::recompute` output, and `Stocks` views. |
| P4-C: Wire the `market_good` layer into the existing dashboard entropy + MI series in `civ-server`'s `sim.snapshot` (already in the F3D0 binary frame per `fr-3d-matrix.md`) | `crates/server/` (or wherever the F3D0 frame is built) | P4-B | No new frame variant; just new fields in the existing `emergence` section. |
| P4-D: Extend `emergence.metrics.v1` replay-bus event with `market_good_entropy`, `rationing_frequency`, `numeraire_persistence`, `numeraire_share`, `cross_region_dispersion`, `mt_mkt_001_threshold_breach`, `mt_mkt_002_threshold_breach`, `mt_mkt_003_threshold_breach` fields | `crates/engine/src/emergence.rs` | P4-B | Same wire format, new fields. |
| P4-E: Tests: P4-B metrics stay bounded for populations of 10, 100, 1000 actors; P4-C snapshot round-trips through `bincode` (preserves FR-3D replay identity for the *new* fields); P4-D replay event deterministically re-emits given a fixed `trades_log` | `crates/engine/` tests | P4-A..D | Determinism of the *metrics* is not required (charter); determinism of the *function-purity* of `compute_market_metrics` IS required for snapshot round-trip testing |

### Phase 5 — Acceptance criteria reachability (validation)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P5-A: Scenario test: "no fixed price table" — start with empty `MarketState` (post P2-C), let the system run for 200 ticks with at least 3 actors posting bids and offers; assert `MarketState.last_emergent_clearing` is populated and the first price is *not* 1_000 (the old hardcoded default) | `crates/economy/tests/` integration test | Phase 2 complete | Charter AC: prices emerge from trades |
| P5-B: Scenario test: "numeraire is data-driven" — construct a scenario where the only good with sustained trade frequency and cross-good acceptance is `Good::Wood`; assert `select_numeraire` returns `Some("wood")`. Then construct a scenario where `Good::Metal` is the only well-traded good; assert numeraire shifts. | same | P5-A, P3 complete | Charter AC: changing which good is most liquid changes the emergent numeraire with no code change (carries over from `polities-markets.md` AC-9) |
| P5-C: Scenario test: "charter conservation" — run a 1000-tick scenario with mixed regime (capitalist + planned); assert `InstitutionLedger::verify_conservation` succeeds every tick; assert no `ClearedTrade::price` is negative; assert the `prices_remain_positive_after_n_steps` invariant from `market.rs:138-146` still holds (now reading off the emergent series, not the deleted `MarketState::step` path) | same | Phase 4 complete | Conservation AC; carries over from `polities-markets.md` AC-8 |
| P5-D: Scenario test: "numeraire shift is a measured event" — run a 5000-tick scenario; assert `numeraire_shift.v1` events fire at most ~5 times (otherwise MT-MKT-002 should fire) and that the shift is always between two goods that were *both* traded above `numeraire_min_trade_count` | same | P3-D | Legibility AC; not a charter requirement but required for the dashboard to make sense |
| P5-E: Scenario test: "criticality edge of chaos" — run a 4096-tick scenario with default `EmergentMarketParams`; assert `market_good_entropy` ∈ [0.6, 0.9] normalised (the §5 target band); assert `rationing_frequency` < 0.15; assert `clearing_persistence_decay` default yields an EWMA that doesn't lock or flicker | same | P4 complete | Performance-gated; runs in CI as a regression |
| P5-F: Scenario test: "criticality knob reachability" — for each knob in §4, set it to its heat-death and explosion extremes; assert the system reaches the corresponding failure mode within 512 ticks (rationing_frequency > 0.5, numeraire flickering, or price series pinning at `min_price_cents`); assert setting it back to default returns the system to the target band within 2048 ticks | same | P5-E | Ensures the knobs are *real* knobs, not constants |

### Phase 6 — Coupling sanity (cross-layer, no new edges)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P6-A: Audit `crates/agents/` for any direct `market_state.prices[g]` reads that depended on the hardcoded default; replace with `MarketState::recompute(...).last_emergent_clearing.get(g)` so the first-ever read in a fresh scenario gets `None` (UI shows "no trade yet") | `crates/agents/` | Phase 2 + Phase 3 | No new crate dep; just a one-line read-site migration |
| P6-B: Audit `crates/diplomacy/` for any reads of price that fed `trade_volume` or `resource_competition` signals; ensure they now read the emergent series (not the deleted `MarketState::step` path) | `crates/diplomacy/` or `crates/agents/src/diplomacy.rs` | P6-A | Same shape as P6-A |
| P6-C: Audit `crates/build/` for any reads of `market_state.prices` that fed building type / tile-set selection (per `emergent-systems-spec.md` §2.3 the build layer is data-driven from culture vec, but the WFC tile-set table may have weighted by local price as a secondary signal); ensure it reads the emergent series | `crates/build/` | P6-A | If no such read exists, document the absence and close the task |
| P6-D: Add a CI-level invariant test: assert that NO production code path mutates `MarketState` except by calling `MarketState::recompute` (static check via `#[test]` that walks the crate's public surface) | `crates/economy/tests/` | Phase 2 complete | Closes the loop on the charter: there is no other way for prices to change |

### DAG summary (critical path)

```
P0-* (all parallel) → P1-A..F → P2-A..G (P2-D depends on P2-B which depends on P1-*; P2-F depends on P0-C audit)
                  ↘ P3-A..E (depend on P2-B) → P4-A..E (depend on P3-D and P1-*)
                                              ↘ P5-A..F (depend on P4 complete)
                                              ↘ P6-A..D (depend on P2 complete, can run in parallel with P5)
```

**Critical path to acceptance:** P0 → P1 → P2 → P3 → P4 → P5. The P6 audit can run in parallel with P5 and is the last gate before charter compliance is complete.

---

## 7. Test strategy summary

- **Unit tests** (property-based via `proptest`): each new pure function in `emergent.rs` has invariant tests (price floor, monotone scarcity drift, numeraire argmax correctness, conservation preserved, VWAP windowed correctly).
- **Integration tests** (hecs World with seeded RNG): a 100-actor 4096-tick scenario with mixed regime; assert market-good entropy is in the target band, no conservation violation, numeraire is non-trivial, and the `prices_remain_positive_after_n_steps` invariant from the deleted `market.rs:138-146` still holds via the new emergent path.
- **Emergence regression**: `cargo test -p civ-economy -- emergent_market_regression` runs the P5-E 4096-tick scenario and asserts `market_good_entropy ∈ [0.6, 0.9]`, `rationing_frequency < 0.15`, conservation holds, and the price series stays strictly positive. This runs in CI as a performance-gated test.
- **No determinism requirement** (per charter): tests assert statistical properties of the macro market (entropy bands, numeraire argmax), not bit-identical outcomes of the per-tick price series. The *function-purity* of `MarketState::recompute`, `select_numeraire`, and `compute_market_metrics` is asserted separately (same inputs → same outputs) so the dashboard / snapshot tests can rely on it.

---

## 8. What this spec does NOT include

- Any `enum MarketType` or `enum Currency` stored on a market locale, an agent, or an institution.
- Any `update_price(g, p)` callable that mutates a price independent of a real `ClearedTrade`.
- Any hardcoded price table (the old `MarketState::default` at `market.rs:14-21` is replaced by an empty default; the new prices only exist because a trade happened).
- Any cross-crate API edge from `civ-economy` to `civ-agents` / `civ-diplomacy` / `civ-build`. All coupling is the shared substrate gradient (Stocks, Allocator order book, InstitutionLedger postings), per the charter.
- Any LLM call in the price-discovery or numeraire path.
- Any LLM garnish for "narration of price changes" — that is the legends engine's job (`legends-engine.md` reads the `numeraire_shift.v1` event as input, not as output).
- A `clearing_persistence_decay` knob that is *zero* (would erase the price series every tick; we want a long EWMA carry-over for stability, not a hard reset).
- Re-implementation of the existing CDA / `Allocator::clear` matching logic; this spec extends the *output* (the `trades_log`) and the *read-out* (the new `MarketState` projector), not the matching math.
- A migration of the existing `market.rs` `step` random-walk — that path is *deleted* in P2-A, not preserved. The `proptest!` invariants on it are ported to the new emergent path (P2-F).

---

*Document authority: this spec supersedes the random-walk `MarketState::step` path and the hardcoded `Default for MarketState` price table. The emergent price field, the emergent numeraire, and the cross-region exchange rate are all read-only projections over `Allocator::trades_log` + `Stocks` + `InstitutionLedger::postings`. There is no other way for a price to be set, a numeraire to be chosen, or a currency to exist. The charter's "measured, emergent pattern over the substrate" is satisfied end-to-end.*
