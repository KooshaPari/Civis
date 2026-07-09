# CIV-ECONEMRG: Emergent Economy — Physics-Field Supply → Markets → Numeraire

> **Status:** Design (planner-only, 2026-06-23). No implementation code in this document.
> **Spec ID:** `economy-emergence` | **Branch:** `research/economy-emergence-design`
> **Pattern ancestors:**
> - [`civ-economy-emergent-markets.md`](civ-economy-emergent-markets.md) — emergent prices, emergent numeraire, allocation-as-read-out projector over `Allocator::trades_log`. *This doc extends it with the physics-field supply side; the market + numeraire substrate is reused unchanged.*
> - [`civ-003-emergent-lifecycle.md`](civ-003-emergent-lifecycle.md) — charter-constrained emergence pattern: measurement, not stored state; shared conserved gradients; explicit lags.
> - [`trade_routes.rs`](../../crates/economy/src/trade_routes.rs) — the gravity-kernel emergent trade-route substrate already on main (FR-ECON-trade): `surplus * deficit / dist²`.
> - [`emergent-systems-spec.md`](emergent-systems-spec.md) — system-level emergence charter.
> - [`emergence-charter.md`](../guides/emergence-charter.md) — "hardcode only the environmental and physical simulation rules."
>
> **Governing canon:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md), [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md).
>
> **Substrate (read-only inputs):**
> - `crates/economy/src/extraction.rs` — `ExtractionSite { Mine, Farm, Quarry, Fishery }`, `ResourceKind { Ore, Grain, Stone, Fish }`, `find_extraction_site`, `tick_extraction`
> - `crates/economy/src/stocks.rs` — `Stocks`, `Good { Food, Water, Wood, Metal, Tools, Energy }`, `ProductionProfile`, `surplus`, `deficit`, `comparative_advantage`
> - `crates/economy/src/trade_routes.rs` — `Settlement`, `TradeRoute`, `route_flow`, `compute_trade_routes` (gravity kernel)
> - `crates/economy/src/allocator.rs` — `Allocator`, `Bid`, `Offer`, `ClearedTrade`, `Allocator::clear` (CDA + rationing fallback)
> - `crates/economy/src/market.rs` — `MarketState`, `MarketState::step` (to be retired), `apply_pressure`, `MultiGoodMarket`, `OrderBook`, `Order`, `Trade`
> - `crates/economy/src/allocation.rs` — `CapitalistAllocator`, `PlannedAllocator`, `JouleAllocator`, `AllocationRegime`, `subsistence_first_allocate`
> - `crates/economy/src/institution.rs` — `InstitutionLedger`, `InstitutionPosting`, `LedgerSide`, `INSTITUTION_MARKET`, `INSTITUTION_TREASURY`
> - `crates/engine/src/engine.rs:1840` — `phase_economy` (the only tick driver that wires the substrate)
> - `crates/voxel/` — `VoxelWorld`, `MaterialId`, `WorldCoord` (the physics substrate: solid / water / air materials)
>
> **Traceability:** FR-CIV-ECON-001..015 (allocation + chains), FR-CIV-ECON-trade (gravity-kernel routes on main), FR-CIV-MARKET-001..008 (carry-over, fully consumed), FR-CIV-EMERGENCE-001 (micro → macro), FR-CIV-EMERGENCE-002 (shared-gradient coupling — no new API edges).
>
> **Supersedes / extends:** the "Hardcoded prices are forbidden" half of `civ-economy-emergent-markets.md` §0 *carries forward unchanged*. This doc is the *physics-field entry point* the existing markets spec assumes but does not derive — i.e. it specifies *how supply gradients enter the CDA* so prices can be emergent.

---

## 0. Charter constraint

The Civis Emergence Charter forbids hardcoding prices, currencies, and market types. Three substrate artifacts already violate it and are explicitly targeted here and in the companion markets spec:

| Artifact | Charter violation | Resolution (this doc) |
|---|---|---|
| `MarketState::default()` seeds `food = 1_000`, `energy = 1_000` cents (`crates/economy/src/market.rs:32-39`) | Hardcoded price table | Replaced by `MarketState::recompute(...)` (already specified in `civ-economy-emergent-markets.md` §1.2 — carried forward unchanged) |
| `MarketState::step` mutates prices via `deterministic_price_delta(tick, good)` (`crates/economy/src/market.rs:111-128`) | Scripted random walk unrelated to any trade | Replaced by the read-out projector over `Allocator::trades_log` (already specified — carried forward unchanged) |
| `crates/engine/src/engine.rs:1858` calls `self.market_state.step(self.state.tick)` | Authored state mutation in the economy phase | Replaced by `MarketState::recompute(allocator, stocks, profile, params, tick)` once `Allocator::trades_log` exists (P2-D in the markets spec) |

