#nullable enable
using System;
using System.Globalization;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Phase 4a unit tests for the bridge HMAC scaffolding: per-session key
/// generation, deterministic receipt signing, and the wire-format DTO
/// round-trip. Verification (Phase 4b) is covered separately.
/// </summary>
public class BridgeHmacTests
{
    [Fact]
    public void SessionHmac_GeneratesUniqueKeyPerInstance()
    {
        using var a = new SessionHmac();
        using var b = new SessionHmac();

        a.SessionId.Should().NotBe(b.SessionId, "every session must have a fresh UUID");
        a.SessionId.Should().HaveLength(32, "Guid.ToString(\"N\") format is 32 hex chars");
        a.KeyMaterial.Should().HaveCount(32, "spec mandates a 256-bit ephemeral key");
        b.KeyMaterial.Should().HaveCount(32);
        a.KeyMaterial.Should().NotEqual(b.KeyMaterial, "RNG must produce distinct key bytes per session");
    }

    [Fact]
    public void SessionHmac_HmacIsDeterministicForSameInputs()
    {
        using var session = new SessionHmac();
        const string ts = "2026-04-25T12:34:56.789Z";
        const long frame = 12345L;
        const string stateHash = "9f2c8a1b4e6d5c0f0e2a3b4c5d6e7f8091a2b3c4d5e6f70819202122232425262";

        string h1 = session.ComputeHmac(ts, frame, stateHash);
        string h2 = session.ComputeHmac(ts, frame, stateHash);

        h1.Should().Be(h2, "HMAC over identical inputs with the same key must be byte-identical");
        h1.Should().HaveLength(64, "HMAC-SHA256 hex output is 64 chars");
        h1.Should().MatchRegex("^[0-9a-f]{64}$", "lowercase hex per spec section 6.7");
    }

    [Fact]
    public void SessionHmac_HmacIsStableAcrossParallelCalls()
    {
        using var session = new SessionHmac();
        const int caseCount = 50;

        string[] timestamps = new string[caseCount];
        long[] frames = new long[caseCount];
        string[] stateHashes = new string[caseCount];
        string[] expected = new string[caseCount];
        string[] actual = new string[caseCount];

        for (int index = 0; index < caseCount; index++)
        {
            timestamps[index] = "2026-04-25T12:34:56." + index.ToString("000", CultureInfo.InvariantCulture) + "Z";
            frames[index] = 10_000L + index;
            stateHashes[index] = index.ToString("x64", CultureInfo.InvariantCulture);
            expected[index] = session.ComputeHmac(timestamps[index], frames[index], stateHashes[index]);
        }

        Parallel.For(0, caseCount, index =>
        {
            actual[index] = session.ComputeHmac(timestamps[index], frames[index], stateHashes[index]);
        });

        actual.Should().Equal(expected, "parallel calls must match the sequential baseline for every input");
    }

    [Fact]
    public void SessionHmac_HmacChangesWhenInputChanges()
    {
        using var session = new SessionHmac();
        const string ts = "2026-04-25T12:34:56.789Z";
        const long frame = 12345L;
        const string stateHash = "9f2c8a1b4e6d5c0f0e2a3b4c5d6e7f8091a2b3c4d5e6f70819202122232425262";

        string baseline = session.ComputeHmac(ts, frame, stateHash);
        string tsChanged = session.ComputeHmac("2026-04-25T12:34:56.790Z", frame, stateHash);
        string frameChanged = session.ComputeHmac(ts, frame + 1, stateHash);
        string hashChanged = session.ComputeHmac(ts, frame, stateHash.Replace('9', 'a'));

        tsChanged.Should().NotBe(baseline, "tampering with the timestamp must change the HMAC");
        frameChanged.Should().NotBe(baseline, "tampering with world_frame must change the HMAC");
        hashChanged.Should().NotBe(baseline, "tampering with state_sha256 must change the HMAC");
    }

    [Fact]
    public void SessionHmac_DifferentKeysProduceDifferentHmacs()
    {
        // Same inputs, different sessions → different HMACs (proves the key is actually used).
        using var a = new SessionHmac();
        using var b = new SessionHmac();
        const string ts = "2026-04-25T12:34:56.789Z";
        const long frame = 1L;
        const string stateHash = "9f2c8a1b4e6d5c0f0e2a3b4c5d6e7f8091a2b3c4d5e6f70819202122232425262";

        string ha = a.ComputeHmac(ts, frame, stateHash);
        string hb = b.ComputeHmac(ts, frame, stateHash);

        ha.Should().NotBe(hb, "different keys over the same input must yield different HMACs");
    }

    [Fact]
    public void SessionHmac_KeyMaterialB64_RoundTrips()
    {
        using var session = new SessionHmac();
        string b64 = session.KeyMaterialB64();

        b64.Should().NotBeNullOrEmpty();
        byte[] decoded = Convert.FromBase64String(b64);
        decoded.Should().Equal(session.KeyMaterial, "base64 round-trip must reproduce the raw key bytes for the client cache");
    }

