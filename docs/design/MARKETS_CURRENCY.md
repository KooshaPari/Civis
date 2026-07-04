# MARKETS_CURRENCY: Emergent Markets + Emergent Currency — Design Spec

> **Status:** Design (planner-only, 2026-06-24). No implementation code in this document.
> **Spec ID:** `civ-markets-currency` | **Layer:** Macro read-out on top of the trade-route + economy substrate that is already on `main` (no new substrate).
> **Governing canon:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) (Layer-0 only is authored; everything above emerges), [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) §3 E1 (economy emergence), [`docs/design/polities-markets.md`](polities-markets.md) §2 (price-discovery / numeraire / credit read-outs — *parent*), [`docs/design/civ-economy-emergent-markets.md`](civ-economy-emergent-markets.md) (companion: pure projectors + market-state migration), [`docs/design/emergence-dashboard.md`](emergence-dashboard.md) (observable metrics + alarms), [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) (sibling pattern: pure read-only classifier functions).
> **Pattern ancestors:** `civ-003-emergent-lifecycle.md` §1.2 (pure read-out classifiers, never mutate substrate), `civ-culture-emergent.md` §1.2 (additive classifier function set).
> **Code substrate (read-only inputs on `main`):** `crates/economy/src/trade_routes.rs` (`Settlement`, `SettlementId`, `TradeRoute`, `route_flow`, `compute_trade_routes`, `routes_lexicographic`), `crates/economy/src/stocks.rs` (`Good`, `GOODS`, `Stocks`, `ProductionProfile`, `surplus`, `deficit`, `comparative_advantage`, `trade_gain`, `propose_trade`, `apply_trade`), `crates/economy/src/allocator.rs` (`Allocator`, `Bid`, `Offer`, `ClearedTrade`, `Allocator::post_bid`, `Allocator::post_offer`, `Allocator::clear`, `best_unmatched_price_for_good`), `crates/economy/src/institution.rs` (`InstitutionId`, `InstitutionLedger`, `InstitutionPosting`, `LedgerSide`, `INSTITUTION_MARKET`, `INSTITUTION_TREASURY`), `crates/economy/src/allocation.rs` (`AllocationRegime`, `AllocationEngine`, `CapitalistAllocator`, `PlannedAllocator`, `JouleAllocator`), `crates/economy/src/market.rs` (`MarketState`, `MultiGoodMarket`, `OrderBook`, `Trade`, `Side`, `DEFAULT_PRICE_CENTS`, `MAX_PRESSURE_DELTA_CENTS`, `MIN_PRICE_CENTS`), `crates/economy/src/lib.rs` (`EconomyState`, `EconomyState::tick`, `step`, `drain_energy_budget`, `verify_ledger_conservation`).
> **Companion (parallel PR #736, `docs/design/ECONOMY_EMERGENCE.md`):** spec for the physics-field → settlement-stocks → CDA chain that supplies the gradient this spec consumes. This document treats that substrate as a given read-only input; it does **not** re-derive it.
> **Traceability:** FR-CIV-MARKET-001..008 (carry-overs from `polities-markets.md` §2 — fully consumed), FR-CIV-ECON-001..005 (conservation / ledger / allocator / taxes / subsistence-first — substrate, not modified), FR-CIV-ECON-trade (trade-route substrate — read-only), FR-CIV-EMERGENCE-001 (micro-driver → macro-pattern mapping), FR-CIV-EMERGENCE-002 (shared-gradient coupling — no API edges between layers).

---

## 0. Charter constraint + scope

The Civis Emergence Charter forbids hardcoded life / society / psyche / polity / **market** (`emergence-charter.md` §"Layer 1+ — What EMERGES"). Three anti-patterns are explicitly forbidden by this spec:

1. **No hardcoded price tables.** The current `MarketState::default()` (`crates/economy/src/market.rs:32-39`) seeds `food = 1_000`, `energy = 1_000` cents. That is an authored price — the charter rules it out. The first-ever price of any good in a fresh scenario must come from a real `ClearedTrade`.
2. **No hardcoded currency.** There is no `enum Currency`, no `currency_id: u32`, no `name: "gold"`, no `symbol: "₲"`. Money is the *role* a good takes when the world starts pricing things against it.
3. **No hardcoded numeraire.** No `enum Numeraire { Gold, Silver, Salt, ... }`, no `primary_money: Good = Good::Metal`. The good that *is* money in a locale is a measured, sliding label.

This spec is **additive** to the existing substrate. It defines **pure read-only projector functions** over the live `Allocator` order book, the `ClearedTrade` log, the `Settlement` / `TradeRoute` snapshots, and the `InstitutionLedger` posting log. It does **not** add new fields to `EconomyState`, new types to `MarketState`, or new API edges from `civ-economy` to `civ-agents` / `civ-diplomacy` / `civ-build`. The P6-A §6 coupling audit guarantees that no production code path other than `MarketState::recompute` (already defined in the companion `civ-economy-emergent-markets.md` §2) mutates the price field.

**Scope of this spec:** emergent price discovery from local supply/demand (per-settlement + cross-settlement), emergent numeraire selection, emergent cross-region exchange rate. Banking / credit / fractional reserve is **explicitly out of scope** here and lives in §8 "Banking/credit — deferred roadmap" (the `polities-markets.md` §FR-CIV-MARKET-008 + `civ-economy-emergent-markets.md` §3.4 credit posture is the seed; full banking is a separate design).

**Out of scope for this document:**
- Re-implementing `Allocator::clear` (the matching math is the substrate — we read its outputs only).
- Re-implementing `compute_trade_routes` (the gravity kernel is the substrate — we read its outputs only).
- Re-implementing `InstitutionLedger::post` (the conservation law is the substrate — we read its outputs only).
- The `MarketState::step` random-walk path (already superseded by the companion spec's `MarketState::recompute` projector; this spec inherits that decision unchanged).
- Authored scripts for "what money is," "what markets look like," or "how prices behave." All of those are read-out projections of substrate.

---

## 1. Core emergence model

### 1.1 The three pillars, summarised

| Pillar | What emerges | Substrate that produces it | Output is a measurement of |
|---|---|---|---|
| **Price** | A clearing price in cents for each `(settlement, good)` | `Allocator::clear` → `Vec<ClearedTrade>` | Real willingness-to-pay cross-pressure at the order book |
| **Money** | A *role* taken by whichever good has the highest trade frequency × acceptability in a locale | `Allocator::trades_log` (already in companion spec §6 P1-A) | The pattern of counterparty-acceptance over a sliding window |
| **Currency** | A region-specific *re-denomination* of all prices against the local money | emergent clearing-price field + numeraire | The market's act of pricing — the act *makes* the money |

There is no fourth pillar. **Money = pricing-act**, not a token. There is no fifth pillar either: banking/credit is a *use* of the institution ledger + numeraire that emerges later (§8).

### 1.2 What the substrate already provides (read-only inputs, in scope on `main`)

| Continuous driver | Source (on `main`) | What it does | Read by |
|---|---|---|---|
| Per-good clearing price | `ClearedTrade::price` returned by `Allocator::clear` (`crates/economy/src/allocator.rs:174-385`) | Mid-point of crossing pair (or rationing midpoint); the **only** place a per-unit price is set | `emergent_clearing_price` projector (§2.1) |
| Bid / Offer book | `Allocator::bids: BTreeMap<OrderId, Bid>` + `Allocator::offers` (`allocator.rs:99-107`) | Live willingness-to-pay / willingness-to-accept | Scarcity drift projector (§2.2), book-depth metric (§4) |
| Cleared trade log | `Vec<ClearedTrade>` returned by `Allocator::clear` (`allocator.rs:174-385`) | Append-only history of every exchange; **the** substrate for numeraire selection | `select_numeraire` projector (§3.1) |
| Per-settlement surplus/deficit | `Settlement::surplus(g)` + `Settlement::deficit(g)` (`crates/economy/src/trade_routes.rs:100-108`) driven by `Stocks` + `ProductionProfile` (`crates/economy/src/stocks.rs:210-220`) | The supply-side pressure that pushes the bid/offer book | All three pillars |
| Inter-settlement routes | `compute_trade_routes(settlements, GOODS, min_flow)` (`trade_routes.rs:185-211`) | The emergent long-range trade volume per `(origin, dest, good)` from the gravity kernel `surplus * deficit / dist²` | Cross-region exchange rate (§3.4) |
| Comparative advantage | `comparative_advantage(profile)` (`stocks.rs:223-234`) | The good each settlement produces most — biases the offer side | Specialization read-out (§2.3) |
| Institution posting log | `InstitutionLedger::postings` (`crates/economy/src/institution.rs:84`) | The conservation layer; every priced transfer posts a balanced pair | Conservation invariant (§4 + §5) |
| Allocation regime | `AllocationRegime` enum + `allocate_with` (`crates/economy/src/allocation.rs:114-137`) | Capitalist / Planned / Joule selection per locale; the regime selector is in `allocation.rs:111-120` | Market-type read-out (carried over from `polities-markets.md` §2) |

The **only** substrate-level data structure that needs to be added (already specified in `civ-economy-emergent-markets.md` §6 P1-A) is `Allocator::trades_log: Vec<ClearedTrade>` — the persistent append-only history of cleared trades. Without that log the numeraire projector has nothing to read.

### 1.3 What is *not* in the substrate (and must stay out)

| Anti-pattern | Why it would violate the charter | What this spec does instead |
|---|---|---|
| `enum Money { Gold, Silver, Copper }` | Authored token list | `Money` is a *role label*, never a stored variant |
| `currency_id: u32` on `InstitutionAccount` | Authored currency identity | The institution's *price field* is the emergent clearing price at settlement time; the institution never names a currency |
| `MarketType::Commodity { numeraire: Good::Metal }` | Hardcoded association | Market type is a derived weight vector over `(gift, barter, commodity, mercantile, credit, planned)` (carried over from `polities-markets.md` §2.2); the *implied* numeraire is read separately from the trade log |
| Hardcoded `DEFAULT_PRICE_CENTS = 1_000` | First-ever price is fictional | Companion spec §2 replaces the `Default for MarketState` with an *empty* struct; this spec inherits that decision |
| Random-walk `MarketState::step` | Scripted price drift unrelated to trade | Companion spec §2 deletes `MarketState::step`; this spec inherits that decision |
| `fn set_price(g, p)` callable | Bypasses real exchange | There is no such callable; the only path that writes a price is `Allocator::clear` returning a `ClearedTrade` |

---

## 2. Pillar 1 — Emergent price discovery from local supply/demand

### 2.1 The crossing-clearing projector (per-settlement, per-good)

```text
fn emergent_clearing_price(
    trades: &[ClearedTrade],
    settlement_tag: SettlementTag,
    good: GoodId,
    lookback_ticks: u64,
    current_tick: u64,
) -> Option<i64>
```

| Property | Definition | Why this is *emergent*, not authored |
|---|---|---|
| Input | Append-only `ClearedTrade` log tagged with a settlement tag (carried on each `ClearedTrade` via a new `settlement: SettlementId` field — see §6 P1-A of the companion spec; this spec assumes it) | The log is the *only* source of truth; if no trade happened, there is no price |
| Output | `Some(price)` for the most recent `ClearedTrade` whose `settlement == settlement_tag`, `good == good`, `tick > current_tick - lookback_ticks`, and `rationed == false`; `None` otherwise | Crossing-pair mid-point from `allocator.rs:253`; the only price ever set by a real exchange |
| Determinism | Two calls with identical inputs always return the same `Option<i64>` (BTreeMap-iteration order on `trades_log` is the canonical sort) | Replay-stable; deterministic by construction |
| Floor | Clamped at `MIN_PRICE_CENTS = 1` (`market.rs:23`) | Preserves the existing positivity invariant; the substrate floor is reused, not bypassed |
| Purity | `&[ClearedTrade]` only; no mutation, no IO, no `thread_rng` | Safe to call from any read-out surface; safe under the charter "determinism is NOT a requirement" since the function itself is deterministic-by-construction regardless of world stochasticity |

**The crossing clearing price is the headline read-out.** It is what overlays show, what the inspector reports, and what the diplomacy `apply_signal(trade_volume)` channel consumes. It is **not** stored on `MarketState`; it is recomputed on read from the trade log (companion spec §1.2 already specifies the migration).

### 2.2 The scarcity-drift projector (supply-side pressure, no trade required)

There are goods that have not traded yet but whose stocks exist. The substrate `MarketState::apply_pressure` (`market.rs:72-97`) already implements a `(supply, demand) → price delta` kernel; the companion spec §1.2 keeps it as a legitimate *secondary* read-out for non-traded goods. This spec uses the same kernel via a thin wrapper:

```text
fn scarcity_drift_price(
    local_stocks: &Stocks,
    profile: &ProductionProfile,
    good: Good,
    anchor: Option<i64>,           // most recent emergent_clearing_price; None if no trade yet
    params: &EmergentMarketParams, // see §4 knobs
) -> i64
```

| Property | Definition |
|---|---|
| Anchor | The `anchor` is whatever `emergent_clearing_price(good, ...)` returned last; if `None`, the function returns `None` too (no trade yet → no drift anchor → "no price yet" — the UI shows "no trade yet" instead of fabricating a price) |
| Direction | Pressure `p = clamp((demand - supply) / max(supply, 1), -9, 9)` (signed fixed-point int); delta `= p * MAX_PRESSURE_DELTA_CENTS / 10` |
| Magnitude | Clamped at `MAX_PRESSURE_DELTA_CENTS = 100` (`market.rs:21`); `emergent_overshoot_cap` knob (§4) further clamps the per-tick fractional change |
| Floor | `max(MIN_PRICE_CENTS, anchor + delta) = 1` minimum |
| Purity | Pure; same inputs → same output |

The scarcity-drift path is a *fallback* for the read-out, not an alternative price-discovery path. The clearing price is always preferred when available; scarcity drift only fills in the *gap* between trades. This is the same posture as `civ-economy-emergent-markets.md` §1.2.

### 2.3 What produces each macro price phenomenon

**Per-good clearing price.** A crossing pair in `Allocator::clear` (`allocator.rs:225-288`) sets `clearing_price = (bid_price + offer_price) / 2`. The crossing-price series for good `g` is the *measurement* of how much one side will pay relative to the other at the moment of exchange. There is no `update_price()` callable that bypasses a real trade (charter test).

**Hyperinflation / deflation.** Sustained unilateral pressure on the book — bids rising while asks stay put, or vice versa — pushes the crossing mid-point each tick. The price is the integrated history of imbalance, not a stochastic delta. Convergence comes from a deeper cause: actors learn (via psyche temperament + utility AI, not via this spec) that posting too high a bid / too low an ask gets cleared instantly, while posting too low / too high sits on the book — so the *distribution* of posted prices self-organises around the true scarcity.

**Specialisation premium.** A settlement whose `comparative_advantage(profile) == Good::Metal` (`stocks.rs:223-234`) posts a metal offer; that offer quantity tracks the settlement's metal stock; the crossing price of metal in that settlement is the *measurement* of how strongly the region is differentiated. Specialisation is *visible* on the order book, not declared.

**Trade-route price gradient.** A `TradeRoute` (`trade_routes.rs:117-128`) with `flow > 0` for `(origin, destination, good)` is a *price gradient* between two settlements for that good. The arbitrage projectors in §3.4 consume it to discover cross-region exchange opportunities.

**Scarcity without trade.** A settlement with `Stocks::get(g) == 0` and `profile.net_flow(g) < 0` is in deficit. The clearing price from `Allocator::clear` is `None` (no one has bid/offered yet); the scarcity-drift projector returns `None` too. The UI shows "no trade yet" — the same path that `propose_trade` (`stocks.rs:252-308`) follows when no counterparty exists. **No fictional price is ever shown.**

### 2.4 The chart of read-out layers (preserved from companion spec §1.2)

| Read-out field | Definition | Why derived, not stored |
|---|---|---|
| **Crossing clearing price** | `emergent_clearing_price` — most recent `ClearedTrade::price` for `good` whose `!rationed` | The only price ever set in a real exchange. No fictional price exists. |
| **Rationing clearing price** | Most recent `ClearedTrade::price` for `good` with `rationed == true` (mid-point of best unmatched bid/ask, `allocator.rs:389-411`) | The "scarcity" price signal; converges toward the crossing price as rationing becomes rarer |
| **Scarcity drift** | `scarcity_drift_price` — `apply_pressure` kernel with the *anchor* = last crossing price (or `None`) | Supply-side pull; never overrides the crossing price |
| **Windowed VWAP** | Volume-weighted average of `ClearedTrade::price * ClearedTrade::quantity` over `vwap_lookback` per good | Smooths noise without storing a price book |
| **Settlement-tagged price** | `emergent_clearing_price` filtered by `settlement == settlement_tag` (this spec's contribution) | Per-settlement read-out, not a locale-aggregate; matches the substrate per-settlement list |

---

## 3. Pillar 2 — Money as the emergent most-tradeable good

### 3.1 The numeraire projector (sliding-window argmax over the trade log)

```text
fn select_numeraire(
    trades_log: &[ClearedTrade],
    lookback_ticks: u64,           // e.g. 4096
    current_tick: u64,
    params: &EmergentMarketParams, // provides numeraire_min_trade_count
) -> Option<GoodId>
```

| Component | Definition | Source |
|---|---|---|
| `trade_count(g, lookback)` | Number of `ClearedTrade` entries for good `g` in `trades_log` whose `tick > current_tick - lookback_ticks` AND whose `settlement_tag` matches the queried locale (per-locale numeraire) | `Allocator::trades_log` (companion spec §6 P1-A) |
| `cross_good_acceptance(g, lookback)` | Number of distinct *other* goods `g'` such that at least one `ClearedTrade` exists in the window where `cleared_trade.good == g` AND there is a separate `ClearedTrade` with `cleared_trade.good == g'` between overlapping institution pairs (counterparty graph) | Same |
| `liquidity_score(g)` | `trade_count(g) * sqrt(cross_good_acceptance(g))` | Derived (integer fixed-point sqrt) |
| `numeraire(g*)` | `argmax_g liquidity_score(g)` over goods with `trade_count(g) >= numeraire_min_trade_count`; ties broken by FIFO `ClearedTrade::tick` (oldest-wins) | Derived |
| Output | `Some(g*)` if any good clears the eligibility floor, else `None` | The numeraire can *be* `None` (no good yet meets the threshold) |

**Two properties follow from the construction:**

1. **Per-settlement numeraire.** The `settlement_tag` filter on `trades_log` means each settlement can have its own numeraire. Two settlements with disjoint trade logs can crown two different goods as money.
2. **Sliding, not fixed.** The numeraire can shift tick-to-tick. A shift is a measurable event (§3.5 `numeraire_shift.v1` replay-bus event).

### 3.2 Money is a *role*, not a token

When the numeraire in settlement `S` is `g*`, all *other* goods' emergent prices can be re-denominated by dividing by the numeraire's emergent clearing price (or by VWAP of `g*`) in `S`:

```text
fn redenominated_price(
    good: GoodId,
    settlement: SettlementId,
    clearing_price_field: &[ClearedTrade],
    numeraire_g: GoodId,
    numeraire_clearing_price: i64,
) -> Option<f64>
```

The denominator `numeraire_clearing_price` is itself an `emergent_clearing_price(numeraire_g, ...)` — there is no stored "price of money." The re-denomination is computed on read, never written to a stored price field. **The act of pricing is what makes `g*` function as money.** This is the same posture as `civ-economy-emergent-markets.md` §3.2 and `polities-markets.md` §2.4.

Two corollaries fall out:
- **No `currency_id` field exists** anywhere on any agent, institution, or economy state. Money is the act of pricing, not a token.
- **Cross-region exchange rates** are an immediate consequence: when settlement `A`'s numeraire is `g_A` and settlement `B`'s is `g_B`, the cross-rate is `emergent_clearing_price(g_A in A) / emergent_clearing_price(g_B in B)`. The rates are derived; arbitrage flows (§3.4) cause the two prices to converge when transport is cheap and diverge when it is expensive.

### 3.3 What the numeraire projector does NOT do

Carried over from `civ-economy-emergent-markets.md` §3.3, restated for completeness:

- It does NOT privilege any good in code. `Good::Metal` and `Good::Tools` are *likely* numeraires by durability + acceptability, but a famine region with no tool production and a salt-flat (if `Good::Salt` were ever added) can have a different numeraire. Acceptance is measured, not declared.
- It does NOT create a `currency_id` field. There is no token; money is the act of pricing.
- It does NOT lock in. The numeraire can shift. The shift is itself a measurable event (§3.5) consumed by legends engine for narrative.
- It does NOT require settlement-level coordination. The per-settlement numeraire is computed purely from the local `trades_log` slice; no synchronisation with other settlements is needed.

### 3.4 Cross-region exchange rate (derived, never authored)

```text
fn cross_region_rate(
    local: &MarketState,        // from MarketState::recompute
    remote: &MarketState,
    transport_cost_cents: i64,  // from the TradeRoute::flow gravity kernel
) -> Option<(Side, i64)>
```

| Output | When |
|---|---|
| `Some((Buy, remote_price - local_price - transport))` | remote price exceeds local price by more than transport cost → buy remote, sell local |
| `Some((Sell, local_price - remote_price - transport))` | symmetric |
| `None` | the gap is below transport cost → no arbitrage flow |

**The arbitrage opportunity is itself a read-out**, not an actor. The actor that *takes* the opportunity is the existing `propose_trade` (`stocks.rs:252-308`) path or the `Allocator::post_bid` call — they read the read-out and respond with a real bid. This is the **shared-gradient coupling** pattern from `civ-003-emergent-lifecycle.md` §2: arbitrage is the visible signature of price divergence, not a `arbitrage_actor.tick()` callback.

When `cross_region_rate(...) == None` for all goods between two settlements (because no `TradeRoute` connects them, or because transport cost exceeds every gap), the two settlements develop independent numeraires permanently — also emergent. This is the *isolation* regime the charter invites.

### 3.5 Numeraire-shift event (replay-bus + dashboard surface)

```text
struct NumeraireShiftV1 {
    schema: "numeraire_shift.v1",
    tick: u64,
    settlement: SettlementId,
    old: Option<GoodId>,
    new: Option<GoodId>,
    liquidity_score_new: u64,
}
```

| Property | Definition |
|---|---|
| Trigger | `select_numeraire(settlement, ...)` returns a `GoodId` different from the previous tick's return for the same settlement |
| Emitted from | `crates/engine/src/engine.rs` economy phase (after `MarketState::recompute`) — wired by the companion spec §6 P3-D |
| Consumed by | `civ-watch` event feed (web inspector), legends engine (narrative "the money changed"), 3D client overlays (currency ticker) |
| Replay | The event is *derived* from the trade log; replaying a snapshot produces the same sequence deterministically |
| Purity | The decision to emit is a pure function of `select_numeraire` outputs across consecutive ticks |

---

## 4. Criticality knobs (edge-of-chaos tuning)

All knobs concentrate in a single `EmergentMarketParams` struct (already declared in the companion spec §4 — this spec inherits that struct unchanged). The numbers below are the *defaults*; all knobs are tunable in scenario RON. **Defaults target weak emergence (Class 4)**: the system is in a price-formation band, not collapsing to barter-rationing and not exploding to hyper-deflation.

| Parameter | Type | Default | Effect | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `tatonnement_lambda` | `i64` (1..1000) | `50` | Step size of the per-tick scarcity-drift adjustment on non-cleared goods | `> 500` (overshoot) | `< 5` (barter deadlock) |
| `tatonnement_scale` | `i64` | `1_000_000` | Denominator that keeps scarcity drift integer | depends on `lambda` | depends on `lambda` |
| `min_price_cents` | `i64` | `1` | Floor on every emergent clearing price (preserves `market.rs:23`) | `0` | `> 100` (friction kills liquidity) |
| `rationing_lookback` | `u64` | `256` | Past-tick window for rationing frequency | `> 4096` (alarm too slow) | `< 16` (noisy) |
| `vwap_lookback` | `u64` | `1024` | Sliding window for the VWAP read-out | `> 16384` (stale prices) | `< 64` (jitter) |
| `numeraire_lookback` | `u64` | `4096` | Sliding window for `select_numeraire` | `> 32768` (numeraire inertia) | `< 256` (numeraire flickers) |
| `numeraire_min_trade_count` | `u64` | `8` | Minimum trade count for numeraire eligibility | `> 64` (bias toward durability) | `1` (single trade can crown a numeraire) |
| `arbitrage_window` | `u64` | `512` | Sliding window for cross-region arbitrage convergence metric | `> 8192` (stale) | `< 64` (noisy) |
| `emergent_overshoot_cap` | `f32` | `0.20` | Maximum fractional price change per tick in scarcity-drift path | `0.0` | `1.0` |
| `clearing_persistence_decay` | `f32` | `0.95` | EWMA weight when the crossing-price series is empty | `1.0` | `0.5` |

Two **new** knobs this spec proposes (the others are inherited unchanged from the companion spec §4):

| Parameter | Type | Default | Effect | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `cross_region_dispersion_threshold` | `f32` | `0.80` | Threshold for the `MT-MKT-003` alarm (cross-region price dispersion > this for adjacent regions with road access → arbitrage dead) | `> 1.0` (alarm never fires) | `< 0.30` (alarm too sensitive, blocks trade) |
| `min_settlement_separation_for_isolated_numeraire` | `u64` (ticks) | `4096` | Minimum time without a `TradeRoute` between two settlements before the *isolation* regime (independent numeraires) is allowed to stabilise | `> 32768` (settlements never stabilise) | `< 256` (numeraires co-mingle too quickly) |

**Rule:** every knob must be *real* — setting it to its heat-death and explosion extremes must reach the corresponding failure mode within 512 ticks (verified by scenario test §6 P5-F in the companion spec). Constants are not knobs.

---

## 5. Observable emergence metrics for the dashboard

These metrics feed the **Emergence Dashboard** (`crates/engine/src/emergence.rs`) and the `emergence.metrics.v1` replay-bus event. They are aggregates computed cheaply from existing `Allocator` + `ClearedTrade` + `Stocks` + `Settlement` views — no new agent-level state.

| Metric | How to compute | Healthy target | Failure mode |
|---|---|---|---|
| **Market-good entropy (per settlement)** | Shannon entropy of `ClearedTrade::good` over `vwap_lookback`, filtered by `settlement_tag`; normalised by `log2 GOODS.len() = log2 6` | High (0.6..0.9 normalised) — many goods actively traded | Low = monopoly/clique in that settlement |
| **Crossing ratio (per settlement)** | `count(cleared_trade.rationed == false && settlement_tag == S) / count(cleared_trade.all with that tag)` over `rationing_lookback` | High (≥ 0.85) — book self-organises | Low = price discovery is broken; agents post blind prices |
| **Rationing frequency (per settlement)** | Same denominator; rationing portion | Low (0.0..0.15) | Sustained > 0.5 = chronic scarcity |
| **Book liquidity (depth, per settlement per good)** | `Σ Bid::quantity` + `Σ Offer::quantity` over live orders for `(settlement_tag, good)` | High and roughly balanced bid↔ask | One side empty = single-sided market |
| **Clearing-price variance (per good)** | Variance of `ClearedTrade::price` per good over `vwap_lookback` | Moderate; oscillates with shocks then settles | Persistent near-zero = barter deadlock |
| **Numeraire persistence (per settlement)** | Consecutive ticks the same `GoodId` is the argmax in `select_numeraire`; reset on shift | Long runs (hundreds of ticks) | Frequent shifts (< 50 ticks) = currency chaos |
| **Numeraire share of trade volume (per settlement)** | `sum(qty for cleared_trade.good == numeraire && settlement_tag == S) / sum(all qty with that tag)` over `numeraire_lookback` | Moderate (0.05..0.30) | Zero = misnamed; > 0.5 = hoarding |
| **Cross-region price dispersion (per pair)** | For each good `g` in two settlements A,B: `|vwap_A(g) - vwap_B(g)| / max(vwap_A(g), vwap_B(g))`, averaged over goods in the `TradeRoute` set | Small (≤ 0.2) when transport is cheap | Persistent near-1.0 in adjacent regions with road access = arbitrage failure (raises `MT-MKT-003`) |
| **Numeraire exchange-rate volatility** | Variance of `cross_region_rate(...)` over `arbitrage_window` | Small | Large and growing = speculation or transport breakdown |
| **Numeraire isolation index (per settlement)** | Number of other settlements whose `select_numeraire` returns the same `GoodId` as `S`'s numeraire, normalised by total settlements within `TradeRoute` reach | Moderate — most settlements share a numeraire via trade, some don't | All-or-nothing = either fully fragmented or fully homogenised (both are failure modes) |
| **Trade-route flow (per route)** | `TradeRoute::flow` straight from `compute_trade_routes` | Variable | Zero everywhere = no commerce; identical everywhere = no comparative advantage |
| **Conservation invariant (per tick)** | `InstitutionLedger::verify_conservation` + `verify_ledger_conservation` per tick | Always Ok | Any failure = critical bug; halt and alarm |
| **Institution balance non-negativity (per account)** | All `InstitutionAccount::balance_joules >= 0` | Always true | Negative = a posting violated the floor; halt and alarm |

The `emergence.metrics.v1` replay-bus event (per `emergence-dashboard.md` §5) is extended with these new fields:
- `market_good_entropy_per_settlement: BTreeMap<SettlementId, f32>`
- `cross_region_dispersion_per_pair: BTreeMap<(SettlementId, SettlementId), f32>`
- `numeraire_persistence_per_settlement: BTreeMap<SettlementId, u64>`
- `numeraire_isolation_index_per_settlement: BTreeMap<SettlementId, f32>`
- `mt_mkt_001_threshold_breach: bool` (chronic rationing > 0.5 for 256 ticks in any settlement)
- `mt_mkt_002_threshold_breach: bool` (numeraire shift > 4 times in 4096 ticks in any settlement — currency chaos)
- `mt_mkt_003_threshold_breach: bool` (cross-region dispersion > `cross_region_dispersion_threshold` in adjacent regions with road access — arbitrage dead)

The `emergence.alarm.v1` event is extended with the same three alarm IDs as the companion spec §5.

---

## 6. Bidirectional coupling — the shared-gradient mechanism

The charter explicitly forbids one emergent layer calling another through an API boundary with no lag. The mechanism here is the same as `civ-003-emergent-lifecycle.md` §2: **shared conserved gradients with explicit lags**. Three gradients carry the coupling:

1. **`Stocks`** (integer stock vector per good) — the supply-side pressure
2. **`Allocator` order book** (per-tick live bids/offers) — the demand-side pressure
3. **`InstitutionLedger` posting log** (append-only double-entry) — the conservation audit

Nothing in this spec introduces a new edge between crates. The P6-A §6 coupling audit guarantees no production code path mutates `MarketState` except by calling `MarketState::recompute` (companion spec §6 P6-D).

### 6.1 Market → Lifecycle / Diplomacy / Agents (downward causation, no API call)

| Market signal (read by what) | Effect on the layer | Mechanism (shared gradient, not call) |
|---|---|---|
| `ClearedTrade::price` rises for `Good::Food` (read by agents / needs) | The *cost* of acquiring food via trade rises; agents whose profile can't produce food switch consumption strategies | The next `Allocator::post_bid` call from a food-short agent posts a *higher* `Bid::price` (driven by psyche `drives[security]` + needs criticality, not by a market→agent API). The trade clears, the price is *read* by the agent's next decision step. |
| `ClearedTrade::price` falls for `Good::Tools` (read by agents / cluster) | Tool-bearing agents' relative bargaining power rises; they post more offers of food-for-tools (long-range mercantile pattern) | The substrate signal is the *price fall* observable on the order book. No `market.on_price_change(...)` callback exists. |
| Numeraire shifts from `Good::Food` to `Good::Metal` (read by diplomacy / polity) | Trade relationships measured in "metal-equivalent" start to make sense; old food-debt postings still exist as historical `ClearedTrade` records but are no longer the natural unit | The numeraire read-out is consumed by the diplomacy `apply_signal` (the existing `trade_volume` channel) and the polity cohesion graph weight `w_econ · payoff_if_coordinated` (`polities-markets.md` §1.2). Both consume the *measured* numeraire; neither mutates it. |
| Rationing trades fire (`ClearedTrade::rationed == true`, `allocator.rs:357`) | Agents downstream read "supply is short for `g`" | The rationing flag is observable on the trade log; the next `Allocator::post_offer` from a long-on-`g` agent *responds* by raising `Offer::price`. Lag = 1 economy tick. |
| Cross-region rate reveals an arbitrage opportunity (read by agents / settlements) | The settlement with the lower price posts a long-range bid via the existing `propose_trade` (`stocks.rs:252-308`) path; the prices converge | The actor is the existing trade-proposer; the read-out is `cross_region_rate`. No new actor is introduced. |

### 6.2 Lifecycle / Diplomacy / Agents → Market (upward causation with lag)

| Layer signal | Market effect | Lag mechanism |
|---|---|---|
| `Stocks::get(Good::Food)` falls toward zero on a settlement | That settlement's next `Allocator::post_bid` posts a *higher* `Bid::price` for food (driven by need decay + `scarcity_drift_price`) | 1 tick lag: stock change → needs tick → utility decision → next bid post |
| `comparative_advantage(profile) == Good::Metal` on settlement X | X's next `Allocator::post_offer` posts a metal offer; offer quantity tracks X's metal stock | 1 tick lag |
| High `DiplomacyMatrix.relation(actor, counterparty)` (alliance) | Counterparty is *more likely* to be on the same book; cross-cluster trade volume rises | Indirect: kinship/culture drift → cluster overlap → spatial co-location → order book co-presence. No direct market↔diplomacy edge. |
| Polity coercion overlap (FR-CIV-POLITY-008) | `AllocationRegime` for that locale flips to `Planned` (`allocation.rs:111-120`); planned override is `polities-markets.md` §FR-CIV-MARKET-006 | 1 tick lag |
| Mortality / immigration shock (lifecycle FR-CIV-LIFE-003) | Number of active bidders / offerers in the locale drops; book thins; price volatility rises | Structural lag = 1 generation (~20 in-game years for a human cohort) |
| Trade-route `flow` shifts between two settlements (driven by Stocks changes) | The cross-region rate projector reflects the new gradient | 1 tick lag |

### 6.3 Arbitrage as a shared-gradient phenomenon, not a call

Arbitrage is the *visible* signature that two settlements' price fields have diverged: a settlement in A with surplus food sees B's food price (the crossing clearing price on B's book) is higher than A's, so it posts a long-range bid on B's book via the existing `propose_trade` (`stocks.rs:252-308`) path and `Allocator::post_bid`. The two prices converge through these *actual bids*, not through an `arbitrage_actor.tick()` callback. The emergent cross-rate settles to the transport-cost band; if no `TradeRoute` connects the two settlements, the rates diverge permanently and the two regions develop independent numeraires — also emergent.

### 6.4 Coupling to the institution ledger (shared conservation)

Every priced transfer posts a balanced `InstitutionPosting` via `InstitutionLedger::post` (`institution.rs:219-246`). A credit posting is a balanced pair that sits in the posting log (`institution.rs:84`); a polity treasury posts a funding transfer to `INSTITUTION_MARKET` that biases the book on that institution's behalf. The price field on the credit settlement reads off the *clearing price at the time the credit is posted* (recorded in the `ClearedTrade` and copied into the institution posting as the unit-of-account anchor). **The settlement price is never *re-priced* later** — late settlement is a separate posting, not a price update.

---

## 7. Phased implementation plan

This is a DAG-structured WBS. No code is written here; file paths, struct names, and function signatures are identified for the implementing agent. Every phase extends *existing* substrate — no new crate, no API edge to lifecycle / diplomacy.

### Phase 0 — Prerequisite audit (no new structs)

| Task | File | Notes |
|---|---|---|
| P0-A | `crates/economy/src/allocator.rs` | Verify `Allocator::clear` returns `Vec<ClearedTrade>` with `price`, `rationed`, `quantity`, `tick` populated. **Already done in the companion spec's P0-A.** |
| P0-B | `crates/economy/src/trade_routes.rs` | Verify `TradeRoute::flow`, `Settlement::surplus`, `Settlement::deficit` are integer-only and clamp at zero. They do. |
| P0-C | `crates/economy/src/market.rs` | Map which `MarketState::prices` consumers exist; flag the hardcoded `food = 1_000`, `energy = 1_000` defaults for deletion in companion spec Phase 2. |
| P0-D | `crates/economy/src/institution.rs` | Confirm `InstitutionLedger::verify_conservation` is the conservation gateway — already is (`institution.rs:319-378`). |
| P0-E | `crates/economy/src/allocator.rs` | Confirm `Allocator::trades_log` does NOT yet exist (companion spec §6 P1-A adds it; this spec inherits). |
| P0-F | `crates/economy/src/trade_routes.rs` | Confirm `TradeRoute` carries no `settlement_tag` yet — the tag is added to `ClearedTrade` in companion spec §6 P1-A as a new field; this spec reads it. |

### Phase 1 — Pure projector functions (read-only, no mutations)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P1-A | `crates/economy/src/markets_currency.rs` (NEW module) | P0-A, P0-E | Add `EmergentClearingPrice`, `ScarcityDriftPrice`, `SelectNumeraire`, `CrossRegionRate`, `RedenominatedPrice` projector functions. All pure. |
| P1-B | same | P1-A | Tests: `EmergentClearingPrice::None` on empty log; most recent non-rationed price when present; `SelectNumeraire::None` when no good clears `numeraire_min_trade_count`; argmax correctness across a known set; `ScarcityDriftPrice::None` when anchor is `None`; monotonic in deficit when anchor > 0; `CrossRegionRate::None` when transport exceeds gap. Property-based via `proptest`. |
| P1-C | `crates/economy/src/lib.rs` | P1-A | Re-export: `pub use markets_currency::{EmergentClearingPrice, ScarcityDriftPrice, SelectNumeraire, CrossRegionRate, RedenominatedPrice};` alongside the existing exports at `lib.rs:20-33`. |

### Phase 2 — Per-settlement wiring (no new types on `MarketState`)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P2-A | `crates/economy/src/markets_currency.rs` | P1-A | Add `pub fn per_settlement_view(trades_log: &[ClearedTrade], settlement: SettlementId, lookback: u64, params: &EmergentMarketParams) -> SettlementMarketView` returning a snapshot struct `{clearing: BTreeMap<GoodId, i64>, vwap: BTreeMap<GoodId, i64>, numeraire: Option<GoodId>, numeraire_persistence_ticks: u64, isolation_index: f32}`. The struct is the **per-settlement** analogue of the global `MarketState::recompute` from the companion spec §1.2. |
| P2-B | `crates/engine/src/engine.rs` | P2-A | Wire `per_settlement_view` into the economy phase: after `MarketState::recompute` (companion spec §6 P2-D), call `per_settlement_view` for every `SettlementId` referenced in the active `TradeRoute` set. Push results into the existing `emergence.metrics.v1` event payload. |
| P2-C | `crates/engine/src/emergence.rs` | P2-B | Extend `compute_market_metrics` (companion spec §6 P4-B) with the per-settlement fields listed in §5. Same shape — pure aggregator over the live state. |
| P2-D | Tests | P2-A, P2-B | Per-settlement view purity (same inputs → same output); per-settlement view respects `numeraire_lookback` window; per-settlement view never returns a negative price; snapshot round-trips through `bincode` for the *new* fields. |

### Phase 3 — Cross-region rate + numeraire-shift event (replay-bus surfaces)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P3-A | `crates/engine/src/engine.rs` | P2-B | Wire `CrossRegionRate` projector into the economy phase. For each `(SettlementId, SettlementId)` pair with a live `TradeRoute`, call `cross_region_rate` and emit a `cross_region_rate.v1` replay-bus event `{tick, settlement_a, settlement_b, side: Option<Side>, gap_cents: Option<i64>, transport_cost_cents: i64}`. |
| P3-B | `crates/engine/src/engine.rs` | P2-B | Wire `NumeraireShiftV1` event emission (companion spec §6 P3-D already declares this; this spec adds per-settlement shifts on top of the global shift). |
| P3-C | `crates/civ-watch/src/event_feed.rs` | P3-A, P3-B | Extend the watch event feed with the two new event types; serialise per-settlement numeraire shifts + per-pair cross-region rates. |
| P3-D | Tests | P3-A, P3-B | `cross_region_rate.v1` fires only on actual gap (not on every tick); `numeraire_shift.v1` fires only on actual `GoodId` change. Replay determinism: given a fixed `trades_log`, the event sequence is identical. |

### Phase 4 — Criticality knobs + dashboard metrics (companion spec Phase 4 + this spec's two new knobs)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P4-A | `crates/economy/src/markets_currency.rs` | P1-A | Add `cross_region_dispersion_threshold` and `min_settlement_separation_for_isolated_numeraire` knobs to `EmergentMarketParams`. |
| P4-B | `crates/engine/src/emergence.rs` | P3-A | Wire `cross_region_dispersion_threshold` into the `MT-MKT-003` alarm logic. Wire `min_settlement_separation_for_isolated_numeraire` into the `numeraire_isolation_index` calculation (a settlement's numeraire is "isolated" only after the gap exceeds the threshold). |
| P4-C | `crates/server/src/snapshot.rs` (or wherever `sim.snapshot` is built) | P4-B | Add the §5 per-settlement fields to the F3D0 binary frame payload. |
| P4-D | Tests | P4-A, P4-B | Knob reachability: setting `cross_region_dispersion_threshold` to extremes reaches the corresponding alarm regime within 512 ticks. `numeraire_isolation_index` stays bounded for populations of 10, 100, 1000 settlements. |

### Phase 5 — Acceptance criteria reachability (validation)

| Task | File | Depends on | Notes |
|---|---|---|---|
| P5-A | `crates/economy/tests/integration_emergent_market.rs` (NEW) | Phase 1 + Phase 2 | Scenario: start with empty `MarketState` (post companion spec §6 P2-C), let the system run for 200 ticks with 3 settlements posting bids/offers across `TradeRoute`s. Assert: (1) the first-ever `ClearedTrade::price` for any good is *not* `DEFAULT_PRICE_CENTS = 1_000` (no hardcoded default sneaks through), (2) the per-settlement view is populated, (3) at least one settlement has a numeraire. |
| P5-B | same | P5-A | Scenario: construct a setup where the only good with sustained trade frequency + cross-good acceptance in settlement `S1` is `Good::Wood`; assert `select_numeraire(S1, ...) == Some("wood")`. Then shift production so `Good::Metal` becomes the only well-traded good; assert the numeraire shifts. (Charter AC: changing which good is most liquid changes the emergent numeraire with no code change.) |
| P5-C | same | P5-A | Scenario: construct two settlements with no `TradeRoute` connecting them for > `min_settlement_separation_for_isolated_numeraire` ticks; assert both develop independent numeraires. (Charter AC: isolation regime is reachable.) |
| P5-D | same | P5-A, P3 complete | Conservation AC: 1000-tick scenario with mixed regime (capitalist + planned via polity overlap); assert `InstitutionLedger::verify_conservation` + `verify_ledger_conservation` succeed every tick; assert no `ClearedTrade::price` is negative; assert the `prices_remain_positive_after_n_steps` invariant from `market.rs:262-270` still holds via the new emergent path. |
| P5-E | same | P5-A, P4 complete | Cross-region convergence AC: two settlements connected by a `TradeRoute` with `flow > 0`; run for 4096 ticks; assert `cross_region_dispersion` for the same good trends toward the transport-cost band (not infinity, not zero). |
| P5-F | same | P5-A | Criticality knob reachability: for each knob in §4 + companion spec §4, set it to its heat-death and explosion extremes; assert the system reaches the corresponding failure mode within 512 ticks; assert setting it back to default returns to the target band within 2048 ticks. |
| P5-G | same | P5-A | Charter AC: there is NO test that asserts a particular `Good` is the default numeraire. The numeraire must come from data, not code. (A test that *would* fail under a hardcoded default is the test we want; it serves as a regression net against future "make `Metal` money by default" PRs.) |

### Phase 6 — Coupling sanity (cross-layer, no new edges)

| Task | File | Depends on | Notes |
|---|---|---|---|
| P6-A | `crates/agents/` | Phase 2 + Phase 3 | Audit any direct `market_state.prices[g]` reads that depended on the hardcoded default; replace with `MarketState::recompute(...).last_emergent_clearing.get(g)` so the first-ever read in a fresh scenario gets `None` (UI shows "no trade yet"). |
| P6-B | `crates/diplomacy/` | P6-A | Audit any reads of price that fed `trade_volume` or `resource_competition` signals; ensure they now read the emergent series (not the deleted `MarketState::step` path). |
| P6-C | `crates/build/` | P6-A | Audit any reads of `market_state.prices` that fed building-type / tile-set selection; ensure they read the emergent series. If no such read exists, document the absence and close the task. |
| P6-D | `crates/economy/tests/` | Phase 2 complete | CI-level invariant test: assert that NO production code path mutates `MarketState` except by calling `MarketState::recompute` (static check via `#[test]` that walks the crate's public surface). Carried over from companion spec §6 P6-D. |

### DAG summary (critical path)

```
P0-* (all parallel) → P1-A..C → P2-A..D (P2-B depends on P2-A; P2-C/D depend on P2-B)
                  ↘ P3-A..D (depend on P2-B) → P4-A..D (depend on P3-A,B)
                                              ↘ P5-A..G (depend on Phase 4 complete)
                                              ↘ P6-A..D (depend on Phase 2 complete, can run in parallel with P5)
```

**Critical path to acceptance:** P0 → P1 → P2 → P3 → P4 → P5. The P6 audit can run in parallel with P5 and is the last gate before charter compliance is complete.

---

## 8. Banking / credit — deferred roadmap (out of scope for this spec)

**Banking and credit are NOT part of this spec.** They are real emergent phenomena that this spec *enables* (by guaranteeing the numeraire is itself emergent), but the actual banking/crédit architecture is a separate design effort. The pointers below are the substrate seeds; the banking spec lives elsewhere when it lands.

### 8.1 Why banking is out of scope here

A bank is *an institution that issues credit denominated in the local numeraire, holds reserves in that numeraire, and clears inter-institution transfers.* That definition has three parts:

1. **Issue credit denominated in the numeraire** — requires the numeraire to exist (this spec) AND the `InstitutionLedger` to support credit postings (substrate, already there: `InstitutionPosting` with split debit/credit amounts).
2. **Hold reserves in the numeraire** — requires an institution balance field tied to a numeraire-tagged sub-ledger (substrate extension; not in scope here).
3. **Clear inter-institution transfers** — requires a *clearing house* concept that is more than the existing `INSTITUTION_MARKET` venue (substrate extension; not in scope here).

This spec guarantees (1) (the numeraire is real). It does not add (2) or (3). Mixing banking into this spec would scope-creep it into a full credit-market design that already lives in `polities-markets.md` §FR-CIV-MARKET-008.

### 8.2 Substrate seeds already in place (for the future banking spec)

| Seed | Source | What it enables |
|---|---|---|
| `InstitutionPosting` with `debit_amount` + `credit_amount` + `amount` | `institution.rs:46-65` | Double-entry postings; a credit is a balanced pair with a *deferred settlement* annotation (substrate extension needed) |
| `ClearedTrade` with `bidder == offerer` self-trade branch | `allocator.rs:443-453` (`post_balanced_trade`) | The seed pattern for a *promissory note*: a settlement where bidder and offerer are the same institution records an audit-trail posting with zero net effect |
| `select_numeraire` returning `Option<GoodId>` per settlement | This spec §3.1 | Banks can denominate their credit in the emergent numeraire without a hardcoded currency field |
| `cross_region_rate` projector | This spec §3.4 | Inter-bank clearing can use the emergent exchange rate rather than a fixed conversion table |
| `TradeRoute::flow` (gravity kernel) | `trade_routes.rs:141-173` | Inter-regional credit is constrained by the same transport-cost band as commodity flows |
| Conservation invariant `verify_conservation` | `institution.rs:319-378` | Any banking layer that bypasses this fails the charter test |

### 8.3 What the future banking spec should cover (this spec does NOT cover these)

The deferred banking spec must:

1. **Define the credit posting extension.** A `CreditPosting` that adds a `settles_at_tick: Option<u64>` and `numeraire_at_issue: GoodId` to `InstitutionPosting`. The settlement is a separate posting at the future tick; the original posting stays in the log as the audit trail. Default-forget is itself a posting (write-off).
2. **Define reserve accounting.** An institution's `balance_joules` becomes a tuple `(numeraire: GoodId, amount: i64)` per numeraire, with conversion to the institution's *home* numeraire via the `cross_region_rate` projector.
3. **Define fractional reserve.** A `reserve_ratio_bp: u32` knob per institution, analogous to the existing `Taxation::rates_bp` (`institution.rs:93-101`). Above 10_000 bp = full reserve (no lending); below 1_000 bp = hyper-fragile. The substrate lets the bank *issue* more credit than its reserves; conservation holds because each issued credit is a balanced posting (creditor asset ↔ debtor liability), but the *risk* emerges from the order-book of issued credit.
4. **Define bank-run dynamics.** When the order-book of credit-to-be-settled grows faster than the reserve posting flow, the bank becomes fragile. The fragility is observable on the existing `emergence.metrics.v1` event as a new `bank_run_risk: f32` field.
5. **No `Bank` enum.** Banks emerge as *institutions whose credit-issuance ratio is non-zero*. The set of "things that look like banks" is a measured label, not a stored type. (Same charter posture as polity, market, money.)
6. **No `interest_rate` field.** The rate emerges from the local supply/demand of *credit itself* — the willingness-to-lend vs the willingness-to-borrow on the credit market. This is a *second-order* order book over `ClearedTrade`-shaped futures, not a hardcoded `rate_bp: u32`.

The deferred banking spec is the natural follow-on to this one. It must wait for this spec to land (so the numeraire it relies on is real), and it must wait for the companion `civ-economy-emergent-markets.md` spec to land (so the price field is real).

### 8.4 Why this deferral is correct under the charter

The charter forbids hardcoded money / market / polity / life / etc. Banking is a layer on top of money — it has no substrate until money exists. Forcing banking into this spec before the numeraire is implemented would either (a) require a placeholder currency (charter violation) or (b) require abstract "credit denominated in nothing" (vacuous). The deferral is *necessary*, not optional.

---

## 9. Test strategy summary

- **Unit tests** (property-based via `proptest`): each new pure function in `markets_currency.rs` has invariant tests (price floor, monotone scarcity drift, numeraire argmax correctness, conservation preserved, VWAP windowed correctly, cross-region rate `None` when transport exceeds gap, per-settlement view purity).
- **Integration tests** (hecs `World` with seeded RNG): a 5-settlement 4096-tick scenario with mixed regime; assert per-settlement market-good entropy is in the target band, no conservation violation, numeraire is non-trivial and per-settlement, cross-region dispersion tracks the transport-cost band, and the `prices_remain_positive_after_n_steps` invariant from `market.rs:262-270` still holds via the new emergent path.
- **Emergence regression**: `cargo test -p civ-economy -- emergent_market_regression` runs the P5-E 4096-tick scenario and asserts `market_good_entropy ∈ [0.6, 0.9]`, `rationing_frequency < 0.15`, `numeraire_isolation_index` bounded, conservation holds, and the price series stays strictly positive. This runs in CI as a performance-gated test.
- **No determinism requirement** (per charter §"Determinism is NOT a requirement"): tests assert *statistical* properties of the macro market (entropy bands, numeraire argmax, isolation index), not bit-identical outcomes of the per-tick price series. The *function-purity* of `emergent_clearing_price`, `select_numeraire`, `cross_region_rate`, and `compute_market_metrics` is asserted separately (same inputs → same outputs) so the dashboard / snapshot tests can rely on it.
- **No `currency_id` existence test** (regression net against hardcoded currency): a test that asserts the absence of any field named `currency` or `money` on `InstitutionAccount`, `Stocks`, or `MarketState`. If a future PR adds one, the test fails.

---

## 10. What this spec does NOT include (explicit non-goals)

- Any `enum Money` or `enum Currency` stored on a market locale, an agent, or an institution.
- Any `currency_id: u32` field anywhere on any agent, institution, or economy state.
- Any `update_price(g, p)` callable that mutates a price independent of a real `ClearedTrade`.
- Any hardcoded price table (the old `MarketState::default` at `market.rs:32-39` is replaced by an empty default in the companion spec §2; this spec inherits that decision).
- Any cross-crate API edge from `civ-economy` to `civ-agents` / `civ-diplomacy` / `civ-build`. All coupling is the shared substrate gradient (Stocks, Allocator order book, InstitutionLedger postings, TradeRoute flow), per the charter.
- Any LLM call in the price-discovery or numeraire path.
- Any LLM garnish for "narration of price changes" — that is the legends engine's job (`legends-engine.md` reads the `numeraire_shift.v1` event as input, not as output).
- Any banking / credit / fractional-reserve / interest-rate mechanism. Deferred to §8.
- Re-implementation of the existing CDA / `Allocator::clear` matching logic; this spec extends the *read-out* (the projectors) and the *event surface* (the replay-bus events), not the matching math.
- Re-implementation of the `compute_trade_routes` gravity kernel; this spec consumes its outputs as substrate.
- A migration of the existing `market.rs` `step` random-walk — that path is *deleted* in the companion spec §2, not preserved here.
- A `clearing_persistence_decay` knob that is *zero* (would erase the price series every tick; we want a long EWMA carry-over for stability, not a hard reset).
- Any `Bank`, `Banker`, `Interest`, or `Reserve` enum or struct. The banking spec is deferred.

---

*Document authority: this spec supersedes the hardcoded `Default for MarketState` price table (inherited from the companion `civ-economy-emergent-markets.md`) and the per-tick `MarketState::step` random-walk (inherited from the companion spec). The emergent price field, the emergent per-settlement numeraire, the emergent cross-region exchange rate, and the emergent money-as-pricing-role are all read-only projections over `Allocator::trades_log` + `Stocks` + `Settlement` + `InstitutionLedger::postings`. There is no other way for a price to be set, a numeraire to be chosen, a currency to exist, or money to function. The charter's "measured, emergent pattern over the substrate" is satisfied end-to-end for the markets-and-currency layer. Banking/credit is the next layer; its substrate seeds are inventoried in §8.*