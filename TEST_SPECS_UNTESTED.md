# TEST_SPECS_UNTESTED.md

> Read-only test-authoring audit for the three untested pure policy helpers
> in `crates/engine/src/engine.rs` flagged by `COVERAGE_GAPS_4.md`:
> `job_type_for_civilian_id`, `faction_wealth_scarcity_shadow`, and
> `faction_unrest_delta_from_shadow`. The blocks below are **ready-to-paste**
> `#[test]` functions; drop them into the existing
> `#[cfg(test)] mod tests { use super::*; ... }` block at
> `crates/engine/src/engine.rs:4378` (the same module already imports
> `JobType`, `Resources`, `Fixed`, `FOOD_SCARCITY_BASELINE`, `unrest_delta`
> and `crate::SCALE` through `use super::*;`).
>
> **Provenance.** All signatures and bodies were copied verbatim from
> `crates/engine/src/engine.rs` at the line numbers cited. Real public
> types/constructors were used throughout:
> - `JobType` enum at `engine.rs:116-125` (with `#[derive(... PartialEq, Eq, ...)]`)
> - `Resources { food, wood, metal, energy }` struct at `engine.rs:216-222`
>   (derives `Default` and `Clone`, but **not** `Copy`)
> - `Fixed { raw: i64 }` from `crates/engine/src/lib.rs:96` with
>   `Fixed::from_num(n)`, `Fixed::ZERO`, and `crate::SCALE = 1_000_000`
> - `FOOD_SCARCITY_BASELINE: i64 = 1_000` at `engine.rs:3277`
>
> **No source edits, no cargo runs, no commits were made** in the production
> of this spec — it is a planning artifact.

---

## 1. `job_type_for_civilian_id` — `engine.rs:127-138`

### Signature
```rust
pub fn job_type_for_civilian_id(id: u64) -> JobType
```

### Body (for reference)
```rust
pub fn job_type_for_civilian_id(id: u64) -> JobType {
    match id % 7 {
        0 => JobType::Farmer,
        1 => JobType::Warrior,
        2 => JobType::Scholar,
        3 => JobType::Trader,
        4 => JobType::Priest,
        5 => JobType::Admin,
        _ => JobType::Unemployed,
    }
}
```

### Edge cases covered
- all seven mod-buckets (`id % 7 ∈ {0..=6}`)
- wrap-around at the modulus (`id = 7` re-enters the `0` bucket)
- `id % 7 == 6` falls through the catch-all `_` arm → `Unemployed`
- sparse id in a far-out range still resolves to the same bucket as its
  remainder (`1_000_000_007 % 7 == 0`)
- `u64::MAX % 7 == 1` (verified: `2_635_249_153_387_078_802 * 7 + 1`) → `Warrior`
- determinism (same `id` → same `JobType`; no state, no panic)

### Test code (drop into `mod tests` at `engine.rs:4378`)

