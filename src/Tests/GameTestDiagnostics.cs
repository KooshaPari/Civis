#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace DINOForge.Tests
{
    /// <summary>
    /// Captures diagnostic state when game-launch tests fail.
    /// Records screenshots, logs, process info, and performs root cause analysis.
    /// </summary>
    public static class GameDiagnosticsCapture
    {
        private static readonly string FailuresDirectory = Path.Combine(
            Environment.GetEnvironmentVariable("TEMP") ?? "C:\\temp",
            "DINOForge", "failures");

        private const string GameProcessName = "Diplomacy is Not an Option";
        private const string GameDir = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option";

        /// <summary>
        /// Captures the current failure state including screenshot, logs, and process info.
        /// </summary>
        /// <param name="testName">Name of the failing test (optional)</param>
        /// <returns>JSON string of failure manifest</returns>
        public static async Task<string> CaptureFailureStateAsync(string? testName = null)
        {
            try
            {
                // Ensure output directory exists
                Directory.CreateDirectory(FailuresDirectory);

                var timestamp = DateTime.UtcNow.ToString("yyyyMMdd_HHmmss");
                var screenshotPath = await CaptureScreenshotAsync(timestamp);
                var processInfo = CaptureProcessInfo();
                var dinoforgeLogTail = ReadLogTail(
                    Path.Combine(GameDir, "BepInEx", "dinoforge_debug.log"), 100);
                var bepinexLogTail = ReadLogTail(
                    Path.Combine(GameDir, "BepInEx", "LogOutput.log"), 100);

                var manifest = new FailureManifest
                {
                    Timestamp = timestamp,
                    TestName = testName,
                    ScreenshotPath = screenshotPath,
                    Process = processInfo,
                    LogTailDinoforge = dinoforgeLogTail,
                    LogTailBepinex = bepinexLogTail,
                    EntityCount = await GetEntityCountAsync(),
                    CapturedAt = DateTime.UtcNow.ToString("O")
                };

                var manifestJson = JsonSerializer.Serialize(manifest, new JsonSerializerOptions
                {
                    WriteIndented = true,
                    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
                });

                var manifestPath = Path.Combine(FailuresDirectory, $"failure_{timestamp}.json");
                await File.WriteAllTextAsync(manifestPath, manifestJson);

                return manifestJson;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Error capturing failure state: {ex.Message}");
                return string.Empty;
            }
        }

        /// <summary>
        /// Analyzes the most recent failure to determine root cause.
        /// </summary>
        /// <returns>Analysis result with root cause and affected systems</returns>
        public static async Task<FailureAnalysis> AnalyzeFailureRootAsync()
        {
            try
            {
                if (!Directory.Exists(FailuresDirectory))
                {
                    return new FailureAnalysis
                    {
                        RootCause = "No failure manifests found",
                        ErrorPattern = "No failures captured"
                    };
                }

                var manifestFiles = Directory.GetFiles(FailuresDirectory, "failure_*.json")
                    .OrderByDescending(f => new FileInfo(f).LastWriteTimeUtc)
                    .FirstOrDefault();

                if (manifestFiles == null)
                {
                    return new FailureAnalysis
                    {
                        RootCause = "No failure manifests found",
                        ErrorPattern = "No failures captured"
                    };
                }

                var manifestJson = await File.ReadAllTextAsync(manifestFiles);
                var manifest = JsonSerializer.Deserialize<FailureManifest>(manifestJson);

                if (manifest == null)
                {
                    return new FailureAnalysis
                    {
                        RootCause = "Failed to parse failure manifest",
                        ErrorPattern = "Manifest parse error"
                    };
                }

                return AnalyzeManifest(manifest);
            }
            catch (Exception ex)
            {
                return new FailureAnalysis
                {
                    RootCause = $"Analysis error: {ex.Message}",
                    ErrorPattern = "Unknown error"
                };
            }
        }

        /// <summary>
        /// Captures a screenshot of the game window.
        /// </summary>
        private static async Task<string> CaptureScreenshotAsync(string timestamp)
        {
            var screenshotPath = Path.Combine(FailuresDirectory, $"{timestamp}_screenshot.png");

            try
            {
                // In production, this would call MCP game_screenshot tool
                // For now, just create a placeholder file
                await File.WriteAllBytesAsync(screenshotPath, Array.Empty<byte>());
                return screenshotPath;
            }
            catch
            {
                return "FAILED_TO_CAPTURE_SCREENSHOT";
            }
        }

        /// <summary>
        /// Captures process information for the running game.
        /// </summary>
        private static ProcessInfo CaptureProcessInfo()
        {
            try
            {
                var processes = Process.GetProcessesByName(GameProcessName);
                if (processes.Length == 0)
                {
                    return new ProcessInfo
                    {
                        Status = "Game process not running"
                    };
                }

                var process = processes[0];
                return new ProcessInfo
                {
                    Pid = process.Id,
                    MainWindowTitle = process.MainWindowTitle ?? string.Empty,
                    MemoryMb = process.WorkingSet64 / (1024 * 1024),
                    Status = "Running"
                };
            }
            catch (Exception ex)
            {
                return new ProcessInfo
                {
                    Status = $"Error capturing process info: {ex.Message}"
                };
            }
        }

        /// <summary>
        /// Gets the current entity count from the game (requires MCP server).
        /// </summary>
        private static async Task<int> GetEntityCountAsync()
        {
            try
            {
                // In production, this would call MCP game_status tool
                // For now, return 0
                return await Task.FromResult(0);
            }
            catch
            {
                return -1;
            }
        }

        /// <summary>
        /// Reads the last N lines from a log file, handling large files efficiently.
        /// </summary>
        private static List<string> ReadLogTail(string filePath, int lineCount = 100)
        {
            var lines = new List<string>();

            try
            {
                if (!File.Exists(filePath))
                {
                    lines.Add($"[LOG NOT FOUND: {filePath}]");
                    return lines;
                }

                // For small files, just read all lines
                var fileInfo = new FileInfo(filePath);
                if (fileInfo.Length < 1024 * 1024) // < 1 MB
                {
                    var allLines = File.ReadAllLines(filePath);
                    return allLines.TakeLast(lineCount).ToList();
                }

                // For large files, read from the end
                using (var stream = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.Read))
                {
                    var buffer = new byte[4096];
                    var queue = new Queue<string>(lineCount);

                    stream.Seek(-Math.Min(stream.Length, 1024 * 100), SeekOrigin.End);
                    int bytesRead;

                    var sb = new StringBuilder();
                    while ((bytesRead = stream.Read(buffer, 0, buffer.Length)) > 0)
                    {
                        var text = Encoding.UTF8.GetString(buffer, 0, bytesRead);
                        sb.Append(text);
                    }

                    var fileLines = sb.ToString().Split(new[] { "\r\n", "\r", "\n" },
                        StringSplitOptions.None);

                    foreach (var line in fileLines.TakeLast(lineCount))
                    {
                        if (!string.IsNullOrEmpty(line))
                        {
                            queue.Enqueue(line);
                        }
                    }

                    return queue.ToList();
                }
            }
            catch (Exception ex)
            {
                lines.Add($"[ERROR READING LOG: {ex.Message}]");
                return lines;
            }
        }

        /// <summary>
        /// Analyzes a failure manifest to extract root cause information.
        /// </summary>
        private static FailureAnalysis AnalyzeManifest(FailureManifest manifest)
        {
            var errorPatterns = new Dictionary<string, string>
            {
                { "ERROR", "Error logged" },
                { "FATAL", "Fatal error" },
                { "Exception", "Exception thrown" },
                { "failed", "Operation failed" },
                { "Failed", "Operation failed" }
            };

            var systemKeywords = new Dictionary<string, string>
            {
                { "Warfare", "Warfare domain" },
                { "Economy", "Economy domain" },
                { "Scenario", "Scenario domain" },
                { "UI", "UI domain" },
                { "Runtime", "DINOForge runtime" },
                { "Plugin", "Plugin system" },
                { "ECS", "ECS system" },
                { "Entity", "Entity system" },
                { "System", "System group" }
            };

            var allLogs = new List<string>();
            allLogs.AddRange(manifest.LogTailDinoforge ?? new List<string>());
            allLogs.AddRange(manifest.LogTailBepinex ?? new List<string>());

            var rootCause = string.Empty;
            var affectedSystems = new HashSet<string>();
            var lastSuccessfulEvent = string.Empty;

            foreach (var line in allLogs)
            {
                foreach (var (pattern, desc) in errorPatterns)
                {
                    if (line.Contains(pattern, StringComparison.OrdinalIgnoreCase))
                    {
                        if (string.IsNullOrEmpty(rootCause))
                        {
                            rootCause = line.Trim();
                        }

                        foreach (var (keyword, systemDesc) in systemKeywords)
                        {
                            if (line.Contains(keyword, StringComparison.OrdinalIgnoreCase))
                            {
                                affectedSystems.Add(systemDesc);
                            }
                        }
                    }
                }

                if (!line.Contains("ERROR", StringComparison.OrdinalIgnoreCase) &&
                    !line.Contains("FATAL", StringComparison.OrdinalIgnoreCase) &&
                    !line.Contains("Exception", StringComparison.OrdinalIgnoreCase) &&
                    !string.IsNullOrWhiteSpace(line))
                {
                    lastSuccessfulEvent = line.Trim();
                }
            }

            return new FailureAnalysis
            {
                RootCause = string.IsNullOrEmpty(rootCause)
                    ? "No explicit error found in logs"
                    : rootCause,
                AffectedSystems = affectedSystems.ToList(),
                LastSuccessfulEvent = lastSuccessfulEvent,
                ErrorPattern = DetermineErrorPattern(rootCause)
            };
        }

        /// <summary>
        /// Determines the high-level error pattern from a root cause line.
        /// </summary>
        private static string DetermineErrorPattern(string rootCause)
        {
            if (string.IsNullOrEmpty(rootCause))
                return "Unknown error";

            if (rootCause.Contains("Fatal", StringComparison.OrdinalIgnoreCase))
                return "Fatal error - immediate crash";

            if (rootCause.Contains("Plugin", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("load", StringComparison.OrdinalIgnoreCase))
                return "Plugin/module loading failure";

            if (rootCause.Contains("Entity", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("ECS", StringComparison.OrdinalIgnoreCase))
                return "Entity Component System failure";

            if (rootCause.Contains("Config", StringComparison.OrdinalIgnoreCase) ||
                rootCause.Contains("Registry", StringComparison.OrdinalIgnoreCase))
                return "Configuration or registry error";

            return "Unclassified error";
        }
    }

    /// <summary>
    /// Represents a captured failure manifest.
    /// </summary>
    public class FailureManifest
    {
        [JsonPropertyName("timestamp")]
        public string? Timestamp { get; set; }

        [JsonPropertyName("test_name")]
        public string? TestName { get; set; }

        [JsonPropertyName("screenshot_path")]
        public string? ScreenshotPath { get; set; }

        [JsonPropertyName("process")]
        public ProcessInfo? Process { get; set; }

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
    /// Represents process information captured at failure time.
    /// </summary>
    public class ProcessInfo
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
    /// Represents root cause analysis of a failure.
    /// </summary>
    public class FailureAnalysis
    {
        [JsonPropertyName("root_cause")]
        public string? RootCause { get; set; }

        [JsonPropertyName("affected_systems")]
        public List<string> AffectedSystems { get; set; } = new();

        [JsonPropertyName("last_successful_event")]
        public string? LastSuccessfulEvent { get; set; }

        [JsonPropertyName("error_pattern")]
        public string? ErrorPattern { get; set; }
    }
}
