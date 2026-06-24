# RND-003: Fixed-Point Arithmetic -- `fixed` Crate vs Manual i64 x SCALE

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**Recommendation: Hybrid approach.** Use three numeric strategies depending on the domain:

1. **`i64` with `SCALE = 1_000_000`** for large-magnitude values (energy in Joules, population
   resources, GDP) where `I32F32` would overflow.
2. **`fixed` crate `I32F32`** (via `FixedI32\<U32\>` or more precisely `FixedI32\<U16\>` for
   range) for ratio/rate values (growth rates, efficiency percentages, tax rates, happiness
   scores) where the values stay in a bounded range and operator ergonomics matter.
3. **`cordic` crate** for trigonometric functions needed by the climate/solar angle subsystem,
   operating on `fixed` types.

The `fixed` crate (v1.30.x) provides `no_std` support, serde, and operator overloading but
**no built-in trig functions**. The `cordic` crate (v0.3.x) fills this gap with CORDIC-based
`sin`, `cos`, `atan2` for fixed-point types. For Joule-scale values, `I32F32` overflows at
~2.1 billion (2^31) which is insufficient for 100k citizens x 1 TJ = 10^17 Joules. These
values must use `i64 x SCALE` where the i64 range (+-9.2 x 10^18) accommodates even extreme
scenarios.

---

## Research Findings

### 1. The Determinism Requirement

CivLab's simulation must be bit-for-bit deterministic across:
- Different machines (x86_64, aarch64, WASM)
- Different Rust compiler versions
- Multiple runs with identical inputs

**Why no `f32`/`f64`:**
- IEEE 754 permits implementation-defined behavior for NaN payloads.
- FMA (fused multiply-add) produces different results than separate mul+add.
- x87 FPU uses 80-bit extended precision internally; SSE uses 32/64-bit. Mixing yields
  different results.
- WASM uses IEEE 754 strictly but may differ from native due to FMA availability.
- Compiler optimizations (e.g., fast-math, reassociation) can change float results.
- The Rust language does not guarantee deterministic float operations across platforms.

**Conclusion:** All simulation arithmetic must use integer or fixed-point types.

### 2. `fixed` Crate Analysis (v1.30.0)

