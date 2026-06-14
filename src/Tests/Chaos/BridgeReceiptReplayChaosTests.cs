#nullable enable
using System;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Chaos;

public class BridgeReceiptReplayChaosTests
{
    [Fact]
    [Trait("Category", "Chaos")]
    public void ReplayedReceipt_IsRejected()
    {
        byte[] key = CreateKey();
        string sessionId = "chaos-session";
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken payload = JToken.FromObject(new { ok = true });
        string timestamp = "2026-04-25T12:34:56.789Z";
        string canonical = CanonicalJson.Canonicalize(payload);
        string stateHash = Sha256Hex(Encoding.UTF8.GetBytes(canonical));
        long frame = 7;
        string hmac = BridgeReceiptVerifier.ComputeReceiptHmac(key, timestamp, frame, stateHash);

        var response = new JsonRpcResponse
        {
            Id = "1",
            Result = payload,
            BridgeReceipt = new BridgeReceipt
            {
                SessionId = sessionId,
                TimestampUtc = timestamp,
                WorldFrame = frame,
                StateSha256Hex = stateHash,
                HmacHex = hmac
            }
        };

        long lastFrame = 0;
        VerificationResult first = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        first.Valid.Should().BeTrue();
        lastFrame.Should().Be(frame);

        VerificationResult replay = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        replay.Valid.Should().BeFalse();
        replay.Reason.Should().Contain("regressed");
        lastFrame.Should().Be(frame);
    }

    private static byte[] CreateKey()
    {
        byte[] key = new byte[32];
        for (int index = 0; index < key.Length; index++)
        {
            key[index] = (byte)index;
        }

        return key;
    }

    private static string Sha256Hex(byte[] input)
    {
        using var sha = SHA256.Create();
        byte[] hash = sha.ComputeHash(input);
        const string hex = "0123456789abcdef";
        var sb = new StringBuilder(hash.Length * 2);
        for (int index = 0; index < hash.Length; index++)
        {
            byte value = hash[index];
            sb.Append(hex[value >> 4]);
            sb.Append(hex[value & 0xF]);
        }

        return sb.ToString();
    }
}
