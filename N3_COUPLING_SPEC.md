# N3 — Settlement Cluster Overlap → Diplomacy Pair Selection

**Status:** Research / design handoff (read-only audit, 2026-06-16)  
**Charter gap:** N3 in `EMERGENCE_CENSUS` / `.cursorlogs/audit-emergence-census.log` item 3 — `ClusterMember` co-location defines emergent settlements, but `phase_diplomacy` selects pairs from the static `WorldState::factions` registry by tick rotation; cluster overlap does not gate who negotiates.  
**Predecessors:** N1 (settlement `cluster_stocks` → market supply) and N2 (`CultureProfile` similarity → diplomacy threshold) are wired on emergence batches 17/24.  
**Scope:** Specify the **optimal minimal first coupling** for N3 (contacting settlements’ dominant alignments → diplomacy pair). No source changes in this artifact.

---

## 1. Census mapping (N1–N5)

| ID | Coupling (from `EMERGENCE_CENSUS`) | Audit ref | Status |
|----|-------------------------------------|-----------|--------|
| **N1** | Settlement `cluster_stocks` → market / faction supply | M2 | Wired |
| **N2** | `CultureProfile` distance → diplomacy / cohesion | M1 (diplomacy slice) | Wired |
| **N3** | **Cluster overlap → polity membership** | M4 | **This spec** |
| N4 | `dispossessed_permille` + institutions → production / labor | M5 | Deferred |
| N5 | Language emergence scaffold | Charter “Missing” layer | Deferred |

N3 is **clearly defined** in the census; this document specs its minimal v1 closure rather than substituting N5 (language) or a legends→belief bridge.

---

## 2. Survey — emergent settlement / cluster state

### 2.1 Micro components (`civ_agents`)

| Component / fn | Role |
|----------------|------|
| `ClusterMember { cluster: ClusterId }` | Written each tick in `phase_life` from co-location clustering |
| `cluster_by_colocation(positions, radius_fp)` | Single-link components; `ClusterId` = min agent id in component |
| `AgentCivilian.alignment` | `Alignment::Faction(u32)` when polity-affiliated; `None` otherwise |
| `Position3d` | Fixed-point coords for contact detection |

**Settlement definition (engine convention):** cluster with **≥ 2** members (`last_settlement_count`, `cluster_member_counts`).

### 2.2 Engine maintenance (`engine.rs`)

| Field | Updated by | Consumed by |
|-------|------------|-------------|
| `cluster_member_counts: BTreeMap<u64, u32>` | `phase_life` §5 | `phase_settlement_consumption`, HUD |
| `cluster_stocks` | `phase_life` §6 + consumption | HUD (N1 may add market supply) |
| `ClusterMember` on entities | `phase_life` §5 | `phase_emergence` (culture, psyche, social) |

**Cluster radius (production):** `cluster_radius = (0.06 * FIXED_SCALE) as i64` (`phase_life`, ~L2420).

### 2.3 Current diplomacy pair selection (`phase_diplomacy`, ~L2705)

```text
faction_ids = sorted(state.factions.keys())          // static registry {0, 1, 2, …}
a = faction_ids[tick % len]
b = faction_ids[(tick + 1) % len]
// threshold + treasury disparity + N2 culture bias → Conflict | TradeAgreement
// faction_relations keyed as ClusterId(u64::from(faction_id))  // partial identity bridge
```

**Gap:** Pair selection ignores whether factions **occupy** emergent settlements or **contact** each other on the map. A registered faction with zero settlement presence is still eligible every cadence tick.

### 2.4 Tick-order constraint