```rust
    /// `job_type_for_civilian_id` covers every mod-7 bucket, including the
    /// catch-all `_` arm for remainder 6. (COVERAGE_GAPS_4 row 1.)
    #[test]
    fn job_type_for_civilian_id_all_seven_buckets() {
        assert_eq!(job_type_for_civilian_id(0), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(1), JobType::Warrior);
        assert_eq!(job_type_for_civilian_id(2), JobType::Scholar);
        assert_eq!(job_type_for_civilian_id(3), JobType::Trader);
        assert_eq!(job_type_for_civilian_id(4), JobType::Priest);
        assert_eq!(job_type_for_civilian_id(5), JobType::Admin);
        assert_eq!(job_type_for_civilian_id(6), JobType::Unemployed);
    }

    /// `id % 7` wraps cleanly: every 7th id resolves to the same `JobType`
    /// (idempotence over the modulus).
    #[test]
    fn job_type_for_civilian_id_mod_wraps_around_seven() {
        assert_eq!(job_type_for_civilian_id(7), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(14), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(42), JobType::Farmer); // 42 % 7 == 0
        assert_eq!(job_type_for_civilian_id(13), JobType::Unemployed); // 13 % 7 == 6
        assert_eq!(job_type_for_civilian_id(20), JobType::Unemployed); // 20 % 7 == 6
    }

    /// Sparse, far-out ids still resolve to the right bucket via `id % 7`.
    /// 1_000_000_007 % 7 == 0 because 1_000_000_007 = 142_857_143 * 7 + 6 — wait,
    /// 1_000_000_007 / 7 = 142_857_143 remainder 6; corrected below to 1_000_000_007 → Unemployed.
    /// Using 1_000_000_008 (1_000_000_008 / 7 = 142_857_144 remainder 0) → Farmer.
    /// The point is: huge ids land in a deterministic bucket, not a panic.
    #[test]
    fn job_type_for_civilian_id_sparse_id_in_resolved_bucket() {
        // 1_000_000_008 % 7 == 0 (1_000_000_008 = 142_857_144 * 7)
        assert_eq!(job_type_for_civilian_id(1_000_000_008), JobType::Farmer);
        // 999_999_999 % 7: 999_999_999 / 7 = 142_857_142 remainder 5 → Admin
        assert_eq!(job_type_for_civilian_id(999_999_999), JobType::Admin);
        // 1_000_000_000_000_000_000 % 7 = 6 (large id, sparse coverage) → Unemployed
        assert_eq!(
            job_type_for_civilian_id(1_000_000_000_000_000_000),
            JobType::Unemployed
        );
    }

    /// `u64::MAX % 7 == 1` (u64::MAX = 2^64 - 1 = 18_446_744_073_709_551_615,
    /// = 2_635_249_153_387_078_802 * 7 + 1) → `JobType::Warrior`. Confirms the
    /// function is total over the full `u64` range, no overflow.
    #[test]
    fn job_type_for_civilian_id_u64_max() {
        assert_eq!(job_type_for_civilian_id(u64::MAX), JobType::Warrior);
    }

    /// Determinism: same id → same `JobType`, no state. Two consecutive
    /// calls must agree; this is what keeps the spawn palette stable across
    /// seeds (FR-CIV-ENGINE spawn determinism).
    #[test]
    fn job_type_for_civilian_id_is_deterministic() {
        for id in [0u64, 1, 6, 7, 42, 100, 999_999_999, u64::MAX] {
            assert_eq!(
                job_type_for_civilian_id(id),
                job_type_for_civilian_id(id),
                "job_type_for_civilian_id({id}) must be a pure function of its input"
            );
        }
    }
```

---

## 2. `faction_wealth_scarcity_shadow` — `engine.rs:3348-3366`

### Signature
```rust
fn faction_wealth_scarcity_shadow(treasury: Fixed, resources: &Resources) -> i64
```

> **Visibility.** This function is private (`fn`, no `pub`). It is
> reachable from the in-file `#[cfg(test)] mod tests` block via
> `use super::*;` at `engine.rs:4380` (same pattern as `unrest_delta` and
> `food_scarcity_birth_factor` use today).

### Body (for reference)
```rust
fn faction_wealth_scarcity_shadow(treasury: Fixed, resources: &Resources) -> i64 {
    const TREASURY_COMFORT: i64 = 8_000;
    const FOOD_COMFORT: i64 = 80;
    const FOOD_WEIGHT: i64 = 50;

    let treasury_i = (treasury.raw / crate::SCALE).max(0);
    let food_i = (resources.food.raw / crate::SCALE).max(0);
    let comfort = TREASURY_COMFORT + FOOD_COMFORT * FOOD_WEIGHT;
    let wealth = treasury_i + food_i * FOOD_WEIGHT;

    if wealth >= comfort {
        FOOD_SCARCITY_BASELINE
    } else {
        FOOD_SCARCITY_BASELINE + (comfort - wealth) / 4
    }
}
```

### Constants
- `TREASURY_COMFORT = 8_000`
- `FOOD_COMFORT = 80`
- `FOOD_WEIGHT = 50`
- `comfort = 8_000 + 80 * 50 = 12_000`
- `FOOD_SCARCITY_BASELINE = 1_000` (`engine.rs:3277`)
- `crate::SCALE = 1_000_000` (`crates/engine/src/lib.rs:101`)

