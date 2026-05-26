using System.Diagnostics;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace DINOForge.Tools.McpServer.Cua;

/// <summary>
/// Shared stdio JSON-RPC 2.0 client for bare-cua / playcua native binaries.
/// </summary>
public class CuaNativeClient : IAsyncDisposable
{
    private readonly CuaNativeClientOptions _options;
    private Process? _proc;
    private int _id;
    private readonly SemaphoreSlim _lock = new(1, 1);

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    protected CuaNativeClient(CuaNativeClientOptions options)
    {
        _options = options;
    }

    /// <summary>Start the native binary and verify it responds to ping.</summary>
    protected static async Task<TClient> StartAsync<TClient>(
        CuaNativeClientOptions options,
        string? nativePath = null,
        string logLevel = "info",
        CancellationToken ct = default)
        where TClient : CuaNativeClient, new()
    {
        string executable = nativePath ?? options.DefaultExecutableName;
        var psi = new ProcessStartInfo(executable)
        {
            UseShellExecute = false,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = false,
            StandardInputEncoding = Encoding.UTF8,
            StandardOutputEncoding = Encoding.UTF8,
        };
        psi.Environment[options.LogEnvironmentVariable] = logLevel;

        var client = new TClient();
        try
        {
            client._proc = Process.Start(psi)
                ?? throw new InvalidOperationException($"Failed to start {executable}");

            JsonElement pong = await client.CallAsync("ping", null, ct).ConfigureAwait(false);
            if (pong.ValueKind == JsonValueKind.Undefined || !pong.TryGetProperty("ok", out _))
            {
                throw new InvalidOperationException($"{options.ProcessDisplayName} did not respond to ping");
            }

            return client;
        }
        catch
        {
            await client.DisposeAsync().ConfigureAwait(false);
            throw;
        }
    }

    public async Task<byte[]> ScreenshotAsync(
        string? windowTitle = null,
        int monitor = 0,
        CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("screenshot", new { window_title = windowTitle, monitor }, ct)
            .ConfigureAwait(false);
        string b64 = result.GetProperty("data").GetString()
            ?? throw new InvalidOperationException("screenshot: missing data field");
        return Convert.FromBase64String(b64);
    }

    public async Task ClickAsync(
        int x,
        int y,
        string button = "left",
        string action = "click",
        CancellationToken ct = default)
    {
        await CallAsync("input.click", new { x, y, button, action }, ct).ConfigureAwait(false);
    }

    public async Task DoubleClickAsync(int x, int y, CancellationToken ct = default)
    {
        await ClickAsync(x, y, "left", "click", ct).ConfigureAwait(false);
        await Task.Delay(50, ct).ConfigureAwait(false);
        await ClickAsync(x, y, "left", "click", ct).ConfigureAwait(false);
    }

    public async Task ScrollAsync(
        int x,
        int y,
        string direction = "down",
        int amount = 3,
        CancellationToken ct = default)
    {
        await CallAsync("input.scroll", new { x, y, direction, amount }, ct).ConfigureAwait(false);
    }

    public async Task MoveMouseAsync(int x, int y, CancellationToken ct = default)
    {
        await CallAsync("input.move", new { x, y }, ct).ConfigureAwait(false);
    }

    public async Task TypeTextAsync(string text, CancellationToken ct = default)
    {
        await CallAsync("input.type", new { text }, ct).ConfigureAwait(false);
    }

    public async Task PressKeyAsync(string key, CancellationToken ct = default)
    {
        await CallAsync("input.key", new { key, action = "press" }, ct).ConfigureAwait(false);
    }

    public async Task KeyDownAsync(string key, CancellationToken ct = default)
    {
        await CallAsync("input.key", new { key, action = "down" }, ct).ConfigureAwait(false);
    }

