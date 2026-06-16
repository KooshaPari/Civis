# Rust hygiene scan — engine / economy / agents

**Date:** 2026-06-16  
**Scope:** `crates/engine/src`, `crates/economy/src`, `crates/agents/src`  
**Method:** Static ripgrep + manual read (no `cargo`, no `clippy` run)  
**Constraint:** Read-only; source not modified.

Likely Clippy families referenced below: `needless_collect`, `needless_pass_by_value` / needless clone, `map_unwrap_or`, `explicit_auto_deref`, `redundant_clone` / `str_to_string`, `iter_count` (iterator length), `unused` / dead API surface.

---

## Summary

| # | File:line | Smell | Severity |
|---|-----------|-------|----------|
| 1 | `engine/src/emergence.rs:586` | Clone vec then borrow | High |
| 2 | `engine/src/emergence.rs:536` | Clone vec for read-only loop | High |
| 3 | `engine/src/emergence.rs:502,512` | `.to_vec()` before immediate iteration | Medium |
| 4 | `engine/src/emergence.rs:553` | `.to_vec()` on diplomacy slice | Medium |
| 5 | `engine/src/emergence.rs:484` | `push(event.clone())` while `event` still used | Medium |
| 6 | `engine/src/emergence.rs:656,666` | Manual deref before `.clone()` | Low |
| 7 | `engine/src/emergence.rs:610-611` | Redundant `.clone()` on owned `String`s | Medium |
| 8 | `engine/src/engine.rs:2444` | Extra `engagements.clone()` | Medium |
| 9 | `engine/src/engine.rs:144` | Full `AgentCivilian` clone; only `id`/`age` needed | Medium |
| 10 | `engine/src/engine.rs:2749-2751` | `.iter().count()` on ECS queries | Low |
| 11 | `agents/src/lib.rs:536` | `.iter().count()` for civilian count | Low |
| 12 | `economy/src/allocator.rs:204-217,276-281` | `map(...).unwrap_or(0)` | Low |
| 13 | `economy/src/chains.rs:345,362` | `&String` → `.to_string()` | Low |
| 14 | `engine/src/engine.rs:870,934` | `format!("{}.civmod", …)` for extension | Low |
| 15 | `engine/src/lib.rs:209` | `pub fn create_rng` — no call sites | Medium (API) |

---

## Top 15 findings

### 1. Needless clone then borrow — `engine/src/emergence.rs:586`

```rust
for event in &self.emergence.last_feed.clone() {
```

**Likely lint:** `needless_collect` / needless clone  
**Fix:** Iterate the existing buffer: `for event in &self.emergence.last_feed {`

---

### 2. Clone entire vec for read-only loop — `engine/src/emergence.rs:536`

```rust
for event in self.emergence.last_sentience.clone() {
```

**Likely lint:** needless clone  
**Fix:** `for event in &self.emergence.last_sentience {`

---

### 3. `.to_vec()` before immediate iteration — `engine/src/emergence.rs:502,512`

```rust
for birth in self.last_births().to_vec() { ... }
for death in self.last_deaths().to_vec() { ... }
```

**Likely lint:** `needless_collect`  
**Fix:** Borrow the slice returned by the accessor, e.g. `for birth in self.last_births() {` (same for deaths), if the accessor already returns `&[T]`.

---

### 4. Diplomacy events copied unnecessarily — `engine/src/emergence.rs:553`

```rust
for dip in self.diplomacy_events().to_vec() {
```

**Likely lint:** `needless_collect`  
**Fix:** `for dip in self.diplomacy_events() {`

---

### 5. Clone on push while event still live — `engine/src/emergence.rs:484`

```rust
self.emergence.last_sentience.push(event.clone());
// … later uses event.cognition_score
```

**Likely lint:** redundant clone  
**Fix:** Hoist fields first, then move:  
`let cognition = event.cognition_score;`  
`self.emergence.last_sentience.push(event);`  
and use `cognition` in the `format!`.

---

### 6. Manual deref before clone — `engine/src/emergence.rs:656,666`

```rust
.map(|p| (*p).clone())
.map(|g| (*g).clone())
```

**Likely lint:** `explicit_auto_deref`  
**Fix:** `.map(|p| p.clone())` and `.map(|g| g.clone())` (or `.cloned()` if the component implements `Clone` through the ref).

---

### 7. Redundant `String` clones — `engine/src/emergence.rs:610-611`

```rust
prompt: prompt.clone(),
output: output.clone(),
```

**Likely lint:** redundant clone (owned bindings)  
**Fix:** Build `CivAiDecision` with `prompt`/`output` by move, then reference `decision.output` in the following `push_feed` `format!`, or reorder so `push_feed` runs before the struct is moved.

---

### 8. Duplicate engagement buffer — `engine/src/engine.rs:2444`

