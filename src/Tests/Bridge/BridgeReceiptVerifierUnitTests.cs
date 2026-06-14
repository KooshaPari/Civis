#nullable enable
using System;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Unit tests for <see cref="BridgeReceiptVerifier"/>.
/// Covers HMAC verification, frame monotonicity, receipt validation modes.
/// </summary>
public class BridgeReceiptVerifierUnitTests
{
    private static byte[] CreateSessionKey(byte seed = 42)
    {
        var key = new byte[32];
        for (int i = 0; i < 32; i++) key[i] = (byte)((seed + i) % 256);
        return key;
    }

    private static BridgeReceipt CreateReceipt(
        string sessionId = "session-123",
        long worldFrame = 1,
        string stateSha256Hex = "abcd1234",
        string? hmacHex = null,
        string? timestampUtc = null)
    {
        timestampUtc ??= "2026-04-25T12:34:56.789Z";
        sessionId ??= "session-123";
        stateSha256Hex ??= "abcd1234";

        var key = CreateSessionKey();
        hmacHex ??= BridgeReceiptVerifier.ComputeReceiptHmac(key, timestampUtc, worldFrame, stateSha256Hex);

        return new BridgeReceipt
        {
            SessionId = sessionId,
            TimestampUtc = timestampUtc,
            WorldFrame = worldFrame,
            StateSha256Hex = stateSha256Hex,
            HmacHex = hmacHex
        };
    }

    private static JsonRpcResponse CreateResponse(BridgeReceipt? receipt = null)
    {
        receipt ??= CreateReceipt();
        return new JsonRpcResponse
        {
            Jsonrpc = "2.0",
            Id = "1",
            Result = JObject.FromObject(new { data = "test" }),
            BridgeReceipt = receipt
        };
    }

    [Fact]
    public void Verify_WithModeOff_AlwaysReturnsValid()
    {
        var cache = new SessionKeyCache();
        var response = CreateResponse(null); // null receipt
        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Off, ref lastFrame, false);

