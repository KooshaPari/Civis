//! Pure pixel-statistics helpers for verification.
//!
//! The output is intentionally narrow and machine-readable. Humans should not
//! eyeball screenshots to make claims when these numbers are available.

use serde::Serialize;

/// A sampled RGB triple in 8-bit sRGB space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SampleRgb {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl SampleRgb {
    /// Construct from raw bytes.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Mean luminance surrogate: simple average channel value.
    #[must_use]
    pub fn mean_channel(self) -> f32 {
        (self.r as f32 + self.g as f32 + self.b as f32) / 3.0
    }

    /// True when the pixel is grayscale (`R = G = B`).
    #[must_use]
    pub fn is_gray(self) -> bool {
        self.r == self.g && self.g == self.b
    }

    /// A near-black predicate used by the gate. This is a conservative low-
    /// light test so a mostly-blank frame is easy to detect in CI logs.
    #[must_use]
    pub fn is_near_black(self, threshold: u8) -> bool {
        self.r <= threshold && self.g <= threshold && self.b <= threshold
    }

    /// Hue bucket in degrees. Greys return `None`.
    #[must_use]
    pub fn hue_bucket(self) -> Option<u16> {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;
        if delta <= f32::EPSILON {
            return None;
        }
        let hue = if (max - r).abs() <= f32::EPSILON {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() <= f32::EPSILON {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        let normalized = hue.rem_euclid(360.0).round() as u16;
        Some(normalized)
    }
}

/// Aggregated statistics emitted by `civis-pixels`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PixelStats {
    /// Number of sampled points.
    pub samples: usize,
    /// Mean red channel.
    pub mean_r: f32,
    /// Mean green channel.
    pub mean_g: f32,
    /// Mean blue channel.
    pub mean_b: f32,
    /// Percent of samples whose RGB values are all <= the near-black threshold.
    pub percent_near_black: f32,
    /// Percent of samples where R == G == B.
    pub percent_gray: f32,
    /// Count of distinct hue buckets among non-gray samples.
    pub distinct_hue_count: usize,
}

/// Compute statistics from sampled RGB values.
#[must_use]
pub fn compute_pixel_stats(samples: &[SampleRgb]) -> PixelStats {
    let samples_len = samples.len();
    if samples_len == 0 {
        return PixelStats {
            samples: 0,
            mean_r: 0.0,
            mean_g: 0.0,
            mean_b: 0.0,
            percent_near_black: 0.0,
            percent_gray: 0.0,
            distinct_hue_count: 0,
        };
    }

    let mut sum_r = 0u64;
    let mut sum_g = 0u64;
    let mut sum_b = 0u64;
    let mut near_black = 0usize;
    let mut gray = 0usize;
    let mut hues = std::collections::BTreeSet::new();

    for s in samples {
        sum_r += s.r as u64;
        sum_g += s.g as u64;
        sum_b += s.b as u64;
        if s.is_near_black(8) {
            near_black += 1;
        }
        if s.is_gray() {
            gray += 1;
        } else if let Some(bucket) = s.hue_bucket() {
            hues.insert(bucket);
        }
    }

    let n = samples_len as f32;
    PixelStats {
        samples: samples_len,
        mean_r: sum_r as f32 / n,
        mean_g: sum_g as f32 / n,
        mean_b: sum_b as f32 / n,
        percent_near_black: (near_black as f32 / n) * 100.0,
        percent_gray: (gray as f32 / n) * 100.0,
        distinct_hue_count: hues.len(),
    }
}

/// Sample a regular grid from a flat RGB buffer.
///
/// `width`/`height` are the image dimensions, `grid` is the number of sample
/// points per axis. The function performs nearest-neighbour sampling at the
/// centre of each cell so it is stable across builds.
#[must_use]
pub fn sample_rgb_grid(width: usize, height: usize, grid: usize, data: &[u8]) -> Vec<SampleRgb> {
    if width == 0 || height == 0 || grid == 0 {
        return Vec::new();
    }
    let expected = width.saturating_mul(height).saturating_mul(3);
    if data.len() < expected {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(grid * grid);
    for gy in 0..grid {
        let py = ((gy * height) + height / 2) / grid;
        let y = py.min(height - 1);
        for gx in 0..grid {
            let px = ((gx * width) + width / 2) / grid;
            let x = px.min(width - 1);
            let idx = (y * width + x) * 3;
            out.push(SampleRgb::new(data[idx], data[idx + 1], data[idx + 2]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stats_are_zeroed() {
        let stats = compute_pixel_stats(&[]);
        assert_eq!(stats.samples, 0);
        assert_eq!(stats.distinct_hue_count, 0);
    }

    #[test]
    fn stats_count_gray_and_hues() {
        let samples = [
            SampleRgb::new(0, 0, 0),
            SampleRgb::new(12, 12, 12),
            SampleRgb::new(255, 0, 0),
            SampleRgb::new(0, 255, 0),
            SampleRgb::new(0, 0, 255),
        ];
        let stats = compute_pixel_stats(&samples);
        assert_eq!(stats.samples, 5);
        assert!((stats.percent_gray - 40.0).abs() < f32::EPSILON);
        assert!((stats.percent_near_black - 20.0).abs() < f32::EPSILON);
        assert_eq!(stats.distinct_hue_count, 3);
    }

    #[test]
    fn grid_sampling_picks_cell_centres() {
        let width = 2;
        let height = 2;
        let data = [
            10, 0, 0, // (0,0)
            20, 0, 0, // (1,0)
            30, 0, 0, // (0,1)
            40, 0, 0, // (1,1)
        ];
        let samples = sample_rgb_grid(width, height, 2, &data);
        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0].r, 10);
        assert_eq!(samples[1].r, 20);
        assert_eq!(samples[2].r, 30);
        assert_eq!(samples[3].r, 40);
    }
}
