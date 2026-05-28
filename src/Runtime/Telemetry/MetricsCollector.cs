#nullable enable
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using System.Threading;
using Newtonsoft.Json;

namespace DINOForge.Runtime.Telemetry
{
    /// <summary>
    /// Thread-safe in-memory metrics collector for DINOForge runtime observability.
    /// Supports counters, values, and durations with zero-allocation hot paths via string interning.
    ///
    /// Metrics are keyed by a dot-separated name (e.g., "asset_swap.update_calls"),
    /// and values are accumulated in-memory. Access is thread-safe via ConcurrentDictionary.
    ///
    /// Example usage:
    /// <code>
    /// MetricsCollector.Instance.IncrementCounter("asset_swap.update_calls");
    /// MetricsCollector.Instance.RecordValue("asset_swap.world_entity_count", 49014);
    /// MetricsCollector.Instance.RecordDuration("pack_load.total_ms", timeSpan);
    ///
    /// string markdown = MetricsCollector.Instance.DumpMarkdown();
    /// string json = MetricsCollector.Instance.DumpJson();
    /// </code>
    /// </summary>
    internal sealed class MetricsCollector
    {
        /// <summary>Singleton instance.</summary>
        private static readonly Lazy<MetricsCollector> _instance =
            new Lazy<MetricsCollector>(() => new MetricsCollector());

        public static MetricsCollector Instance => _instance.Value;

        private sealed class MetricEntry
        {
            public MetricEntry(string name, MetricType type)
            {
                Name = string.Intern(name);  // String interning for zero-alloc lookup
                Type = type;
            }

            public string Name { get; }
            public MetricType Type { get; }

            // Backing fields for Interlocked operations (thread-safe atomic RMW).
            // Public property accessors read these via volatile read (Interlocked.Read).
            internal long _counterValue;
            internal long _sampleCount;
            internal double _numericValue;   // last-write-wins; acceptable for gauge metrics
            internal double _totalDurationMs; // protected by _durationLock per-entry

            // Per-entry lock for the two-field (TotalDurationMs, SampleCount) duration update
            // which cannot be made atomic with a single Interlocked operation.
            internal readonly object _durationLock = new object();

            public long CounterValue => Interlocked.Read(ref _counterValue);
            public double NumericValue => _numericValue;
            public long SampleCount => Interlocked.Read(ref _sampleCount);

            /// <summary>For duration metrics: total milliseconds.</summary>
            public double TotalDurationMs => _totalDurationMs;

            /// <summary>For duration metrics: running average (ms).</summary>
            public double AvgDurationMs => SampleCount > 0 ? TotalDurationMs / SampleCount : 0;
        }

        private enum MetricType
        {
            Counter,
            Value,
            Duration
        }

        private readonly ConcurrentDictionary<string, MetricEntry> _metrics =
            new ConcurrentDictionary<string, MetricEntry>(StringComparer.Ordinal);

        private readonly object _dumpLock = new object();
        private DateTime _lastDumpUtc = DateTime.UtcNow;

        private MetricsCollector()
        {
        }

        /// <summary>
        /// Increment a counter metric by 1.
        /// If the metric doesn't exist, it is created with initial value 1.
        /// Thread-safe.
        /// </summary>
        public void IncrementCounter(string name)
        {
            if (string.IsNullOrEmpty(name)) return;

            try
            {
                var entry = _metrics.GetOrAdd(name, n => new MetricEntry(n, MetricType.Counter));
                Interlocked.Increment(ref entry._counterValue);
            }
            catch (Exception ex)
            {
                // safe-swallow: metric recording must never throw in hot paths
                System.Diagnostics.Debug.WriteLine($"[MetricsCollector] IncrementCounter failed for '{name}': {ex.Message}");
            }
        }

        /// <summary>
        /// Record a numeric value metric.
        /// If the metric doesn't exist, it is created. If it exists, the value is overwritten
        /// (not accumulated). Use RecordDuration for accumulating time measurements.
        /// Thread-safe.
        /// </summary>
        public void RecordValue(string name, double value)
        {
            if (string.IsNullOrEmpty(name)) return;

            try
            {
                var entry = _metrics.GetOrAdd(name, n => new MetricEntry(n, MetricType.Value));
                // NumericValue is last-write-wins (gauge semantics); volatile write via Thread.VolatileWrite
                // is sufficient here — we accept a torn read for gauge values on 32-bit platforms.
                entry._numericValue = value;
                Interlocked.Increment(ref entry._sampleCount);
            }
            catch (Exception ex)
            {
                // safe-swallow: metric recording must never throw in hot paths
                System.Diagnostics.Debug.WriteLine($"[MetricsCollector] RecordValue failed for '{name}': {ex.Message}");
            }
        }