    [Fact]
    public void BridgeReceipt_RoundTripsThroughJsonSerialization()
    {
        var receipt = new BridgeReceipt
        {
            SessionId = "550e8400e29b41d4a716446655440000",
            TimestampUtc = "2026-04-25T12:34:56.789Z",
            WorldFrame = 42L,
            StateSha256Hex = "9f2c8a1b4e6d5c0f0e2a3b4c5d6e7f8091a2b3c4d5e6f70819202122232425262",
            HmacHex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        };

        string json = JsonConvert.SerializeObject(receipt);
        BridgeReceipt? round = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        round.Should().NotBeNull();
        round!.SessionId.Should().Be(receipt.SessionId);
        round.TimestampUtc.Should().Be(receipt.TimestampUtc);
        round.WorldFrame.Should().Be(receipt.WorldFrame);
        round.StateSha256Hex.Should().Be(receipt.StateSha256Hex);
        round.HmacHex.Should().Be(receipt.HmacHex);

        // Wire-format keys must use snake_case as per the spec.
        json.Should().Contain("\"session_id\"");
        json.Should().Contain("\"timestamp\"");
        json.Should().Contain("\"world_frame\"");
        json.Should().Contain("\"state_sha256\"");
        json.Should().Contain("\"hmac\"");
    }

    [Fact]
    public void JsonRpcResponse_BridgeReceiptSerializesUnderSnakeCaseKey()
    {
        var response = new JsonRpcResponse
        {
            Id = "42",
            Result = Newtonsoft.Json.Linq.JToken.FromObject(new { ok = true }),
            BridgeReceipt = new BridgeReceipt
            {
                SessionId = "abc",
                TimestampUtc = "2026-04-25T00:00:00.000Z",
                WorldFrame = 7,
                StateSha256Hex = "deadbeef",
                HmacHex = "cafebabe",
            },
        };

        string json = JsonConvert.SerializeObject(response);

        json.Should().Contain("\"bridge_receipt\"", "envelope must expose the receipt under snake_case");
        json.Should().Contain("\"hmac\":\"cafebabe\"");

        // Sanity round-trip
        var round = JsonConvert.DeserializeObject<JsonRpcResponse>(json);
        round.Should().NotBeNull();
        round!.BridgeReceipt.Should().NotBeNull();
        round.BridgeReceipt!.HmacHex.Should().Be("cafebabe");
    }

    [Fact]
    public void JsonRpcResponse_OmitsBridgeReceiptWhenNull()
    {
        var response = new JsonRpcResponse
        {
            Id = "1",
            Result = Newtonsoft.Json.Linq.JToken.FromObject(new { ok = true }),
            BridgeReceipt = null,
        };

        string json = JsonConvert.SerializeObject(response);

        json.Should().NotContain("bridge_receipt",
            "Connect handshake must not carry a receipt because the client doesn't yet have the session key");
    }

    // ─────────────────────────────────────────────────────────────────
    //  Phase 4a wiring tests (#223): BridgeReceiptBuilder produces
    //  receipts that round-trip through BridgeReceiptVerifier.
    // ─────────────────────────────────────────────────────────────────

    [Fact]
    [Trait("Category", "BridgeHmac")]
    public void BridgeReceiptBuilder_BuildsReceiptThatRoundTripsThroughVerifier()
    {
        // Server-side: holds the session, builds the receipt.
        using var session = new SessionHmac();
        var result = JToken.FromObject(new { entity_count = 45776, world_name = "Default World" });

        BridgeReceipt receipt = BridgeReceiptBuilder.BuildReceipt(session, result, worldFrame: 12345L);

        // Receipt fields populated correctly.
        receipt.Should().NotBeNull();
        receipt.SessionId.Should().Be(session.SessionId);
        receipt.WorldFrame.Should().Be(12345L);
        receipt.StateSha256Hex.Should().MatchRegex("^[0-9a-f]{64}$",
            "payload hash must be lowercase hex SHA-256");
        receipt.HmacHex.Should().MatchRegex("^[0-9a-f]{64}$",
            "HMAC must be lowercase hex SHA-256 per spec section 6.7");
        receipt.TimestampUtc.Should().MatchRegex(@"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$",
            "timestamp must be ISO 8601 UTC with millisecond precision and trailing Z");

        // Client-side: caches the key, verifies the receipt over the wire.
        var response = new JsonRpcResponse
        {
            Id = "17",
            Result = result,
            BridgeReceipt = receipt,
        };
        using var cache = new SessionKeyCache();
        cache.Set(session.SessionId, session.KeyMaterial);

        long lastFrame = 0L;
        VerificationResult verdict = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        verdict.Valid.Should().BeTrue(
            "the receipt was just signed under this session key — strict verification must accept it. "
            + "Reason returned: " + verdict.Reason);
        verdict.Reason.Should().BeEmpty();
        lastFrame.Should().Be(12345L, "verifier advances the frame on success");
    }

    [Fact]
    [Trait("Category", "BridgeHmac")]
    public void BridgeReceiptBuilder_TamperedPayloadIsRejectedByVerifier()
    {
        // Build a valid receipt for one payload, then mutate the payload before verifying.
        using var session = new SessionHmac();
        JToken originalPayload = JToken.FromObject(new { entity_count = 100 });
        BridgeReceipt receipt = BridgeReceiptBuilder.BuildReceipt(session, originalPayload, worldFrame: 1L);

        var tamperedResponse = new JsonRpcResponse
        {
            Id = "1",
            Result = JToken.FromObject(new { entity_count = 999 }), // attacker mutated the result
            BridgeReceipt = receipt,
        };
        using var cache = new SessionKeyCache();
        cache.Set(session.SessionId, session.KeyMaterial);

        long lastFrame = 0L;
        VerificationResult verdict = BridgeReceiptVerifier.Verify(
            tamperedResponse, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        verdict.Valid.Should().BeFalse(
            "tampered payload must produce a state_sha256 mismatch under Strict mode");
        verdict.Reason.Should().Contain("state_sha256",
            "the reason should call out which canonical-input field diverged");
    }
}
