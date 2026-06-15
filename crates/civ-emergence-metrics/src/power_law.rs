//! Power-law fit metric.
//!
//! See `docs/design/emergence-dashboard.md` §3.4 for the design rationale.
//!
//! The metric consumes a rank-frequency histogram (e.g. city sizes, trade
//! volumes, cluster populations) and returns the exponent `α` of the
//! best-fit power law `P(k) ∝ k^(-α)` estimated via linear regression on
//! log-log data, plus the coefficient of determination `R²`.
//!
//! A true power-law process (e.g. preferential attachment) yields
//! `α ≈ 2..3` with `R² > 0.95`; a uniform or exponential distribution
//! yields `α ≈ 0` and `R² ≪ 0.9`.
//!
//! ## Why linear regression on log-log?
//!
//! Maximum-likelihood Clauset–Shalizi–Newman fitting is more accurate
//! but requires `O(n)` sorting and an expensive Kolmogorov–Smirnov
//! minimiser. For the dashboard's 50-tick cadence and the small sample
//! sizes typical of early-game clusters (8–64 points), ordinary least
//! squares on the log-log CCDF is accurate enough and compiles in
//! ~20 lines of std-only code.

use crate::{Histogram, Metric};

/// Result of a power-law fit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerLawResult {
    /// Exponent `α` of the best-fit `P(k) ∝ k^(-α)`.
    pub alpha: f32,
    /// Coefficient of determination `R²` in `[0, 1]`.
    /// Values near `1.0` indicate a strong power-law signal.
    pub r_squared: f32,
}

/// Power-law fit metric.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PowerLawFit;

impl PowerLawFit {
    /// Construct a new instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Fit a power law to a rank-frequency histogram.
    ///
    /// The histogram bins are treated as frequencies in rank order
    /// (bin 0 = rank 1, bin 1 = rank 2, …).  Empty bins are ignored.
    /// Returns `alpha = 0.0, r_squared = 0.0` when there are fewer
    /// than two non-empty bins or when the regression is degenerate.
    #[must_use]
    pub fn compute_rank_frequency(&self, input: &Histogram) -> PowerLawResult {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for (rank, &count) in input.bins().iter().enumerate() {
            if count > 0 {
                xs.push((rank + 1) as f32);
                ys.push(count as f32);
            }
        }
        let n = xs.len();
        if n < 2 {
            return PowerLawResult {
                alpha: 0.0,
                r_squared: 0.0,
            };
        }
        // Log-log transform.
        let log_x: Vec<f32> = xs.iter().map(|&x| x.ln()).collect();
        let log_y: Vec<f32> = ys.iter().map(|&y| y.ln()).collect();
        let mean_x = log_x.iter().sum::<f32>() / n as f32;
        let mean_y = log_y.iter().sum::<f32>() / n as f32;
        let mut ss_xx = 0.0_f32;
        let mut ss_xy = 0.0_f32;
        let mut ss_yy = 0.0_f32;
        for i in 0..n {
            let dx = log_x[i] - mean_x;
            let dy = log_y[i] - mean_y;
            ss_xx += dx * dx;
            ss_xy += dx * dy;
            ss_yy += dy * dy;
        }
        if ss_xx == 0.0 || ss_yy == 0.0 {
            return PowerLawResult {
                alpha: 0.0,
                r_squared: 0.0,
            };
        }
        let slope = ss_xy / ss_xx;
        let r = ss_xy / (ss_xx * ss_yy).sqrt();
        PowerLawResult {
            alpha: -slope,
            r_squared: r * r,
        }
    }
}

impl Metric for PowerLawFit {
    const NAME: &'static str = "power_law_alpha";

    fn compute(&self, input: &Histogram) -> f32 {
        self.compute_rank_frequency(input).alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 5e-3,
            "expected {b} got {a} (|diff|={})",
            (a - b).abs()
        );
    }

    #[test]
    fn power_law_fit_on_perfect_data() {
        // Build a perfect power law: count_i = 1000 / (i+1)^2.
        // α = 2.0, R² should be 1.0.
        let mut counts = Vec::new();
        for i in 1..=10 {
            counts.push((1000.0 / (i as f32).powi(2)).round() as u64);
        }
        let h = Histogram::from_counts(counts);
        let r = PowerLawFit.compute_rank_frequency(&h);
        approx(r.alpha, 2.0);
        approx(r.r_squared, 1.0);
    }

    #[test]
    fn power_law_fit_on_uniform_data_has_low_r_squared() {
        // Uniform frequencies: every rank has the same count.
        // The slope should be ≈ 0 and R² should be low.
        let h = Histogram::uniform(10, 100);
        let r = PowerLawFit.compute_rank_frequency(&h);
        assert!(
            r.r_squared < 0.5,
            "uniform data should not fit a power law, got R²={}",
            r.r_squared
        );
        assert!(
            r.alpha.abs() < 0.5,
            "uniform data should give alpha ≈ 0, got {}",
            r.alpha
        );
    }

    #[test]
    fn power_law_fit_on_zipf_like_data() {
        // Classic Zipf: count_i = 1000 / (i+1).
        // α = 1.0, R² very high.
        let mut counts = Vec::new();
        for i in 1..=10 {
            counts.push((1000.0 / (i as f32)).round() as u64);
        }
        let h = Histogram::from_counts(counts);
        let r = PowerLawFit.compute_rank_frequency(&h);
        approx(r.alpha, 1.0);
        assert!(r.r_squared > 0.98, "Zipf data should have R² > 0.98, got {}", r.r_squared);
    }

    #[test]
    fn metric_name_is_stable() {
        assert_eq!(PowerLawFit::NAME, "power_law_alpha");
    }

    #[test]
    fn trait_compute_returns_alpha() {
        let h = Histogram::from_counts(vec![1000, 500, 333, 250, 200]);
        let v = PowerLawFit.compute(&h);
        let r = PowerLawFit.compute_rank_frequency(&h);
        assert_eq!(v, r.alpha);
    }

    #[test]
    fn empty_histogram_returns_zero() {
        let h = Histogram::default();
        let r = PowerLawFit.compute_rank_frequency(&h);
        assert_eq!(r.alpha, 0.0);
        assert_eq!(r.r_squared, 0.0);
    }

    #[test]
    fn single_bin_returns_zero() {
        let h = Histogram::from_counts(vec![42]);
        let r = PowerLawFit.compute_rank_frequency(&h);
        assert_eq!(r.alpha, 0.0);
        assert_eq!(r.r_squared, 0.0);
    }
}
