#nullable enable
using System;
using System.Collections.Generic;
using System.Collections.Concurrent;
using System.IO;
using System.IO.Pipes;
using System.Net;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Tests.BDD.Support;

internal sealed class BridgeRegressionServer : IAsyncDisposable
{
    private readonly bool _replaySamePingFrame;
    private readonly string _pipeName;
    private readonly CancellationTokenSource _cts = new();
    private readonly ConcurrentQueue<string> _receivedMethods = new();
    private readonly TaskCompletionSource _ready = new(TaskCreationOptions.RunContinuationsAsynchronously);

    private Task? _serverTask;
    private byte[]? _sessionKey;
    private string? _sessionId;
    private int _pingCount;

    public BridgeRegressionServer(bool replaySamePingFrame)
    {
        _replaySamePingFrame = replaySamePingFrame;
        _pipeName = $"dinoforge-bdd-{Guid.NewGuid():N}";
    }

    public string PipeName => _pipeName;

    public IReadOnlyCollection<string> ReceivedMethods => _receivedMethods.ToArray();

    public async Task StartAsync()
    {
        if (_serverTask != null)
        {
            return;
        }

        _serverTask = RunAsync(_cts.Token);
        await _ready.Task.ConfigureAwait(false);
    }

    public async ValueTask DisposeAsync()
    {
        _cts.Cancel();

        try
        {
            await (_serverTask ?? Task.CompletedTask).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
        }
        catch (IOException)
        {
        }
        catch (ObjectDisposedException)
        {
        }
        finally
        {
            _cts.Dispose();
        }
    }

    private async Task RunAsync(CancellationToken ct)
    {
        using var server = new NamedPipeServerStream(
            _pipeName,
            PipeDirection.InOut,
            1,
            PipeTransmissionMode.Byte,
            PipeOptions.Asynchronous);

        _ready.TrySetResult();
        await server.WaitForConnectionAsync(ct).ConfigureAwait(false);

        while (!ct.IsCancellationRequested)
        {
            string? requestJson = await ReadFramedMessageAsync(server, ct).ConfigureAwait(false);
            if (requestJson == null)
            {
                break;
            }

            JsonRpcRequest? request = JsonConvert.DeserializeObject<JsonRpcRequest>(requestJson);
            if (request == null)
            {
                await WriteFramedMessageAsync(server, Serialize(new JsonRpcResponse
                {
                    Error = new JsonRpcError { Code = -32700, Message = "Parse error" }
                }), ct).ConfigureAwait(false);
                continue;
            }

            _receivedMethods.Enqueue(request.Method);

            JsonRpcResponse response = request.Method.Equals("connect", StringComparison.OrdinalIgnoreCase)
                ? BuildHandshakeResponse(request)
                : request.Method.Equals("ping", StringComparison.OrdinalIgnoreCase)
                    ? BuildPingResponse(request)
                    : new JsonRpcResponse
                    {
                        Id = request.Id,
                        Error = new JsonRpcError { Code = -32601, Message = $"Method not found: {request.Method}" }
                    };

            await WriteFramedMessageAsync(server, Serialize(response), ct).ConfigureAwait(false);
        }
    }

    private JsonRpcResponse BuildHandshakeResponse(JsonRpcRequest request)
    {
        EnsureSessionMaterial();

        JObject result = new()
        {
            ["session_id"] = _sessionId,
            ["session_key_b64"] = Convert.ToBase64String(_sessionKey!)
        };

        return CreateSignedResponse(request.Id, result, worldFrame: 0);
    }

    private JsonRpcResponse BuildPingResponse(JsonRpcRequest request)
    {
        EnsureSessionMaterial();

        int pingOrdinal = Interlocked.Increment(ref _pingCount);
        long worldFrame = _replaySamePingFrame ? 1L : pingOrdinal;

        JObject result = JObject.FromObject(new
        {
            pong = true,
            version = "bdd-bridge",
            uptimeSeconds = 12.5d
        });

        return CreateSignedResponse(request.Id, result, worldFrame);
    }

    private JsonRpcResponse CreateSignedResponse(string? id, JToken result, long worldFrame)
    {
        string canonicalPayload = CanonicalJson.Canonicalize(result);
        string stateSha256Hex = Sha256Hex(Encoding.UTF8.GetBytes(canonicalPayload));
        string timestampUtc = "2026-06-08T12:00:00.000Z";
        string hmacHex = BridgeReceiptVerifier.ComputeReceiptHmac(_sessionKey!, timestampUtc, worldFrame, stateSha256Hex);

        return new JsonRpcResponse
        {
            Id = id,
            Result = result,
            BridgeReceipt = new BridgeReceipt
            {
                SessionId = _sessionId!,
                TimestampUtc = timestampUtc,
                WorldFrame = worldFrame,
                StateSha256Hex = stateSha256Hex,
                HmacHex = hmacHex
            }
        };
    }

    private void EnsureSessionMaterial()
    {
        if (_sessionId != null && _sessionKey != null)
        {
            return;
        }

        _sessionId = Guid.NewGuid().ToString("N");
        _sessionKey = new byte[32];
        RandomNumberGenerator.Fill(_sessionKey);
    }

    private static string Serialize(JsonRpcResponse response) =>
        JsonConvert.SerializeObject(response, Formatting.None,
            new JsonSerializerSettings { NullValueHandling = NullValueHandling.Ignore });

    private static async Task<string?> ReadFramedMessageAsync(PipeStream pipe, CancellationToken ct)
    {
        byte[] lengthBuffer = new byte[4];
        int totalRead = 0;
        while (totalRead < 4)
        {
            int bytesRead = await pipe.ReadAsync(lengthBuffer, totalRead, 4 - totalRead, ct).ConfigureAwait(false);
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

        byte[] messageBuffer = new byte[frameLength];
        int offset = 0;
        while (offset < frameLength)
        {
            int bytesRead = await pipe.ReadAsync(messageBuffer, offset, (int)frameLength - offset, ct).ConfigureAwait(false);
            if (bytesRead == 0)
            {
                return null;
            }

            offset += bytesRead;
        }

        return Encoding.UTF8.GetString(messageBuffer);
    }

    private static async Task WriteFramedMessageAsync(PipeStream pipe, string message, CancellationToken ct)
    {
        byte[] payload = Encoding.UTF8.GetBytes(message);
        byte[] lengthBytes = BitConverter.GetBytes((uint)payload.Length);
        if (BitConverter.IsLittleEndian)
        {
            Array.Reverse(lengthBytes);
        }

        await pipe.WriteAsync(lengthBytes, 0, lengthBytes.Length, ct).ConfigureAwait(false);
        await pipe.WriteAsync(payload, 0, payload.Length, ct).ConfigureAwait(false);
        await pipe.FlushAsync(ct).ConfigureAwait(false);
    }

    private static string Sha256Hex(byte[] input)
    {
        using SHA256 sha = SHA256.Create();
        byte[] hash = sha.ComputeHash(input);
        StringBuilder builder = new(hash.Length * 2);
        foreach (byte value in hash)
        {
            builder.Append("0123456789abcdef"[value >> 4]);
            builder.Append("0123456789abcdef"[value & 0x0F]);
        }

        return builder.ToString();
    }
}
