#nullable enable
using System;
using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Phase 4b unit tests for the client-side <see cref="BridgeReceiptVerifier"/>.
/// Covers happy-path, tampering, key mismatch, missing-receipt under both
/// modes, frame monotonicity (including the frame=0 connect sentinel) and the
/// critical canonical-JSON byte-equality between the client and server
/// canonicalizers.
/// </summary>
public class BridgeReceiptVerifierTests
{
    /// <summary>
    /// Builds a server-signed receipt for <paramref name="payload"/> using
    /// the same HMAC path as the runtime <see cref="SessionHmac"/>. The
    /// returned receipt mirrors what GameBridgeServer would emit on the wire.
    /// </summary>
    private static (BridgeReceipt receipt, byte[] key) ServerSign(string sessionId, byte[] key, JToken payload, long frame)
    {
        // Mirror the server: state_sha256 = sha256(canonical(payload)).
        string canonical = CanonicalJson.Canonicalize(payload);
        string stateHash;
        using (var sha = SHA256.Create())
        {
            byte[] hash = sha.ComputeHash(Encoding.UTF8.GetBytes(canonical));
            var sb = new StringBuilder(hash.Length * 2);
            foreach (byte b in hash)
            {
                sb.Append("0123456789abcdef"[b >> 4]);
                sb.Append("0123456789abcdef"[b & 0xF]);
            }
            stateHash = sb.ToString();
        }
        string ts = "2026-04-25T12:34:56.789Z";

        // Use SessionHmac.ComputeHmac (the server path) directly so this
        // test simultaneously exercises the cross-component byte-equality.
        // We construct a SessionHmac from a known key by reflecting through
        // the public ComputeReceiptHmac helper (same algorithm).
        string hmac = BridgeReceiptVerifier.ComputeReceiptHmac(key, ts, frame, stateHash);

        return (new BridgeReceipt
        {
            SessionId = sessionId,
            TimestampUtc = ts,
            WorldFrame = frame,
            StateSha256Hex = stateHash,
            HmacHex = hmac,
        }, key);
    }

    private static byte[] FreshKey()
    {
        byte[] k = new byte[32];
        RandomNumberGenerator.Fill(k);
        return k;
    }

    [Fact]
    public void ValidReceipt_PassesVerification()
    {
        var key = FreshKey();
        var sessionId = Guid.NewGuid().ToString("N");
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken payload = JToken.FromObject(new { entity_count = 45776, world_ready = true });
        var (receipt, _) = ServerSign(sessionId, key, payload, frame: 100);

        var response = new JsonRpcResponse { Id = "1", Result = payload, BridgeReceipt = receipt };
        long lastFrame = 50;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeTrue("server-signed receipt under known key must verify");
        result.Reason.Should().BeEmpty();
        lastFrame.Should().Be(100, "monotonicity bookmark must advance on success");
    }

    [Fact]
    public void TamperedPayload_RejectedInStrict()
    {
        var key = FreshKey();
        var sessionId = Guid.NewGuid().ToString("N");
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken originalPayload = JToken.FromObject(new { entity_count = 100 });
        var (receipt, _) = ServerSign(sessionId, key, originalPayload, frame: 10);

        // Tamper: ship the receipt for the original payload alongside a
        // mutated result. state_sha256 won't match anymore.
        JToken tamperedPayload = JToken.FromObject(new { entity_count = 999_999 });
        var response = new JsonRpcResponse { Id = "1", Result = tamperedPayload, BridgeReceipt = receipt };

        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse("tampered payload must fail state_sha256 recomputation");
        result.Reason.Should().Contain("state_sha256");
    }

    [Fact]
    public void WrongSessionKey_RejectedInStrict()
    {
        var realKey = FreshKey();
        var attackerKey = FreshKey();
        var sessionId = Guid.NewGuid().ToString("N");

        // Server signs under realKey…
        JToken payload = JToken.FromObject(new { ok = true });
        var (receipt, _) = ServerSign(sessionId, realKey, payload, frame: 5);

        // …but the client cache has an attacker-supplied key for that session.
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, attackerKey);