        /// <summary>
        /// Record a duration metric in milliseconds.
        /// Accumulates samples and maintains a running average.
        /// Thread-safe.
        /// </summary>
        public void RecordDuration(string name, TimeSpan duration)
        {
            if (string.IsNullOrEmpty(name)) return;

            try
            {
                var durationMs = duration.TotalMilliseconds;
                var entry = _metrics.GetOrAdd(name, n => new MetricEntry(n, MetricType.Duration));
                // TotalDurationMs and SampleCount must be updated together atomically.
                // Use per-entry lock (lightweight — held only for two field writes, no I/O).
                lock (entry._durationLock)
                {
                    entry._totalDurationMs += durationMs;
                    Interlocked.Increment(ref entry._sampleCount);
                }
            }
            catch (Exception ex)
            {
                // safe-swallow: metric recording must never throw in hot paths
                System.Diagnostics.Debug.WriteLine($"[MetricsCollector] RecordDuration failed for '{name}': {ex.Message}");
            }
        }

        /// <summary>
        /// Clear all metrics and reset the collector.
        /// Thread-safe.
        /// </summary>
        public void Clear()
        {
            try
            {
                _metrics.Clear();
            }
            catch
            {
                // Best-effort
            }
        }

        /// <summary>
        /// Dump current metrics as a Markdown table.
        /// Format: | Metric | Value | Type | Samples |
        /// Thread-safe via lock.
        /// </summary>
        public string DumpMarkdown()
        {
            lock (_dumpLock)
            {
                _lastDumpUtc = DateTime.UtcNow;

                var sb = new StringBuilder();
                sb.AppendLine("# DINOForge Telemetry Snapshot");
                sb.AppendLine($"Captured at: {_lastDumpUtc:O}");
                sb.AppendLine();
                sb.AppendLine("| Metric | Value | Type | Samples |");
                sb.AppendLine("|--------|-------|------|---------|");

                foreach (var entry in _metrics.Values.OrderBy(e => e.Name))
                {
                    string value = FormatMetricValue(entry);
                    string type = entry.Type.ToString();
                    string samples = entry.SampleCount > 1 ? entry.SampleCount.ToString() : "—";

                    sb.AppendLine($"| {entry.Name} | {value} | {type} | {samples} |");
                }

                return sb.ToString();
            }
        }

        /// <summary>
        /// Dump current metrics as JSON.
        /// Format: { "timestamp": "...", "metrics": { "name": { "value": ..., "type": "...", "samples": ... } } }
        /// Thread-safe via lock.
        /// </summary>
        public string DumpJson()
        {
            lock (_dumpLock)
            {
                _lastDumpUtc = DateTime.UtcNow;

                var jsonData = new
                {
                    timestamp = _lastDumpUtc.ToString("O"),
                    metrics = _metrics.Values.OrderBy(e => e.Name).ToDictionary(
                        e => e.Name,
                        e => new
                        {
                            value = FormatMetricValue(e),
                            type = e.Type.ToString(),
                            samples = e.SampleCount,
                            raw = e.Type switch
                            {
                                MetricType.Counter => (object)e.CounterValue,
                                MetricType.Value => (object)e.NumericValue,
                                MetricType.Duration => (object)new
                                {
                                    total_ms = e.TotalDurationMs,
                                    avg_ms = e.AvgDurationMs
                                },
                                _ => (object?)null
                            }
                        })
                };

                return JsonConvert.SerializeObject(jsonData, Formatting.Indented);
            }
        }

        /// <summary>
        /// Get the timestamp of the last dump operation.
        /// </summary>
        public DateTime LastDumpUtc => _lastDumpUtc;

        /// <summary>
        /// Get the count of tracked metrics.
        /// </summary>
        public int MetricCount => _metrics.Count;

        /// <summary>
        /// Format a metric value for display (markdown/JSON).
        /// </summary>
        private static string FormatMetricValue(MetricEntry entry)
        {
            return entry.Type switch
            {
                MetricType.Counter => entry.CounterValue.ToString(CultureInfo.InvariantCulture),
                MetricType.Value => entry.NumericValue.ToString("F2", CultureInfo.InvariantCulture),
                MetricType.Duration => $"Σ {entry.TotalDurationMs:F1}ms, avg {entry.AvgDurationMs:F1}ms",
                _ => "N/A"
            };
        }

        /// <summary>
        /// Get a metric value by name (for CLI/API access).
        /// Returns null if not found or on error.
        /// </summary>
        public object? GetMetricValue(string name)
        {
            if (_metrics.TryGetValue(name, out var entry))
            {
                return entry.Type switch
                {
                    MetricType.Counter => entry.CounterValue,
                    MetricType.Value => entry.NumericValue,
                    MetricType.Duration => new
                    {
                        total_ms = entry.TotalDurationMs,
                        avg_ms = entry.AvgDurationMs,
                        samples = entry.SampleCount
                    },
                    _ => null
                };
            }

            return null;
        }
    }
}