`phase_diplomacy` (#6) runs **before** `phase_life` (#13) in `tick_with_emergence_source`. N3 reads:

- `cluster_member_counts` from tick **T−1**
- `ClusterMember` + `Position3d` + `AgentCivilian` from tick **T−1**

Same one-tick lag pattern as N2 (`cluster_cultures` read at diplomacy tick). Acceptable: diplomacy cadence is 500 ticks; settlement layout is slow-moving.

---

## 3. Gap statement (N3)

| Layer | Evolves | Shapes diplomacy membership? |
|-------|---------|------------------------------|
| `ClusterMember` co-location | Yes, every tick | **No** |
| Settlement count / stocks | Yes | **No** (N1 adds economy only) |
| `AgentCivilian.alignment` | Spawn + infer | **No** for pair gating |
| `WorldState::factions` registry | Static at init | **Yes** (sole pair source today) |
| `faction_relations` matrix | Yes | Pairwise scores only; pairs not emergent |

**Charter intent (`docs/guides/emergence-charter.md`):** “Polities emerge from co-location + kinship + culture”; “membership is emergent cluster overlap, NOT `faction: u32`.”

**N3 v1 does not replace the faction registry** — it makes diplomacy **conditional on demonstrated settlement presence and geographic contact**, the smallest step toward emergent polity membership.

---

## 4. Optimal minimal first coupling

### 4.1 Choice: **adjacent-settlement dominant factions → diplomacy pair selection**

**Why this (not full polity migration, not legends→belief):**

| Alternative | Verdict |
|-------------|---------|
| Full `BTreeMap<u32, PolityMacroState>` per-polity phases | Correct end state (`MULTIPOLITY_DESIGN.txt`) but multi-phase; not minimal N3 |
| Replace `faction: u32` registry with dynamic polity ids | Save-format + UI break; out of N3 v1 |
| Round-robin only among factions with any settlement agent | Weaker — ignores **contact** (charter: co-location drives politics) |
| Legends graph salience → `belief` | Closes religion/**legends** gap (honorable mention in audit §7) but is **not N3**; schedule as N3-alt or post-N5 |
| `SocialGraph` tie density → pair pick | Indirect; settlement contact is the charter grain for polity overlap |
| Reorder `phase_diplomacy` after `phase_life` | Unnecessary given 1-tick lag at 500-tick cadence |

**Mechanism:** Derive each settlement’s **dominant faction** from alignment plurality among its members. Build **settlement contact edges** when two multi-member clusters have agents within contact radius. At each diplomacy event, select faction pair `(fa, fb)` from **contacting settlements with different dominant factions**; fallback gracefully to present-then-registry rotation.

### 4.2 Micro signal

**Per-agent row (read-only scan):**

```text
(agent_id, cluster_id, faction_id?, position)
  cluster_id  ← ClusterMember.cluster.0 (skip if missing)
  faction_id  ← Some(id) if Alignment::Faction(id), else None
  position    ← Position3d.coord
```

**Aggregates:**

1. **Settlement filter:** `cluster_member_counts[cluster_id] >= SETTLEMENT_MIN_MEMBERS` (constant `2`).

2. **Dominant faction per settlement:**

```text
fn settlement_dominant_factions(
    world: &hecs::World,
    cluster_member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u64, u32>   // cluster_id → winning faction_id
```

Logic: for each qualifying cluster, count `Alignment::Faction(id)` among members; plurality wins; ties broken by **smallest faction id** (deterministic). Clusters with zero explicit faction members are **omitted** (no invented alignment).

3. **Settlement contact pairs:**

```text
const SETTLEMENT_CONTACT_RADIUS_FP: i64 = cluster_radius * 2;   // 0.12 × FIXED_SCALE

fn settlement_contact_pairs(
    world: &hecs::World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    contact_radius_fp: i64,
) -> BTreeSet<(u64, u64)>   // canonical (min_cluster, max_cluster)
```

Logic: for each unordered settlement pair `(ca, cb)`, if ∃ agent `a ∈ ca`, agent `b ∈ cb` with squared distance ≤ `contact_radius_fp²`, insert `(min(ca,cb), max(ca,cb))`.

4. **Faction contact candidates:**

```text
fn diplomacy_faction_pairs_from_settlement_contact(
    dominant: &BTreeMap<u64, u32>,
    contacts: &BTreeSet<(u64, u64)>,
) -> Vec<(u32, u32)>   // sorted, deduped (min_f, max_f)
```

For each contact `(ca, cb)`: let `fa = dominant[ca]`, `fb = dominant[cb]`; if `fa != fb`, push `(min(fa,fb), max(fa,fb))`. Sort + dedupe.

### 4.3 Macro consumer

**New pure function** (suggested location: `engine.rs` next to `diplomacy_relation_threshold_bias`):

```text
fn diplomacy_pair_from_settlement_overlap(
    world: &hecs::World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    registered_factions: &[u32],   // sorted ascending
    tick: u64,
) -> (u32, u32)
```

**Selection order (deterministic):**

```text
let dominant = settlement_dominant_factions(world, cluster_member_counts);
let contacts = settlement_contact_pairs(world, cluster_member_counts, SETTLEMENT_CONTACT_RADIUS_FP);
let mut pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);

if !pairs.is_empty() {
    let idx = (tick as usize / 500) % pairs.len();
    return pairs[idx];
}

// Fallback A: factions with any settlement presence
let present: Vec<u32> = dominant.values().copied().collect::<BTreeSet<_>>().into_iter().collect();
if present.len() >= 2 {
    let idx = (tick as usize) % present.len();
    let a = present[idx];
    let b = present[(idx + 1) % present.len()];
    return (a, b);
}

// Fallback B: current registry rotation (preserve legacy sims with no settlements)
let idx = (tick as usize) % registered_factions.len();
return (
    registered_factions[idx],
    registered_factions[(idx + 1) % registered_factions.len()],
);
```

**Sink (replace two lines in `phase_diplomacy`):**

```text
// BEFORE:
// let a = faction_ids[(self.state.tick as usize) % faction_ids.len()];
// let b = faction_ids[((self.state.tick as usize) + 1) % faction_ids.len()];

let (a, b) = diplomacy_pair_from_settlement_overlap(
    &self.world,
    &self.cluster_member_counts,
    &faction_ids,
    self.state.tick,
);
```

All downstream threshold / treasury / N2 culture / relation logic stays unchanged.

### 4.4 Exact fields touched

| Read | Write |
|------|-------|
| `ClusterMember.cluster` | — |
| `AgentCivilian.alignment` | — |
| `Position3d.coord` | — |
| `cluster_member_counts` | — |
| `state.factions` keys (fallback) | — |
| — | `diplomacy_events[].faction_a/b` (indirect) |
| — | `faction_treasury`, `faction_relations` (existing side effects) |

**No new `WorldState` fields.** No serde migration. Empty settlement graph → Fallback B (today’s behavior).

---

## 5. Test specification

### 5.1 Unit test — dominant faction plurality

**Name:** `settlement_dominant_factions_picks_plurality_and_tiebreaks`  
**File:** `crates/engine/src/engine.rs` `#[cfg(test)]`

**Cases (hand-built `hecs::World`):**

```text
// Cluster 10: 3× Faction(1), 1× Faction(2) → dominant 1
// Cluster 20: 2× Faction(2) → dominant 2
// Cluster 30: 1 member only → omitted
// Cluster 40: 2× Alignment::None → omitted
```

Assert keys `{10: 1, 20: 2}` only.

### 5.2 Unit test — contact detection

**Name:** `settlement_contact_pairs_detects_adjacent_settlements`  
**File:** same

Two settlements pinned `contact_radius_fp` apart → one contact edge; far apart → empty.

### 5.3 Unit test — pair selection priority

**Name:** `diplomacy_pair_from_settlement_overlap_prefers_contact`  
**File:** same

```text
dominant = {10: 0, 20: 1}
contacts = {(10, 20)}
registered = [0, 1, 2]
tick = 500
→ (0, 1) even though faction 2 is registered
```

Second case: no contacts but `dominant = {10: 0, 30: 1}` → Fallback A returns `(0, 1)`.

Third case: empty dominant → Fallback B matches legacy `(tick % n, (tick+1) % n)`.

### 5.4 Integration test — diplomacy respects settlement contact

**Name:** `adjacent_settlements_select_diplomacy_pair_over_absent_faction`  
**Pattern:** mirror `phase_diplomacy_emerges_trade_among_peers`

**Setup:**

1. `Simulation::with_seed(42)`; default factions `{0, 1, 2}`.
2. Spawn **two adjacent settlement cohorts** (≥ 2 agents each, pinned positions):
   - Cohort A at `(FIXED_SCALE/2, FIXED_SCALE/2)`: 4 agents, `Alignment::Faction(0)`.
   - Cohort B at `(FIXED_SCALE/2 + contact_offset, FIXED_SCALE/2)`: 4 agents, `Alignment::Faction(1)`.
   - `contact_offset = SETTLEMENT_CONTACT_RADIUS_FP - 1` (within contact, separate clusters).
   - No agents aligned to faction `2`.
3. Run `sim.tick()` once (or manually set `cluster_member_counts` + insert `ClusterMember` to match post-`phase_life` state).
4. `sim.state.tick = 500`; pin treasuries equal (trade outcome); clear macro scalars like existing diplomacy tests.
5. `sim.phase_diplomacy()`.
6. **Assert:** last event `(faction_a, faction_b)` is `{0, 1}` in either order — **not** involving faction `2`.

**Control:** same layout but all agents `Alignment::None` → Fallback B; pair may include faction `2` (legacy path).

**Regression:** fresh `Simulation::with_seed(5)` with no settlements → pair identical to pre-N3 test expectations at tick 500.

---

## 6. What N3 v1 does *not* do

| Deferred | Rationale |
|----------|-----------|
| Per-polity macro shadow (`PolityMacroState` migration) | `MULTIPOLITY_DESIGN.txt` Phase 2+ |
| Dynamic polity creation / dissolution | Requires registry + save schema |
| Kinship / culture in contact weighting | N2 covers culture at threshold; contact is geographic v1 |
| `ClusterId` ≠ `faction_id` unification | Keep `ClusterId(u64::from(faction))` bridge for relations |
| Legends / chronicle → diplomacy or belief | Honorable mention tier; weakest **legends** layer, not N3 |
| Language vectors → anything | N5 scope |
| Reorder tick phases | 1-tick lag sufficient at 500-tick cadence |

---

## 7. Tick-order DAG (N3 slice)

```mermaid
flowchart LR
  PL_prev[phase_life T-1<br/>ClusterMember + counts]
  PD[phase_diplomacy T<br/>pair from settlement contact]
  PL[phase_life T<br/>refresh clusters]
  PL_prev --> PD
  PD --> PL
```

**Depends on:** at least one prior `phase_life` with multi-member clusters before first gated diplomacy event.  
**Composes with:** N2 culture bias on same `(a, b)` pair; N1 supply term unaffected.

---

## 8. Phased WBS (follow-on)

| Phase | Task ID | Description | Depends on |
|-------|---------|-------------|------------|
| 1 | **N3-A** | `settlement_dominant_factions` + `settlement_contact_pairs` + unit tests | — |
| 2 | N3-A2 | `diplomacy_pair_from_settlement_overlap` + `phase_diplomacy` wire | N3-A |
| 3 | N3-A-int | Integration test `adjacent_settlements_select_diplomacy_pair_over_absent_faction` | N3-A2 |
| 4 | N3-B | Contact → `faction_relations.apply_signal` (small trade/contact nudge) | N3-A2 |
| 5 | N3-C | Dominant faction → per-faction unrest shadow split | N3-A2, multipolity |
| 6 | N3-D | Full `PolityMacroState` per settlement cluster | MULTIPOLITY_DESIGN |

**Agent effort (aggressive):** N3-A + N3-A2 + tests ≈ 10–15 tool calls, ~5 min wall clock.

---

## 9. Cross-project reuse

| Candidate | Location | Notes |
|-----------|----------|-------|
| `cluster_by_colocation` | `civ_agents::cluster` | Already tested; N3 consumes positions + radius |
| `settlement_dominant_factions` | `civ-emergence-metrics` or `civ_agents` | Pure; optional split mirroring N2 pattern |
| Contact radius constant | Share `cluster_radius` from `phase_life` | Extract `SETTLEMENT_CLUSTER_RADIUS_FP` to avoid drift |

---

## 10. References

| Artifact | Path |
|----------|------|
| Diplomacy phase + pair pick | `crates/engine/src/engine.rs` (`phase_diplomacy`, ~L2705) |
| Settlement clustering + counts | `crates/engine/src/engine.rs` (`phase_life`, ~L2413) |
| Cluster primitives | `crates/agents/src/cluster.rs` |
| Agent alignment | `crates/agents/src/lib.rs` (`Alignment::Faction`) |
| N2 precedent | `N2_CULTURE_DIPLOMACY_SPEC.md` |
| Coupling audit M4 | `EMERGENCE_COUPLING_AUDIT.txt` §7 M4 |
| Emergence census N3 | `.cursorlogs/audit-emergence-census.log` item 3 |
| Multipolity roadmap | `MULTIPOLITY_DESIGN.txt` |
| Emergence charter | `docs/guides/emergence-charter.md` |

---

## 11. Summary

**Gap:** Emergent settlements exist (`ClusterMember`, `cluster_member_counts`), but diplomacy pairs are chosen from the static faction registry by tick index — cluster overlap does not determine who negotiates.

**Minimal closure:** Before treasury/threshold logic, compute **dominant faction per settlement** and **contact edges between adjacent settlements**; select the diplomacy pair from **cross-faction contact** first, then settlement presence, then legacy registry rotation.

**Proof:** Unit tests on dominant-faction plurality, contact detection, and selection priority; integration test showing adjacent Faction(0) and Faction(1) settlements negotiate at tick 500 while registered Faction(2) with no settlement is excluded.

**Not chosen instead:** Legends→belief or language scaffold (N5) — valuable for the weakest **language/legends** charter layers but explicitly **N4/N5/census item 5**, not N3.
