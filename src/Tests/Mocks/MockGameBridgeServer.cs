#nullable enable
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipes;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.Bridge;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// Mock JSON-RPC 2.0 game bridge server using named pipes.
/// Wraps an IGameBridge implementation to enable offline game automation testing.
/// </summary>
/// <remarks>
/// Supports concurrent client connections, protocol error handling, and message tracking.
/// All methods are async and cancellation-aware. Can be started and stopped multiple times.
/// </remarks>
public sealed class MockGameBridgeServer : IAsyncDisposable
{
    private readonly IGameBridge _bridge;
    private readonly string _pipeName;
    private readonly ConcurrentBag<(string method, object? request)> _receivedMessages;
    private readonly ConcurrentBag<Task> _clientHandlers;
    private CancellationTokenSource? _serverCts;
    private NamedPipeServerStream? _pipeServer;
    private Task? _listenerTask;
    private bool _disposed;

    /// <summary>
    /// Creates a new mock bridge server with optional pipe name and bridge implementation.
    /// </summary>
    /// <param name="pipeName">Named pipe name (defaults to "dinoforge-game-bridge"). If null, a UUID-based name is generated.</param>
    /// <param name="bridge">IGameBridge implementation (defaults to FakeGameBridge if null).</param>
    public MockGameBridgeServer(string? pipeName = null, IGameBridge? bridge = null)
    {
        _pipeName = !string.IsNullOrWhiteSpace(pipeName)
            ? pipeName
            : $"dinoforge-mock-{Guid.NewGuid():N}";

        _bridge = bridge ?? new FakeGameBridge();
        _receivedMessages = new ConcurrentBag<(string, object?)>();
        _clientHandlers = new ConcurrentBag<Task>();
    }

    /// <summary>
    /// Gets the actual pipe name being used by this server.
    /// </summary>
    public string PipeName => _pipeName;

    /// <summary>
    /// Gets a read-only list of all messages received by the server.
    /// Each entry contains the method name and the deserialized request parameters (or null).
    /// </summary>
    public IReadOnlyList<(string method, object? request)> ReceivedMessages
        => _receivedMessages.ToList().AsReadOnly();

    /// <summary>
    /// Starts the mock bridge server listening on the named pipe.
    /// If already running, this is a no-op.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    public async Task StartAsync(CancellationToken ct = default)
    {
        ThrowIfDisposed();

        if (_listenerTask != null && !_listenerTask.IsCompleted)
            return; // Already running

        _serverCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        _listenerTask = ListenerLoopAsync(_serverCts.Token);

        // Give the listener a moment to start accepting connections
        await Task.Delay(100, ct).ConfigureAwait(false);
    }

    /// <summary>
    /// Stops the mock bridge server and waits for all clients to disconnect.
    /// </summary>
    public async Task StopAsync()
    {
        if (_disposed)
            return;

        if (_serverCts == null)
            return;

        _serverCts.Cancel();

        try { _pipeServer?.Dispose(); }
        catch { }
        _pipeServer = null;

        if (_listenerTask != null)
        {
            try { await _listenerTask.ConfigureAwait(false); }
            catch (OperationCanceledException) { }
        }

        // Wait for all client handlers to complete (with timeout)
        using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        try
        {
            var incompleteTasks = _clientHandlers.Where(t => !t.IsCompleted).ToList();
            if (incompleteTasks.Count > 0)
                await Task.WhenAll(incompleteTasks).ConfigureAwait(false);
        }
        catch (OperationCanceledException) { }
    }

    /// <summary>
    /// Disposes the server and releases all resources.
    /// </summary>
    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;

        await StopAsync().ConfigureAwait(false);

