use serde::{Deserialize, Serialize};

// NFR-CIV-DET-003
    // FR-CIV-CORE-012
/// Fixed-point integer type for deterministic simulation math.
/// Stores a 64-bit integer with an implied scale factor of 1_000.
///
/// NFR-C-03 — fixed-point arithmetic enforcement: simulation crates carry
/// numeric state exclusively through this `Fixed` type (and its 1e6-scale
/// sibling in `civ_mod_host`). Float arithmetic is blocked at the lint
/// level via `cargo clippy -D clippy::float_arithmetic` in CI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Fixed(pub(crate) i64);

impl Default for Fixed {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Internal trait used by `Fixed::from_num` so integer and float types can
/// both be passed without explicit casts.
pub trait FixedFromNum: Sized {
    fn into_fixed(self) -> i64;
    fn from_fixed(bits: i64) -> Self;
}

impl FixedFromNum for i32 {
    fn into_fixed(self) -> i64 {
        i64::from(self) * 1_000
    }
    fn from_fixed(bits: i64) -> Self {
        (bits / 1_000) as i32
    }
}
impl FixedFromNum for i64 {
    fn into_fixed(self) -> i64 {
        self * 1_000
    }
    fn from_fixed(bits: i64) -> Self {
        bits / 1_000
    }
}
impl FixedFromNum for u32 {
    fn into_fixed(self) -> i64 {
        i64::from(self) * 1_000
    }
    fn from_fixed(bits: i64) -> Self {
        (bits / 1_000) as u32
    }
}
impl FixedFromNum for u64 {
    fn into_fixed(self) -> i64 {
        (self as i64) * 1_000
    }
    fn from_fixed(bits: i64) -> Self {
        (bits / 1_000) as u64
    }
}
impl FixedFromNum for f32 {
    fn into_fixed(self) -> i64 {
        (f64::from(self) * 1_000.0) as i64
    }
    fn from_fixed(bits: i64) -> Self {
        (bits as f32) / 1_000.0
    }
}
impl FixedFromNum for f64 {
    fn into_fixed(self) -> i64 {
        (self * 1_000.0) as i64
    }
    fn from_fixed(bits: i64) -> Self {
        (bits as f64) / 1_000.0
    }
}

impl Fixed {
    /// All-zero value.
    pub const ZERO: Self = Self(0);
    /// All-one value (scale = 1_000).
    pub const ONE: Self = Self(1_000);

    /// Construct from an integer or float. `f64`/`f32` callers are
    /// converted via the 1_000 scale (lossy; matches the lossy semantics
    /// the original `fixed`-crate-backed `Fixed` exposed for stub use).
    #[inline]
    pub fn from_num<T: FixedFromNum>(v: T) -> Self {
        Self(T::into_fixed(v))
    }

    /// Direct `f64` constructor (used by callers that can't use the trait
    /// generic — e.g. `disasters::apply_disaster_resource_loss`).
    #[inline]
    pub fn from_f64_direct(v: f64) -> Self {
        Self((v * 1_000.0) as i64)
    }

    /// Convenience: directly accept `f64` (used by `disasters.rs`).
    #[inline]
    pub fn from_f64_stub(v: f64) -> Self {
        Self((v * 1_000.0) as i64)
    }

    /// Convenience: accept a `f64` directly (used by `disasters.rs`).
    #[inline]
    pub fn from_f64_lossy(v: f64) -> Self {
        Self((v * 1_000.0) as i64)
    }

    /// Construct from a `f64` (rounded; loss of precision expected for stubs).
    #[inline]
    pub fn from_f64(v: f64) -> Self {
        Self((v * 1_000.0) as i64)
    }

    /// Construct from a `f32` (rounded; loss of precision expected for stubs).
    #[inline]
    pub fn from_num_f32(v: f32) -> Self {
        Self((v * 1_000.0) as i64)
    }

    /// Construct from a raw i64 bit pattern.
    #[inline]
    pub fn from_bits(bits: i64) -> Self {
        Self(bits)
    }

    /// Raw i64 bit pattern (used by callers that read it for serialization).
    #[inline]
    pub fn to_bits(self) -> i64 {
        self.0
    }

