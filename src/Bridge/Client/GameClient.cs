#nullable enable
using System;
using System.IO;
using System.IO.Pipes;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Bridge.Client;

/// <summary>
/// Client for communicating with the DINOForge in-game IPC bridge server
/// over named pipes using JSON-RPC 2.0.
/// </summary>
/// <remarks>
/// Thread-safe. All public methods use internal locking to ensure that
/// only one request is in flight at a time on the underlying pipe stream.
/// </remarks>
public sealed class GameClient : IGameClient
{
    private readonly GameClientOptions _options;
    private readonly SemaphoreSlim _sendLock = new(1, 1);
    private readonly object _stateLock = new();

    private NamedPipeClientStream? _pipe;
    private StreamReader? _reader;
    private StreamWriter? _writer;
    private ConnectionState _state = ConnectionState.Disconnected;
    private bool _disposed;

    /// <summary>
    /// Initializes a new instance of <see cref="GameClient"/> with default options.
    /// </summary>
    public GameClient() : this(new GameClientOptions()) { }

    /// <summary>
    /// Initializes a new instance of <see cref="GameClient"/> with the specified options.
    /// </summary>
    /// <param name="options">Client configuration options.</param>
    public GameClient(GameClientOptions options)
    {
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    /// <summary>Gets the current connection state.</summary>
    public ConnectionState State
    {
        get { lock (_stateLock) return _state; }
        private set { lock (_stateLock) _state = value; }
    }

    /// <summary>Gets whether the client is currently connected to the game bridge.</summary>
    public bool IsConnected => State == ConnectionState.Connected;

    /// <summary>
    /// Connects to the game bridge named pipe server.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <exception cref="GameClientException">Thrown when the connection fails.</exception>
    public async Task ConnectAsync(CancellationToken ct = default)
    {
        ThrowIfDisposed();

        if (IsConnected)
            return;

        State = ConnectionState.Connecting;
        try
        {
            _pipe = new NamedPipeClientStream(".", _options.PipeName, PipeDirection.InOut, PipeOptions.Asynchronous);
            await _pipe.ConnectAsync(_options.ConnectTimeoutMs, ct).ConfigureAwait(false);

            _reader = new StreamReader(_pipe);
            _writer = new StreamWriter(_pipe) { AutoFlush = true };

            State = ConnectionState.Connected;
        }
        catch (Exception ex)
        {
            State = ConnectionState.Error;
            CleanupPipe();
            if (ex is OperationCanceledException)
                throw;
            throw new GameClientException($"Failed to connect to pipe '{_options.PipeName}'.", ex);
        }
    }

    /// <summary>
    /// Disconnects from the game bridge server.
    /// </summary>
    public void Disconnect()
    {
        CleanupPipe();
        State = ConnectionState.Disconnected;
    }

    /// <inheritdoc />
    public Task<PingResult> PingAsync(CancellationToken ct = default) =>
        SendRequestAsync<PingResult>("ping", null, ct);

    /// <inheritdoc />
    public Task<GameStatus> StatusAsync(CancellationToken ct = default) =>
        SendRequestAsync<GameStatus>("status", null, ct);

    /// <inheritdoc />
    public Task<WaitResult> WaitForWorldAsync(int? timeoutMs = null, CancellationToken ct = default) =>
        SendRequestAsync<WaitResult>("waitForWorld", timeoutMs.HasValue ? new { timeoutMs } : null, ct);

    /// <inheritdoc />
    public Task<QueryResult> QueryEntitiesAsync(string? componentType = null, string? category = null, CancellationToken ct = default) =>
        SendRequestAsync<QueryResult>("queryEntities", new { componentType, category }, ct);

    /// <inheritdoc />
    public Task<StatResult> GetStatAsync(string sdkPath, int? entityIndex = null, CancellationToken ct = default) =>
        SendRequestAsync<StatResult>("getStat", new { sdkPath, entityIndex }, ct);

    /// <inheritdoc />
    public Task<OverrideResult> ApplyOverrideAsync(string sdkPath, float value, string? mode = null, string? filter = null, CancellationToken ct = default) =>
        SendRequestAsync<OverrideResult>("applyOverride", new { sdkPath, value, mode, filter }, ct);

    /// <inheritdoc />
    public Task<ReloadResult> ReloadPacksAsync(string? path = null, CancellationToken ct = default) =>
        SendRequestAsync<ReloadResult>("reloadPacks", path != null ? new { path } : null, ct);

    /// <inheritdoc />
    public Task<CatalogSnapshot> GetCatalogAsync(CancellationToken ct = default) =>
        SendRequestAsync<CatalogSnapshot>("getCatalog", null, ct);

    /// <inheritdoc />
    public Task<CatalogSnapshot> DumpStateAsync(string? category = null, CancellationToken ct = default) =>
        SendRequestAsync<CatalogSnapshot>("dumpState", category != null ? new { category } : null, ct);

    /// <inheritdoc />
    public Task<ResourceSnapshot> GetResourcesAsync(CancellationToken ct = default) =>
        SendRequestAsync<ResourceSnapshot>("getResources", null, ct);

    /// <inheritdoc />
    public Task<ScreenshotResult> ScreenshotAsync(string? path = null, CancellationToken ct = default) =>
        SendRequestAsync<ScreenshotResult>("screenshot", path != null ? new { path } : null, ct);

    /// <inheritdoc />
    public Task<LoadSceneResult> LoadSceneAsync(string scene, CancellationToken ct = default) =>
        SendRequestAsync<LoadSceneResult>("loadScene", new { scene }, ct);

    /// <inheritdoc />
    public Task<StartGameResult> StartGameAsync(CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("startGame", null, ct);

    /// <summary>Lists available save files discovered by the game bridge.</summary>
    public Task<JObject> ListSavesAsync(CancellationToken ct = default) =>
        SendRequestAsync<JObject>("listSaves", null, ct);

    /// <summary>Dismisses the "Press Any Key to Continue" loading screen.</summary>
    public Task<StartGameResult> DismissLoadScreenAsync(CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("dismissLoadScreen", null, ct);

    /// <summary>Loads a save file by name (e.g. "AUTOSAVE_1" or "CONTINUE").</summary>
    public Task<StartGameResult> LoadSaveAsync(string saveName = "AUTOSAVE_1", CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("loadSave", new { saveName }, ct);

    /// <summary>
    /// Clicks a named Unity UI button. Pass empty string to list all active buttons.
    /// Use "DINOForge_ModsButton" to click the injected Mods button.
    /// </summary>
    public Task<StartGameResult> ClickButtonAsync(string buttonName, CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("clickButton", new { buttonName }, ct);

    /// <summary>
    /// Toggles a DINOForge UI panel. target="modmenu" (F10) or "debug" (F9).
    /// </summary>
    public Task<StartGameResult> ToggleUiAsync(string target = "modmenu", CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("toggleUi", new { target }, ct);

    /// <summary>
    /// Dumps active MonoBehaviours and their void() methods. filter narrows by type/GO name.
    /// Uses the pressKey bridge endpoint (repurposed as scanScene).
    /// </summary>
    public Task<StartGameResult> ScanSceneAsync(string filter = "", CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("pressKey", new { filter }, ct);

    /// <summary>
    /// Invokes a void(0-param) method on any active MonoBehaviour matching target (type or GO name).
    /// </summary>
    public Task<StartGameResult> InvokeMethodAsync(string target, string method, CancellationToken ct = default) =>
        SendRequestAsync<StartGameResult>("invokeMethod", new { target, method }, ct);

    /// <inheritdoc />
    public Task<VerifyResult> VerifyModAsync(string packPath, CancellationToken ct = default) =>
        SendRequestAsync<VerifyResult>("verifyMod", new { packPath }, ct);

    /// <inheritdoc />
    public Task<ComponentMapResult> GetComponentMapAsync(string? sdkPath = null, CancellationToken ct = default) =>
        SendRequestAsync<ComponentMapResult>("getComponentMap", sdkPath != null ? new { sdkPath } : null, ct);

    /// <summary>
    /// Captures a live snapshot of the active Unity UI hierarchy.
    /// </summary>
    /// <param name="selector">Optional selector string for future filtering.</param>
    /// <param name="ct">Cancellation token.</param>
    public Task<UiTreeResult> GetUiTreeAsync(string? selector = null, CancellationToken ct = default) =>
        SendRequestAsync<UiTreeResult>("getUiTree", selector != null ? new { selector } : null, ct);

    /// <summary>
    /// Queries the live Unity UI hierarchy using a simple selector grammar.
    /// </summary>
    public Task<UiActionResult> QueryUiAsync(string selector, CancellationToken ct = default) =>
        SendRequestAsync<UiActionResult>("queryUi", new { selector }, ct);

    /// <summary>
    /// Clicks the first live Unity UI node matching the given selector.
    /// </summary>
    public Task<UiActionResult> ClickUiAsync(string selector, CancellationToken ct = default) =>
        SendRequestAsync<UiActionResult>("clickUi", new { selector }, ct);

    /// <summary>
    /// Waits for a live Unity UI selector to reach the requested state.
    /// </summary>
    public Task<UiWaitResult> WaitForUiAsync(string selector, string? state = null, int? timeoutMs = null, CancellationToken ct = default) =>
        SendRequestAsync<UiWaitResult>("waitForUi", new { selector, state, timeoutMs }, ct);

    /// <summary>
    /// Asserts a condition against the first node matching the given selector.
    /// </summary>
    public Task<UiExpectationResult> ExpectUiAsync(string selector, string condition, CancellationToken ct = default) =>
        SendRequestAsync<UiExpectationResult>("expectUi", new { selector, condition }, ct);

    /// <summary>
    /// Sends a JSON-RPC request and returns the deserialized result.
    /// Handles serialization, pipe I/O, error checking, timeout, and retries.
    /// </summary>
    /// <typeparam name="T">The expected result type.</typeparam>
    /// <param name="method">The JSON-RPC method name.</param>
    /// <param name="parameters">Optional method parameters.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The deserialized result of type <typeparamref name="T"/>.</returns>
    /// <exception cref="GameClientException">Thrown on communication or server errors.</exception>
    internal async Task<T> SendRequestAsync<T>(string method, object? parameters, CancellationToken ct = default)
    {
        ThrowIfDisposed();

        Exception? lastException = null;

        for (int attempt = 0; attempt <= _options.RetryCount; attempt++)
        {
            if (attempt > 0)
            {
                await Task.Delay(_options.RetryDelayMs, ct).ConfigureAwait(false);
            }

            try
            {
                return await SendRequestCoreAsync<T>(method, parameters, ct).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception ex)
            {
                lastException = ex;
                // If the pipe broke, try to reconnect
                if (!IsConnected)
                {
                    try
                    {
                        await ConnectAsync(ct).ConfigureAwait(false);
                    }
                    catch
                    {
                        // Will retry on next iteration
                    }
                }
            }
        }

        throw new GameClientException(
            $"Failed to execute '{method}' after {_options.RetryCount + 1} attempts.",
            lastException!);
    }

    private async Task<T> SendRequestCoreAsync<T>(string method, object? parameters, CancellationToken ct)
    {
        if (!IsConnected || _writer is null || _reader is null)
            throw new GameClientException("Not connected to the game bridge. Call ConnectAsync first.");

        JsonRpcRequest request = new()
        {
            Id = Guid.NewGuid().ToString("N"),
            Method = method,
            Params = parameters != null ? JObject.FromObject(parameters) : null
        };

        string requestJson = JsonConvert.SerializeObject(request, Formatting.None,
            new JsonSerializerSettings { NullValueHandling = NullValueHandling.Ignore });

        await _sendLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            await _writer.WriteLineAsync(requestJson).ConfigureAwait(false);

            using CancellationTokenSource timeoutCts = new(_options.ReadTimeoutMs);
            using CancellationTokenSource linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct, timeoutCts.Token);

            string? responseLine = await ReadLineAsync(_reader, linkedCts.Token).ConfigureAwait(false);

            if (responseLine is null)
            {
                State = ConnectionState.Error;
                throw new GameClientException("Connection closed by the game bridge server.");
            }

            JsonRpcResponse? response = JsonConvert.DeserializeObject<JsonRpcResponse>(responseLine);
            if (response is null)
                throw new GameClientException("Received invalid JSON-RPC response.");

            if (response.Error is not null)
                throw new GameClientException($"Server error [{response.Error.Code}]: {response.Error.Message}");

            if (response.Result is null)
                throw new GameClientException($"Server returned null result for '{method}'.");

            T? result = JsonConvert.DeserializeObject<T>(response.Result.ToString()!);
            if (result is null)
                throw new GameClientException($"Failed to deserialize result for '{method}'.");

            return result;
        }
        catch (OperationCanceledException ex) when (!ct.IsCancellationRequested)
        {
            throw new GameClientException($"Request '{method}' timed out after {_options.ReadTimeoutMs}ms.", ex);
        }
        finally
        {
            _sendLock.Release();
        }
    }

    private static async Task<string?> ReadLineAsync(StreamReader reader, CancellationToken ct)
    {
        // Wrap StreamReader.ReadLineAsync in a cancellation-aware Task.WhenAny.
        // StreamReader.ReadLineAsync doesn't accept CancellationToken in older .NET,
        // so we race it against a delay loop that checks the token.
        Task<string?> readTask = reader.ReadLineAsync();

        while (!readTask.IsCompleted)
        {
            ct.ThrowIfCancellationRequested();
            Task delayTask = Task.Delay(200, ct);
            await Task.WhenAny(readTask, delayTask).ConfigureAwait(false);
        }

        return readTask.Result;
    }

    private void CleanupPipe()
    {
        try { _reader?.Dispose(); } catch { }
        try { _writer?.Dispose(); } catch { }
        try { _pipe?.Dispose(); } catch { }
        _reader = null;
        _writer = null;
        _pipe = null;
    }

    private void ThrowIfDisposed()
    {
        if (_disposed) throw new ObjectDisposedException(GetType().Name);
    }

    /// <summary>
    /// Disposes the client and releases all resources.
    /// </summary>
    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        CleanupPipe();
        _sendLock.Dispose();
        State = ConnectionState.Disconnected;
    }
}