        result.Valid.Should().BeTrue();
        result.Mode.Should().Be(VerificationMode.Off);
        result.Reason.Should().BeEmpty();
    }

    [Fact]
    public void Verify_WithNullResponse_ThrowsArgumentNullException()
    {
        var cache = new SessionKeyCache();
        long lastFrame = 0;

        Action act = () => BridgeReceiptVerifier.Verify(null!, cache, VerificationMode.Strict, ref lastFrame, false);

        act.Should().Throw<ArgumentNullException>().WithParameterName("response");
    }

    [Fact]
    public void Verify_WithNullCache_ThrowsArgumentNullException()
    {
        var response = CreateResponse();
        long lastFrame = 0;

        Action act = () => BridgeReceiptVerifier.Verify(response, null!, VerificationMode.Strict, ref lastFrame, false);

        act.Should().Throw<ArgumentNullException>().WithParameterName("cache");
    }

    [Fact]
    public void Verify_WithMissingReceipt_WarnOnlyReturnsValid()
    {
        var cache = new SessionKeyCache();
        var response = new JsonRpcResponse
        {
            Jsonrpc = "2.0",
            Id = "1",
            Result = JObject.FromObject(new { data = "test" }),
            BridgeReceipt = null
        };
        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.WarnOnly, ref lastFrame, false);

        result.Valid.Should().BeTrue();
        result.Reason.Should().Contain("missing bridge_receipt");
    }

    [Fact]
    public void Verify_WithMissingReceipt_StrictReturnsFalse()
    {
        var cache = new SessionKeyCache();
        var response = new JsonRpcResponse
        {
            Jsonrpc = "2.0",
            Id = "1",
            Result = JObject.FromObject(new { data = "test" }),
            BridgeReceipt = null
        };
        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, false);

        result.Valid.Should().BeFalse();
        result.Reason.Should().Contain("missing bridge_receipt");
    }

    [Fact]
    public void Verify_WithUnknownSessionId_StrictReturnsFalse()
    {
        var cache = new SessionKeyCache();
        var receipt = CreateReceipt(sessionId: "unknown-session");
        var response = CreateResponse(receipt);
        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, false);

        result.Valid.Should().BeFalse();
        result.Reason.Should().Contain("unknown session_id");
    }

    [Fact]
    public void Verify_WithUnknownSessionId_IsHandshakeReturnsValid()
    {
        var cache = new SessionKeyCache();
        var receipt = CreateReceipt(sessionId: "unknown-session");
        var response = CreateResponse(receipt);
        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: true);

        result.Valid.Should().BeTrue();
        result.Reason.Should().BeEmpty();
    }

    [Fact]
    public void Verify_WithValidReceipt_ReturnsValid()
    {
        var cache = new SessionKeyCache();
        var key = CreateSessionKey();
        cache.Set("session-123", key);

        var response = CreateResponse();
        var stateSha = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(response.Result)));
        var receipt = CreateReceipt(sessionId: "session-123", worldFrame: 1, stateSha256Hex: stateSha);
        response.BridgeReceipt = receipt;

        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, false);

        result.Valid.Should().BeTrue();
        result.Reason.Should().BeEmpty();
    }

    [Fact]
    public void Verify_WithValidReceipt_UpdatesLastFrame()
    {
        var cache = new SessionKeyCache();
        var key = CreateSessionKey();
        cache.Set("session-123", key);

        var response = CreateResponse();
        var stateSha = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(response.Result)));
        var receipt = CreateReceipt(sessionId: "session-123", worldFrame: 42, stateSha256Hex: stateSha);
        response.BridgeReceipt = receipt;

        long lastFrame = 0;

        BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, false);

        lastFrame.Should().Be(42);
    }

    [Fact]
    public void Verify_WithRegressedFrame_ReturnsFalse()
    {
        var cache = new SessionKeyCache();
        var key = CreateSessionKey();
        cache.Set("session-123", key);

        // Create the response first to get the correct payload hash
        var response = CreateResponse();
        var stateSha = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(response.Result)));
        var receipt = CreateReceipt(sessionId: "session-123", worldFrame: 10, stateSha256Hex: stateSha);
        response.BridgeReceipt = receipt;

        long lastFrame = 20;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, false);

        result.Valid.Should().BeFalse();
        result.Reason.Should().Contain("regressed");
    }

    [Fact]
    public void Verify_WithZeroFrameOutsideHandshake_ReturnsFalse()
    {
        var cache = new SessionKeyCache();
        var key = CreateSessionKey();
        cache.Set("session-123", key);

        var response = CreateResponse();
        var stateSha = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(response.Result)));
        var receipt = CreateReceipt(sessionId: "session-123", worldFrame: 0, stateSha256Hex: stateSha);
        response.BridgeReceipt = receipt;

        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse();
        result.Reason.Should().Contain("world_frame=0 outside Connect");
    }

    [Fact]
    public void Verify_WithZeroFrameInHandshake_ReturnsValid()
    {
        var cache = new SessionKeyCache();
        var key = CreateSessionKey();
        cache.Set("session-123", key);

        var response = CreateResponse();
        var stateSha = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(response.Result)));
        var receipt = CreateReceipt(sessionId: "session-123", worldFrame: 0, stateSha256Hex: stateSha);
        response.BridgeReceipt = receipt;

        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: true);

        result.Valid.Should().BeTrue();
    }

    [Fact]
    public void Verify_WithTamperedHmac_ReturnsFalse()
    {
        var cache = new SessionKeyCache();
        var key = CreateSessionKey();
        cache.Set("session-123", key);

        var response = CreateResponse();
        var stateSha = Sha256Hex(Encoding.UTF8.GetBytes(CanonicalJson.Canonicalize(response.Result)));
        var receipt = CreateReceipt(sessionId: "session-123", worldFrame: 1, stateSha256Hex: stateSha, hmacHex: "badbadbadbadbad");
        response.BridgeReceipt = receipt;

        long lastFrame = 0;

        var result = BridgeReceiptVerifier.Verify(response, cache, VerificationMode.Strict, ref lastFrame, false);

        result.Valid.Should().BeFalse();
        result.Reason.Should().Contain("hmac mismatch");
    }

    [Fact]
    public void ComputeReceiptHmac_WithValidInputs_ReturnsHex()
    {
        var key = CreateSessionKey();
        var timestamp = "2026-04-25T12:34:56.789Z";
        var frame = 1L;
        var stateSha = "abcd1234";

        var hmac = BridgeReceiptVerifier.ComputeReceiptHmac(key, timestamp, frame, stateSha);

        hmac.Should().NotBeNullOrEmpty();
        hmac.Should().HaveLength(64); // SHA256 hex = 64 chars
        hmac.Should().MatchRegex("^[0-9a-f]+$");
    }

    [Fact]
    public void ComputeReceiptHmac_WithNullKey_ThrowsArgumentNullException()
    {
        Action act = () => BridgeReceiptVerifier.ComputeReceiptHmac(null!, "2026-04-25T12:34:56.789Z", 1, "abcd");

        act.Should().Throw<ArgumentNullException>().WithParameterName("sessionKey");
    }

    [Fact]
    public void ComputeReceiptHmac_WithNullTimestamp_ThrowsArgumentNullException()
    {
        var key = CreateSessionKey();
        Action act = () => BridgeReceiptVerifier.ComputeReceiptHmac(key, null!, 1, "abcd");

        act.Should().Throw<ArgumentNullException>().WithParameterName("timestampUtc");
    }

    [Fact]
    public void ComputeReceiptHmac_WithNullStateSha_ThrowsArgumentNullException()
    {
        var key = CreateSessionKey();
        Action act = () => BridgeReceiptVerifier.ComputeReceiptHmac(key, "2026-04-25T12:34:56.789Z", 1, null!);

        act.Should().Throw<ArgumentNullException>().WithParameterName("stateSha256Hex");
    }

    [Fact]
    public void ComputeReceiptHmac_WithDifferentInputs_ProducesDifferentOutputs()
    {
        var key = CreateSessionKey();
        var hmac1 = BridgeReceiptVerifier.ComputeReceiptHmac(key, "2026-04-25T12:34:56.789Z", 1, "abcd");
        var hmac2 = BridgeReceiptVerifier.ComputeReceiptHmac(key, "2026-04-25T12:34:56.789Z", 2, "abcd");

        hmac1.Should().NotBe(hmac2);
    }

    [Theory]
    [InlineData("0123456789abcdef", "0123456789abcdef", true)]
    [InlineData("0123456789ABCDEF", "0123456789abcdef", true)]
    [InlineData("0123456789abcdef", "0123456789abcdee", false)]
    [InlineData("0123456789abcdef", "0123456789abcde", false)]
    [InlineData("0123456789abcdeg", "0123456789abcdeg", false)]
    [InlineData(null, "0123456789abcdef", false)]
    [InlineData("0123456789abcdef", null, false)]
    public void ConstantTimeHexEquals_CoversValidAndInvalidInputs(
        string? left,
        string? right,
        bool expected)
    {
        BridgeReceiptVerifier.ConstantTimeHexEquals(left, right).Should().Be(expected);
    }

    [Fact]
    public void VerificationResult_Construction_StoredCorrectly()
    {
        var result = new VerificationResult(true, VerificationMode.Strict, "test reason");

        result.Valid.Should().BeTrue();
        result.Mode.Should().Be(VerificationMode.Strict);
        result.Reason.Should().Be("test reason");
    }

    [Fact]
    public void VerificationResult_WithNullReason_DefaultsToEmpty()
    {
        var result = new VerificationResult(false, VerificationMode.WarnOnly, null!);

        result.Reason.Should().BeEmpty();
    }

    private static string Sha256Hex(byte[] input)
    {
        using var sha = SHA256.Create();
        return ToHexLower(sha.ComputeHash(input));
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
