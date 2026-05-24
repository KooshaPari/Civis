#nullable enable
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipes;
using System.Linq;
using System.Net;
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
    private readonly ConcurrentDictionary<int, PipeStream> _activeClientPipes;
    private CancellationTokenSource? _serverCts;
    private NamedPipeServerStream? _pipeServer;
    private Task? _listenerTask;
    private TaskCompletionSource _stoppedTcs = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private bool _disposed;
    private int _nextConnectionId;

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
        _activeClientPipes = new ConcurrentDictionary<int, PipeStream>();
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
    /// Gets a task that completes when the server has fully stopped and all
    /// tracked client handlers have completed.
    /// </summary>
    public Task Stopped => _stoppedTcs.Task;

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

        _stoppedTcs = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);

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

        foreach (PipeStream activePipe in _activeClientPipes.Values)
        {
            try { activePipe.Dispose(); }
            catch { }
        }

        if (_listenerTask != null)
        {
            try { await _listenerTask.ConfigureAwait(false); }
            catch (OperationCanceledException) { }
        }

        // Wait briefly for client handlers to observe the shutdown, but do not
        // block indefinitely on a connected client that is still blocked in a read.
        var incompleteTasks = _clientHandlers.Where(t => !t.IsCompleted).ToList();
        if (incompleteTasks.Count > 0)
        {
            Task allHandlersTask = Task.WhenAll(incompleteTasks);
            Task timeoutTask = Task.Delay(TimeSpan.FromSeconds(5));

            try
            {
                await Task.WhenAny(allHandlersTask, timeoutTask).ConfigureAwait(false);
                if (allHandlersTask.IsCompleted)
                {
                    await allHandlersTask.ConfigureAwait(false);
                }
            }
            catch (OperationCanceledException)
            {
                // StopAsync is a best-effort shutdown path; return once the timeout elapses.
            }
        }

        _stoppedTcs.TrySetResult();
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
                    int connectionId = Interlocked.Increment(ref _nextConnectionId);
                    _activeClientPipes[connectionId] = connectedPipe;
                    Task handlerTask = HandleClientAsync(connectionId, connectedPipe);
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

    private async Task HandleClientAsync(int connectionId, PipeStream pipe)
    {
        // Phase 4c sub-task A + sub-task C (#249/#279):
        // Each connection owns its own SessionHmac instance so the connect
        // handshake can mint a fresh session_id + 32-byte ephemeral key, and
        // subsequent responses can be signed with a monotonic world_frame.
        // The session is null until the first `connect` verb arrives — which
        // models real-server behaviour where receipts only appear post-handshake.
        SessionHmac? session = null;
        long worldFrame = 0;
        MessageProtocol? protocol = null;

        try
        {
            using (pipe)
            {
                while (true)
                {
                    try
                    {
                        (string? line, MessageProtocol? detectedProtocol) = await ReadMessageAsync(pipe, protocol).ConfigureAwait(false);
                        if (detectedProtocol != null)
                        {
                            protocol = detectedProtocol;
                        }
                        if (line == null)
                        {
                            break;
                        }

                        // Deserialize request
                        var request = JsonConvert.DeserializeObject<JsonRpcRequest>(line);
                        if (request == null)
                        {
                            await SendErrorAsync(pipe, null, -32700, "Parse error", protocol).ConfigureAwait(false);
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
                        await WriteMessageAsync(pipe, responseJson, protocol ?? MessageProtocol.Framed).ConfigureAwait(false);
                    }
                    catch (JsonReaderException)
                    {
                        await SendErrorAsync(pipe, null, -32700, "Parse error", protocol).ConfigureAwait(false);
                    }
                    catch (Exception ex)
                    {
                        await SendErrorAsync(pipe, null, -32603, "Internal server error", protocol, ex.Message).ConfigureAwait(false);
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
            _activeClientPipes.TryRemove(connectionId, out _);
        }
    }

    private static async Task SendErrorAsync(PipeStream pipe, string? id, int code, string message, MessageProtocol? protocol, string? data = null)
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
        await WriteMessageAsync(pipe, json, protocol ?? MessageProtocol.Framed).ConfigureAwait(false);
    }

    private static async Task<(string? message, MessageProtocol? protocol)> ReadMessageAsync(PipeStream pipe, MessageProtocol? protocol)
    {
        if (protocol == MessageProtocol.Line)
        {
            return (await ReadLineMessageAsync(pipe).ConfigureAwait(false), protocol);
        }

        if (protocol == MessageProtocol.Framed)
        {
            return (await ReadFramedMessageAsync(pipe).ConfigureAwait(false), protocol);
        }

        byte[] prefix = new byte[4];
        int totalRead = 0;
        while (totalRead < 4)
        {
            int bytesRead = await pipe.ReadAsync(prefix, totalRead, 4 - totalRead).ConfigureAwait(false);
            if (bytesRead == 0)
            {
                return (null, null);
            }
            totalRead += bytesRead;
        }

        if (LooksLikeFramedPrefix(prefix))
        {
            return (await ReadFramedMessageAsync(pipe, prefix).ConfigureAwait(false), MessageProtocol.Framed);
        }

        return (await ReadLineMessageAsync(pipe, prefix).ConfigureAwait(false), MessageProtocol.Line);
    }

    private static bool LooksLikeFramedPrefix(IReadOnlyList<byte> prefix)
    {
        // Valid framed messages use a 4-byte big-endian length prefix. With the
        // current max-frame cap, the high byte must be zero; this keeps normal
        // JSON-RPC lines from being misclassified when their first 4 bytes happen
        // to form a plausible small integer after byte swapping.
        if (prefix[0] != 0)
        {
            return false;
        }

        uint frameLength = BitConverter.ToUInt32(prefix.ToArray(), 0);
        if (BitConverter.IsLittleEndian)
        {
            frameLength = (uint)IPAddress.NetworkToHostOrder((int)frameLength);
        }

        return frameLength > 0 && frameLength <= 1_000_000;
    }

    private static async Task<string?> ReadFramedMessageAsync(PipeStream pipe, byte[]? initialPrefix = null)
    {
        byte[] lengthBuffer = initialPrefix ?? new byte[4];
        int totalRead = 0;
        if (initialPrefix != null)
        {
            totalRead = 4;
        }
        while (totalRead < 4)
        {
            int bytesRead = await pipe.ReadAsync(lengthBuffer, totalRead, 4 - totalRead).ConfigureAwait(false);
            if (bytesRead == 0)
            {
                return null;
            }
            totalRead += bytesRead;
        }

        uint frameLength = BitConverter.ToUInt32(lengthBuffer, 0);
        if (BitConverter.IsLittleEndian)
        {
            frameLength = (uint)IPAddress.NetworkToHostOrder((int)frameLength);
        }

        if (frameLength == 0)
        {
            return null;
        }

        var buffer = new byte[frameLength];
        int offset = 0;
        while (offset < frameLength)
        {
            int bytesRead = await pipe.ReadAsync(buffer, offset, (int)frameLength - offset).ConfigureAwait(false);
            if (bytesRead == 0)
            {
                return null;
            }
            offset += bytesRead;
        }

        return System.Text.Encoding.UTF8.GetString(buffer);
    }

    private static async Task<string?> ReadLineMessageAsync(PipeStream pipe, byte[]? initialPrefix = null)
    {
        using var buffer = new MemoryStream();
        if (initialPrefix != null)
        {
            buffer.Write(initialPrefix, 0, initialPrefix.Length);
        }

        var byteBuffer = new byte[1];
        while (true)
        {
            int bytesRead = await pipe.ReadAsync(byteBuffer, 0, 1).ConfigureAwait(false);
            if (bytesRead == 0)
            {
                if (buffer.Length == 0)
                {
                    return null;
                }

                break;
            }

            if (byteBuffer[0] == (byte)'\n')
            {
                break;
            }

            buffer.WriteByte(byteBuffer[0]);
        }

        return System.Text.Encoding.UTF8.GetString(buffer.ToArray());
    }

    private static async Task WriteFramedMessageAsync(PipeStream pipe, string message)
    {
        byte[] payload = System.Text.Encoding.UTF8.GetBytes(message);
        byte[] lengthBytes = BitConverter.GetBytes((uint)payload.Length);
        if (BitConverter.IsLittleEndian)
        {
            Array.Reverse(lengthBytes);
        }

        await pipe.WriteAsync(lengthBytes, 0, lengthBytes.Length).ConfigureAwait(false);
        await pipe.WriteAsync(payload, 0, payload.Length).ConfigureAwait(false);
        await pipe.FlushAsync().ConfigureAwait(false);
    }

    private static async Task WriteLineMessageAsync(PipeStream pipe, string message)
    {
        byte[] payload = System.Text.Encoding.UTF8.GetBytes(message + Environment.NewLine);
        await pipe.WriteAsync(payload, 0, payload.Length).ConfigureAwait(false);
        await pipe.FlushAsync().ConfigureAwait(false);
    }

    private static Task WriteMessageAsync(PipeStream pipe, string message, MessageProtocol protocol)
        => protocol == MessageProtocol.Line
            ? WriteLineMessageAsync(pipe, message)
            : WriteFramedMessageAsync(pipe, message);

    private enum MessageProtocol
    {
        Framed,
        Line
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
            throw new ObjectDisposedException(GetType().Name);
    }
}
