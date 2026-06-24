# Criticality Re-Verification 2 — N3 / N4 couplings

**Date:** 2026-06-16  
**Method:** Read-only static audit (`git show origin/main:crates/engine/src/engine.rs` + prior `CRITICALITY_REVERIFY.md`). No `cargo`, no source edits, no commit.  
**Ref audited:** `origin/main` @ `73ef4b71` (`feat(engine): birth emergent trade routes from sustained TradeAgreement (N4) (#529)`, 2026-06-16)  
**Prior audit:** `CRITICALITY_REVERIFY.md` (`main` @ `eacd4361`, pre-N3/N4)

**Scope:** New couplings since last verify — **N3** (cluster → diplomacy-pairs), **N4** (emergent trade-route birth). Confirm N4 route topology stays bounded; flag any new unbounded accumulator or positive-feedback loop.

---

## Executive summary

| Coupling | Verdict | Notes |
|----------|---------|-------|
| **N3** cluster → diplomacy pair | **CLEAN** | Pure selection; no new persistent state; no accumulator |
| **N4** diplomacy → trade-route birth | **BOUNDED** ✓ | `MAX_TRADE_ROUTES=64`, idle decay `2_000` ticks, conflict teardown |
| **N3 × N4** cross-coupling | **LOW** | Geographic pair bias accelerates route birth among neighbors; capped by N4 bounds |
| **Prior [U5], [R1], [FC-3]** | **CLOSED** on this ref | Decay, material gate, cluster food sink present |
| **Legacy [U1]–[U4], [U6]** | **Still open** | Unchanged by N3/N4 |

**N4 route creation:** **Yes — bounded.** Hard cap on `trade_routes.len()`, emergent-only idle decay, conflict removes emergent edges. One minor streak-scalar drift (see [N4-S1]).

**Overall for N3/N4 delta:** **CLEAN** for unbounded topology / runaway birth. One **low-severity** scalar drift; no new high-severity positive-feedback loop beyond pre-existing treasury path [U6].

---

## 1. N3 — settlement cluster overlap → diplomacy pair selection

**Wired at:** `phase_diplomacy` (~L2853) calls `diplomacy_pair_from_settlement_overlap`.  
**Helpers:** `settlement_dominant_factions`, `settlement_contact_pairs`, `diplomacy_faction_pairs_from_settlement_contact` (~L3936–4078).

### Mechanism (read-only)

1. Dominant `Alignment::Faction` per multi-member cluster (`SETTLEMENT_MIN_MEMBERS = 2`).
2. Contact edges when any cross-cluster agents within `SETTLEMENT_CONTACT_RADIUS_FP` (`2 × cluster_radius`).
3. Pair pick order: contacting cross-faction pairs → settlement presence → legacy registry rotation.
4. Selection index: `(tick / 500) % pairs.len()` — one pair per diplomacy event.

### Criticality table

| Check | Result |
|-------|--------|
| New `WorldState` fields | **None** |
| Persistent accumulators | **None** |
| Writes per tick | **None** (diplomacy cadence 500 ticks only) |
| Contact-pair compute | O(C²) over settlement count C; ephemeral per call, not stored |
| Positive feedback into scalars | **No direct path** — only changes `(a,b)` for existing diplomacy economics |

### Verdict: **CLEAN**

N3 is a deterministic pair selector over bounded candidate sets. It does not integrate a scalar without sink.

---

## 2. N4 — emergent trade-route birth (implemented model)

**Note:** `N4_COUPLING_SPEC.md` describes settlement food-exchange → `faction_exchange_ledger` → route upsert. **On `origin/main`, N4 is diplomacy-driven:** sustained `DiplomacyKind::TradeAgreement` → `faction_trade_agreement_streak` → `trade_routes.push` when streak ≥ 2. Settlement-exchange phase is **not** present.

### 2.1 Birth path (`phase_diplomacy`, ~L2944–2974)

| Guard | Constant / behavior |
|-------|---------------------|
| Agreement streak | `TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD = 2` (≥2 TradeAgreement events on same pair) |
| Relation floor | `TRADE_ROUTE_MIN_RELATION = 0.0` |
| Duplicate block | `already_exists` on `(from, to, goods)` |
| Global cap | `MAX_TRADE_ROUTES = 64` (`at_cap` skips birth) |
| Initial volume | `Fixed::from_num(8)` — **fixed at birth; no per-agreement volume growth** |
| Conflict sink | `reset_trade_agreement_streak` + `remove_emergent_routes_between` (~L2939–2940) |

### 2.2 Idle decay (`tick_trade_routes` → `decay_idle_emergent_trade_routes`, ~L3152 / ~L4176)

| Mechanism | Detail |
|-----------|--------|
| Track set | `emergent_trade_route_keys` — bootstrap triangle **excluded** |
| Flow reset | Route key in `flowed_keys` (resource moved this tick) → idle = 0 |
| Decay | `trade_route_idle_ticks[key] += 1` when no flow |
| Removal | `idle >= TRADE_ROUTE_UNUSED_DECAY_TICKS` (2_000) → remove from `trade_routes`, keys, idle map |

