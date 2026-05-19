#nullable enable
using System;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Golden-output tests for the shared <see cref="CanonicalJson"/> implementation
/// (#217 Phase A). Locks down the byte-for-byte canonical form for the cases
/// that matter to bridge HMAC receipts: sorted keys, empty containers, nested
/// recursion, float .0 stripping, boolean/null literals, escaped strings, and
/// ordinal property-key sort (capital letters sort before lowercase).
/// </summary>
/// <remarks>
/// These tests exist because <see cref="CanonicalJson"/> is the byte-equality
/// contract between server (state_sha256) and client (receipt verifier). A
/// silent change to canonical output here breaks every receipt across the wire,
/// so the goldens are the early-warning that keeps drift from landing.
/// </remarks>
public class CanonicalJsonGoldensTests
{
    [Theory]
    // Sorted keys (object property order)
    [InlineData("{\"b\":1,\"a\":2}", "{\"a\":2,\"b\":1}")]
    // Empty containers
    [InlineData("{}", "{}")]
    [InlineData("[]", "[]")]
    // Nested recursion: child object inside array inside object
    [InlineData("{\"x\":[{\"b\":1,\"a\":2}]}", "{\"x\":[{\"a\":2,\"b\":1}]}")]
    // Float .0 stripping (DOCUMENTED behavior — JSON.NET parses 1.0 as Integer when
    // it has no fractional component; "R" formatter on a long emits "1", not "1.0")
    [InlineData("{\"x\":1.0}", "{\"x\":1}")]
    [InlineData("{\"x\":42.0}", "{\"x\":42}")]
    // True float (preserves precision via "R" round-trip)
    [InlineData("{\"x\":3.14}", "{\"x\":3.14}")]
    // Boolean lowercase
    [InlineData("{\"x\":true}", "{\"x\":true}")]
    [InlineData("{\"x\":false}", "{\"x\":false}")]
    // Null literal
    [InlineData("{\"x\":null}", "{\"x\":null}")]
    // Special chars in key (escaped quote in key)
    [InlineData("{\"a\\\"b\":1}", "{\"a\\\"b\":1}")]
    // Property keys ordinal sort: capital 'Z' (0x5A) sorts before lowercase 'a' (0x61)
    [InlineData("{\"a\":1,\"Z\":2}", "{\"Z\":2,\"a\":1}")]
    public void Canonicalize_ProducesExpectedOutput(string input, string expected)
    {
        var token = JToken.Parse(input);
        var actual = CanonicalJson.Canonicalize(token);
        actual.Should().Be(expected);
    }

    [Fact]
    public void Canonicalize_RejectsNaN()
    {
        // JToken.FromObject(double.NaN) routes via JValue(double) which preserves NaN.
        var token = JToken.FromObject(double.NaN);
        Action act = () => CanonicalJson.Canonicalize(token);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*NaN/Infinity*");
    }

    [Fact]
    public void Canonicalize_RejectsPositiveInfinity()
    {
        var token = JToken.FromObject(double.PositiveInfinity);
        Action act = () => CanonicalJson.Canonicalize(token);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*NaN/Infinity*");
    }

    [Fact]
    public void Canonicalize_RejectsNegativeInfinity()
    {
        var token = JToken.FromObject(double.NegativeInfinity);
        Action act = () => CanonicalJson.Canonicalize(token);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*NaN/Infinity*");
    }

    [Fact]
    public void Canonicalize_HandlesLongOverflow()
    {
        // 2^64 doesn't fit in long (max long is 2^63-1). Newtonsoft surfaces
        // these as JTokenType.Integer backed by BigInteger; the canonicalizer
        // falls back to BigInteger.ToString instead of throwing.
        var settings = new JsonLoadSettings();
        var reader = new JsonTextReader(new System.IO.StringReader("{\"x\":18446744073709551616}"));
        var token = JToken.ReadFrom(reader, settings);
        var output = CanonicalJson.Canonicalize(token);
        output.Should().Be("{\"x\":18446744073709551616}");
    }

    [Fact]
    public void Canonicalize_NullToken_ReturnsLiteralNull()
    {
        // The null-input contract: the SHA-256 of "no payload" is well-defined
        // as sha256("null"), matching the legacy server behaviour.
        var actual = CanonicalJson.Canonicalize(null);
        actual.Should().Be("null");
    }
}