### Edge cases covered
- **comfort threshold (≥ branch):** `treasury + food*50 >= 12_000` →
  shadow equals `FOOD_SCARCITY_BASELINE` exactly (`1_000`).
- **empty `Resources` (all fields zero) + zero treasury:** wealth = 0,
  shadow = `1_000 + 12_000/4 = 4_000` — the deepest "deep scarcity" reachable
  in one call (no clamp at the top inside the function; the function only
  floors the shadow at `FOOD_SCARCITY_BASELINE` for the comfort branch).
- **food-only shortfall:** treasury = 0, food = 10 → wealth = 500 →
  shadow = `1_000 + 11_500/4 = 1_000 + 2_875 = 3_875`.
- **treasury-only shortfall:** treasury = 4_000, food = 0 → wealth = 4_000 →
  shadow = `1_000 + 8_000/4 = 3_000`. (COVERAGE_GAPS_4 prose claims "treasury
  above comfort → shadow = 0 hedges"; the function does not implement that
  hedge — `treasury_i` is additive in the same units as the food-weighted
  wealth, so treasury alone cannot push the shadow to baseline unless it
  covers the entire `12_000` comfort. The tests assert what the code does.)
- **exact boundary:** wealth = `12_000` → shadow = `1_000` (the `>=` branch).
- **shadow never below `FOOD_SCARCITY_BASELINE`:** every legal input
  produces a shadow `>= 1_000`, matching the COVERAGE_GAPS_4 row
  "saturates at SCARCITY_BASELINE" claim.
- **raw / SCALE conversion:** `treasury.raw = 5_000 * SCALE` →
  `treasury_i = 5_000`; this guards against a regression that would drop
  the `/ SCALE` term and treat `raw` directly as a wealth value.

### Test code (drop into `mod tests` at `engine.rs:4378`)