    public async Task KeyUpAsync(string key, CancellationToken ct = default)
    {
        await CallAsync("input.key", new { key, action = "up" }, ct).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<WindowInfo>> ListWindowsAsync(CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("windows.list", new { }, ct).ConfigureAwait(false);
        return JsonSerializer.Deserialize<List<WindowInfo>>(result.GetRawText(), JsonOptions) ?? [];
    }

    public async Task<WindowInfo?> FindWindowAsync(
        string? title = null,
        int? pid = null,
        CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("windows.find", new { title, pid }, ct).ConfigureAwait(false);
        if (result.ValueKind == JsonValueKind.Null)
            return null;
        return JsonSerializer.Deserialize<WindowInfo>(result.GetRawText(), JsonOptions);
    }

    public async Task FocusWindowAsync(long hwnd, CancellationToken ct = default)
    {
        await CallAsync("windows.focus", new { hwnd }, ct).ConfigureAwait(false);
    }

    public async Task<int> LaunchProcessAsync(
        string path,
        string[]? args = null,
        string? cwd = null,
        CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("process.launch", new { path, args, cwd }, ct).ConfigureAwait(false);
        return result.GetProperty("pid").GetInt32();
    }

    public async Task KillProcessAsync(int pid, CancellationToken ct = default)
    {
        await CallAsync("process.kill", new { pid }, ct).ConfigureAwait(false);
    }

    public async Task<ProcessStatus> ProcessStatusAsync(int pid, CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("process.status", new { pid }, ct).ConfigureAwait(false);
        return JsonSerializer.Deserialize<ProcessStatus>(result.GetRawText(), JsonOptions)
            ?? new ProcessStatus(false, null);
    }

    public async Task<bool> FramesDifferAsync(
        byte[] imageA,
        byte[] imageB,
        double threshold = 0.02,
        CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("analysis.diff", new
        {
            image_a = Convert.ToBase64String(imageA),
            image_b = Convert.ToBase64String(imageB),
            threshold,
        }, ct).ConfigureAwait(false);
        return result.GetProperty("changed").GetBoolean();
    }

    public async Task<string> ImageHashAsync(byte[] image, CancellationToken ct = default)
    {
        JsonElement result = await CallAsync("analysis.hash", new { image = Convert.ToBase64String(image) }, ct)
            .ConfigureAwait(false);
        return result.GetProperty("hash").GetString() ?? string.Empty;
    }

    public async Task<bool> PingAsync(CancellationToken ct = default)
    {
        try
        {
            JsonElement result = await CallAsync("ping", new { }, ct).ConfigureAwait(false);
            return result.TryGetProperty("ok", out JsonElement ok) && ok.GetBoolean();
        }
        catch
        {
            return false;
        }
    }

    private async Task<JsonElement> CallAsync(string method, object? @params, CancellationToken ct = default)
    {
        if (_proc is null || _proc.HasExited)
            throw new ObjectDisposedException(GetType().Name, "Native process is not running");

        await _lock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            int id = Interlocked.Increment(ref _id);
            var request = new { jsonrpc = "2.0", id, method, @params = @params ?? new { } };

            string reqJson = JsonSerializer.Serialize(request, JsonOptions) + "\n";
            await _proc.StandardInput.WriteAsync(reqJson.AsMemory(), ct).ConfigureAwait(false);
            await _proc.StandardInput.FlushAsync(ct).ConfigureAwait(false);

            string? respLine = await _proc.StandardOutput.ReadLineAsync(ct).ConfigureAwait(false);
            if (respLine is null)
            {
                throw new InvalidOperationException($"{_options.ProcessDisplayName} closed stdout unexpectedly");
            }

            using JsonDocument doc = JsonDocument.Parse(respLine);
            JsonElement root = doc.RootElement.Clone();

            if (root.TryGetProperty("error", out JsonElement errEl) && errEl.ValueKind != JsonValueKind.Null)
            {
                int code = errEl.TryGetProperty("code", out JsonElement c) ? c.GetInt32() : -1;
                string msg = errEl.TryGetProperty("message", out JsonElement m)
                    ? m.GetString() ?? "unknown"
                    : "unknown";
                throw new RpcException(code, msg);
            }

            if (!root.TryGetProperty("result", out JsonElement resultEl))
                return default;

            return resultEl;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (_proc is not null)
        {
            try
            {
                _proc.StandardInput.Close();
                await _proc.WaitForExitAsync(CancellationToken.None)
                    .WaitAsync(TimeSpan.FromSeconds(3))
                    .ConfigureAwait(false);
            }
            catch
            {
                try { _proc.Kill(); } catch { /* ignore */ }
            }

            _proc.Dispose();
            _proc = null;
        }

        _lock.Dispose();
    }
}
