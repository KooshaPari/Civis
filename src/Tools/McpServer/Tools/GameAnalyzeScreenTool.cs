#nullable enable
using System.ComponentModel;
using System.Text;
using System.Text.Json;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that captures a game screenshot and optionally analyzes it with OmniParser
/// to detect UI elements (health bars, unit portraits, buttons, faction indicators).
/// OmniParser integration supports both local Docker REST API and Replicate cloud API.
/// Configuration:
///   OMNIPARSER_ENDPOINT - local REST endpoint (default: http://localhost:8000)
///   OMNIPARSER_MODE     - "local" or "replicate" (default: local)
///   REPLICATE_API_KEY   - required when OMNIPARSER_MODE=replicate
/// </summary>
[McpServerToolType]
public sealed class GameAnalyzeScreenTool
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = false,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    /// <summary>
    /// Captures a game screenshot and analyzes it with OmniParser to detect UI elements.
    /// Returns detected elements with bounding boxes. Gracefully degrades if OmniParser is unavailable.
    /// </summary>
    [McpServerTool(Name = "game_analyze_screen"), Description("Capture game screenshot and detect UI elements (health bars, unit portraits, buttons, faction indicators) using OmniParser. Returns bounding boxes and labels for all detected elements.")]
    public static async Task<string> AnalyzeScreenAsync(
        GameClient client,
        [Description("Optional file path to save the screenshot")] string? path = null,
        CancellationToken ct = default)
    {
        try
        {
            // Step 1: Capture screenshot via GameScreenshotTool
            string screenshotJson = await GameScreenshotTool.ScreenshotAsync(client, path, returnBase64: true, ct).ConfigureAwait(false);

            // Step 2: Parse the screenshot result
            string? screenshotPath = null;
            string? base64Image = null;
            bool screenshotSuccess = false;

            using (var doc = JsonDocument.Parse(screenshotJson))
            {
                var root = doc.RootElement;
                screenshotSuccess = root.TryGetProperty("success", out var s) && s.GetBoolean();
                screenshotPath = root.TryGetProperty("path", out var p) ? p.GetString() : null;
                base64Image = root.TryGetProperty("base64", out var b) ? b.GetString() : null;
            }

            if (!screenshotSuccess || string.IsNullOrEmpty(screenshotPath))
            {
                return GameClientHelper.ToJson(new
                {
                    success = false,
                    error = "Failed to capture screenshot for analysis."
                });
            }

            // Step 3: If OmniParser is configured, call it
            string omniparserMode = Environment.GetEnvironmentVariable("OMNIPARSER_MODE")?.ToLowerInvariant() ?? "local";
            string omniparserEndpoint = Environment.GetEnvironmentVariable("OMNIPARSER_ENDPOINT") ?? "http://localhost:8000";

            if (string.IsNullOrEmpty(base64Image))
            {
                // Couldn't get base64 from screenshot, try reading file directly
                if (File.Exists(screenshotPath))
                {
                    byte[] bytes = await File.ReadAllBytesAsync(screenshotPath, ct).ConfigureAwait(false);
                    base64Image = Convert.ToBase64String(bytes);
                }
            }

            OmniParserElement[]? elements = null;
            if (!string.IsNullOrEmpty(base64Image))
            {
                elements = omniparserMode == "replicate"
                    ? await CallReplicateApiAsync(base64Image, ct).ConfigureAwait(false)
                    : await CallLocalApiAsync(base64Image, omniparserEndpoint, ct).ConfigureAwait(false);
            }

            // Step 4: Return result with graceful degradation
            if (elements == null)
            {
                return GameClientHelper.ToJson(new
                {
                    success = false,
                    screenshotPath,
                    elements = Array.Empty<object>(),
                    note = $"Screenshot captured but OmniParser unavailable at {omniparserEndpoint}. " +
                           "Set OMNIPARSER_ENDPOINT and OMNIPARSER_MODE to enable element detection. " +
                           "Start OmniParser: docker run -p 8000:8000 microsoft/omniparser:latest"
                });
            }

            return GameClientHelper.ToJson(new
            {
                success = true,
                screenshotPath,
                elements,
                elementCount = elements.Length,
                timestamp = DateTime.UtcNow.ToString("o")
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = ex.Message });
        }
    }

    private static async Task<OmniParserElement[]?> CallLocalApiAsync(string base64Image, string endpoint, CancellationToken ct)
    {
        try
        {
            using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(30) };
            string url = endpoint.TrimEnd('/') + "/parse/";
            string body = JsonSerializer.Serialize(new { image = base64Image }, JsonOptions);
            using var content = new StringContent(body, Encoding.UTF8, "application/json");
            using var response = await http.PostAsync(url, content, ct).ConfigureAwait(false);
            if (!response.IsSuccessStatusCode) return null;
            string json = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
            return ParseElements(json);
        }
        catch { return null; }
    }

    private static async Task<OmniParserElement[]?> CallReplicateApiAsync(string base64Image, CancellationToken ct)
    {
        try
        {
            string? apiKey = Environment.GetEnvironmentVariable("REPLICATE_API_KEY");
            if (string.IsNullOrEmpty(apiKey)) return null;

            using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(60) };
            http.DefaultRequestHeaders.Add("Authorization", $"Token {apiKey}");
            string body = JsonSerializer.Serialize(new
            {
                version = "latest",
                input = new { image = $"data:image/png;base64,{base64Image}" }
            }, JsonOptions);
            using var content = new StringContent(body, Encoding.UTF8, "application/json");
            using var response = await http.PostAsync("https://api.replicate.com/v1/predictions", content, ct).ConfigureAwait(false);
            if (!response.IsSuccessStatusCode) return null;
            string json = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
            return ParseElements(json);
        }
        catch { return null; }
    }

    private static OmniParserElement[]? ParseElements(string json)
    {
        try
        {
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            // Look for "elements" array in response
            if (!root.TryGetProperty("elements", out var elemArray) || elemArray.ValueKind != JsonValueKind.Array)
                return null;

            var result = new List<OmniParserElement>();
            foreach (var elem in elemArray.EnumerateArray())
            {
                if (elem.ValueKind != JsonValueKind.Object) continue;
                string? label = elem.TryGetProperty("label", out var l) ? l.GetString() : null;
                if (string.IsNullOrEmpty(label)) continue;

                int[]? bbox = null;
                if (elem.TryGetProperty("bbox", out var b) && b.ValueKind == JsonValueKind.Array)
                {
                    var coords = b.EnumerateArray().Select(c => c.TryGetInt32(out int v) ? v : 0).ToArray();
                    if (coords.Length == 4) bbox = coords;
                }

                result.Add(new OmniParserElement
                {
                    Label = label,
                    BBox = bbox ?? Array.Empty<int>(),
                    Confidence = elem.TryGetProperty("confidence", out var c) && c.TryGetDouble(out double cv) ? cv : 0.0,
                    Type = elem.TryGetProperty("type", out var t) ? t.GetString() : null,
                    Text = elem.TryGetProperty("text", out var tx) ? tx.GetString() : null
                });
            }

            return result.ToArray();
        }
        catch { return null; }
    }

    public sealed class OmniParserElement
    {
        public string? Label { get; set; }
        public int[] BBox { get; set; } = Array.Empty<int>();
        public double Confidence { get; set; }
        public string? Type { get; set; }
        public string? Text { get; set; }
    }
}
