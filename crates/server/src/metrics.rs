//! Runtime metrics collection for the Civis Bevy godgame server.
//!
//! Tracks tick durations, connected clients, event throughput, and memory
//! usage. Designed to be polled by the JSON-RPC layer and optionally
//! exported to Prometheus.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

// ---------------------------------------------------------------------------
// SimMetrics — the core snapshot struct
// ---------------------------------------------------------------------------

/// A point-in-time snapshot of simulation metrics.
///
/// Serializable via serde so it can be embedded directly in JSON-RPC
/// responses.
#[derive(Debug, Clone, Serialize)]
pub struct SimMetrics {
    /// Total number of simulation ticks processed since startup.
    pub tick_count: u64,
    /// Duration of the most recent tick in milliseconds.
    pub tick_duration_ms: f64,
    /// Number of currently connected client sessions.
    pub connected_clients: u32,
    /// Cumulative count of events processed since startup.
    pub events_processed: u64,
    /// Approximate memory usage in bytes (as reported by the allocator or OS).
    pub memory_usage_bytes: u64,
    /// Seconds elapsed since the server started.
    pub uptime_seconds: u64,
    /// Unix-epoch millisecond timestamp of the last completed tick.
    pub last_tick_timestamp: u64,
}

impl fmt::Display for SimMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mem_mb = self.memory_usage_bytes as f64 / (1024.0 * 1024.0);
        write!(
            f,
            "SimMetrics {{ ticks: {}, tick_ms: {:.2}, clients: {}, events: {}, \
             mem: {:.2} MB, uptime: {}s, last_tick: {} }}",
            self.tick_count,
            self.tick_duration_ms,
            self.connected_clients,
            self.events_processed,
            mem_mb,
            self.uptime_seconds,
            self.last_tick_timestamp,
        )
    }
}

// ---------------------------------------------------------------------------
// TickHistogram — derived from recent tick history
// ---------------------------------------------------------------------------

/// Statistical summary of recent tick durations.
#[derive(Debug, Clone, Serialize)]
pub struct TickHistogram {
    /// 50th-percentile tick duration (ms).
    pub p50: f64,
    /// 95th-percentile tick duration (ms).
    pub p95: f64,
    /// 99th-percentile tick duration (ms).
    pub p99: f64,
    /// Minimum tick duration in the window (ms).
    pub min: f64,
    /// Maximum tick duration in the window (ms).
    pub max: f64,
    /// Arithmetic mean of tick durations in the window (ms).
    pub avg: f64,
}

// ---------------------------------------------------------------------------
// MetricsSummary — human-friendly / JSON-RPC ready
// ---------------------------------------------------------------------------

/// Pre-formatted metrics summary intended for JSON-RPC response payloads.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    /// Human-readable uptime string (e.g. "2h 14m 03s").
    pub uptime: String,
    /// Total tick count.
    pub total_ticks: u64,
    /// Average tick duration in milliseconds.
    pub avg_tick_ms: f64,
    /// Current connected client count.
    pub client_count: u32,
    /// Estimated events per second since server start.
    pub events_per_second: f64,
    /// Memory usage in megabytes.
    pub memory_mb: f64,
}

impl fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MetricsSummary {{ uptime: {}, ticks: {}, avg_tick: {:.2}ms, \
             clients: {}, events/s: {:.1}, mem: {:.2}MB }}",
            self.uptime,
            self.total_ticks,
            self.avg_tick_ms,
            self.client_count,
            self.events_per_second,
            self.memory_mb,
        )
    }
}

// ---------------------------------------------------------------------------
// MetricsCollector — the main API
// ---------------------------------------------------------------------------

/// Collects and retains simulation metrics.
///
/// Tick durations are stored in a circular buffer of the most recent 100
/// samples so that percentiles can be computed cheaply.
pub struct MetricsCollector {
    metrics: SimMetrics,
    /// Rolling buffer of recent tick durations (max 100).
    tick_history: Vec<f64>,
    /// Unix-epoch milliseconds when the collector was created.
    start_time: u64,
}

impl MetricsCollector {
    /// Maximum number of tick durations retained for histogram computation.
    const HISTORY_CAP: usize = 100;

    /// Create a new collector, recording the current wall-clock time as the
    /// start time.
    pub fn new() -> Self {
        let now = Self::now_ms();
        Self {
            metrics: SimMetrics {
                tick_count: 0,
                tick_duration_ms: 0.0,
                connected_clients: 0,
                events_processed: 0,
                memory_usage_bytes: 0,
                uptime_seconds: 0,
                last_tick_timestamp: 0,
            },
            tick_history: Vec::with_capacity(Self::HISTORY_CAP),
            start_time: now,
        }
    }