**Repository:** [gitlab.com/tspiteri/fixed](https://gitlab.com/tspiteri/fixed)
**License:** MIT/Apache-2.0
**MSRV:** Rust 1.85.0

#### 2.1 Available Types

| Type | Bits | Signed | Fractional Bits | Integer Range | Fractional Precision |
|------|------|--------|-----------------|---------------|---------------------|
| `FixedI8\<UX\>` | 8 | Yes | 0-8 | Depends on X | Depends on X |
| `FixedI16\<UX\>` | 16 | Yes | 0-16 | Depends on X | Depends on X |
| `FixedI32\<UX\>` | 32 | Yes | 0-32 | Depends on X | Depends on X |
| `FixedI64\<UX\>` | 64 | Yes | 0-64 | Depends on X | Depends on X |
| `FixedI128\<UX\>` | 128 | Yes | 0-128 | Depends on X | Depends on X |
| `FixedU*` variants | * | No | * | * | * |

**Key configurations for CivLab:**

| Type Alias | Type | Integer Bits | Frac Bits | Integer Range | Precision |
|-----------|------|-------------|-----------|---------------|-----------|
| `I32F32` | Not a real type -- `FixedI32\<U32\>` has 0 integer bits! | 0 | 32 | -0.5..0.5 | ~2.3e-10 |
| `FixedI32\<U16\>` | 32-bit | 16 | 16 | -32768..32767 | ~1.5e-5 |
| `FixedI32\<U8\>` | 32-bit | 24 | 8 | -8388608..8388607 | ~0.004 |
| `FixedI64\<U32\>` | 64-bit | 32 | 32 | -2^31..2^31-1 | ~2.3e-10 |

**IMPORTANT CORRECTION:** The notation "I32F32" commonly refers to a 64-bit type with 32
integer bits and 32 fractional bits -- i.e., `FixedI64\<U32\>`. A `FixedI32\<U32\>` has zero
integer bits (range -0.5 to 0.5), which is useless for most purposes. CivLab should use:
- `FixedI64\<U32\>` for "I32F32" semantics (32 int + 32 frac, 64 bits total)
- `FixedI32\<U16\>` for "I16F16" semantics (16 int + 16 frac, 32 bits total)

#### 2.2 Overflow Analysis

**Scenario: 100k citizens x 1 TJ (10^12 Joules)**

Total energy = 100,000 x 10^12 = 10^17 Joules.

| Type | Max Value | Overflows? |
|------|-----------|------------|
| `FixedI32\<U16\>` (I16F16) | 32,767 | YES -- overflows at 32k |
| `FixedI64\<U32\>` (I32F32) | ~2.15 x 10^9 | YES -- overflows at 2.1 billion |
| `i64 x SCALE(10^6)` | ~9.2 x 10^12 | YES if SCALE=10^6, max representable = 9.2e12 |
| `i64` (raw, no scale) | ~9.2 x 10^18 | NO -- 10^17 fits comfortably |
| `FixedI128\<U32\>` | ~1.7 x 10^29 | NO -- but 128-bit math is slow |

**Finding:** For Joule-scale values, even `FixedI64\<U32\>` overflows. The only viable options
are:
1. **`i64` with reduced scale (SCALE=1000 or SCALE=100):** Max = 9.2e15 or 9.2e16 --
   sufficient for 10^17 with SCALE=100.
2. **`i64` without scale (integer Joules):** If we don't need sub-Joule precision for energy.
   Most energy calculations don't need fractional Joules.
3. **`i128`:** Overkill and slower on 32-bit / WASM targets.
4. **Domain-specific units:** Store energy in kJ or MJ instead of J. 10^17 J = 10^14 kJ =
   10^11 MJ. `FixedI64\<U32\>` handles 10^11 MJ fine (max ~2.1e9 with fraction... still tight).
   10^11 MJ > 2.1e9. Still overflows.
5. **`i64` with SCALE = 1_000 (milliJoules? No -- scale for sub-unit precision):**
   For energy: use raw `i64` Joules (no fractional precision needed).
   For rates: use `FixedI32\<U16\>` or `FixedI64\<U32\>`.

**Recommendation:** Energy values use plain `i64` (integer Joules or kiloJoules). No scaling
needed because sub-Joule precision is unnecessary for a civilization simulation. Rates and
ratios use `fixed` types.

#### 2.3 Arithmetic Operations

The `fixed` crate provides full operator overloading:

```rust
use fixed::types::extra::U16;
use fixed::FixedI32;

type Fix = FixedI32<U16>;

let a = Fix::from_num(3.5);
let b = Fix::from_num(2.0);
let c = a + b;          // 5.5
let d = a * b;          // 7.0
let e = a / b;          // 1.75
let f = a % b;          // 1.5
let g = -a;             // -3.5
let cmp = a > b;        // true
```

**Overflow behavior:**
- Default operations (`+`, `-`, `*`, `/`) panic on overflow in debug, wrap in release.
- `checked_*` variants return `Option\<Self\>`.
- `saturating_*` variants clamp to min/max.
- `wrapping_*` variants explicitly wrap.
- `strict_*` (renamed from `unwrapped_*` in v1.30) always panic on overflow.

**CivLab policy:** Use `checked_*` operations in simulation code, with explicit error
propagation. Overflow in a simulation is a logic bug that should be detected and reported,
not silently wrapped.

#### 2.4 Conversion Operations

```rust
// From integer:
let x = Fix::from_num(42);         // 42.0
let x = Fix::from_num(42_i64);     // 42.0

// From float (compile-time only for determinism):
let x = Fix::lit("3.14159");       // Exact binary representation of closest value

// To integer:
let n: i32 = x.to_num();           // Truncates toward zero
let n: i32 = x.round().to_num();   // Rounds to nearest

// To/from raw bits:
let bits: i32 = x.to_bits();       // Raw bit representation
let x = Fix::from_bits(bits);      // From raw bits (for serialization)
```

#### 2.5 Features and Compatibility

```toml
[dependencies]
fixed = { version = "1.30", features = ["serde"] }  # optional serde
```

- `no_std` by default (only `serde-str` requires `std`)
- `serde` feature: serializes as the numeric value (not raw bits)
- WASM compatible: all operations are pure integer math
- `#[repr(transparent)]` over the underlying integer type -- same layout as `i32`/`i64`

### 3. `cordic` Crate Analysis (v0.3.x)

**Repository:** [github.com/sebcrozet/cordic](https://github.com/sebcrozet/cordic)
**License:** BSD-3-Clause
**Dependency:** `fixed = "^1"` (compatible with our version)

#### 3.1 Available Functions

```rust
use cordic;
use fixed::types::extra::U16;
use fixed::FixedI32;

type Fix = FixedI32<U16>;

// Trigonometric
let angle = Fix::from_num(1.0);  // 1 radian
let s = cordic::sin(angle);       // sine
let c = cordic::cos(angle);       // cosine
let (s, c) = cordic::sin_cos(angle);  // both at once (faster)
let t = cordic::tan(angle);       // tangent

// Inverse trigonometric
let a = cordic::asin(Fix::from_num(0.5));  // arcsine
let a = cordic::acos(Fix::from_num(0.5));  // arccosine
let a = cordic::atan(Fix::from_num(1.0));  // arctangent
let a = cordic::atan2(y, x);              // atan2 with quadrant correction

// Other
let r = cordic::sqrt(Fix::from_num(2.0));  // square root
```

#### 3.2 Precision

CORDIC achieves precision proportional to the number of fractional bits:
- With 16 fractional bits (`FixedI32\<U16\>`): ~4-5 decimal digits of precision
- With 32 fractional bits (`FixedI64\<U32\>`): ~9-10 decimal digits of precision

For climate angle calculations (solar altitude, latitude effects), 4-5 digits of precision
is more than sufficient. The sun angle doesn't need sub-arcsecond precision for a civilization
game.

#### 3.3 Determinism

CORDIC algorithms are fully deterministic:
- Pure integer arithmetic internally (shift-and-add)
- Lookup table based (compile-time generated)
- No floating-point operations
- Platform-independent results for the same input type

#### 3.4 Alternative: `fixed_trigonometry` Crate

```toml
[dependencies]
fixed_trigonometry = "0.3"  # Alternative to cordic
```

- Also provides `sin`, `cos`, `tan`, `atan2` for `fixed` types
- `no_std` compatible
- Uses polynomial approximation rather than CORDIC iterations
- Similar precision (~4-5 digits for 16 frac bits)

Either `cordic` or `fixed_trigonometry` works. `cordic` is more established and has `sqrt`.

### 4. Manual `i64 x SCALE` Approach

The manual approach uses plain `i64` with a constant scale factor:

```rust
/// Scale factor: 1_000_000 = 10^6
/// This gives 6 decimal digits of fractional precision.
const SCALE: i64 = 1_000_000;

/// A "fixed-point" value is just an i64 where the real value = raw / SCALE.
type Scaled = i64;

fn from_int(n: i64) -> Scaled { n * SCALE }
fn from_frac(n: i64, d: i64) -> Scaled { n * SCALE / d }
fn to_int(s: Scaled) -> i64 { s / SCALE }
fn mul(a: Scaled, b: Scaled) -> Scaled { a * b / SCALE }
fn div(a: Scaled, b: Scaled) -> Scaled { a * SCALE / b }
```

**Pros:**
- Zero dependencies
- Full control over scale factor
- Easy to understand and debug
- `i64` range: +-9.2 x 10^18 / SCALE = +-9.2 x 10^12 representable values

**Cons:**
- **No operator overloading:** Must use `mul(a, b)` instead of `a * b`. Every arithmetic
  expression becomes verbose and error-prone.
- **Scale factor discipline:** Every multiplication must divide by SCALE, every division must
  multiply by SCALE. Forgetting produces wrong results silently.
- **Overflow risk in intermediates:** `a * b` can overflow `i64` before the `/ SCALE`
  normalization. Need `i128` intermediates or careful ordering.
- **No type safety:** A `Scaled` value and a raw `i64` are the same type. Can accidentally
  mix them (e.g., add a scaled value to an unscaled one).
- **No trig functions:** Must implement CORDIC or polynomial approximation from scratch.

### 5. Comparison Summary

| Criterion | `fixed` crate | Manual i64 x SCALE | Winner |
|-----------|---------------|---------------------|--------|
| Type safety | Strong (distinct type) | None (just i64) | `fixed` |
| Operator overloading | Yes (`+`, `-`, `*`, `/`) | No (function calls) | `fixed` |
| Overflow detection | `checked_*` variants | Manual | `fixed` |
| Range for energy | FixedI64\<U32\>: +-2.1e9 | i64: +-9.2e12 (SCALE=10^6) | Manual |
| Trig functions | Via `cordic` | Must implement | `fixed` |
| Serde | Built-in feature | Manual | `fixed` |
| no_std | Yes | Yes | Tie |
| WASM compat | Yes | Yes | Tie |
| Dependencies | 1 crate | 0 crates | Manual |
| Ergonomics | Excellent | Poor | `fixed` |
| Precision control | Per-type (U8, U16, U32) | Per-constant (SCALE) | `fixed` |
| Debuggability | `.to_num::\<f64\>()` for display | `raw / SCALE` | Tie |

---

## Decision

**Hybrid approach: domain-specific numeric types.**

### Domain 1: Large-Magnitude Values (Energy, Resources, GDP)

**Use `i64` with domain-specific units, no fractional scaling.**

- Energy: store in **kiloJoules (kJ)** as raw `i64`. Range: +-9.2e18 kJ = +-9.2e21 J.
  Even 10^17 J = 10^14 kJ, well within range.
- Resources (food, minerals): store in **grams** or **kilograms** as raw `i64`.
- GDP / currency: store in **milli-credits** as raw `i64`.

No need for `SCALE` constant -- the unit itself provides the precision. This avoids the
error-prone manual scaling math.

Newtype wrappers for type safety:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct KiloJoules(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MilliCredits(pub i64);
```

### Domain 2: Ratios, Rates, Bounded Values

**Use `fixed` crate types.**

| Value | Type | Range | Precision | Justification |
|-------|------|-------|-----------|---------------|
| Growth rate | `FixedI32\<U16\>` | -32768..32767 | ~1.5e-5 | Rates are small numbers (0.01-0.10 typical) |
| Tax rate | `FixedI32\<U16\>` | -32768..32767 | ~1.5e-5 | 0.0-1.0 range |
| Happiness | `FixedI32\<U16\>` | -32768..32767 | ~1.5e-5 | 0.0-100.0 range |
| Efficiency | `FixedI32\<U16\>` | -32768..32767 | ~1.5e-5 | 0.0-1.0 multiplier |
| Temperature | `FixedI32\<U16\>` | -32768..32767 | ~1.5e-5 | Kelvin (200-400 typical) |
| Latitude/angle | `FixedI32\<U16\>` | -32768..32767 | ~1.5e-5 | Radians (-pi to pi) |

### Domain 3: Trigonometric Computations

**Use `cordic` crate with `FixedI32\<U16\>` inputs.**

Only needed for:
- Solar angle calculation (climate system)
- Latitude-based temperature modifiers
- Possibly orbital mechanics if the game includes planetary features

---

## Implementation Contract

### Cargo.toml

```toml
[dependencies]
fixed = { version = "1.30", features = ["serde"] }
cordic = "0.3"
```

### Type Aliases Module

```rust
// In crates/engine/src/numeric.rs

use fixed::types::extra::{U16, U32};
use fixed::{FixedI32, FixedI64};

// === Domain 1: Large Magnitudes (i64 newtypes) ===

/// Energy in kiloJoules. Range: +-9.2e18 kJ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct KiloJoules(pub i64);

/// Currency in milli-credits (1 credit = 1000 milli-credits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct MilliCredits(pub i64);

/// Mass in grams. Range: +-9.2e18 grams = +-9.2e15 kg.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Grams(pub i64);

/// Population count. Plain integer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Population(pub i64);

// Implement basic arithmetic for newtypes via macro:
macro_rules! impl_i64_newtype_ops {
    ($T:ident) => {
        impl std::ops::Add for $T {
            type Output = Self;
            fn add(self, rhs: Self) -> Self { $T(self.0.checked_add(rhs.0).expect(concat!(stringify!($T), " overflow"))) }
        }
        impl std::ops::Sub for $T {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self { $T(self.0.checked_sub(rhs.0).expect(concat!(stringify!($T), " overflow"))) }
        }
        // Multiply by scalar (not by same type -- that changes units)
        impl std::ops::Mul<i64> for $T {
            type Output = Self;
            fn mul(self, rhs: i64) -> Self { $T(self.0.checked_mul(rhs).expect(concat!(stringify!($T), " overflow"))) }
        }
        impl std::ops::Div<i64> for $T {
            type Output = Self;
            fn div(self, rhs: i64) -> Self { $T(self.0.checked_div(rhs).expect(concat!(stringify!($T), " division"))) }
        }
    };
}

impl_i64_newtype_ops!(KiloJoules);
impl_i64_newtype_ops!(MilliCredits);
impl_i64_newtype_ops!(Grams);
impl_i64_newtype_ops!(Population);

// === Domain 2: Ratios and Rates (fixed-point) ===

/// 32-bit fixed-point with 16 integer bits and 16 fractional bits.
/// Range: -32768.0 to ~32767.99998. Precision: ~0.0000153.
/// Use for: rates, ratios, percentages, temperatures, small multipliers.
pub type Ratio = FixedI32<U16>;

/// 64-bit fixed-point with 32 integer bits and 32 fractional bits.
/// Range: -2,147,483,648 to ~2,147,483,647.999. Precision: ~2.3e-10.
/// Use for: high-precision rates where I16F16 is insufficient.
pub type HiRatio = FixedI64<U32>;

// === Domain 3: Angles (fixed-point with trig) ===

/// Angle in radians, stored as FixedI32<U16>.
/// Range: -32768..32767 radians (way more than 2*pi).
/// Precision: ~0.0000153 radians (~0.0009 degrees).
pub type Angle = FixedI32<U16>;

/// Compute sine of an angle. Fully deterministic (CORDIC algorithm).
pub fn sin(angle: Angle) -> Ratio {
    cordic::sin(angle)
}

/// Compute cosine of an angle. Fully deterministic.
pub fn cos(angle: Angle) -> Ratio {
    cordic::cos(angle)
}

/// Compute both sine and cosine. More efficient than calling each separately.
pub fn sin_cos(angle: Angle) -> (Ratio, Ratio) {
    cordic::sin_cos(angle)
}

/// Compute arctangent with quadrant correction.
pub fn atan2(y: Ratio, x: Ratio) -> Angle {
    cordic::atan2(y, x)
}

/// Compute square root. Input must be non-negative.
pub fn sqrt(value: Ratio) -> Ratio {
    cordic::sqrt(value)
}
```

### Usage Examples

#### Climate System (Trig)

```rust
use crate::numeric::{Angle, Ratio, sin, cos, sin_cos};
use fixed::traits::FromFixed;

/// Calculate solar altitude angle based on latitude and day-of-year.
/// All computations are fixed-point. No f32/f64.
pub fn solar_altitude(
    latitude: Angle,     // radians
    day_of_year: i32,    // 1-365
    hour: i32,           // 0-23
) -> Angle {
    // Earth's axial tilt: ~23.44 degrees = 0.4091 radians
    let tilt = Angle::lit("0.4091");

    // Declination angle: delta = tilt * sin(2*pi*(day-80)/365)
    let two_pi = Angle::lit("6.2832");
    let day_angle = two_pi * (day_of_year - 80) / 365;
    let declination = Ratio::from_num(tilt) * sin(day_angle);
    let declination = Angle::from_num(declination);

    // Hour angle: omega = pi/12 * (hour - 12)
    let hour_angle = Angle::lit("0.2618") * (hour - 12); // pi/12 ~= 0.2618

    // Solar altitude: sin(alt) = sin(lat)*sin(dec) + cos(lat)*cos(dec)*cos(ha)
    let (sin_lat, cos_lat) = sin_cos(latitude);
    let (sin_dec, cos_dec) = sin_cos(declination);
    let cos_ha = cos(hour_angle);

    let sin_alt = sin_lat * sin_dec + cos_lat * cos_dec * cos_ha;

    // Return altitude as angle (asin would be needed for exact angle,
    // but sin_alt as a ratio is sufficient for temperature modifiers)
    Angle::from_num(sin_alt)
}
```

#### Economy System (Large Values)

```rust
use crate::numeric::{KiloJoules, MilliCredits, Ratio};

/// Calculate food production for a tile.
/// Energy uses KiloJoules (i64), efficiency uses Ratio (fixed-point).
pub fn calculate_food_production(
    base_energy: KiloJoules,
    soil_fertility: Ratio,    // 0.0 - 1.0
    technology_bonus: Ratio,  // 1.0 = no bonus, 1.5 = 50% bonus
) -> KiloJoules {
    // Multiply base energy by fertility ratio
    // fixed * i64 -> i64: convert ratio to scaled integer multiplication
    let fertility_scaled = soil_fertility.to_num::<i64>();  // Truncates to integer
    // Better: use intermediate fixed-point for the multiplication
    let base_ratio = Ratio::from_num(base_energy.0 / 1000); // Scale down to fit Ratio
    let production_ratio = base_ratio * soil_fertility * technology_bonus;
    let production_kj = production_ratio.to_num::<i64>() * 1000; // Scale back up

    KiloJoules(production_kj)
}
```

#### Mixing Domains Safely

```rust
use crate::numeric::{KiloJoules, Ratio, Population};

/// Calculate per-capita energy consumption.
/// Returns a Ratio (energy per person per tick), not a KiloJoules.
pub fn per_capita_consumption(
    total_energy: KiloJoules,
    population: Population,
) -> Ratio {
    // i64 / i64 -> Ratio: careful to preserve precision
    // Option 1: If total_energy.0 / population.0 fits in Ratio range
    if population.0 == 0 {
        return Ratio::ZERO;
    }
    // Use i64 division with remainder for precision
    let quotient = total_energy.0 / population.0;
    let remainder = total_energy.0 % population.0;
    Ratio::from_num(quotient)
        + Ratio::from_num(remainder) / Ratio::from_num(population.0.min(i32::MAX as i64))
}
```

### Clippy Lint Configuration

```rust
// In crates/engine/src/lib.rs or build.rs:
//
// Deny all floating-point usage in the simulation crate.
#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]
#![deny(clippy::float_cmp_const)]
// These lints catch accidental f32/f64 usage at compile time.
```

### Serialization Contract

```rust
/// All numeric types serialize deterministically:
/// - i64 newtypes: serialize as JSON numbers (integer)
/// - Ratio/HiRatio/Angle: serialize as JSON numbers (decimal via serde feature)
///
/// For binary serialization (network/snapshot):
/// - i64 newtypes: little-endian i64 bytes
/// - Fixed-point: .to_bits() -> i32/i64 -> little-endian bytes
///
/// NEVER serialize fixed-point via to_num::<f64>() -- this introduces f64
/// and loses precision. Always use to_bits() for binary formats.

#[cfg(test)]
mod serde_tests {
    use super::*;

    #[test]
    fn fixed_point_roundtrips_via_bits() {
        let original = Ratio::lit("3.14159");
        let bits = original.to_bits();
        let restored = Ratio::from_bits(bits);
        assert_eq!(original, restored);
    }

    #[test]
    fn kj_roundtrips_via_json() {
        let original = KiloJoules(1_000_000_000);
        let json = serde_json::to_string(&original).unwrap();
        let restored: KiloJoules = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}
```

### Cross-Domain Conversion Safety Rules

```text
RULES FOR NUMERIC DOMAIN CROSSING:

1. i64 newtype -> Ratio:
   - Only when value fits in Ratio range (-32768..32767).
   - Scale down first if needed (e.g., divide kJ by 1000 before converting).
   - Use checked conversion: Ratio::checked_from_num(value).expect("overflow")

2. Ratio -> i64 newtype:
   - Use .to_num::<i64>() which truncates toward zero.
   - Or .round().to_num::<i64>() for rounding.
   - Wrap in appropriate newtype: KiloJoules(ratio.to_num())

3. Ratio * i64 newtype:
   - Convert the i64 to Ratio first (if in range), multiply, convert back.
   - OR: multiply i64 by ratio's numerator, divide by denominator.
   - Example: kj.0 * ratio.to_bits() as i64 / (1 << 16)

4. FORBIDDEN:
   - Never convert to f32/f64 for intermediate computation.
   - Never use .to_num::<f32>() in simulation code.
   - Never construct Ratio from runtime f32 values.
   - Compile-time float literals: use Ratio::lit("3.14") (parsed at compile time).
```

---

## Open Questions Remaining

1. **`FixedI32\<U16\>` vs `FixedI64\<U32\>` for Ratio:** The current recommendation uses
   `FixedI32\<U16\>` for most rates. If any rate computation involves multiplication of two
   rates (rate * rate), the intermediate product may lose significant precision with only
   16 fractional bits. Profiling needed to determine if `FixedI64\<U32\>` is needed for any
   hot paths. Preliminary recommendation: start with `FixedI32\<U16\>`, upgrade to
   `FixedI64\<U32\>` only for domains where precision matters (e.g., compound interest over
   many ticks).

2. **Overflow handling policy:** The contract specifies `checked_*` operations that panic
   on overflow. In production, panicking in the simulation is a hard crash. Alternative:
   use `saturating_*` for non-critical values (e.g., happiness saturates at max rather than
   crashing). Need to define per-domain overflow policy. Preliminary: panic on financial
   overflow (indicates a game balance bug), saturate on cosmetic values.

3. **Compile-time float literal precision:** `Ratio::lit("3.14159")` rounds to the nearest
   representable fixed-point value. With U16 fractional bits, `3.14159` becomes `3.14154...`
   (error ~5e-5). For game-critical constants (PI, e, tilt angle), verify that the rounded
   values are acceptable. They should be -- the game doesn't need 10-digit precision for
   any physical constant.

4. **Performance of `cordic` vs lookup table:** For trig functions called every tick for every
   tile (climate system), CORDIC's iterative algorithm may be slower than a precomputed lookup
   table. For 16-bit precision, a 65536-entry lookup table for sin/cos is only 256KB and gives
   O(1) lookup. Profile both approaches. `cordic` is the safer starting point.

5. **Interaction with MCTS rollouts:** MCTS (RND-011) will run lightweight forward simulations.
   These must use the same numeric types for determinism. Verify that the simplified rollout
   model can operate efficiently with fixed-point types (no conversion overhead in hot loops).

---

## References

- [fixed crate docs.rs](https://docs.rs/fixed/latest/fixed/)
- [fixed crate crates.io](https://crates.io/crates/fixed)
- [cordic crate docs.rs](https://docs.rs/cordic/latest/cordic/)
- [fixed_trigonometry crate](https://crates.io/crates/fixed_trigonometry)
- [Working with Fixed-Point Numbers in Rust](https://blog.implrust.com/posts/2025/12/fixed-point-crate-in-rust/)
- [CORDIC algorithm (Wikipedia)](https://en.wikipedia.org/wiki/CORDIC)
- [Deterministic lockstep networking](https://gafferongames.com/post/deterministic_lockstep/)
- [IEEE 754 cross-platform issues](https://randomascii.wordpress.com/2013/07/16/floating-point-determinism/)
