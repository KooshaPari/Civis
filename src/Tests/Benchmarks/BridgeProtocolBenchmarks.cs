using System;
using System.Security.Cryptography;
using System.Text;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Order;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json.Linq;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for Bridge/GameClient throughput and latency-sensitive paths.
/// Measures serialization roundtrip, receipt HMAC computation, and canonical JSON ordering.
/// Context: Bridge protocol carries traffic between GameClient (CLI/MCP) and GameBridgeServer (game plugin).
/// These operations run on the hot path during game state queries and override application.
/// </summary>
[MemoryDiagnoser]
[Orderer(SummaryOrderPolicy.FastestToSlowest)]
[RankColumn]
[SimpleJob(warmupCount: 3, iterationCount: 5)]
public class BridgeProtocolBenchmarks
{
    private JsonRpcRequest _sampleRequest = null!;
    private JsonRpcResponse _sampleResponse = null!;
    private BridgeReceipt _sampleReceipt = null!;
    private byte[] _sessionKey = null!;
    private JObject _samplePayload = null!;

    /// <summary>
    /// Initializes benchmark fixtures.
    /// </summary>
    [GlobalSetup]
    public void Setup()
    {
        // Sample JSON-RPC request (e.g., GameClient.QueryEntitiesByComponent)
        _sampleRequest = new JsonRpcRequest
        {
            Id = "req-001",
            Method = "query_entities",
            Params = JObject.FromObject(new { component = "Health", limit = 100 })
        };

        // Sample session key (32 bytes for HMAC-SHA256)
        _sessionKey = new byte[32];
        using (var rng = RandomNumberGenerator.Create())
        {
            rng.GetBytes(_sessionKey);
        }

        // Sample canonical JSON payload (~300 chars, typical response body)
        _samplePayload = JObject.FromObject(new
        {
            entities = new[]
            {
                new { id = 1, health = 100, armor = 5 },
                new { id = 2, health = 80, armor = 8 },
                new { id = 3, health = 150, armor = 10 }
            },
            metadata = new { query_time_ms = 12, frame = 4521 }
        });

        // Sample bridge receipt (Phase 4a+ tamper detection)
        var stateSha256 = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(_samplePayload)));
        _sampleReceipt = new BridgeReceipt
        {
            SessionId = "sess-test-001",
            StateSha256Hex = stateSha256,
            TimestampUtc = "2026-05-18T10:30:45.123456Z",
            WorldFrame = 4521,
            HmacHex = ComputeReceiptHmac(_sessionKey, "2026-05-18T10:30:45.123456Z", 4521, stateSha256)
        };

        // Sample response with receipt
        _sampleResponse = new JsonRpcResponse
        {
            Id = "req-001",
            Jsonrpc = "2.0",
            Result = _samplePayload,
            BridgeReceipt = _sampleReceipt
        };
    }

    /// <summary>
    /// Baseline: Serialize a JSON-RPC request to JSON and deserialize it back.
    /// Simulates a round-trip through the named-pipe bridge (as seen in GameClient usage).
    /// Target: sub-microsecond per op on modern .NET 8+ (under 1,000 ns/op).
    /// </summary>
    [Benchmark]
    public JsonRpcRequest JsonRpcRequest_Serialize_Roundtrip()
    {
        // Serialize
        string json = Newtonsoft.Json.JsonConvert.SerializeObject(_sampleRequest);

        // Deserialize
        var deserialized = Newtonsoft.Json.JsonConvert.DeserializeObject<JsonRpcRequest>(json);

        return deserialized!;
    }

    /// <summary>
    /// Hot path: Compute HMAC-SHA256 over a canonical receipt (state_sha256, timestamp, world_frame).
    /// Called by BridgeReceiptVerifier.Verify() for every response in Strict mode.
    /// Target: sub-10 microsecond per op (HMAC-SHA256 is crypto-bound, ~5-8 μs on modern CPUs).
    /// </summary>
    [Benchmark]
    public string BridgeReceipt_HmacCompute()
    {
        return BridgeReceiptVerifier.ComputeReceiptHmac(
            _sessionKey,
            _sampleReceipt.TimestampUtc,
            _sampleReceipt.WorldFrame,
            _sampleReceipt.StateSha256Hex);
    }

    /// <summary>
    /// Hot path: Canonicalize a JSON response payload for state hash verification.
    /// Called by BridgeReceiptVerifier before HMAC to ensure tamper-proof delivery.
    /// Payload varies in size (100-1000+ chars); this test uses ~300 char typical response.
    /// Target: sub-5 microsecond per op (key sorting + no-whitespace formatting).
    /// </summary>
    [Benchmark]
    public string CanonicalJson_Sort()
    {
        return CanonicalJson.Canonicalize(_samplePayload);
    }

    /// <summary>
    /// Helper: SHA256 hash to hex. Used to compute state_sha256 in BridgeReceipt.
    /// Not a direct benchmark (internal utility), but included for context.
    /// </summary>
    private static string Sha256Hex(byte[] input)
    {
        using var sha = System.Security.Cryptography.SHA256.Create();
        return ToHexLower(sha.ComputeHash(input));
    }

    /// <summary>
    /// Helper: Convert byte array to lowercase hex. Used by receipt HMAC computation.
    /// </summary>
    private static string ComputeReceiptHmac(byte[] sessionKey, string timestampUtc, long worldFrame, string stateSha256Hex)
    {
        // Mirrors BridgeReceiptVerifier.ComputeReceiptHmac
        string canonical = "{\"state_sha256\":\"" + stateSha256Hex
            + "\",\"timestamp\":\"" + timestampUtc
            + "\",\"world_frame\":" + worldFrame.ToString(System.Globalization.CultureInfo.InvariantCulture)
            + "}";

        byte[] bytes = Encoding.UTF8.GetBytes(canonical);
        using var hmac = new HMACSHA256(sessionKey);
        return ToHexLower(hmac.ComputeHash(bytes));
    }

    private static string ToHexLower(byte[] bytes)
    {
        const string hex = "0123456789abcdef";
        var sb = new StringBuilder(bytes.Length * 2);
        for (int i = 0; i < bytes.Length; i++)
        {
            byte b = bytes[i];
            sb.Append(hex[b >> 4]);
            sb.Append(hex[b & 0xF]);
        }
        return sb.ToString();
    }
}