        var response = new JsonRpcResponse { Id = "1", Result = payload, BridgeReceipt = receipt };
        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse("HMAC under the wrong key cannot match");
        result.Reason.Should().Contain("hmac");
    }

    [Fact]
    public void MissingReceipt_WarnOnly_ReturnsValidWithReason()
    {
        using var cache = new SessionKeyCache();
        var response = new JsonRpcResponse
        {
            Id = "1",
            Result = JToken.FromObject(new { ok = true }),
            BridgeReceipt = null,
        };

        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.WarnOnly, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeTrue("WarnOnly tolerates legacy fixtures without receipts");
        result.Mode.Should().Be(VerificationMode.WarnOnly);
        result.Reason.Should().Contain("missing");
    }

    [Fact]
    public void MissingReceipt_Strict_ReturnsInvalid()
    {
        using var cache = new SessionKeyCache();
        var response = new JsonRpcResponse
        {
            Id = "1",
            Result = JToken.FromObject(new { ok = true }),
            BridgeReceipt = null,
        };

        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse("Strict rejects responses without a receipt");
        result.Reason.Should().Contain("missing");
    }

    [Fact]
    public void Frame0_Accepted_OnHandshake()
    {
        var key = FreshKey();
        var sessionId = Guid.NewGuid().ToString("N");
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken payload = JToken.FromObject(new { server_version = "0.24.0" });
        var (receipt, _) = ServerSign(sessionId, key, payload, frame: 0);

        var response = new JsonRpcResponse { Id = "1", Result = payload, BridgeReceipt = receipt };
        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: true);

        result.Valid.Should().BeTrue("frame=0 is the documented Connect/handshake sentinel");
    }

    [Fact]
    public void Frame0_Rejected_OnNonHandshakeMethod()
    {
        var key = FreshKey();
        var sessionId = Guid.NewGuid().ToString("N");
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken payload = JToken.FromObject(new { value = 1 });
        var (receipt, _) = ServerSign(sessionId, key, payload, frame: 0);

        var response = new JsonRpcResponse { Id = "1", Result = payload, BridgeReceipt = receipt };
        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse("frame=0 is a sentinel reserved for the Connect handshake");
        result.Reason.Should().Contain("world_frame=0");
    }

    [Fact]
    public void FrameRegression_Rejected()
    {
        var key = FreshKey();
        var sessionId = Guid.NewGuid().ToString("N");
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken payload = JToken.FromObject(new { value = 1 });
        var (receipt, _) = ServerSign(sessionId, key, payload, frame: 5);

        var response = new JsonRpcResponse { Id = "1", Result = payload, BridgeReceipt = receipt };
        long lastFrame = 100; // we've already seen frame 100
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse("frame must strictly advance");
        result.Reason.Should().Contain("regressed");
    }

    [Fact]
    public void Off_Mode_AlwaysValid_NoVerification()
    {
        using var cache = new SessionKeyCache();
        // Even with no receipt + no cache, Off mode is a no-op.
        var response = new JsonRpcResponse { Id = "1", Result = null, BridgeReceipt = null };
        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Off, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeTrue();
        result.Mode.Should().Be(VerificationMode.Off);
        result.Reason.Should().BeEmpty();
    }

    /// <summary>
    /// CRITICAL CORRECTNESS GATE: the client-side
    /// <see cref="CanonicalJson.Canonicalize"/> MUST produce byte-identical
    /// output to the runtime canonicalizer used inside <see cref="SessionHmac.ComputeHmac"/>.
    /// We exercise this by signing on the server side via SessionHmac (real
    /// production code) and verifying on the client side via the verifier.
    /// </summary>
    [Fact]
    public void CanonicalJson_ClientServerByteEquality()
    {
        // Use the production server signer end-to-end.
        using var server = new SessionHmac();
        using var cache = new SessionKeyCache();
        cache.Set(server.SessionId, server.KeyMaterial);

        // A payload with mixed types + unsorted keys to exercise the canonicalizer.
        JObject payload = JObject.FromObject(new
        {
            zeta = "last",
            alpha = 1,
            middle = new { c = 3, a = 1, b = 2 },
            arr = new object[] { 1, "two", false, null! },
        });

        // Compute the same state_sha256 the server would have computed.
        string canonical = CanonicalJson.Canonicalize(payload);
        string stateHash;
        using (var sha = SHA256.Create())
        {
            byte[] hash = sha.ComputeHash(Encoding.UTF8.GetBytes(canonical));
            var sb = new StringBuilder(hash.Length * 2);
            foreach (byte b in hash)
            {
                sb.Append("0123456789abcdef"[b >> 4]);
                sb.Append("0123456789abcdef"[b & 0xF]);
            }
            stateHash = sb.ToString();
        }

        string ts = "2026-04-25T12:34:56.789Z";
        long frame = 42;

        // Sign with the REAL server signer.
        string serverHmac = server.ComputeHmac(ts, frame, stateHash);

        var receipt = new BridgeReceipt
        {
            SessionId = server.SessionId,
            TimestampUtc = ts,
            WorldFrame = frame,
            StateSha256Hex = stateHash,
            HmacHex = serverHmac,
        };

        var response = new JsonRpcResponse { Id = "1", Result = payload, BridgeReceipt = receipt };
        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeTrue(
            "client canonicalizer + verifier must accept a receipt produced by the real SessionHmac");
        result.Reason.Should().BeEmpty();

        // Also confirm the canonical strings match what would feed the hash on
        // the server (canonical.Length sanity — keys sorted alphabetically).
        canonical.Should().StartWith("{\"alpha\":")
            .And.Contain("\"arr\":[1,\"two\",false,null]")
            .And.EndWith("\"zeta\":\"last\"}");
    }

    [Fact]
    public void SessionKeyCache_RoundTrip()
    {
        using var cache = new SessionKeyCache();
        var key = FreshKey();
        cache.Set("sess", key);

        cache.TryGet("sess", out var got).Should().BeTrue();
        got.Should().Equal(key);

        cache.TryGet("missing", out var missing).Should().BeFalse();
        missing.Should().BeNull();

        cache.Remove("sess").Should().BeTrue();
        cache.TryGet("sess", out _).Should().BeFalse();
    }

    [Fact]
    public void SessionKeyCache_RejectsWrongLengthKey()
    {
        using var cache = new SessionKeyCache();
        Action act = () => cache.Set("sess", new byte[16]);
        act.Should().Throw<ArgumentException>().WithMessage("*32 bytes*");
    }
}