### 2.3 Boundedness checklist

| Container | Bound | Mechanism |
|-----------|-------|-----------|
| `trade_routes` | **≤ 64** | `MAX_TRADE_ROUTES` pre-push check |
| `emergent_trade_route_keys` | **≤ 64** | Subset of routes |
| `trade_route_idle_ticks` | **≤ emergent keys** | 1:1 with emergent routes |
| Route `volume` | **Fixed at 8** | No upsert growth loop |
| Bootstrap routes (3) | **Static** | Not in emergent set; no idle decay (legacy) |

### Verdict: **BOUNDED** ✓

Route **topology** and **tick cost** are capped. Unused emergent routes decay. Conflict tears down emergent edges between warring factions.

---

## 3. New-coupling risks (field + feeding term + fix)

| ID | Severity | Field | Feeding term | Fix |
|----|----------|-------|--------------|-----|
| — | — | *(N3)* | — | **No risk flagged** |
| **[N4-S1]** | Low | `WorldState.faction_trade_agreement_streak[(a,b)]` | `record_trade_agreement_streak`: `+= 1` every TradeAgreement (500-tick cadence); reset only on Conflict; **no cap after route birth** | `streak = streak.min(TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD)` after increment, or `streak.remove(pair)` on successful birth |
| **[N4×U6]** | Pre-existing | `faction_treasury` | Emergent routes add `tick_trade_routes` profit each tick; N3 biases neighbor pairs → more agreements/routes among contacts → **amplifies** treasury drift | Treasury soft cap / progressive tax (same as [U6] in prior audit) |

**No new high-severity unbounded accumulator or positive-feedback loop** from N3/N4 beyond [N4-S1] (cosmetic scalar) and amplification of legacy [U6].

### Closed / bounded N3×N4 loops

```text
N3 contact pair → TradeAgreement → N4 route (capped)
  → tick_trade_routes (volume × multipliers ≤ 2× arbitrage)
  → relation_trade_factor (relation ∈ [-1, 1])
  → diplomacy threshold / outcome
Conflict arm → streak reset + route removal (negative feedback)
Idle arm → route removal after 2000 no-flow ticks (negative feedback)
at_cap → birth blocked (hard stop)
```

---

## 4. Prior audit flags on `origin/main` @ `73ef4b71`

| ID | Prior status (`eacd4361`) | This ref |
|----|---------------------------|----------|
| **[U5]** cluster food | BOUNDED | **BOUNDED** ✓ |
| **[R1]** `faction_unrest` | NOT BOUNDED | **BOUNDED** ✓ (`FACTION_UNREST_DECAY_DIVISOR = 200`, ~L2086–2109) |
| **[FC-3]** building_graph | NOT BOUNDED | **BOUNDED** ✓ (`building_materials_affordable` + wood/metal debit, ~L2287, ~L3662+) |
| **[U1]** `state.unrest` | Open | Open |
| **[U2]** `state.cohesion` | Open | Open |
| **[U3]** `research_progress` | Open | Open |
| **[U4]** `state.belief` | Open | Open |
| **[U6]** `faction_treasury` | Open | Open (N4 can increase flow) |

---

## 5. Phase scan delta (N3/N4 touchpoints only)

| Phase | N3/N4 change | Status |
|-------|--------------|--------|
| `phase_diplomacy` | N3 pair pick; N4 streak + route birth; conflict route teardown | **BOUNDED** (N4 caps + sinks) |
| `phase_economy` → `tick_trade_routes` | `decay_idle_emergent_trade_routes` | **BOUNDED** |
| `phase_life` / `phase_settlement_consumption` | Unchanged substrate for N3 reads (T−1 lag) | **CLEAN** |

---

## 6. Verdict

### N3 + N4 coupling delta

**CLEAN** for catastrophic criticality: no unbounded route graph, no uncapped birth loop, idle decay and conflict removal present.

**N4 route creation stays BOUNDED:** capped (`64`), decayed (emergent idle `2000` ticks), torn down on conflict.

**Residual (low):** [N4-S1] streak scalar drift; [N4×U6] neighbor-biased diplomacy may increase treasury accumulation rate on legacy unbounded field.

### Full engine (informational)

Prior legacy scalars [U1]–[U4] and [U6] remain the dominant long-horizon risks; they are **not introduced** by N3/N4.

---

## 7. Tick-order DAG (N3 / N4 slice)

```text
phase_life (T−1) → cluster_member_counts, ClusterMember
phase_diplomacy (T, every 500t)
  N3: diplomacy_pair_from_settlement_overlap
  → TradeAgreement/Conflict
  N4: streak → trade_routes.push (if guards pass)
phase_economy (T, every tick)
  tick_trade_routes → flow → decay_idle_emergent_trade_routes
```

---

*Generated by read-only static audit. No tests executed.*