```rust
    /// Comfort branch: when `treasury_i + food_i*50 >= 12_000` the shadow
    /// pins to `FOOD_SCARCITY_BASELINE` (= 1_000). (COVERAGE_GAPS_4 row 5.)
    #[test]
    fn faction_wealth_scarcity_shadow_above_comfort_returns_baseline() {
        // treasury=100_000, food=10_000 → wealth = 100_000 + 10_000*50 = 600_000
        let res = Resources {
            food: Fixed::from_num(10_000),
            wood: Fixed::ZERO,
            metal: Fixed::ZERO,
            energy: Fixed::ZERO,
        };
        let shadow = faction_wealth_scarcity_shadow(Fixed::from_num(100_000), &res);
        assert_eq!(shadow, FOOD_SCARCITY_BASELINE);
    }

    /// Exact comfort boundary: `wealth == 12_000` still pins to baseline
    /// because the function uses `>=` (not strict `>`).
    #[test]
    fn faction_wealth_scarcity_shadow_exactly_at_comfort() {
        // treasury=12_000, food=0 → wealth = 12_000 + 0 = 12_000
        let res = Resources::default();
        let shadow = faction_wealth_scarcity_shadow(Fixed::from_num(12_000), &res);
        assert_eq!(shadow, FOOD_SCARCITY_BASELINE);
    }

    /// Empty `Resources` + zero treasury = "deep scarcity": wealth = 0,
    /// shadow = 1_000 + 12_000/4 = 4_000. (No upper clamp inside the
    /// function; the COVERAGE_GAPS_4 "saturates at SCARCITY_BASELINE"
    /// claim refers to the LOWER floor, which we cover below.)
    #[test]
    fn faction_wealth_scarcity_shadow_empty_resources_zero_treasury() {
        let res = Resources::default();
        let shadow = faction_wealth_scarcity_shadow(Fixed::ZERO, &res);
        assert_eq!(
            shadow,
            FOOD_SCARCITY_BASELINE + 12_000 / 4,
            "empty Resources + zero treasury lands at the maximum shadow"
        );
    }

    /// Food-only shortfall: treasury = 0, food = 10 → wealth = 500.
    /// Shadow = 1_000 + (12_000 - 500)/4 = 1_000 + 2_875 = 3_875.
    #[test]
    fn faction_wealth_scarcity_shadow_food_only_shortfall() {
        let res = Resources {
            food: Fixed::from_num(10),
            wood: Fixed::ZERO,
            metal: Fixed::ZERO,
            energy: Fixed::ZERO,
        };
        let shadow = faction_wealth_scarcity_shadow(Fixed::ZERO, &res);
        assert_eq!(shadow, FOOD_SCARCITY_BASELINE + (12_000 - 500) / 4);
    }

    /// Treasury-only shortfall: treasury = 4_000, food = 0 → wealth = 4_000.
    /// Shadow = 1_000 + (12_000 - 4_000)/4 = 3_000. The function does NOT
    /// implement the "treasury hedges food" claim from COVERAGE_GAPS_4 prose
    /// (treasury is additive in the same units as the food-weighted wealth,
    /// not a separate hedge channel) — this test pins the actual behavior.
    #[test]
    fn faction_wealth_scarcity_shadow_treasury_only_shortfall() {
        let res = Resources::default();
        let shadow = faction_wealth_scarcity_shadow(Fixed::from_num(4_000), &res);
        assert_eq!(shadow, FOOD_SCARCITY_BASELINE + (12_000 - 4_000) / 4);
    }

    /// The shadow never falls below `FOOD_SCARCITY_BASELINE` (= 1_000) for
    /// any legal input: the comfort branch pins to it, the shortfall branch
    /// adds to it. COVERAGE_GAPS_4 row 5 "saturates at SCARCITY_BASELINE"
    /// reads as "lower-bound at SCARCITY_BASELINE" — this is the floor.
    #[test]
    fn faction_wealth_scarcity_shadow_floor_at_baseline() {
        let cases: Vec<(i64, Resources)> = vec![
            (0, Resources::default()),
            (10_000, Resources::default()),
            (0, Resources { food: Fixed::from_num(1), ..Resources::default() }),
            (Fixed::from_num(5_000).raw, Resources::default()), // 5_000 * SCALE raw
            (Fixed::from_num(99_999_999).raw, Resources::default()),
        ];
        for (treasury_raw, res) in cases {
            let treasury = Fixed { raw: treasury_raw };
            let shadow = faction_wealth_scarcity_shadow(treasury, &res);
            assert!(
                shadow >= FOOD_SCARCITY_BASELINE,
                "shadow ({shadow}) fell below FOOD_SCARCITY_BASELINE ({FOOD_SCARCITY_BASELINE})"
            );
        }
    }

    /// `treasury.raw / SCALE` is the integer wealth — the function divides
    /// the raw `i64` by `crate::SCALE` (1_000_000) before mixing with
    /// `food_i`. This guards against a regression that would treat
    /// `treasury.raw` directly as a wealth value (e.g. dropping the `/ SCALE`
    /// would make treasury = 5_000 * SCALE look like wealth = 5e9 and
    /// trivially satisfy the comfort branch — but that is not the
    /// semantic of the field).
    #[test]
    fn faction_wealth_scarcity_shadow_treasury_raw_is_divided_by_scale() {
        // treasury = 5_000 in fixed-point → treasury_i = 5_000
        // food = 0 → food_i = 0 → wealth = 5_000 < 12_000 → shortfall branch
        // shadow = 1_000 + (12_000 - 5_000) / 4 = 1_000 + 1_750 = 2_750
        let res = Resources::default();
        let treasury = Fixed::from_num(5_000);
        let shadow = faction_wealth_scarcity_shadow(treasury, &res);
        assert_eq!(shadow, FOOD_SCARCITY_BASELINE + (12_000 - 5_000) / 4);
    }
```

---

## 3. `faction_unrest_delta_from_shadow` — `engine.rs:3368-3372`

### Signature
```rust
fn faction_unrest_delta_from_shadow(scarcity_shadow: i64) -> i64
```

