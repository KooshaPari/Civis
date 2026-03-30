#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace DINOForge.Tools.DumpTools
{
    /// <summary>
    /// Analyzes game launch failure manifests and generates diagnostic reports.
    /// Parses logs, identifies error patterns, affected systems, and provides recommendations.
    /// </summary>
    public class GameLaunchFailureAnalyzer
    {
        private readonly string _failuresDirectory;
        private readonly string _outputDirectory;

        public GameLaunchFailureAnalyzer(string? failuresDir = null, string? outputDir = null)
        {
            _failuresDirectory = failuresDir ?? Path.Combine(
                Environment.GetEnvironmentVariable("TEMP") ?? "C:\\temp",
                "DINOForge", "failures");

            _outputDirectory = outputDir ?? Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments),
                "DINOForge", "sessions");

            Directory.CreateDirectory(_outputDirectory);
        }

        /// <summary>
        /// Analyzes the most recent failure manifest and generates a markdown report.
        /// </summary>
        /// <returns>Path to generated report file, or empty string if no failure found</returns>
        public async Task<string> GenerateFailureReportAsync()
        {
            try
            {
                if (!Directory.Exists(_failuresDirectory))
                {
                    return string.Empty;
                }

                var manifests = Directory.GetFiles(_failuresDirectory, "failure_*.json")
                    .OrderByDescending(f => new FileInfo(f).LastWriteTimeUtc)
                    .ToList();

                if (manifests.Count == 0)
                {
                    return string.Empty;
                }

                var latestManifest = await File.ReadAllTextAsync(manifests[0]);
                var failureData = JsonSerializer.Deserialize<FailureManifestDto>(latestManifest);

                if (failureData == null)
                {
                    return string.Empty;
                }

                var analysis = AnalyzeFailureData(failureData);
                var report = GenerateMarkdownReport(failureData, analysis);

                var timestamp = DateTime.UtcNow.ToString("yyyyMMdd_HHmmss");
                var reportPath = Path.Combine(_outputDirectory, $"{timestamp}_failure_analysis.md");

                await File.WriteAllTextAsync(reportPath, report);
                return reportPath;
            }
            catch
            {
                return string.Empty;
            }
        }

        /// <summary>
        /// Analyzes logs from a failure manifest to extract error patterns and root causes.
        /// </summary>
        public FailureAnalysisResult AnalyzeLogs(List<string> dinoforgeLog, List<string> bepinexLog)
        {
            var result = new FailureAnalysisResult
            {
                AllLogs = new List<string>(dinoforgeLog ?? new List<string>())
            };

            if (bepinexLog != null)
            {
                result.AllLogs.AddRange(bepinexLog);
            }

            // Find error patterns
            var errorPatterns = new Dictionary<string, string>
            {
                { "ERROR", "Error logged" },
                { "FATAL", "Fatal error" },
                { "Exception", "Exception thrown" },
                { "failed", "Operation failed" },
                { "Failed", "Operation failed" },
                { "NullReferenceException", "Null reference error" },
                { "IndexOutOfRangeException", "Index out of range error" },
                { "InvalidOperationException", "Invalid operation" }
            };

            var systemPatterns = new Dictionary<string, string>
            {
                { "Warfare", "Warfare domain plugin" },
                { "Economy", "Economy domain plugin" },
                { "Scenario", "Scenario domain plugin" },
                { "UI", "UI domain plugin" },
                { "Runtime", "DINOForge runtime" },
                { "Plugin", "Plugin loader" },
                { "ECS", "Entity Component System" },
                { "Entity", "ECS entities" },
                { "System", "ECS system group" },
                { "Registry", "Registry system" },
                { "Bridge", "Game bridge" },
                { "ContentLoader", "Content loader service" }
            };

            // Extract timeline
            var timelineEvents = new List<string>();
            foreach (var line in result.AllLogs)
            {
                if (!string.IsNullOrWhiteSpace(line) && line.Length > 0)
                {
                    timelineEvents.Add(line.Trim());
                }
            }

            result.TimelineLastEvents = timelineEvents.TakeLast(20).ToList();

            // Find root cause
            string? rootCause = null;
            foreach (var line in result.AllLogs)
            {
                foreach (var (pattern, desc) in errorPatterns)
                {
                    if (line.Contains(pattern, StringComparison.OrdinalIgnoreCase))
                    {
                        if (rootCause == null)
                        {
                            rootCause = line.Trim();
                        }

                        foreach (var (keyword, systemDesc) in systemPatterns)
                        {
                            if (line.Contains(keyword, StringComparison.OrdinalIgnoreCase) &&
                                !result.AffectedSystems.Contains(systemDesc))
                            {
                                result.AffectedSystems.Add(systemDesc);
                            }
                        }

                        break;
                    }
                }
            }

            result.RootCause = rootCause ?? "Unknown - no explicit error found in logs";

            // Find last successful event
            foreach (var line in result.AllLogs.Reverse<string>())
            {
                if (!string.IsNullOrWhiteSpace(line) &&
                    !line.Contains("ERROR", StringComparison.OrdinalIgnoreCase) &&
                    !line.Contains("FATAL", StringComparison.OrdinalIgnoreCase) &&
                    !line.Contains("Exception", StringComparison.OrdinalIgnoreCase) &&
                    !line.Contains("failed", StringComparison.OrdinalIgnoreCase))
                {
                    result.LastSuccessfulEvent = line.Trim();
                    break;
                }
            }

            result.ErrorPattern = DetermineErrorPattern(result.RootCause);

            return result;
        }

        /// <summary>
        /// Generates recommendations based on the failure analysis.
        /// </summary>
        public List<string> GenerateRecommendations(FailureAnalysisResult analysis)
        {
            var recommendations = new List<string>();

            if (analysis.ErrorPattern.Contains("Fatal", StringComparison.OrdinalIgnoreCase))
            {
                recommendations.Add("Fatal error detected — check game logs for stack trace and module information");
                recommendations.Add("Verify DINOForge Runtime DLL is correctly deployed to BepInEx/plugins/");
                recommendations.Add("Check for incompatible BepInEx version or missing core plugins");
            }

            if (analysis.AffectedSystems.Contains("Plugin loader"))
            {
                recommendations.Add("Plugin loading failure — check BepInEx LogOutput.log for load errors");
                recommendations.Add("Verify all domain plugins are built and deployed");
                recommendations.Add("Check for circular dependencies in plugin initialization");
            }

            if (analysis.AffectedSystems.Contains("Entity Component System"))
            {
                recommendations.Add("ECS failure — check if ECS world initialization completed");
                recommendations.Add("Verify all system groups registered correctly");
                recommendations.Add("Check for race conditions during world creation");
            }

            if (analysis.AffectedSystems.Contains("Warfare domain plugin") ||
                analysis.AffectedSystems.Contains("Economy domain plugin") ||
                analysis.AffectedSystems.Contains("Scenario domain plugin") ||
                analysis.AffectedSystems.Contains("UI domain plugin"))
            {
                var domain = analysis.AffectedSystems
                    .FirstOrDefault(s => s.Contains("domain"))?
                    .Replace(" domain plugin", "") ?? "domain";
                recommendations.Add($"{domain} plugin initialization failed — check pack manifest and schema");
                recommendations.Add($"Verify {domain} content packs are valid and compatible");
            }

            if (analysis.ErrorPattern.Contains("Config", StringComparison.OrdinalIgnoreCase) ||
                analysis.ErrorPattern.Contains("Registry", StringComparison.OrdinalIgnoreCase))
            {
                recommendations.Add("Configuration or registry error — check YAML/JSON manifest format");
                recommendations.Add("Verify all required fields are present in pack manifest");
                recommendations.Add("Run 'dotnet run --project src/Tools/PackCompiler -- validate packs/' to validate packs");
            }

            if (recommendations.Count == 0)
            {
                recommendations.Add("Unknown error pattern — review full logs in failure manifest");
                recommendations.Add("Enable verbose logging in BepInEx configuration");
                recommendations.Add("Check BepInEx console window for additional debug output");
            }

            return recommendations;
        }

        /// <summary>
        /// Analyzes failure manifest data.
        /// </summary>
        private FailureAnalysisResult AnalyzeFailureData(FailureManifestDto data)
        {
            return AnalyzeLogs(
                data.LogTailDinoforge ?? new List<string>(),
                data.LogTailBepinex ?? new List<string>());
        }

        /// <summary>
        /// Generates a markdown report from failure data and analysis.
        /// </summary>
        private string GenerateMarkdownReport(FailureManifestDto failureData, FailureAnalysisResult analysis)
        {
            var sb = new StringBuilder();
            var timestamp = failureData.CapturedAt ?? DateTime.UtcNow.ToString("O");

            sb.AppendLine("# Game Launch Failure Analysis Report");
            sb.AppendLine();
            sb.AppendLine($"**Generated:** {timestamp}");
            sb.AppendLine();

            if (!string.IsNullOrEmpty(failureData.TestName))
            {
                sb.AppendLine($"**Failing Test:** `{failureData.TestName}`");
                sb.AppendLine();
            }

            // Process Information
            sb.AppendLine("## Process Information");
            sb.AppendLine();
            if (failureData.Process != null)
            {
                sb.AppendLine($"- **Status:** {failureData.Process.Status}");
                if (failureData.Process.Pid > 0)
                {
                    sb.AppendLine($"- **PID:** {failureData.Process.Pid}");
                    sb.AppendLine($"- **Window Title:** {failureData.Process.MainWindowTitle ?? "(none)"}");
                    sb.AppendLine($"- **Memory:** {failureData.Process.MemoryMb} MB");
                }
            }
            sb.AppendLine();

            // Root Cause
            sb.AppendLine("## Root Cause Analysis");
            sb.AppendLine();
            sb.AppendLine($"**Error Pattern:** {analysis.ErrorPattern}");
            sb.AppendLine();
            sb.AppendLine($"**Root Cause:**");
            sb.AppendLine($"```");
            sb.AppendLine(analysis.RootCause);
            sb.AppendLine($"```");
            sb.AppendLine();

            // Affected Systems
            if (analysis.AffectedSystems.Count > 0)
            {
                sb.AppendLine("## Affected Systems");
                sb.AppendLine();
                foreach (var system in analysis.AffectedSystems)
                {
                    sb.AppendLine($"- {system}");
                }
                sb.AppendLine();
            }

            // Timeline
            if (analysis.TimelineLastEvents.Count > 0)
            {
                sb.AppendLine("## Event Timeline (Last 20 Events)");
                sb.AppendLine();
                sb.AppendLine("```");
                foreach (var evt in analysis.TimelineLastEvents)
                {
                    sb.AppendLine(evt);
                }
                sb.AppendLine("```");
                sb.AppendLine();
            }

            // Last Successful Event
            if (!string.IsNullOrEmpty(analysis.LastSuccessfulEvent))
            {
                sb.AppendLine("## Last Successful Event");
                sb.AppendLine();
                sb.AppendLine($"```");
                sb.AppendLine(analysis.LastSuccessfulEvent);
                sb.AppendLine($"```");
                sb.AppendLine();
            }

            // Recommendations
            var recommendations = GenerateRecommendations(analysis);
            if (recommendations.Count > 0)
            {
                sb.AppendLine("## Recommendations");
                sb.AppendLine();
                for (int i = 0; i < recommendations.Count; i++)
                {
                    sb.AppendLine($"{i + 1}. {recommendations[i]}");
                }
                sb.AppendLine();
            }

            // Full Logs
            sb.AppendLine("## Full Logs");
            sb.AppendLine();
            sb.AppendLine("### DINOForge Debug Log");
            sb.AppendLine();
            sb.AppendLine("```");
            if (failureData.LogTailDinoforge != null && failureData.LogTailDinoforge.Count > 0)
            {
                foreach (var line in failureData.LogTailDinoforge)
                {
                    sb.AppendLine(line);
                }
            }
            else
            {
                sb.AppendLine("(no log available)");
            }
            sb.AppendLine("```");
            sb.AppendLine();

            sb.AppendLine("### BepInEx Log");
            sb.AppendLine();
            sb.AppendLine("```");
            if (failureData.LogTailBepinex != null && failureData.LogTailBepinex.Count > 0)
            {
                foreach (var line in failureData.LogTailBepinex)
                {
                    sb.AppendLine(line);
                }
            }
            else
            {
                sb.AppendLine("(no log available)");
            }
            sb.AppendLine("```");
            sb.AppendLine();

            return sb.ToString();
        }

        /// <summary>
        /// Determines the error pattern classification from a root cause line.
        /// </summary>
        private string DetermineErrorPattern(string rootCause)
        {
            if (string.IsNullOrEmpty(rootCause) || rootCause.Contains("Unknown", StringComparison.OrdinalIgnoreCase))
                return "Unknown error - no explicit failure logged";

            if (rootCause.Contains("Fatal", StringComparison.OrdinalIgnoreCase))
                return "Fatal error - immediate crash";

            if (rootCause.Contains("NullReference", StringComparison.OrdinalIgnoreCase))
                return "Null reference exception - invalid object access";

            if (rootCause.Contains("Plugin", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("load", StringComparison.OrdinalIgnoreCase))
                return "Plugin/module loading failure";

            if (rootCause.Contains("Entity", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("ECS", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("System", StringComparison.OrdinalIgnoreCase))
                return "Entity Component System failure";

            if (rootCause.Contains("Config", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("Registry", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("Manifest", StringComparison.OrdinalIgnoreCase))
                return "Configuration or registry error";

            if (rootCause.Contains("timeout", StringComparison.OrdinalIgnoreCase))
                return "Operation timeout - resource not available in time";

            if (rootCause.Contains("memory", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("OutOfMemory", StringComparison.OrdinalIgnoreCase))
                return "Memory error - insufficient resources";

            return "Unclassified error";
        }
    }

    /// <summary>
    /// DTO for failure manifest deserialization.
    /// </summary>
    public class FailureManifestDto
    {
        [JsonPropertyName("timestamp")]
        public string? Timestamp { get; set; }

        [JsonPropertyName("test_name")]
        public string? TestName { get; set; }

        [JsonPropertyName("screenshot_path")]
        public string? ScreenshotPath { get; set; }

        [JsonPropertyName("process")]
        public ProcessInfoDto? Process { get; set; }

        [JsonPropertyName("log_tail_dinoforge")]
        public List<string>? LogTailDinoforge { get; set; }

        [JsonPropertyName("log_tail_bepinex")]
        public List<string>? LogTailBepinex { get; set; }

        [JsonPropertyName("entity_count")]
        public int EntityCount { get; set; }

        [JsonPropertyName("captured_at")]
        public string? CapturedAt { get; set; }
    }

    /// <summary>
    /// DTO for process information.
    /// </summary>
    public class ProcessInfoDto
    {
        [JsonPropertyName("pid")]
        public int Pid { get; set; }

        [JsonPropertyName("main_window_title")]
        public string? MainWindowTitle { get; set; }

        [JsonPropertyName("memory_mb")]
        public long MemoryMb { get; set; }

        [JsonPropertyName("status")]
        public string? Status { get; set; }
    }

    /// <summary>
    /// Result of failure analysis.
    /// </summary>
    public class FailureAnalysisResult
    {
        public string RootCause { get; set; } = "Unknown";
        public List<string> AffectedSystems { get; set; } = new();
        public List<string> AllLogs { get; set; } = new();
        public List<string> TimelineLastEvents { get; set; } = new();
        public string? LastSuccessfulEvent { get; set; }
        public string ErrorPattern { get; set; } = "Unknown error";
    }
}