        _serverCts?.Dispose();
        _pipeServer?.Dispose();
    }

    private async Task ListenerLoopAsync(CancellationToken ct)
    {
        try
        {
            while (!ct.IsCancellationRequested)
            {
                try
                {
                    _pipeServer = new NamedPipeServerStream(
                        _pipeName,
                        PipeDirection.InOut,
                        NamedPipeServerStream.MaxAllowedServerInstances,
                        PipeTransmissionMode.Byte,
                        PipeOptions.Asynchronous);

                    await _pipeServer.WaitForConnectionAsync(ct).ConfigureAwait(false);

                    // Spawn a handler for this client without blocking the listener
                    var connectedPipe = _pipeServer;
                    Task handlerTask = HandleClientAsync(connectedPipe);
                    _clientHandlers.Add(handlerTask);

                    _pipeServer = null; // Will be recreated for next connection
                }
                catch (OperationCanceledException) when (ct.IsCancellationRequested)
                {
                    break;
                }
                catch (IOException)
                {
                    // Pipe closed or error, try to accept a new connection
                    continue;
                }
            }
        }
        finally
        {
            try { _pipeServer?.Dispose(); }
            catch { }
        }
    }

    private async Task HandleClientAsync(PipeStream pipe)
    {
        // Phase 4c sub-task A + sub-task C (#249/#279):
        // Each connection owns its own SessionHmac instance so the connect
        // handshake can mint a fresh session_id + 32-byte ephemeral key, and
        // subsequent responses can be signed with a monotonic world_frame.
        // The session is null until the first `connect` verb arrives — which
        // models real-server behaviour where receipts only appear post-handshake.
        SessionHmac? session = null;
        long worldFrame = 0;

        try
        {
            using (pipe)
            using (var reader = new StreamReader(pipe))
            using (var writer = new StreamWriter(pipe) { AutoFlush = true })
            {
                string? line;
                while ((line = await reader.ReadLineAsync().ConfigureAwait(false)) != null)
                {
                    try
                    {
                        // Deserialize request
                        var request = JsonConvert.DeserializeObject<JsonRpcRequest>(line);
                        if (request == null)
                        {
                            await SendErrorAsync(writer, null, -32700, "Parse error").ConfigureAwait(false);
                            continue;
                        }

                        // Track the received message
                        _receivedMessages.Add((request.Method, request.Params));

                        JsonRpcResponse response;
                        bool isHandshake = string.Equals(request.Method, "connect", StringComparison.OrdinalIgnoreCase);

                        if (isHandshake)
                        {
                            // Phase 4c sub-task A: mint a fresh session and reply
                            // with the (session_id, session_key_b64) envelope that
                            // GameClient.PerformHandshakeAsync expects. Replace any
                            // prior session on this connection (reconnect semantics).
                            session?.Dispose();
                            session = new SessionHmac();
                            worldFrame = 0;

                            var envelope = new JObject
                            {
                                ["session_id"] = session.SessionId,
                                ["session_key_b64"] = session.KeyMaterialB64(),
                            };

                            response = new JsonRpcResponse
                            {
                                Id = request.Id,
                                Result = envelope,
                            };
                        }
                        else
                        {
                            // Dispatch to bridge for all other verbs
                            var dispatcher = new BridgeProtocolDispatcher(_bridge);
                            response = await dispatcher.DispatchAsync(request).ConfigureAwait(false);
                        }

                        // Phase 4c sub-task C (#279): if a session was minted,
                        // attach a signed BridgeReceipt to every response. The
                        // handshake response itself uses world_frame=0 sentinel;
                        // subsequent responses advance the frame strictly to
                        // satisfy BridgeReceiptVerifier monotonicity.
                        if (session != null && response.Error == null)
                        {
                            long frameForReceipt;
                            if (isHandshake)
                            {
                                frameForReceipt = 0;
                            }
                            else
                            {
                                worldFrame++;
                                frameForReceipt = worldFrame;
                            }

                            response.BridgeReceipt = BridgeReceiptBuilder.BuildReceipt(
                                session,
                                response.Result,
                                frameForReceipt);
                        }

                        // Send response
                        string responseJson = JsonConvert.SerializeObject(response, Formatting.None,
                            new JsonSerializerSettings { NullValueHandling = NullValueHandling.Ignore });
                        await writer.WriteLineAsync(responseJson).ConfigureAwait(false);
                    }
                    catch (JsonReaderException)
                    {
                        await SendErrorAsync(writer, null, -32700, "Parse error").ConfigureAwait(false);
                    }
                    catch (Exception ex)
                    {
                        await SendErrorAsync(writer, null, -32603, "Internal server error", ex.Message).ConfigureAwait(false);
                    }
                }
            }
        }
        catch
        {
            // Connection closed, handler completes
        }
        finally
        {
            session?.Dispose();
        }
    }

    private static async Task SendErrorAsync(StreamWriter writer, string? id, int code, string message, string? data = null)
    {
        var response = new JsonRpcResponse
        {
            Id = id,
            Error = new JsonRpcError
            {
                Code = code,
                Message = message,
                Data = data != null ? JToken.FromObject(new { detail = data }) : null
            }
        };

        string json = JsonConvert.SerializeObject(response, Formatting.None,
            new JsonSerializerSettings { NullValueHandling = NullValueHandling.Ignore });
        await writer.WriteLineAsync(json).ConfigureAwait(false);
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
            throw new ObjectDisposedException(GetType().Name);
    }
}
