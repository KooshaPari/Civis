#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using DINOForge.Bridge.Protocol;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Captures action traces for UI automation debugging and replay.
    /// Stores tree snapshots, screenshots, and action history.
    /// </summary>
    internal static class UiActionTrace
    {
        private static readonly object _lock = new object();
        private static readonly List<UiActionEntry> _entries = new List<UiActionEntry>();
        private static bool _enabled = true;
        private static string? _traceDirectory;

        public static bool Enabled
        {
            get => _enabled;
            set => _enabled = value;
        }

        public static string? TraceDirectory
        {
            get => _traceDirectory;
            set => _traceDirectory = value;
        }

        public static void Record(string action, string selector, object? result)
        {
            if (!_enabled)
            {
                return;
            }

            lock (_lock)
            {
                var entry = new UiActionEntry
                {
                    TimestampUtc = DateTime.UtcNow,
                    Action = action,
                    Selector = selector,
                    ResultType = result?.GetType().Name ?? "null",
                    ResultSummary = SummarizeResult(result)
                };

                // Attach tree snapshot for significant actions
                if (action == "tree" || action == "click" || action == "expect" || action == "wait")
                {
                    try
                    {
                        var treeResult = UiTreeSnapshotBuilder.Capture(selector);
                        entry.TreeSnapshot = treeResult;
                    }
                    catch { /* safe-swallow: trace snapshot capture is best-effort and must not block recording */ }
                }

                _entries.Add(entry);
            }
        }

        public static void Record(string action, string selector, object? result, UiNode? matchedNode)
        {
            if (!_enabled)
            {
                return;
            }

            lock (_lock)
            {
                var entry = new UiActionEntry
                {
                    TimestampUtc = DateTime.UtcNow,
                    Action = action,
                    Selector = selector,
                    MatchedNodeId = matchedNode?.Id,
                    MatchedNodePath = matchedNode?.Path,
                    ResultType = result?.GetType().Name ?? "null",
                    ResultSummary = SummarizeResult(result)
                };

                // Attach tree snapshot for significant actions
                if (action == "tree" || action == "click" || action == "expect" || action == "wait")
                {
                    try
                    {
                        var treeResult = UiTreeSnapshotBuilder.Capture(selector);
                        entry.TreeSnapshot = treeResult;
                    }
                    catch { /* safe-swallow: trace snapshot capture is best-effort and must not block recording */ }
                }

                _entries.Add(entry);
            }
        }

        public static List<UiActionEntry> GetHistory(int? maxEntries = null)
        {
            lock (_lock)
            {
                if (maxEntries.HasValue && maxEntries.Value > 0 && maxEntries.Value < _entries.Count)
                {
                    var result = new List<UiActionEntry>();
                    int start = _entries.Count - maxEntries.Value;
                    for (int i = start; i < _entries.Count; i++)
                    {
                        result.Add(_entries[i]);
                    }
                    return result;
                }
                return new List<UiActionEntry>(_entries);
            }
        }

        public static void Clear()
        {
            lock (_lock)
            {
                _entries.Clear();
            }
        }

        public static string ExportToJson()
        {
            lock (_lock)
            {
                var sb = new StringBuilder(4096);  // Capacity ~= trace entry count × 7 fields × 100 chars
                sb.AppendLine("{");
                sb.AppendLine("  \"traceEntries\": [");

                for (int i = 0; i < _entries.Count; i++)
                {
                    var entry = _entries[i];
                    sb.AppendLine("    {");
                    sb.AppendLine($"      \"timestampUtc\": \"{entry.TimestampUtc:O}\",");
                    sb.AppendLine($"      \"action\": \"{EscapeJson(entry.Action)}\",");
                    sb.AppendLine($"      \"selector\": \"{EscapeJson(entry.Selector)}\",");
                    sb.AppendLine($"      \"matchedNodeId\": \"{EscapeJson(entry.MatchedNodeId ?? "")}\",");
                    sb.AppendLine($"      \"matchedNodePath\": \"{EscapeJson(entry.MatchedNodePath ?? "")}\",");
                    sb.AppendLine($"      \"resultType\": \"{EscapeJson(entry.ResultType)}\",");
                    sb.AppendLine($"      \"resultSummary\": \"{EscapeJson(entry.ResultSummary)}\"");
                    sb.Append("    }");
                    if (i < _entries.Count - 1)
                    {
                        sb.AppendLine(",");
                    }
                    else
                    {
                        sb.AppendLine();
                    }
                }

                sb.AppendLine("  ]");
                sb.AppendLine("}");
                return sb.ToString();
            }
        }

        public static void SaveToFile(string? path = null)
        {
            string filePath = path ?? Path.Combine(_traceDirectory ?? Path.GetTempPath(), $"ui_trace_{DateTime.UtcNow:yyyyMMdd_HHmmss}.json");

            lock (_lock)
            {
                try
                {
                    File.WriteAllText(filePath, ExportToJson(), Encoding.UTF8);
                }
                catch
                {
                    // safe-swallow: trace file writes are best-effort diagnostics and must not break UI actions
                }
            }
        }

        private static string SummarizeResult(object? result)
        {
            if (result == null)
                return "null";

            if (result is UiActionResult ar)
                return $"Success={ar.Success}, Message={ar.Message}, Actionable={ar.Actionable}";

            if (result is UiTreeResult tr)
                return $"NodeCount={tr.NodeCount}, Success={tr.Success}";

            if (result is UiWaitResult wr)
                return $"Ready={wr.Ready}, State={wr.State}";

            if (result is UiExpectationResult er)
                return $"Success={er.Success}, Condition={er.Condition}, MatchCount={er.MatchCount}";

            return result.ToString() ?? result.GetType().Name;
        }

        private static string EscapeJson(string? s)
        {
            if (s == null) return "";
            return s.Replace("\\", "\\\\")
                    .Replace("\"", "\\\"")
                    .Replace("\n", "\\n")
                    .Replace("\r", "\\r")
                    .Replace("\t", "\\t");
        }
    }

    /// <summary>
    /// Single action entry in the trace log.
    /// </summary>
    internal class UiActionEntry
    {
        public DateTime TimestampUtc { get; set; }
        public string Action { get; set; } = "";
        public string Selector { get; set; } = "";
        public string? MatchedNodeId { get; set; }
        public string? MatchedNodePath { get; set; }
        public string ResultType { get; set; } = "";
        public string ResultSummary { get; set; } = "";
        public UiTreeResult? TreeSnapshot { get; set; }
    }
}
