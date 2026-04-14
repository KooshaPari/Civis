#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.IO.Pipes;
using System.Net;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Serilog;
using Serilog.Events;

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
    private readonly ILogger _logger;

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
        _logger = InitializeLogger();
    }

    /// <summary>
    /// Initializes the Serilog logger for structured logging to JSON/JSONL.
    /// </summary>
    private static ILogger InitializeLogger()
    {
        // Read request ID from environment variable (set by automation scripts)
        var requestId = Environment.GetEnvironmentVariable("DINO_REQUEST_ID") ?? "no-request-id";

        var logConfig = new LoggerConfiguration()
            .MinimumLevel.Debug()
            .Enrich.FromLogContext()
            .Enrich.WithProperty("ProcessName", Process.GetCurrentProcess().ProcessName)
            .Enrich.WithProperty("ProcessId", Process.GetCurrentProcess().Id)
            .Enrich.WithProperty("MachineName", Environment.MachineName)
            .Enrich.WithProperty("RequestId", requestId)
            .WriteTo.Console()
            .WriteTo.File(
                path: "logs/dinoforge-.jsonl",
                outputTemplate: "{Timestamp:u} [{Level:u3}] {Message:lj}{NewLine}{Exception}",
                rollingInterval: RollingInterval.Day);

        return logConfig.CreateLogger();
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
        await ConnectAsync(connectTimeout: null, ct);
    }

    /// <summary>
    /// Connects to the game bridge named pipe server with optional custom timeout.
    /// </summary>
    /// <param name="connectTimeout">Optional timeout override. If null, uses options default.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <exception cref="GameClientException">Thrown when the connection fails.</exception>
    public async Task ConnectAsync(TimeSpan? connectTimeout, CancellationToken ct = default)
    {
        ThrowIfDisposed();

        if (IsConnected)
        {
            _logger.Debug("Already connected to pipe '{PipeName}'", _options.PipeName);
            return;
        }

        State = ConnectionState.Connecting;
        var timeout = connectTimeout ?? TimeSpan.FromMilliseconds(_options.ConnectTimeoutMs);
        _logger.Information("Connecting to pipe '{PipeName}' with timeout {TimeoutMs}ms",
            _options.PipeName, timeout.TotalMilliseconds);

        try
        {
            _pipe = new NamedPipeClientStream(".", _options.PipeName, PipeDirection.InOut, PipeOptions.Asynchronous);

            using var timeoutCts = new CancellationTokenSource(timeout);
            using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct, timeoutCts.Token);

            try
            {
                await _pipe.ConnectAsync(linkedCts.Token).ConfigureAwait(false);
            }
            catch (OperationCanceledException ex) when (timeoutCts.Token.IsCancellationRequested)
            {
                throw new TimeoutException($"Connection timeout after {timeout.TotalSeconds}s", ex);
            }

            _reader = new StreamReader(_pipe);
            _writer = new StreamWriter(_pipe) { AutoFlush = true };

            State = ConnectionState.Connected;
            _logger.Information("Successfully connected to pipe '{PipeName}'", _options.PipeName);
        }
        catch (Exception ex)
        {
            State = ConnectionState.Error;
            CleanupPipe();
            _logger.Error(ex, "Failed to connect to pipe '{PipeName}'", _options.PipeName);
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
        _logger.Information("Disconnecting from pipe '{PipeName}'", _options.PipeName);
        CleanupPipe();
        State = ConnectionState.Disconnected;
        _logger.Debug("Disconnection complete");
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
    /// Invokes an arbitrary bridge method and returns the raw JSON result.
    /// Useful for debugging or calling methods not yet wrapped.
    /// </summary>
    /// <param name="method">The JSON-RPC method name.</param>
    /// <param name="parameters">Method parameters as an anonymous object.</param>
    /// <param name="ct">Cancellation token.</param>
    public Task<JObject> InvokeBridgeMethodAsync(string method, object? parameters = null, CancellationToken ct = default) =>
        SendRequestAsync<JObject>(method, parameters, ct);

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
                _logger.Warning("Retrying request '{Method}' (attempt {Attempt}/{MaxAttempts})", method, attempt + 1, _options.RetryCount + 1);
                await Task.Delay(_options.RetryDelayMs, ct).ConfigureAwait(false);
            }

            try
            {
                _logger.Debug("Sending request '{Method}' to pipe '{PipeName}' (attempt {Attempt}/{MaxAttempts})",
                    method, _options.PipeName, attempt + 1, _options.RetryCount + 1);
                return await SendRequestCoreAsync<T>(method, parameters, ct).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception ex)
            {
                lastException = ex;
                _logger.Warning(ex, "Request '{Method}' failed on attempt {Attempt}: {ErrorMessage}",
                    method, attempt + 1, ex.Message);
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

        _logger.Error(lastException, "Failed to execute '{Method}' after {RetryCount} attempts",
            method, _options.RetryCount + 1);
        throw new GameClientException(
            $"Failed to execute '{method}' after {_options.RetryCount + 1} attempts.",
            lastException!);
    }

    private async Task<T> SendRequestCoreAsync<T>(string method, object? parameters, CancellationToken ct,
        TimeSpan? sendTimeout = null, TimeSpan? readTimeout = null)
    {
        if (!IsConnected || (_writer is null && !_options.UseMessageFraming) || (_pipe is null && _options.UseMessageFraming))
        {
            _logger.Error("Cannot send request - not connected to pipe '{PipeName}'", _options.PipeName);
            throw new GameClientException("Not connected to the game bridge. Call ConnectAsync first.");
        }

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
            var sw = System.Diagnostics.Stopwatch.StartNew();

            // Send request with configurable timeout
            var effectiveSendTimeout = sendTimeout ?? TimeSpan.FromMilliseconds(_options.SendTimeoutMs);
            using var sendTimeoutCts = new CancellationTokenSource(effectiveSendTimeout);
            using var sendLinkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct, sendTimeoutCts.Token);

            try
            {
                if (_options.UseMessageFraming)
                {
                    await WriteFramedMessageAsync(requestJson, sendLinkedCts.Token).ConfigureAwait(false);
                }
                else
                {
                    await _writer!.WriteLineAsync(requestJson).ConfigureAwait(false);
                }
            }
            catch (OperationCanceledException ex) when (sendTimeoutCts.Token.IsCancellationRequested)
            {
                _logger.Error(ex, "Send timeout for request '{Method}' after {TimeoutMs}ms",
                    method, effectiveSendTimeout.TotalMilliseconds);
                throw new GameClientException(
                    $"Send timeout for request '{method}' after {effectiveSendTimeout.TotalMilliseconds}ms", ex);
            }

            // Read response with configurable timeout
            var effectiveReadTimeout = readTimeout ?? TimeSpan.FromMilliseconds(_options.ReadTimeoutMs);
            using var readTimeoutCts = new CancellationTokenSource(effectiveReadTimeout);
            using var readLinkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct, readTimeoutCts.Token);

            string? responseLine;
            try
            {
                if (_options.UseMessageFraming)
                {
                    responseLine = await ReadFramedMessageAsync(readLinkedCts.Token).ConfigureAwait(false);
                }
                else
                {
                    responseLine = await ReadLineAsync(_reader!, readLinkedCts.Token).ConfigureAwait(false);
                }
            }
            catch (OperationCanceledException ex) when (readTimeoutCts.Token.IsCancellationRequested)
            {
                _logger.Error(ex, "Read timeout for request '{Method}' after {TimeoutMs}ms",
                    method, effectiveReadTimeout.TotalMilliseconds);
                State = ConnectionState.Error;
                throw new GameClientException(
                    $"Read timeout for request '{method}' after {effectiveReadTimeout.TotalMilliseconds}ms", ex);
            }

            if (responseLine is null)
            {
                State = ConnectionState.Error;
                _logger.Error("Connection closed by game bridge server for request '{Method}'", method);
                throw new GameClientException("Connection closed by the game bridge server.");
            }

            JsonRpcResponse? response = JsonConvert.DeserializeObject<JsonRpcResponse>(responseLine);
            if (response is null)
            {
                _logger.Error("Received invalid JSON-RPC response for request '{Method}': {Response}", method, responseLine);
                throw new GameClientException("Received invalid JSON-RPC response.");
            }

            if (response.Error is not null)
            {
                _logger.Error("Server returned error for request '{Method}': [{ErrorCode}] {ErrorMessage}",
                    method, response.Error.Code, response.Error.Message);
                throw new GameClientException($"Server error [{response.Error.Code}]: {response.Error.Message}");
            }

            if (response.Result is null)
            {
                _logger.Warning("Server returned null result for request '{Method}'", method);
                throw new GameClientException($"Server returned null result for '{method}'.");
            }

            T? result = JsonConvert.DeserializeObject<T>(response.Result.ToString()!);
            if (result is null)
            {
                _logger.Error("Failed to deserialize result for request '{Method}'", method);
                throw new GameClientException($"Failed to deserialize result for '{method}'.");
            }

            sw.Stop();
            _logger.Information("Request '{Method}' completed successfully in {ElapsedMs}ms", method, sw.ElapsedMilliseconds);
            return result;
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

    /// <summary>
    /// Writes a framed message to the pipe with a 4-byte big-endian length prefix.
    /// </summary>
    /// <param name="message">The message content (JSON string).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <exception cref="IOException">Thrown on write failure.</exception>
    /// <exception cref="ProtocolException">Thrown on protocol violations.</exception>
    private async Task WriteFramedMessageAsync(string message, CancellationToken ct)
    {
        if (_pipe is null || !_pipe.IsConnected)
            throw new IOException("Pipe not connected");

        var messageBytes = Encoding.UTF8.GetBytes(message);

        if (messageBytes.Length > _options.MaxMessageSizeBytes)
            throw new ProtocolException(
                $"Message size {messageBytes.Length} bytes exceeds maximum {_options.MaxMessageSizeBytes}");

        // Write 4-byte length prefix (big-endian)
        var lengthBytes = BitConverter.GetBytes((uint)messageBytes.Length);
        if (BitConverter.IsLittleEndian)
            Array.Reverse(lengthBytes);

        await _pipe.WriteAsync(lengthBytes, 0, 4, ct).ConfigureAwait(false);
        await _pipe.WriteAsync(messageBytes, 0, messageBytes.Length, ct).ConfigureAwait(false);

        _logger.Debug("Wrote framed message: {LengthBytes} byte header + {MessageLengthBytes} byte payload",
            4, messageBytes.Length);
    }

    /// <summary>
    /// Reads a framed message from the pipe with a 4-byte big-endian length prefix.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The decoded message content.</returns>
    /// <exception cref="IOException">Thrown on read failure or connection closed.</exception>
    /// <exception cref="ProtocolException">Thrown on protocol violations (bad frame size, incomplete data).</exception>
    private async Task<string> ReadFramedMessageAsync(CancellationToken ct)
    {
        if (_pipe is null || !_pipe.IsConnected)
            throw new IOException("Pipe not connected");

        // Read 4-byte frame length header
        var lengthBuffer = new byte[4];
        int headerRead = await _pipe.ReadAsync(lengthBuffer, 0, 4, ct).ConfigureAwait(false);

        if (headerRead == 0)
            throw new IOException("Connection closed by peer while reading frame header");

        if (headerRead != 4)
            throw new ProtocolException(
                $"Expected 4-byte frame header, received {headerRead} bytes");

        // Decode length (big-endian)
        uint frameLength = BitConverter.ToUInt32(lengthBuffer, 0);
        if (BitConverter.IsLittleEndian)
            frameLength = (uint)IPAddress.NetworkToHostOrder((int)frameLength);

        if (frameLength == 0)
            throw new ProtocolException("Frame length cannot be zero");

        if (frameLength > _options.MaxMessageSizeBytes)
            throw new ProtocolException(
                $"Frame size {frameLength} bytes exceeds maximum {_options.MaxMessageSizeBytes}");

        // Read message payload
        var messageBuffer = new byte[frameLength];
        int messageRead = 0;
        int offset = 0;

        while (offset < frameLength)
        {
            int bytesRead = await _pipe.ReadAsync(messageBuffer, offset, (int)(frameLength - offset), ct)
                .ConfigureAwait(false);

            if (bytesRead == 0)
                throw new ProtocolException(
                    $"Incomplete frame: expected {frameLength} bytes, got {offset} bytes before EOF");

            messageRead += bytesRead;
            offset += bytesRead;
        }

        string message = Encoding.UTF8.GetString(messageBuffer);
        _logger.Debug("Read framed message: {LengthBytes} byte header + {MessageLengthBytes} byte payload",
            4, messageRead);

        return message;
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
