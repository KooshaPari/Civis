#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Numerics;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.BridgeMutation;

/// <summary>
/// Focused mutation-kill tests for <see cref="CanonicalJson"/>.
/// The cases here target the branches and rare paths most likely to survive
/// broad coverage-only testing: scalar roots, negative zero, finite float
/// formatting, unsafe numeric rejection, and canonical ordering for objects
/// and nested containers.
/// </summary>
public class CanonicalJsonMutationTests
{
    [Theory]
    [InlineData("{\"b\":1,\"a\":2}", "{\"a\":2,\"b\":1}")]
    [InlineData("{\"x\":[{\"b\":1,\"a\":2}]}", "{\"x\":[{\"a\":2,\"b\":1}]}")]
    [InlineData("{\"a\":1,\"Z\":2}", "{\"Z\":2,\"a\":1}")]
    public void Canonicalize_ProducesExpectedOutput(string input, string expected)
    {
        JToken token = JToken.Parse(input);

        string actual = CanonicalJson.Canonicalize(token);

        actual.Should().Be(expected);
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
    public void Canonicalize_NegativeZero_NormalizesToZero()
    {
        JToken token = JToken.FromObject(-0.0d);

        string actual = CanonicalJson.Canonicalize(token);

        actual.Should().Be("0");
    }

    [Fact]
    public void Canonicalize_RejectsNonFiniteScalars()
    {
        Action nan = () => CanonicalJson.Canonicalize(JToken.FromObject(double.NaN));
        Action positiveInfinity = () => CanonicalJson.Canonicalize(JToken.FromObject(double.PositiveInfinity));
        Action negativeInfinity = () => CanonicalJson.Canonicalize(JToken.FromObject(double.NegativeInfinity));

        nan.Should().Throw<InvalidOperationException>().WithMessage("*NaN/Infinity*");
        positiveInfinity.Should().Throw<InvalidOperationException>().WithMessage("*NaN/Infinity*");
        negativeInfinity.Should().Throw<InvalidOperationException>().WithMessage("*NaN/Infinity*");
    }

    [Fact]
    public void Canonicalize_HandlesBigIntegerNestedObject()
    {
        string input = "{\"outer\":[{\"b\":2,\"a\":18446744073709551616}],\"z\":1}";
        JToken token = JToken.ReadFrom(
            new JsonTextReader(new StringReader(input)),
            new JsonLoadSettings());

        string actual = CanonicalJson.Canonicalize(token);

        actual.Should().Be("{\"outer\":[{\"a\":18446744073709551616,\"b\":2}],\"z\":1}");
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
}
