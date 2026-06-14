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

public class BridgeReceiptChaosTests
{
    [Fact]
    [Trait("Category", "Chaos")]
    public void TamperedReceipt_IsRejected()
    {
        byte[] key = CreateKey();
        string sessionId = "chaos-session";
        using var cache = new SessionKeyCache();
        cache.Set(sessionId, key);

        JToken payload = JToken.FromObject(new { ok = true });
        string timestamp = "2026-04-25T12:34:56.789Z";
        string canonical = CanonicalJson.Canonicalize(payload);
        string stateHash = Sha256Hex(Encoding.UTF8.GetBytes(canonical));
        string hmac = BridgeReceiptVerifier.ComputeReceiptHmac(key, timestamp, 1, stateHash);

        var response = new JsonRpcResponse
        {
            Id = "1",
            Result = payload,
            BridgeReceipt = new BridgeReceipt
            {
                SessionId = sessionId,
                TimestampUtc = timestamp,
                WorldFrame = 1,
                StateSha256Hex = stateHash,
                HmacHex = hmac[..60] + "ffff"
            }
        };

        long lastFrame = 0;
        VerificationResult result = BridgeReceiptVerifier.Verify(
            response, cache, VerificationMode.Strict, ref lastFrame, isHandshake: false);

        result.Valid.Should().BeFalse();
        result.Reason.Should().Contain("hmac mismatch");
        lastFrame.Should().Be(0);
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
