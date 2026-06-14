#nullable enable
using System;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json.Linq;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace DINOForge.Tests.BridgeSmoke;

[TestClass]
public class CanonicalJsonSmokeTests
{
    [TestMethod]
    public void Canonicalize_SortsObjectProperties()
    {
        AssertCanonical("{\"b\":1,\"a\":2}", "{\"a\":2,\"b\":1}");
    }

    [TestMethod]
    public void Canonicalize_PreservesNestedOrdering()
    {
        AssertCanonical("{\"x\":[{\"b\":1,\"a\":2}]}", "{\"x\":[{\"a\":2,\"b\":1}]}");
    }

    [TestMethod]
    public void Canonicalize_PreservesBooleanLiterals()
    {
        AssertCanonical("{\"x\":true}", "{\"x\":true}");
    }

    [TestMethod]
    public void Canonicalize_NullToken_ReturnsLiteralNull()
    {
        string actual = CanonicalJson.Canonicalize(null);

        actual.Should().Be("null");
    }

    [TestMethod]
    public void Canonicalize_NegativeZero_NormalizesToZero()
    {
        JToken token = JToken.FromObject(-0.0d);

        string actual = CanonicalJson.Canonicalize(token);

        actual.Should().Be("0");
    }

    [TestMethod]
    public void Canonicalize_RejectsNaN()
    {
        JToken token = JToken.FromObject(double.NaN);

        Action act = () => CanonicalJson.Canonicalize(token);

        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*NaN/Infinity*");
    }

    private static void AssertCanonical(string input, string expected)
    {
        JToken token = JToken.Parse(input);

        string actual = CanonicalJson.Canonicalize(token);

        actual.Should().Be(expected);
    }
}