This doc adds the **fourth artifact** that the charter demands we expose explicitly: **the supply side of every bid is currently authored**. The engine constructs `Bid { price, quantity }` values from internal heuristics, not from measured `surplus`/`deficit` on `Stocks`. As long as that is true, prices are half-authored (they depend on trades, but the trades' willingness-to-pay is hardcoded). The fix is the `bid_from_need` / `offer_from_surplus` projector pair specified in §3 below — the analog of the lifecycle spec's `classify_lifecycle(age, health, maturity, ...)` projector, but for bid / offer *intent*.

The charter test applied to every field below: *can this emerge from Layer-0 rules?* Yes — prices from CDA crossing pressure on `Allocator`, numeraire from trade frequency over `Allocator::trades_log`, money from the role of pricing, supply gradients from voxel material density under a hand-of-workers profile. **No hardcoded price table, no fixed currency, no enum currency, no per-resource fixed value.**

---

## 1. Core emergence model

### 1.1 The economy is a measurement over a supply-gradient + a crossing book

A `MarketState` whose `step` mutates prices independent of any trade, an `AllocationRegime::Capitalist` that has no measurement of supply, and an `Engine::phase_economy` that calls `market_state.step(tick)` at the end of every tick are three faces of the same authored behavior: *prices as a clock-driven signal*. The substrate that should drive them already exists:

| Continuous driver | Crate + field | What it does |
|---|---|---|
| Voxel material density | `civ_voxel::VoxelWorld::read(coord) → MaterialId` (`crates/economy/src/extraction.rs:113-189`) | The physics-field gradient: ore vs farm vs quarry vs fishery at world position |
| Extraction site classification | `ExtractionSite { Mine, Farm, Quarry, Fishery }`, `ResourceKind { Ore, Grain, Stone, Fish }` (`extraction.rs:22-43`) | The four "bands" the simulator currently recognizes — the *only* authored enum, and only because four geological categories deserve distinct yield functions |
| Per-site yield | `tick_extraction(extractor, tick) → (ResourceKind, f32)` (`extraction.rs:81-105`) | Output of voxel density × workers × tool-quality × depletion (mine only) |
| Settlement stocks | `Settlement::stocks: Stocks` (`trade_routes.rs:71-84`) | Per-settlement inventory snapshot |
| Production profile | `ProductionProfile::net_flow(good) → i64` (`stocks.rs:160-162`) | Per-tick production − consumption, the canonical supply-side pressure signal |
| Surplus / deficit | `surplus(&Stocks, &Profile, Good)` / `deficit(...)` (`stocks.rs:210-220`) | Signed supply gradient; the kernel input to the gravity routes |
| Gravity-kernel routes | `route_flow(origin, destination, good, min_flow) → Option<TradeRoute>` (`trade_routes.rs:141-173`) | The emergent route substrate already on main; `flow = surplus * deficit / dist²` |
| Bid order book | `Allocator::bids: BTreeMap<OrderId, Bid>` (`allocator.rs:102`) | Live willingness-to-pay per `(bidder, good)` |
| Offer order book | `Allocator::offers: BTreeMap<OrderId, Offer>` (`allocator.rs:104`) | Live willingness-to-accept per `(offerer, good)` |
| Cleared trades | `Vec<ClearedTrade>` from `Allocator::clear` (`allocator.rs:174-384`) | The *only* place a per-unit price is set (midpoint of crossing pair, or rationing midpoint) |
| Trade frequency log | `Allocator::trades_log: Vec<ClearedTrade>` (NEW — §3 P1-A, P3-A in the markets spec) | Append-only history of every cleared trade; the substrate for numeraire selection |
| Institution ledger | `InstitutionLedger::postings` (`institution.rs:84`) | The conservation layer — every price-conveying transfer posts a balanced pair |
| Allocation regime | `AllocationRegime { Capitalist, Planned, Joule }` (`allocation.rs:115-124`) | Selectable regime selector; emergent *which* locales are which regime is FR-CIV-MARKET-006 (out of scope here) |

The chain from a voxel to a market price is:

```
voxel(MaterialId)
  → find_extraction_site() → ExtractionSite (Mine/Farm/Quarry/Fishery)
  → tick_extraction(workers, tool_quality, tick) → ResourceKind, qty
  → Settlement.stocks[Good]
  → ProductionProfile.net_flow(Good) = prod - cons
  → surplus(...) / deficit(...)               [per-tick supply gradient]
  → Trader posts bid(offer_from_surplus(...)) [P3-B, this doc]
  → Allocator::clear()                         [CDA crossing; per-unit price]
  → MarketState::recompute(allocator, ...)     [read-out projector; no mutation]
  → emergent_clearing_price(good)              [pure function of trades_log]
```

Each link in the chain is a function of the link above it. **Nothing is hardcoded.**

### 1.2 Prices are a read-out, not a stored state

Carried forward verbatim from `civ-economy-emergent-markets.md` §1.2. The new contribution is the *input* to that read-out: instead of constructing `Bid { price, quantity }` heuristically in the engine, the engine (or any other caller) reads `bid_from_need(stocks, profile, good, anchor)` (§3 below), which is a *pure* function of the local supply gradient. The price then falls out of crossing in the CDA.

### 1.3 Currency is a measured role

Carried forward verbatim from `civ-economy-emergent-markets.md` §3. The numeraire is the good with the highest `(trade_count × cross_good_acceptance)` over a sliding window; the good with the highest *realized trade flow* in the gravity-kernel routes is a strong prior but not a fixed role. There is no `currency_id` field anywhere on any agent, institution, or settlement.

---

## 2. Bidirectional coupling — the substrate gradient, not the API edge

The charter forbids one emergent layer calling another through an API boundary with no lag. The coupling mechanism is the same as `civ-003-emergent-lifecycle.md` §2: **shared conserved gradients with explicit lags**. Four gradients carry the coupling: the voxel material field, `Stocks`, the `Allocator` order book, and the institution posting log. Nothing in this spec introduces a new edge between crates.

### 2.1 Voxel physics → Economy (upward causation, the supply side)

| Physics signal | Economy effect | Lag mechanism |
|---|---|---|
| Local `MaterialId` density above a mineral band | `find_extraction_site` returns `ExtractionSite::Mine { ore_density }` → `tick_extraction` produces `ResourceKind::Ore` qty | 1 tick: voxel read → extraction step → `Stocks.add(Metal, qty)` |
| Voxel water band | Fishery classification → `ResourceKind::Fish` qty → `Stocks.add(Food, qty)` (per the diet-grain → Food mapping in §4.2) | 1 tick |
| Depletion (mine only): `0.95^(tick % 100)` | Ore output falls over the century, no scripted "this is depleted" enum | Continuous, no flag |
| Biome / climate (FR-CIV-PLANET) | Per-settlement `ProductionProfile::consumption(Water)` rises with climate dryness | 1 climate phase, ~tens of ticks |

The point: **no economic state is hardcoded against the physics.** Every `Stocks` write can be traced back to (a) an extraction event, (b) a chain conversion (FR-CIV-ECON-015), or (c) a consumption deduction. The supply side of every bid is downstream of a voxel.

### 2.2 Economy → Voxel physics (downward causation, the demand side)

| Economy signal | Voxel effect | Mechanism (no direct call) |
|---|---|---|
| Many agents at a settlement with `deficit(Wood) > 0` | They mark `VoxelWorld` cells as desired-for-harvest (`tool_quality > threshold`) → next extraction tick consumes `MaterialId::Wood` voxels | The mark is a placement; the voxel write happens through the existing `VoxelWriteProxy` (`engine.rs:3250`) |
| Sustained ore clearing trade volume | Settlement's ore-band workers auto-expand (more `Extractor { workers }` from the same settlement) | Indirect: `comparative_advantage(profile)` re-ranks each tick; new extractors are spawned at the highest-advantage good |
| Coinage (numeraire) shift | Workers in numeraire-band settlements stockpile; the `Stocks` vector shifts, the `ProductionProfile` shifts | Continuous |

### 2.3 Gravity-kernel routes are already emergent; the new wiring is just upstream supply

The trade-route gravity kernel (`crates/economy/src/trade_routes.rs:141-211`) is *already* emergent from supply gradients: `flow = surplus * deficit / dist²`. The substrate this doc adds makes `surplus` and `deficit` *also* emergent from voxel material density. The chain is then end-to-end emergent: **voxel → extraction → stocks → surplus/deficit → route flow → bid posting → CDA clearing → emergent price → emergent numeraire**.

### 2.4 What this spec does NOT couple

- No call from `civ-economy` into `civ-voxel`. The voxel side reads its own substrate; the economy side reads `Stocks`. `find_extraction_site` already lives in `civ-economy` and takes a `&VoxelWorld` by reference (`extraction.rs:113-189`), so the coupling happens at construction time, not per-tick call time.
- No new edge between `civ-agents` and `civ-economy`. Agents post bids via `Allocator::post_bid`, which is the existing API; the *content* of the bid is the new `bid_from_need` projector (§3), but the posting call is unchanged.
- No numeraire field on any actor. The numeraire is read by the dashboard from `Allocator::trades_log`.

---

## 3. Bid / offer intent projectors (the new piece)

These are the analogs of `classify_lifecycle(...)` (`civ-003-emergent-lifecycle.md` §1.2) — pure functions of measured state that produce the *intent* actors would have posted if their agency were continuous rather than authored.

### 3.1 `offer_from_surplus`

```text
fn offer_from_surplus(
    stocks: &Stocks,
    profile: &ProductionProfile,
    market: &MarketState,        // read-out; from previous tick
    offerer: InstitutionId,
    params: &EmergentMarketParams,
) -> Vec<Offer>
```

For each `Good` in `GOODS` where `surplus(stocks, profile, good) > 0`:

- `quantity = surplus(stocks, profile, good)` (the kernel's own supply)
- `price = market.last_emergent_clearing[good].unwrap_or(scarcity_drift_price(stocks, profile, good, ANCHOR_PRICE_CENTS, params))`
- The offerer posts one `Offer { offerer, good, quantity, price }` per surplus good

`AnchorPriceCents` is a single integer floor used when no trade has occurred yet for the good — *not* a hardcoded price table. It exists so that an empty book in a brand-new scenario can still post sensible first offers; it is the same default for every good (one cent of willingness to accept, derived from the substrate floor `MIN_PRICE_CENTS = 1` already on main at `market.rs:22`).

### 3.2 `bid_from_need`

```text
fn bid_from_need(
    stocks: &Stocks,
    profile: &ProductionProfile,
    need: Good,
    need_kind: NeedKind,         // Subsistence | Luxury (from allocation.rs:159-165)
    market: &MarketState,        // read-out; from previous tick
    bidder: InstitutionId,
    params: &EmergentMarketParams,
) -> Bid
```

- `quantity = max(deficit(stocks, profile, need), 0)`
- `price = market.last_emergent_clearing[need].unwrap_or(scarcity_drift_price(stocks, profile, need, ANCHOR_PRICE_CENTS, params))`
  - For `Subsistence` needs, the price is *not* capped — the bidder pays whatever the book demands.
  - For `Luxury` needs, the price is floored at `params.luxury_price_cap_pct` of the last clearing price (a §4 knob; default 50%) so luxury demand does not run away during scarcity.

`NeedKind` is the same enum already on main at `allocation.rs:159-165`; it is reused unchanged.

### 3.3 Why these are pure functions, not callables

`bid_from_need` and `offer_from_surplus` are *not* mutators. They return a `Bid` or `Vec<Offer>` for the caller to post via `Allocator::post_bid` / `Allocator::post_offer`. The substrate never posts on the actor's behalf — the agent's psyche / utility AI (or, in scenarios, a deterministic actor model) is the *caller*. This is the same separation the lifecycle spec enforces: `classify_lifecycle` returns a label; the ECS does the entity despawn.

The two helpers together ensure that **the only prices that exist in `MarketState` are prices that arose from a real `ClearedTrade`** (via `Allocator::clear`), and **the only quantities posted are quantities measured from `surplus` / `deficit`**, which in turn are measured from `Stocks`, which in turn are downstream of voxel material density.

### 3.4 Cross-locale arbitrage

When two settlements have different numeraire prices for the same good (because their supply gradients differ), the price gap is observable on each settlement's emergent clearing series. The `propose_trade` helper on main (`crates/economy/src/stocks.rs:252-308`) already implements the *intent* — propose a trade if both sides have surplus. The new `cross_locale_arbitrage_opportunity(local_market, remote_market, good, transport_cost_cents) → Option<(Side, i64)>` (specified in the companion markets spec P3-C) is the *signal*: it reads two `MarketState`s and tells the caller whether the gap exceeds transport cost. The caller is `propose_trade`, not the engine. No new API edge.

---

## 4. Criticality knobs — edge of chaos

All knobs concentrate in `EmergentMarketParams` (already specified in `civ-economy-emergent-markets.md` §4). The physics-field coupling adds three knobs specific to the supply-side projector; the rest are carried forward unchanged.

### 4.1 Carried-forward knobs (markets spec §4)

| Parameter | Default | Effect |
|---|---|---|
| `tatonnement_lambda` | `50` | Per-tick scarcity-drift step size |
| `min_price_cents` | `1` | Floor on every emergent clearing price (preserves `market.rs:144` invariant) |
| `rationing_lookback` | `256` | Window for rationing-frequency metric |
| `vwap_lookback` | `1024` | Window for VWAP read-out |
| `numeraire_lookback` | `4096` | Window for numeraire argmax |
| `numeraire_min_trade_count` | `8` | Min trades in window for numeraire eligibility |
| `arbitrage_window` | `512` | Cross-region arbitrage convergence window |
| `emergent_overshoot_cap` | `0.20` | Max fractional price change per tick (antitrust fuse) |
| `clearing_persistence_decay` | `0.95` | EWMA weight when crossing series is empty |

### 4.2 NEW: supply-side knobs (this doc)

| Parameter | Type | Default | Effect | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `resource_band_to_good_gain_bps` | `i64` (0..=10_000) | `10_000` | Confidence that a `ResourceKind` band is mapped to its default `Good`. 10_000 bps = always map; 0 = never map (every extraction is its own latent good) | `0` (every extraction is a unique good; no economies of scale; barter-only) | `10_000` (one fixed map; same as hardcoded) |
| `luxury_price_cap_pct` | `i64` (0..=100) | `50` | Luxury bid price is capped at this percent of the last clearing price; subsistence bids are uncapped | `0` (luxury dies even on healthy book) | `100` (luxury can outbid subsistence on tight book — equity violation) |
| `extraction_yield_scale_bps` | `i64` (1..=100_000) | `10_000` | Multiplier on `tick_extraction` output before it enters `Stocks` (1.0×). Pure scaling of the physics→economy coupling; does *not* shift relative prices | `< 100` (extraction can't seed supply; trade is barter forever) | `> 50_000` (extraction dominates; chains + trades become irrelevant) |

**Charter test:** all three knobs are *parameters*, not *constants*. The §6 P5-F acceptance criterion (markets spec §5) extends naturally: setting `resource_band_to_good_gain_bps` to its heat-death extreme should reduce `market_good_entropy` below the healthy band within 512 ticks (every extraction is a distinct good; trades become barter-one-offs).

### 4.3 ResourceKind → Good: emergent, not authored

The naive mapping is `Ore → Metal`, `Grain → Food`, `Stone → Wood`, `Fish → Food`. **This naive mapping is *not* what the substrate emits.** Instead:

| Layer | What it knows | Where it lives |
|---|---|---|
| `ResourceKind { Ore, Grain, Stone, Fish }` | The four geological bands the *physics* recognizes — these are authored because four categories deserve distinct yield / depletion curves | `extraction.rs:8-18` (already on main) |
| `Good { Food, Water, Wood, Metal, Tools, Energy }` | The six categories the *stocks* substrate tracks — these are authored because they are the canonical inventory vector and a finite inventory space is required for conservation | `stocks.rs:13-26` (already on main) |
| `BandToGoodMap` (NEW) | A *probabilistic* projector that returns a `BTreeMap<(ResourceKind, Good), u16>` of basis-point confidences | `crates/economy/src/emergent.rs` (this doc) |

The default `BandToGoodMap` ships with the naive mapping at `10_000 bps` (always map), but **the substrate accepts any mapping**. A scenario with different bands (e.g. `Brine` from salt flats) can ship its own mapping at, say, `5_000 bps` (50% probability the brine becomes `Water` and 50% it becomes `Tools` after processing) — the bookkeeping resolves through the chain substrate (`chains.rs`) and the `Stocks` accounting. The price for `Brine` is then *emergent* from the chains that consume it and the consumers that bid on it.

`resource_band_to_good_gain_bps` is a knob, not a fixed mapping: `10_000 bps` is *one* configuration; `0 bps` is the fully-emergent extreme where every band stays distinct (no good is privileged). The acceptance criterion P5-F tests both extremes.

---

## 5. Observable emergence metrics for the dashboard

These extend `civ-economy-emergent-markets.md` §5 with the supply-side metrics. All metrics aggregate from existing `VoxelWorld`, `Stocks`, `Allocator::trades_log`, and `MarketState::recompute` outputs — no new agent-level state.

| Metric | How to compute | Target signature (healthy) | Failure mode |
|---|---|---|---|
| **Supply-side entropy** | Histogram of `Stocks` totals across `(Settlement, Good)`; Shannon entropy normalised by `log2(SETTLEMENTS * GOODS.len())` | High (0.6..0.9) — many `(settlement, good)` cells are non-empty | Low (all settlements have all goods) = autarky; high (one cell) = monoculture |
| **Resource-band entropy** | Histogram of `ResourceKind` produced per extraction site over `vwap_lookback`; Shannon entropy normalised by `log2 4` | High (0.7..1.0) — all four bands active | Low = geology is one-band (desert island with only fish); harmless, but the market degenerates to one-good |
| **Bid-anchor respect** | Fraction of `Bid::price` posted in the last `vwap_lookback` that are within `[emergent_clearing_price(g) * (1 − 0.2), emergent_clearing_price(g) * (1 + 0.2)]` (the `emergent_overshoot_cap` band) | High (≥ 0.7) — bidders read the book | Low (< 0.3) = bidders are blind / scripted; charter violation |
| **Numeraire persistence** | Number of consecutive ticks the same `GoodId` is the argmax in `select_numeraire`; reset to 0 on shift | Long runs (hundreds of ticks) | Frequent shifts (< 50 ticks) = currency chaos |
| **Numeraire share of trade volume** | `sum(cleared_trade.quantity for cleared_trade.good == numeraire) / sum(all quantity)` over `numeraire_lookback` | Moderate (0.05..0.30) | Zero = misnamed; > 0.5 = hoarding |
| **Cross-region price dispersion** | `|vwap_A(g) - vwap_B(g)| / max(vwap_A(g), vwap_B(g))` averaged over goods | Small (≤ 0.2) with road access | Persistent near-1.0 in adjacent regions with road access = arbitrage dead |
| **Gravity-route viability** | `count(trade_routes.flow >= params.min_flow) / count(trade_routes)` over `vwap_lookback` | Moderate (0.3..0.7) — most pairs produce at least *some* flow when supply gradients exist | Near 0 = no trade ever viable (the substrate has no surplus anywhere); near 1.0 = every pair is a route (uniform geography, no comparative advantage) |
| **Conservation invariant** | `InstitutionLedger::verify_conservation` + `verify_ledger_conservation` per tick | Always Ok | Any failure = critical bug |
| **Institution balance non-negativity** | All `InstitutionAccount::balance_joules >= 0` | Always true | Negative = a posting violated the floor |

The `emergence.metrics.v1` replay-bus event is extended with `supply_side_entropy`, `resource_band_entropy`, `bid_anchor_respect`, and `gravity_route_viability`. The `emergence.alarm.v1` event adds `MT-SUP-001` (resource-band entropy < 0.2 for 1024 ticks — band collapse, churn to barter), `MT-SUP-002` (bid-anchor respect < 0.3 — scripted price behavior, charter violation), `MT-SUP-003` (gravity-route viability > 0.95 — autarky has set in, no comparative advantage).

---

## 6. Phased implementation plan

This is a DAG-structured WBS. No code is written here; file paths, struct names, and function signatures are identified for the implementing agent. Every phase extends *existing* substrate — no new crate, no new API edge.

### Phase 0 — Prerequisite audit (no new structs)

| Task | File | Depends on | Effort |
|---|---|---|---|
| P0-A: Confirm `find_extraction_site(world, pos, radius) → Option<ExtractionSite>` is the only voxel→economy entry point (it is; `extraction.rs:113-189`) | `crates/economy/src/extraction.rs` | — | 1 tool call |
| P0-B: Confirm `Allocator::trades_log` does NOT yet exist; the `Vec<ClearedTrade>` returned by `Allocator::clear` is currently discarded by every caller (`crates/engine/src/engine.rs:1858` only calls `market_state.step`) | `crates/economy/src/allocator.rs` + engine callers | — | 2 tool calls |
| P0-C: Confirm the engine's `phase_economy` (`engine.rs:1840-1859`) is the only place `MarketState::step` is called and that no other crate holds a `MarketState` (search for `MarketState::step` and `market_state.step` across the tree) | `crates/` (search) | — | 3 tool calls |
| P0-D: Confirm the `ResourceKind → Good` mapping does NOT exist anywhere in the tree (search for `ResourceKind::`, `Grain`, `Ore`, `Stone`, `Fish` → `Good` conversions). The naive mapping at `crates/engine/src/engine.rs:3222-3230` (`route_resource`) is for `ResourceType`, not `ResourceKind`, and lives in the engine's `tick_trade_routes` only | `crates/` (search) | — | 2 tool calls |
| P0-E: Confirm `bid_from_need` / `offer_from_surplus` do not exist; the only bid construction today is `Allocator::post_bid(Bid { ... })` calls in scenario / agent code | `crates/` (search) | — | 2 tool calls |

### Phase 1 — Extend `Allocator` with append-only trade log + bid/offer intent projectors

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P1-A: Add `pub trades_log: Vec<ClearedTrade>` field to `Allocator` (`allocator.rs:99-107`); append every `ClearedTrade` inside `clear`; bound memory with `trades_log.retain(\|t\| t.tick > current_tick.saturating_sub(self.max_log_ticks))` | `crates/economy/src/allocator.rs` | P0-B | Pure append; no breaking changes to existing callers |
| P1-B: Add `EmergentMarketParams` struct (all §4 knobs — both carried-forward markets-spec and NEW §4.2 supply-side) to new `crates/economy/src/emergent.rs` | new `crates/economy/src/emergent.rs` | P0-A..E | Re-exported via `lib.rs:32` |
| P1-C: Add `pub fn offer_from_surplus(...) -> Vec<Offer>` (§3.1) | `crates/economy/src/emergent.rs` | P1-B | Pure function; same GoodId type as `Allocator::Offer.good` (`String`) |
| P1-D: Add `pub fn bid_from_need(...) -> Bid` (§3.2) | same | P1-B | Pure function |
| P1-E: Add `pub fn BandToGoodMap::default() -> BandToGoodMap` returning the naive mapping at `10_000 bps`; add `pub fn BandToGoodMap::apply(&self, kind: ResourceKind, params: &EmergentMarketParams) -> BTreeMap<Good, u16>` | same | P1-B | The default is reproducible across runs; the API allows scenarios to override |
| P1-F: Add `pub fn scarcity_drift_price(stocks, profile, good, anchor, params) -> i64` (carried forward from markets spec P1-E, integer arithmetic) | same | P1-B | Used by both `bid_from_need` and `offer_from_surplus` for the no-trade-yet fallback |
| P1-G: Tests: P1-A retain window does not drop fresh entries; P1-C offer count = count of surplus goods; P1-D bid price rises monotonically in deficit when `Subsistence`; P1-D bid price capped at `luxury_price_cap_pct` for `Luxury`; P1-E naive map returns one good per band at `10_000 bps`; P1-F drift monotone in deficit when anchor > 0 | `crates/economy/src/emergent.rs` tests | P1-A..F | Property-based via `proptest` |

### Phase 2 — Replace `MarketState::step` with the read-out projector (carried forward)

All P2-A..G tasks are *unchanged* from `civ-economy-emergent-markets.md` §6 Phase 2. The only addition is wiring `Bid`/`Offer` intent through the new projectors (P2-H):

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P2-A: Change `MarketState` to the read-only struct (markets spec §1.2). Remove `MarketState::step` | `crates/economy/src/market.rs` | P1-C, P1-D, P1-F | No `step` method |
| P2-B: Add `pub fn recompute(allocator, stocks, profile, params, tick) -> MarketState` | same | P2-A | Pure function |
| P2-C: Empty `Default for MarketState`; helper `is_empty()` | same | P2-A | |
| P2-D: Wire `MarketState::recompute` into `EconomyState::step` and `engine.rs:1858` | `crates/economy/src/lib.rs` + `crates/engine/src/engine.rs:1858` | P2-B | The engine's `market_state.step(tick)` becomes `market_state = MarketState::recompute(&allocator, &stocks_view, &profile_view, &params, tick)` |
| P2-E: Re-exports `pub use emergent::{EmergentMarketParams, Bid, Offer, ClearedTrade, Allocator, allocator::post_balanced_trade};` | `crates/economy/src/lib.rs` | P1-B, P2-A | |
| P2-F: Migrate every `MarketState::step` caller found in P0-C | callers from P0-C | P2-D | |
| P2-G: Tests: empty default; recompute is pure; never returns a negative price | `crates/economy/src/market.rs` tests | P2-A..F | |
| **P2-H (NEW)** | `crates/engine/src/engine.rs` | P2-D, P1-C, P1-D | At the engine level, after `recompute`, the engine (or a caller inside `phase_economy`) constructs one set of bids from `bid_from_need` and one set of offers from `offer_from_surplus` for each settlement whose `Stocks` and `ProductionProfile` are non-default. The settlement list is whatever the substrate owns (no per-settlement enum; settlements are derived from world state — see §6.1 in the markets spec). The constructed bids/offers are posted via `Allocator::post_bid` / `Allocator::post_offer`. This is the first place the supply side is wired through the projectors. |

### Phase 3 — Physics-field coupling (the new layer)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P3-A: Add `pub fn extract_to_stocks(extractor, settlement_stocks, tick, params, band_map) -> (Stocks, ResourceKind, i64)` to `crates/economy/src/emergent.rs` | `crates/economy/src/emergent.rs` | P1-E, P1-F | Pure function: calls `tick_extraction(extractor, tick)`, scales by `extraction_yield_scale_bps`, samples `BandToGoodMap::apply(kind, params)`, applies the basis-point distribution to `settlement_stocks` additively (the residual un-mapped quantity is *not* lost — it stays in a `latent: BTreeMap<ResourceKind, i64>` field on `Settlement` for chains to consume later). |
| P3-B: Add `pub fn tick_settlement_extraction(settlement, world, params, band_map) -> Settlement` (pure; returns the updated settlement) | same | P3-A | One settlement at a time; the engine iterates its settlement list |
| P3-C: Wire `tick_settlement_extraction` into `phase_economy` (alongside P2-D's `recompute`). The extractor list lives on each `Settlement` as `extractors: Vec<Extractor>` (a new optional field — non-breaking; absent on main is fine, falls back to no-extraction pass) | `crates/engine/src/engine.rs` | P3-A, P3-B | One `tick_settlement_extraction` call per settlement per tick |
| P3-D: Tests: P3-A scaling by `extraction_yield_scale_bps`; P3-A `BandToGoodMap::apply` produces exactly one unit of mapped Good per unit of band; P3-B zero-extractors is a no-op; P3-B pure function (same inputs → same outputs) | `crates/economy/src/emergent.rs` tests + engine integration test | P3-A..C | Property-based |
| P3-E: (Optional, future) `pub fn emergent_extractor_assignment(settlement, world, profile, params) -> Vec<Extractor>` — the substrate decides how many workers to place on each band based on `comparative_advantage(profile)`. Out of scope for v1; agent / polity layer decides assignment per the existing `Allocator` / utility AI path | `crates/economy/src/emergent.rs` | P3-C | Documents the *future* wiring; not implemented in this slice |

### Phase 4 — Criticality knobs + dashboard metrics (carried forward + extension)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P4-A: Add `EmergentMarketParams` field to `EconomyState` (markets spec P4-A) | `crates/economy/src/lib.rs` | P1-B | |
| P4-B: Extend `crates/engine/src/emergence.rs` with the §5 metrics (`supply_side_entropy`, `resource_band_entropy`, `bid_anchor_respect`, `gravity_route_viability`) | `crates/engine/src/emergence.rs` | P3-C, P2-D | All metrics aggregate from `VoxelWorld` + `Stocks` + `Allocator::trades_log` + `MarketState::recompute` |
| P4-C: Wire the new metrics into `civ-server`'s `sim.snapshot` (F3D0 frame already in `fr-3d-matrix.md`) | `crates/server/` | P4-B | No new frame variant |
| P4-D: Extend `emergence.metrics.v1` replay-bus event with the four new metrics + the three new `MT-SUP-*` alarm IDs | `crates/engine/src/emergence.rs` | P4-B | Same wire format |
| P4-E: Tests: P4-B metrics stay bounded for populations of 10, 100, 1000 actors; P4-C snapshot round-trips through `bincode`; P4-D replay event deterministically re-emits given a fixed `trades_log` | `crates/engine/` tests | P4-A..D | |

### Phase 5 — Acceptance criteria reachability (validation)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P5-A: "No hardcoded price table" — start with empty `MarketState`, run 200 ticks with ≥ 3 actors posting via `bid_from_need` / `offer_from_surplus`; assert `MarketState.last_emergent_clearing` is populated and the first price is NOT 1_000 (the old hardcoded default) | `crates/economy/tests/` | Phase 2 complete | Charter AC |
| P5-B: "Numeraire is data-driven" — construct a scenario where the only well-traded good is `Good::Wood`; assert `select_numeraire` returns `Some("wood")`. Then swap to `Good::Metal` only and assert the shift (carried forward from markets spec P5-B) | same | P5-A | Charter AC |
| P5-C: "Charter conservation" — 1000-tick scenario with mixed regime; assert `InstitutionLedger::verify_conservation` succeeds every tick; assert no `ClearedTrade::price` is negative; assert the `prices_remain_positive_after_n_steps` invariant from `market.rs:144` still holds via the new emergent path | same | Phase 4 | Conservation AC |
| P5-D: "Bid price is supply-gradient derived" — construct a scenario where settlement A has `surplus(Food) = 100` and settlement B has `deficit(Food) = 100`; assert that the bid posted by an actor at B with `bid_from_need(..., Food, Subsistence, ...)` has a price *strictly above* the bid posted by the same actor at A with `bid_from_need(..., Food, Subsistence, ...)`. (B should outbid A because B's deficit is local.) | same | Phase 1 + Phase 2 | Charter AC: prices derive from gradients, not from a script |
| P5-E: "Currency emerges from physics" — construct a scenario with only `ResourceKind::Ore` extractors; assert the `select_numeraire` after 4096 ticks is `Some(Good::Metal)`. Then re-run with only `ResourceKind::Grain`; assert the numeraire shifts to `Some(Good::Food)`. | same | Phase 3 | Charter AC: currency derives from physical resource base |
| P5-F: "Criticality knob reachability" — for each knob in §4.2 (and the carried-forward §4.1 knobs), set to its heat-death and explosion extremes; assert the system reaches the corresponding failure mode within 512 ticks; setting back to default returns to the target band within 2048 ticks | same | P5-C | Performance-gated |
| P5-G: "Gravity-route substrate still works" — re-run `fr_econ_trade_compute_is_deterministic_and_conserving` and `fr_econ_trade_route_flow_is_gravity_kernel` (`crates/economy/src/trade_routes.rs:264-401`) as integration tests; assert no regression | `crates/economy/tests/` | Phase 3 | The substrate this doc adds is *upstream* of trade routes; the routes themselves must continue to work |

### Phase 6 — Coupling sanity (cross-layer, no new edges)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P6-A: Audit `crates/agents/` for any direct `market_state.prices[g]` reads that depended on the hardcoded default; replace with `MarketState::recompute(...).last_emergent_clearing.get(g)` so first-ever reads in fresh scenarios return `None` | `crates/agents/` | Phase 2 + Phase 3 | No new crate dep |
| P6-B: Audit `crates/diplomacy/` and `crates/build/` for any price reads; ensure they consume the emergent series | `crates/diplomacy/`, `crates/build/` | P6-A | |
| P6-C: Add a CI-level invariant test: assert that NO production code path mutates `MarketState` except by calling `MarketState::recompute` (or by re-assigning to a fresh `recompute(...)` return value). Static walk of the crate's public surface | `crates/economy/tests/` | Phase 2 complete | Closes the loop on the charter |
| P6-D: Add a CI-level invariant test: assert that NO production code path constructs a `Bid` or `Offer` with a hardcoded price that does not derive from `last_emergent_clearing[g]` or `scarcity_drift_price(...)`. Walk the constructor sites; allow the `AuctionTester` test harness to opt out | `crates/economy/tests/` | Phase 1 + Phase 2 complete | Closes the supply-side charter loop |

### DAG summary

```
P0-* (all parallel)
  → P1-A..G
    → P2-A..G + P2-H (P2-H needs P1-C, P1-D)
      → P3-A..E (P3-C needs P2-D)
        → P4-A..E
          → P5-A..G
            → P6-A..D (P6-C and P6-D can run in parallel with P5)
```

**Critical path to acceptance:** P0 → P1 → P2 (incl. P2-H) → P3 → P4 → P5. The P6 audit can run in parallel with P5 and is the last gate before charter compliance is complete.

---

## 7. Test strategy summary

- **Unit tests** (property-based via `proptest`): each new pure function in `emergent.rs` has invariant tests — bid price monotone in deficit (with cap for `Luxury`), offer count equals surplus count, scarcity drift monotone, `BandToGoodMap::apply` is a basis-point distribution, `tick_settlement_extraction` never reduces `Stocks`, recompute purity (same inputs → same outputs).
- **Integration tests** (hecs World + `VoxelWorld` + seeded RNG): a 100-actor 4096-tick scenario with mixed regime and three extraction bands (mine / farm / fishery); assert supply-side entropy in target band, no conservation violation, numeraire derives from physical resource base (not hardcoded), `bid_from_need` and `offer_from_surplus` produce realistic price distributions.
- **Emergence regression**: `cargo test -p civ-economy -- emergent_economy_regression` runs the P5-E "currency emerges from physics" scenario with only `ResourceKind::Ore` extractors and asserts `select_numeraire` converges to `Some(Good::Metal)`. Runs in CI as a performance-gated test.
- **No determinism requirement** (per charter; `emergence-charter.md` §"Determinism is NOT a requirement"): tests assert statistical properties of the macro market (entropy bands, numeraire derives from physical base, conservation holds), not bit-identical outcomes. *Function-purity* of `MarketState::recompute`, `select_numeraire`, `bid_from_need`, `offer_from_surplus`, `tick_settlement_extraction`, and `BandToGoodMap::apply` is asserted separately (same inputs → same outputs) so dashboard / snapshot tests can rely on it.

---

## 8. What this spec does NOT include

- Any `enum MarketType` or `enum Currency` stored on a market locale, an agent, or an institution.
- Any `update_price(g, p)` callable that mutates a price independent of a real `ClearedTrade`.
- Any hardcoded price table (the old `MarketState::default` at `market.rs:32-39` is replaced by an empty default in P2-C; the new prices only exist because a trade happened).
- Any cross-crate API edge from `civ-economy` to `civ-voxel`, `civ-agents`, `civ-diplomacy`, or `civ-build`. The voxel side reads its own substrate; the economy side reads `Stocks`. `find_extraction_site` already takes a `&VoxelWorld` by reference, so the coupling happens at construction time, not per-tick call time.
- Any LLM call in the price-discovery or numeraire path.
- Any LLM garnish for "narration of price changes" — that is the legends engine's job (`legends-engine.md` reads the `numeraire_shift.v1` event as input, not as output).
- A `resource_band_to_good_gain_bps` knob that is `> 10_000` (would be a malformed basis-point distribution); the knob is always clamped to `[0, 10_000]`.
- A `luxury_price_cap_pct` knob that is `> 100` (would let luxury bid above subsistence; clamped to `[0, 100]`).
- A re-implementation of the existing CDA / `Allocator::clear` matching logic; this spec extends the *output* (the `trades_log` in P1-A) and the *input* (the `bid_from_need` / `offer_from_surplus` projectors in P1-C, P1-D), not the matching math.
- A migration of the existing `market.rs` `step` random-walk — that path is *deleted* in P2-A (carried forward from markets spec), not preserved.
- A direct per-resource `ResourceKind::Ore → Good::Metal` mapping (this doc explicitly avoids that; §4.3 mandates a `BandToGoodMap` projector with `resource_band_to_good_gain_bps` confidence).
- `Emergent extractor assignment` (P3-E): the substrate does not decide which band each worker is on. Agents / polities decide; the substrate measures the consequences. (Future wiring; out of scope for this slice.)

---

## 9. Cross-references

| Concept | Reference |
|---|---|
| Emergent prices + numeraire (carried forward, *unchanged*) | [`docs/design/civ-economy-emergent-markets.md`](civ-economy-emergent-markets.md) |
| Emergent lifecycle + classifier pattern | [`docs/design/civ-003-emergent-lifecycle.md`](civ-003-emergent-lifecycle.md) |
| Emergence charter (single governing principle) | [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) |
| System-level emergence spec | [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) |
| Emergence dashboard (existing) | [`docs/design/emergence-dashboard.md`](emergence-dashboard.md), [`docs/design/EMERGENCE_DASHBOARD.md`](EMERGENCE_DASHBOARD.md) |
| Emergence tests plan | [`docs/design/EMERGENCE_TESTS_PLAN.md`](EMERGENCE_TESTS_PLAN.md) |
| Emergence wiring patchplan | [`docs/design/EMERGENCE_WIRING_PATCHPLAN.md`](EMERGENCE_WIRING_PATCHPLAN.md) |
| Emergent trade routes (already on main) | [`crates/economy/src/trade_routes.rs`](../../crates/economy/src/trade_routes.rs) (FR-ECON-trade) |
| Resource extraction from voxel (already on main) | [`crates/economy/src/extraction.rs`](../../crates/economy/src/extraction.rs) |
| CDA auction substrate (already on main) | [`crates/economy/src/allocator.rs`](../../crates/economy/src/allocator.rs) |
| Stocks / ProductionProfile (already on main) | [`crates/economy/src/stocks.rs`](../../crates/economy/src/stocks.rs) |
| Allocation regime (already on main) | [`crates/economy/src/allocation.rs`](../../crates/economy/src/allocation.rs) |
| Institution ledger (already on main) | [`crates/economy/src/institution.rs`](../../crates/economy/src/institution.rs) |
| Engine phase_economy (existing call site) | [`crates/engine/src/engine.rs:1840-1859`](../../crates/engine/src/engine.rs) |

---

*Document authority: this spec, together with `civ-economy-emergent-markets.md`, replaces the random-walk `MarketState::step` path, the hardcoded `Default for MarketState` price table, and any `Bid`/`Offer` constructor that uses a hardcoded price. The emergent price field, the emergent numeraire, and the emergent currency all derive from the physics-resource supply gradient (voxel material density → extraction → stocks → surplus/deficit → bid/offer on the CDA → clearing → price → trade frequency → numeraire) and from the gravity-kernel trade routes already on main. There is no other way for a price to be set, a numeraire to be chosen, a currency to exist, or a supply gradient to enter the market. The charter's "measured, emergent pattern over the substrate" is satisfied end-to-end from voxel to wallet.*