> **Visibility.** Private (no `pub`); reachable from the in-file
> `#[cfg(test)] mod tests` via `use super::*;`.

### Body (for reference)
```rust
fn faction_unrest_delta_from_shadow(scarcity_shadow: i64) -> i64 {
    unrest_delta(scarcity_shadow)
}
```

It is a thin wrapper that delegates to `unrest_delta` (private, defined at
`engine.rs:3333`). `unrest_delta` itself is:

```rust
fn unrest_delta(food_price: i64) -> i64 {
    const MAX_RISE: i64 = 50;
    const CENTS_PER_UNREST: i64 = 20;
    const DECAY: i64 = 10;
    let scarcity = food_price - FOOD_SCARCITY_BASELINE;
    if scarcity > 0 {
        (scarcity / CENTS_PER_UNREST).clamp(1, MAX_RISE)
    } else {
        -DECAY
    }
}
```

### Edge cases covered
- **shadow at or below `FOOD_SCARCITY_BASELINE`:** `scarcity <= 0` →
  return `-DECAY = -10` (decay, negative). The "clamp at 0" referenced in
  COVERAGE_GAPS_4 row 6 is enforced by the **caller** of this delta (the
  running `faction_unrest` total is floored at zero in
  `phase_faction_unrest`); the delta itself only knows `-10` and `[1, 50]`.
- **shadow just above baseline:** `scarcity ∈ [1, 19]` → integer divide
  by 20 → `0` → `.clamp(1, 50) = 1` (minimum rise).
- **shadow scaling with shortfall:** `shadow = 1_100` → scarcity = 100 →
  delta = 5; `shadow = 1_400` → scarcity = 400 → delta = 20.
- **shadow at the upper bound:** `scarcity = 1_000` → 1_000/20 = 50 →
  clamp(1, 50) = 50; larger shadows stay at 50.
- **shadow at extreme (`i64::MAX`):** still returns 50 (the
  `.clamp(1, MAX_RISE)` ceiling holds against `i64::MAX`).
- **identity with `unrest_delta`:** the wrapper is a pure pass-through, so
  the test pins `faction_unrest_delta_from_shadow(s) == unrest_delta(s)`
  for a representative range.

### Test code (drop into `mod tests` at `engine.rs:4378`)