    /// Record the duration of a completed tick.
    pub fn record_tick(&mut self, duration_ms: f64) {
        self.metrics.tick_count += 1;
        self.metrics.tick_duration_ms = duration_ms;
        self.metrics.last_tick_timestamp = Self::now_ms();

        if self.tick_history.len() == Self::HISTORY_CAP {
            // Drop the oldest sample (ring buffer via VecDeque-like behaviour).
            self.tick_history.remove(0);
        }
        self.tick_history.push(duration_ms);

        self.metrics.uptime_seconds = self.elapsed_seconds();
    }

    /// Set the number of currently connected clients.
    pub fn set_clients(&mut self, n: u32) {
        self.metrics.connected_clients = n;
    }

    /// Increment the cumulative event counter by `n`.
    pub fn increment_events(&mut self, n: u64) {
        self.metrics.events_processed += n;
    }

    /// Set the current memory usage in bytes.
    pub fn set_memory(&mut self, bytes: u64) {
        self.metrics.memory_usage_bytes = bytes;
    }

    /// Return a snapshot of the current metrics.
    pub fn get_metrics(&mut self) -> SimMetrics {
        self.metrics.uptime_seconds = self.elapsed_seconds();
        self.metrics.clone()
    }

    /// Compute a histogram (percentiles, min, max, avg) from the tick
    /// history buffer.
    pub fn histogram(&self) -> TickHistogram {
        if self.tick_history.is_empty() {
            return TickHistogram {
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
                min: 0.0,
                max: 0.0,
                avg: 0.0,
            };
        }

        let mut sorted = self.tick_history.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = sorted.len();
        let min = sorted[0];
        let max = sorted[len - 1];
        let avg = sorted.iter().sum::<f64>() / len as f64;

        TickHistogram {
            p50: percentile(&sorted, 0.50),
            p95: percentile(&sorted, 0.95),
            p99: percentile(&sorted, 0.99),
            min,
            max,
            avg,
        }
    }

    /// Build a pre-formatted [`MetricsSummary`] suitable for JSON-RPC
    /// responses.
    pub fn summary(&self) -> MetricsSummary {
        let uptime_secs = self.elapsed_seconds();
        let uptime = format_duration(uptime_secs);

        let total_ticks = self.metrics.tick_count;
        let avg_tick_ms = if total_ticks > 0 {
            // Prefer the windowed average from the rolling histogram.
            let hist = self.histogram();
            if hist.avg > 0.0 {
                hist.avg
            } else {
                self.metrics.tick_duration_ms
            }
        } else {
            0.0
        };

        let events_per_second = if uptime_secs > 0 {
            self.metrics.events_processed as f64 / uptime_secs as f64
        } else {
            0.0
        };

        MetricsSummary {
            uptime,
            total_ticks,
            avg_tick_ms,
            client_count: self.metrics.connected_clients,
            events_per_second,
            memory_mb: self.metrics.memory_usage_bytes as f64 / (1024.0 * 1024.0),
        }
    }

    /// Current Unix-epoch time in milliseconds.
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn elapsed_seconds(&self) -> u64 {
        let now = Self::now_ms();
        now.saturating_sub(self.start_time) / 1000
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute a percentile from a **sorted** slice using linear interpolation.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = p * (sorted.len() as f64 - 1.0);
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = rank - lo as f64;
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }
}