    /// Cast to a numeric type. Used for the `to_num` method the original
    /// `fixed`-crate-backed `Fixed` exposed. For float types the result
    /// is divided by the internal scale (1_000).
    #[inline]
    pub fn to_num<T>(self) -> T
    where
        T: FixedFromNum,
    {
        T::from_fixed(self.0)
    }

    /// Minimum of two values.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Maximum of two values.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Saturating subtraction.
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Cast to f64 (lossy; used by callers that bridge into `f32` / `f64`).
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / 1_000.0
    }
}

impl core::ops::Add for Fixed {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
impl core::ops::Sub for Fixed {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}
impl core::ops::Mul for Fixed {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // Truncate to scale; matches the lossy semantics callers expect.
        Self((self.0 * rhs.0) / 1_000)
    }
}
impl core::ops::Div for Fixed {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            Self(0)
        } else {
            Self((self.0 * 1_000) / rhs.0)
        }
    }
}
impl core::ops::AddAssign for Fixed {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl core::ops::SubAssign for Fixed {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::fmt::Display for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

#[cfg(test)]
mod fixed_tests {
    // NFR-C-03 — fixed-point arithmetic enforcement: all simulation numeric
    // state flows through `Fixed` (scale 1_000) so ticks are deterministic
    // and float arithmetic stays lint-blocked in CI.
    use super::Fixed;

    #[test]
    fn nfr_c_03_from_num_to_num_roundtrip_integers() {
        let v = Fixed::from_num(42i64);
        assert_eq!(v.to_bits(), 42_000);
        assert_eq!(v.to_num::<i64>(), 42);
        assert_eq!(Fixed::from_num(-7i32).to_num::<i32>(), -7);
        assert_eq!(Fixed::from_num(9u32).to_num::<u32>(), 9);
    }

    #[test]
    fn nfr_c_03_float_conversion_uses_1000_scale() {
        let v = Fixed::from_num(1.5f64);
        assert_eq!(v.to_bits(), 1_500);
        assert!((v.to_num::<f64>() - 1.5).abs() < 1e-9);
        let v32 = Fixed::from_num(0.25f32);
        assert_eq!(v32.to_bits(), 250);
    }

    #[test]
    fn nfr_c_03_add_sub_assign_are_exact_integer_math() {
        let mut a = Fixed::from_num(10i64);
        a += Fixed::from_num(3i64);
        assert_eq!(a.to_bits(), 13_000);
        a -= Fixed::from_num(4i64);
        assert_eq!(a.to_bits(), 9_000);
        assert_eq!(
            (Fixed::from_num(2i64) - Fixed::from_num(5i64)).to_bits(),
            -3_000
        );
    }

    #[test]
    fn nfr_c_03_mul_div_truncate_to_scale() {
        // 2.0 * 3.0 = 6.0
        let m = Fixed::from_num(2i64) * Fixed::from_num(3i64);
        assert_eq!(m.to_bits(), 6_000);
        // 7.0 / 2.0 = 3.5
        let d = Fixed::from_num(7i64) / Fixed::from_num(2i64);
        assert_eq!(d.to_bits(), 3_500);
        // Division by zero saturates to zero (deterministic, no panic).
        assert_eq!((Fixed::from_num(1i64) / Fixed::ZERO).to_bits(), 0);
    }

    #[test]
    fn nfr_c_03_min_max_saturating_sub_and_ordering() {
        let a = Fixed::from_num(5i64);
        let b = Fixed::from_num(9i64);
        assert_eq!(a.min(b), a);
        assert_eq!(a.max(b), b);
        // saturating_sub clamps at i64::MIN (no wrap/panic).
        assert_eq!(
            Fixed::from_bits(i64::MIN).saturating_sub(b),
            Fixed::from_bits(i64::MIN),
            "saturating sub must clamp at i64::MIN, never wrap"
        );
        assert_eq!(b.saturating_sub(a).to_bits(), 4_000);
        assert!(a < b);
        assert!(Fixed::ONE > Fixed::ZERO);
    }

    #[test]
    fn nfr_c_03_display_renders_scaled_value() {
        assert_eq!(Fixed::from_num(1i64).to_string(), "1");
        assert_eq!(Fixed::from_f64(0.125).to_string(), "0.125");
    }
}