```rust
    /// `shadow <= FOOD_SCARCITY_BASELINE` produces a *decay* delta of `-10`
    /// (not zero, not positive). The function never returns zero: the
    /// running-total "clamp at 0" in COVERAGE_GAPS_4 row 6 lives in the
    /// caller's accumulator, not in this delta. (COVERAGE_GAPS_4 row 6.)
    #[test]
    fn faction_unrest_delta_from_shadow_below_baseline_decays() {
        for shadow in [0i64, 100, 500, 999] {
            let delta = faction_unrest_delta_from_shadow(shadow);
            assert_eq!(
                delta, -10,
                "shadow={shadow} (below baseline) must decay by 10"
            );
        }
    }

    /// At the boundary `shadow == FOOD_SCARCITY_BASELINE` the function
    /// takes the `else` branch (scarcity is not `> 0`) and returns `-10`,
    /// not zero. Pin this so a future `>=` refactor doesn't silently flip
    /// the boundary into the rise branch.
    #[test]
    fn faction_unrest_delta_from_shadow_at_baseline_decays() {
        let delta = faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE);
        assert_eq!(delta, -10);
    }

    /// Just above baseline, the rise is clamped to a minimum of `+1` (the
    /// `clamp(1, MAX_RISE)` lower bound kicks in for any `scarcity > 0`,
    /// even when `scarcity / 20 == 0`).
    #[test]
    fn faction_unrest_delta_from_shadow_just_above_baseline_minimal_rise() {
        assert_eq!(faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 1), 1);
        assert_eq!(faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 19), 1);
    }

    /// The rise scales linearly with the shortfall (`scarcity / 20`) until
    /// it hits the `MAX_RISE` ceiling of 50. (COVERAGE_GAPS_4 row 6: "sign
    /// of shadow" — when shadow > baseline, delta is positive and bounded.)
    #[test]
    fn faction_unrest_delta_from_shadow_scales_with_shortfall_then_clamps() {
        // shadow = 1_100 → scarcity = 100 → 100 / 20 = 5
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 100),
            5
        );
        // shadow = 1_400 → scarcity = 400 → 400 / 20 = 20
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 400),
            20
        );
        // shadow = 2_000 → scarcity = 1_000 → 1_000 / 20 = 50 (at ceiling)
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 1_000),
            50
        );
    }

    /// Large shadows still clamp to `MAX_RISE = 50`. (COVERAGE_GAPS_4 row
    /// 6: "large shadow".) This is what stops a price spike from
    /// instantly maxing faction unrest.
    #[test]
    fn faction_unrest_delta_from_shadow_large_shadow_still_clamps_at_50() {
        for shadow in [10_000i64, 1_000_000, 1_000_000_000, i64::MAX] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                50,
                "shadow={shadow} must clamp at MAX_RISE=50"
            );
        }
    }

    /// The wrapper is a pure pass-through to `unrest_delta` — pin that
    /// identity across the full sign range so a future refactor can't
    /// diverge the two without breaking a test.
    #[test]
    fn faction_unrest_delta_from_shadow_is_identity_with_unrest_delta() {
        for shadow in [
            0i64,
            FOOD_SCARCITY_BASELINE - 1,
            FOOD_SCARCITY_BASELINE,
            FOOD_SCARCITY_BASELINE + 1,
            FOOD_SCARCITY_BASELINE + 100,
            FOOD_SCARCITY_BASELINE + 1_000,
            FOOD_SCARCITY_BASELINE + 100_000,
            i64::MAX,
        ] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                unrest_delta(shadow),
                "wrapper must equal unrest_delta at shadow={shadow}"
            );
        }
    }
```

---

## 4. Test-count summary

| # | Function | `#[test]` count | Assertion summary |
|---|----------|-----------------|-------------------|
| 1 | `job_type_for_civilian_id` | 5 | all 7 mod-buckets (incl. `_`-arm); `id % 7` wrap; sparse id resolved correctly; `u64::MAX` → `Warrior`; determinism |
| 2 | `faction_wealth_scarcity_shadow` | 7 | comfort branch pins to `FOOD_SCARCITY_BASELINE`; exact `12_000` boundary; empty `Resources` → `4_000`; food-only shortfall; treasury-only shortfall; lower floor at `1_000`; `treasury.raw / SCALE` conversion |
| 3 | `faction_unrest_delta_from_shadow` | 6 | decay below baseline (`-10`); decay at boundary (`-10`); minimum rise `+1` just above baseline; linear scaling with shortfall; clamp at `MAX_RISE = 50` (incl. `i64::MAX`); identity with `unrest_delta` |

Total: **18** ready-to-paste `#[test]` functions, all referencing real types
(`JobType`, `Resources`, `Fixed`) and real constants (`FOOD_SCARCITY_BASELINE`,
`crate::SCALE`) — no mocks, no source edits to `engine.rs`.

## 5. Paste instructions

1. Open `crates/engine/src/engine.rs`.
2. Scroll to the existing `#[cfg(test)] mod tests {` block at
   `engine.rs:4378`. The block already contains `use super::*;` at L4380,
   which makes `job_type_for_civilian_id`, `faction_wealth_scarcity_shadow`,
   `faction_unrest_delta_from_shadow`, `unrest_delta`, `JobType::*`,
   `Resources`, `Fixed::*`, `FOOD_SCARCITY_BASELINE`, and `crate::SCALE`
   all directly in scope. **No additional `use` statements are required.**
3. Append the 18 test fns from sections 1, 2, and 3 above to the end of
   the module (immediately before the closing `}` of `mod tests` at the
   end of the file, near `engine.rs:7708`).
4. Run `cargo test -p civ-engine` (per `AGENTS.md` "Verify before you
   claim done" — though this spec was authored in read-only mode; the
   paste + run is left to the human agent).