```rust
self.last_tick_engagements = engagements.clone();
for engagement in &engagements {
```

**Likely lint:** redundant clone  
**Fix:** Move once and reuse storage:  
`self.last_tick_engagements = engagements;`  
`for engagement in &self.last_tick_engagements {`

---

### 9. Over-cloning ECS component — `engine/src/engine.rs:144`

```rust
.map(|(entity, civilian)| (entity, civilian.clone()))
```

Only `civilian.id` and `civilian.age` are read in the loop.

**Likely lint:** needless clone / `cloned` on full struct  
**Fix:** Snapshot minimal fields:  
`.map(|(entity, civilian)| (entity, civilian.id, civilian.age))`  
and adjust the `Citizen` construction accordingly.

---

### 10. `.iter().count()` on hecs queries — `engine/src/engine.rs:2749-2751`

```rust
let citizen_count = self.world.query::<&Citizen>().iter().count();
let building_count = self.world.query::<&Building>().iter().count();
let military_count = self.world.query::<&MilitaryUnit>().iter().count();
```

**Likely lint:** `iter_count` (same class as slice `.iter().count()`)  
**Fix:** Prefer a direct count if the ECS API exposes one (e.g. `query.into_iter().count()` without an intermediate adapter, or a dedicated `count()` helper). At minimum, factor a small `count_query<T>(world: &World) -> usize` to avoid three copy-pasted patterns.

---

### 11. `.iter().count()` for civilians — `agents/src/lib.rs:536`

```rust
world.query::<&Civilian>().iter().count()
```

**Likely lint:** `iter_count`  
**Fix:** Same as #10 — shared helper or native query count; keeps hot-path tick code consistent with `engine::Simulation::snapshot`.

---

### 12. Manual `map` + `unwrap_or` — `economy/src/allocator.rs:204-217,276-281`

```rust
self.bids.get(a).map(|b| b.price).unwrap_or(0)
```

(repeated for bids, offers, and quantity checks)

**Likely lint:** `map_unwrap_or`  
**Fix:** `self.bids.get(a).map_or(0, |b| b.price)`

---

### 13. `&String` converted via `.to_string()` — `economy/src/chains.rs:345,362`

```rust
outcomes.push(ChainStepOutcome::skipped(name.to_string()));
// …
outcomes.push(ChainStepOutcome::fired(name.to_string(), recipe.joule_yield));
```

`name` comes from `BTreeMap<String, Recipe>::iter()` (`&String`).

**Likely lint:** `clone_on_ref_with_to_string` / inefficient conversion  
**Fix:** `name.clone()` (or change `ChainStepOutcome` to accept `impl Into<String>` and pass `name.clone()` once).

---

### 14. `format!` for file extension — `engine/src/engine.rs:870,934`

```rust
let archive = dir.join(format!("{}.civmod", name.to_string_lossy()));
```

**Likely lint:** `format_push_string` / needlessly allocated path  
**Fix:** When `dir` is the mod folder, prefer `dir.with_extension("civmod")` (or `PathBuf::from(name).with_extension("civmod")` resolved under `repo_root`) instead of allocating a formatted string.

---

### 15. Unused public API — `engine/src/lib.rs:209`

```rust
pub fn create_rng(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}
```

**Static reachability:** No references anywhere else in the workspace (callers use `SimRng::seed_from_u64` directly).

**Likely lint:** `dead_code` / `unused` (with `pub` visibility)  
**Fix:** Remove the wrapper, or wire callers to it; if it must stay for API stability, add `#[allow(dead_code)]` with a comment (least preferred).

**Honorable mentions (unused `pub`, same scan):**

| Symbol | Location | Notes |
|--------|----------|-------|
| `compute` / `compute_fixed` | `engine/src/metrics.rs` | Re-exported from `lib.rs`; only used in module tests |
| `poi_kind_for_need` | `agents/src/daily_path.rs:77` | Re-exported; inverse `need_for_poi_kind` is what production uses |
| `relation_label` | `agents/src/social.rs:109` | Re-exported; only referenced in unit tests |
| `should_tick_entity` | `engine/src/lod.rs:50` | Production path uses `should_tick_entity_with_policy` |
| `trade_gain` | `economy/src/stocks.rs:192` | Only exercised in `stocks` tests |

---

## Not observed in scope

- `push_str("x")` single-character patterns (none in these three crates’ `src/`).
- `.len() == 0` / `.len() > 0` (codebase already uses `.is_empty()` where checked).

---

## Limitations

- Static scan only: no CFG, no type-aware Clippy, no cross-crate `#[allow]` context.
- Some clones may be intentional for borrow-checker or determinism; verify before deleting.
- `unused pub` list is grep-based; dynamic linking, FFI, or macro-generated calls would not appear.
