#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Numerics;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Focused edge-case tests for Bridge canonical JSON and payload-hash helpers.
/// These cases target the branches the existing golden tests do not hit, while
/// staying small and deterministic.
/// </summary>
public class CanonicalJsonEdgeCaseTests
{
    [Fact]
    public void ComputePayloadHash_NestedObjectsAndArrays_UsesCanonicalJson()
    {
        JToken payload = JToken.ReadFrom(
            new JsonTextReader(new StringReader("{\"z\":1,\"outer\":[{\"b\":2,\"a\":18446744073709551616},{\"d\":4,\"c\":3}]}")),
            new JsonLoadSettings());

        string canonical = "{\"outer\":[{\"a\":18446744073709551616,\"b\":2},{\"c\":3,\"d\":4}],\"z\":1}";
        string expectedHash = Sha256Hex(Encoding.UTF8.GetBytes(canonical));

        BridgeReceiptBuilder.ComputePayloadHash(payload).Should().Be(expectedHash);
    }

    [Fact]
    public void Canonicalize_StringScalarRoot_ReturnsJsonLiteral()
    {
        string actual = CanonicalJson.Canonicalize(JToken.Parse("\"alpha\""));

        actual.Should().Be("\"alpha\"");
    }

    [Theory]
    [MemberData(nameof(ScalarFloatingPointRoots))]
    public void Canonicalize_ScalarFloatingPointRoots_UseExpectedFormatting(JToken token, string expected)
    {
        string actual = CanonicalJson.Canonicalize(token);

        actual.Should().Be(expected);
    }

    public static IEnumerable<object[]> ScalarFloatingPointRoots()
    {
        yield return new object[] { new JValue(1.25f), "1.25" };
        yield return new object[] { new JValue(1.25m), "1.25" };
    }

    [Fact]
    public void Canonicalize_RejectsInfinityStringRoot()
    {
        Action act = () => CanonicalJson.Canonicalize(new JValue("Infinity"));

        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*NaN/Infinity*");
    }

    [Fact]
    public void Canonicalize_RejectsUnsafeNumericInNestedContainer()
    {
        var payload = new JObject
        {
            ["outer"] = new JArray
            {
                new JObject
                {
                    ["b"] = 2,
                    ["a"] = new JValue(BigInteger.Parse("18446744073709551616", CultureInfo.InvariantCulture))
                },
                new JObject
                {
                    ["d"] = 4,
                    ["c"] = new JValue(float.NaN)
                }
            },
            ["z"] = 1
        };

        Action act = () => CanonicalJson.Canonicalize(payload);

        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*NaN/Infinity*");
    }

    private static string Sha256Hex(byte[] input)
    {
        using SHA256 sha = SHA256.Create();
        byte[] hash = sha.ComputeHash(input);
        var sb = new StringBuilder(hash.Length * 2);

        foreach (byte b in hash)
        {
            sb.Append("0123456789abcdef"[b >> 4]);
            sb.Append("0123456789abcdef"[b & 0xF]);
        }

        return sb.ToString();
    }
}
