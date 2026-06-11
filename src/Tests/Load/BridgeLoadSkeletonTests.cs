#nullable enable
using System;
using System.IO;
using System.IO.Pipes;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Load;

/// <summary>
/// Minimal load-test skeleton for Bridge client concurrency.
/// </summary>
public sealed class BridgeLoadSkeletonTests
{
    [Fact]
    public async Task PingAsync_ParallelBurst_Completes()
    {
        string pipeName = $"dinoforge-load-{Guid.NewGuid():N}";

        LineBridgeServer server = new(pipeName);
        await using (server.ConfigureAwait(true))
        {
            await server.StartAsync().ConfigureAwait(true);

            using GameClient client = new(new GameClientOptions
            {
                PipeName = pipeName,
                UseMessageFraming = false,
                PerformConnectHandshake = false,
                RetryCount = 0,
                // Generous timeouts: this is a load/burst-completion test, not a latency test.
                // Under full-suite CPU contention a normally-fast response can exceed a tight
                // timeout, causing flaky failures. Assert the burst COMPLETES, not that it's fast.
                ConnectTimeoutMs = 30_000,
                ReadTimeoutMs = 30_000
            });

            client.HmacVerificationMode = VerificationMode.Off;

            await client.ConnectAsync().ConfigureAwait(true);

            const int rounds = 3;
            const int parallelCalls = 8;

            for (int round = 0; round < rounds; round++)
            {
                Task<PingResult>[] calls = Enumerable.Range(0, parallelCalls)
                    .Select(_ => client.PingAsync())
                    .ToArray();

                PingResult[] results = await Task.WhenAll(calls).ConfigureAwait(true);

                results.Should().HaveCount(parallelCalls);
                results.Should().OnlyContain(result => result.Pong);
            }

            server.RequestCount.Should().Be(rounds * parallelCalls);
        }
    }

    private sealed class LineBridgeServer : IAsyncDisposable
    {
        private readonly string _pipeName;
        private readonly CancellationTokenSource _cancellation = new();
        private Task? _loopTask;
        private NamedPipeServerStream? _pipeServer;
        private int _requestCount;

        public LineBridgeServer(string pipeName)
        {
            _pipeName = pipeName;
        }

        public int RequestCount => Volatile.Read(ref _requestCount);

        public async Task StartAsync()
        {
            if (_loopTask is not null)
            {
                return;
            }

            _loopTask = Task.Run(() => ServeAsync(_cancellation.Token));
            await Task.Delay(50, _cancellation.Token).ConfigureAwait(false);
        }

        public async ValueTask DisposeAsync()
        {
            _cancellation.Cancel();

            try
            {
                _pipeServer?.Dispose();
            }
            catch
            {
            }

            if (_loopTask is not null)
            {
                try
                {
                    await _loopTask.ConfigureAwait(false);
                }
                catch (OperationCanceledException)
                {
                }
                catch (ObjectDisposedException)
                {
                }
            }

            _cancellation.Dispose();
        }

        private async Task ServeAsync(CancellationToken ct)
        {
            _pipeServer = new NamedPipeServerStream(
                _pipeName,
                PipeDirection.InOut,
                1,
                PipeTransmissionMode.Byte,
                PipeOptions.Asynchronous);

            await _pipeServer.WaitForConnectionAsync(ct).ConfigureAwait(false);

            using StreamReader reader = new(_pipeServer);
            using StreamWriter writer = new(_pipeServer) { AutoFlush = true };

            while (!ct.IsCancellationRequested)
            {
                string? requestLine = await reader.ReadLineAsync().ConfigureAwait(false);
                if (requestLine is null)
                {
                    break;
                }

                JObject request = JObject.Parse(requestLine);
                string? id = request.Value<string>("id");
                string? method = request.Value<string>("method");

                if (!string.Equals(method, "ping", StringComparison.OrdinalIgnoreCase))
                {
                    break;
                }

                Interlocked.Increment(ref _requestCount);

                string response = JsonConvert.SerializeObject(new
                {
                    jsonrpc = "2.0",
                    id,
                    result = new PingResult
                    {
                        Pong = true,
                        Version = "load-test",
                        UptimeSeconds = 0d
                    }
                }, Formatting.None);

                await writer.WriteLineAsync(response).ConfigureAwait(false);
            }
        }
    }
}