/// Format a number of seconds into a human-readable string like `"2h 14m 03s"`.
fn format_duration(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{:02}h {:02}m {:02}s", h, m, s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Basic construction
    #[test]
    fn collector_starts_with_zeroed_metrics() {
        let mut c = MetricsCollector::new();
        let m = c.get_metrics();
        assert_eq!(m.tick_count, 0);
        assert_eq!(m.tick_duration_ms, 0.0);
        assert_eq!(m.connected_clients, 0);
        assert_eq!(m.events_processed, 0);
        assert_eq!(m.memory_usage_bytes, 0);
        // tick_history lives on the collector, not the metrics snapshot
        assert!(c.tick_history.is_empty());
    }

    // 2. record_tick increments tick_count and stores duration
    #[test]
    fn record_tick_updates_state() {
        let mut c = MetricsCollector::new();
        c.record_tick(12.5);
        let m = c.get_metrics();
        assert_eq!(m.tick_count, 1);
        assert!((m.tick_duration_ms - 12.5).abs() < f64::EPSILON);
        assert!(m.last_tick_timestamp > 0);
    }

    // 3. record_tick fills history buffer up to 100 then evicts oldest
    #[test]
    fn tick_history_caps_at_100() {
        let mut c = MetricsCollector::new();
        for i in 0..150 {
            c.record_tick(i as f64);
        }
        assert_eq!(c.tick_history.len(), MetricsCollector::HISTORY_CAP);
        // The oldest surviving value should be the 50th sample (index 50).
        assert!((c.tick_history[0] - 50.0).abs() < f64::EPSILON);
    }

    // 4. histogram on empty history returns all zeros
    #[test]
    fn empty_histogram_returns_zeros() {
        let c = MetricsCollector::new();
        let h = c.histogram();
        assert!((h.p50).abs() < f64::EPSILON);
        assert!((h.p95).abs() < f64::EPSILON);
        assert!((h.p99).abs() < f64::EPSILON);
        assert!((h.min).abs() < f64::EPSILON);
        assert!((h.max).abs() < f64::EPSILON);
        assert!((h.avg).abs() < f64::EPSILON);
    }

    // 5. histogram percentiles for a known sorted distribution
    #[test]
    fn histogram_percentiles_correct() {
        let mut c = MetricsCollector::new();
        // Insert 100 values: 1.0 .. 100.0
        for i in 1..=100 {
            c.record_tick(i as f64);
        }
        let h = c.histogram();
        assert!((h.min - 1.0).abs() < f64::EPSILON);
        assert!((h.max - 100.0).abs() < f64::EPSILON);
        // avg of 1..=100 is 50.5
        assert!((h.avg - 50.5).abs() < 0.01);
        // p50 ≈ 50.5
        assert!((h.p50 - 50.5).abs() < 1.0);
        // p95 ≈ 95.05
        assert!((h.p95 - 95.05).abs() < 1.5);
        // p99 ≈ 99.01
        assert!((h.p99 - 99.01).abs() < 1.5);
    }

    // 6. set_clients / increment_events / set_memory
    #[test]
    fn setters_update_metrics() {
        let mut c = MetricsCollector::new();
        c.set_clients(5);
        c.increment_events(42);
        c.set_memory(1024 * 1024);
        let m = c.get_metrics();
        assert_eq!(m.connected_clients, 5);
        assert_eq!(m.events_processed, 42);
        assert_eq!(m.memory_usage_bytes, 1024 * 1024);
    }

    // 7. increment_events accumulates
    #[test]
    fn increment_events_accumulates() {
        let mut c = MetricsCollector::new();
        c.increment_events(10);
        c.increment_events(20);
        c.increment_events(5);
        assert_eq!(c.get_metrics().events_processed, 35);
    }

    // 8. summary includes formatted uptime and correct fields
    #[test]
    fn summary_uses_histogram_avg() {
        let mut c = MetricsCollector::new();
        for i in 1..=10 {
            c.record_tick(i as f64 * 10.0); // 10,20,...,100 ms
        }
        c.set_clients(3);
        c.increment_events(500);
        c.set_memory(2 * 1024 * 1024); // 2 MB

        let s = c.summary();
        assert_eq!(s.client_count, 3);
        assert_eq!(s.total_ticks, 10);
        assert!((s.memory_mb - 2.0).abs() < 0.01);
        // avg of 10..100 step 10 = 55.0
        assert!((s.avg_tick_ms - 55.0).abs() < 1.0);
        // uptime string format: "XXh XXm XXs"
        assert!(s.uptime.ends_with('s'));
    }

    // 9. Display impl produces a non-empty human-readable string
    #[test]
    fn display_impl_contains_key_info() {
        let mut c = MetricsCollector::new();
        c.record_tick(42.0);
        c.set_clients(7);
        let m = c.get_metrics();
        let s = m.to_string();
        assert!(s.contains("tick_ms"));
        assert!(s.contains("42"));
        assert!(s.contains("clients: 7"));
    }

    // 10. Histogram with a single tick
    #[test]
    fn single_tick_histogram() {
        let mut c = MetricsCollector::new();
        c.record_tick(33.3);
        let h = c.histogram();
        assert!((h.min - 33.3).abs() < f64::EPSILON);
        assert!((h.max - 33.3).abs() < f64::EPSILON);
        assert!((h.avg - 33.3).abs() < f64::EPSILON);
        assert!((h.p50 - 33.3).abs() < f64::EPSILON);
        assert!((h.p95 - 33.3).abs() < f64::EPSILON);
        assert!((h.p99 - 33.3).abs() < f64::EPSILON);
    }

    // 11. format_duration helper
    #[test]
    fn format_duration_basic() {
        assert_eq!(format_duration(0), "00h 00m 00s");
        assert_eq!(format_duration(3661), "01h 01m 01s");
        assert_eq!(format_duration(60), "00h 01m 00s");
    }

    // 12. SimMetrics serializes to valid JSON
    #[test]
    fn sim_metrics_serialize() {
        let m = SimMetrics {
            tick_count: 100,
            tick_duration_ms: 16.67,
            connected_clients: 4,
            events_processed: 1000,
            memory_usage_bytes: 512 * 1024,
            uptime_seconds: 3600,
            last_tick_timestamp: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(json.contains("\"tick_count\":100"));
        assert!(json.contains("\"connected_clients\":4"));
    }
}
