#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Coverage tests for <see cref="ProtocolException"/> (constructor + inheritance contract)
/// and <see cref="ReloadResult"/> (defaults + JSON round-trip).
/// </summary>
public class ProtocolExceptionCoverageTests
{
    [Fact]
    public void Ctor_WithMessage_SetsMessageAndIsInvalidOperationException()
    {
        ProtocolException ex = new("bridge handshake failed");

        ex.Message.Should().Be("bridge handshake failed");
        ex.InnerException.Should().BeNull();
        ex.Should().BeAssignableTo<InvalidOperationException>();
    }

    [Fact]
    public void Ctor_WithMessageAndInner_PreservesInnerException()
    {
        Exception inner = new InvalidOperationException("pipe closed");

        ProtocolException ex = new("bridge call failed", inner);

        ex.Message.Should().Be("bridge call failed");
        ex.InnerException.Should().BeSameAs(inner);
    }

    [Fact]
    public void IsThrowableAndCatchableAsBaseType()
    {
        Action act = () => throw new ProtocolException("boom");

        act.Should().Throw<InvalidOperationException>().WithMessage("boom");
    }

    [Fact]
    public void ReloadResult_Defaults_AreFalseAndEmpty()
    {
        ReloadResult result = new();

        result.Success.Should().BeFalse();
        result.LoadedPacks.Should().NotBeNull().And.BeEmpty();
        result.Errors.Should().NotBeNull().And.BeEmpty();
    }

    [Fact]
    public void ReloadResult_RoundTripsThroughJson()
    {
        ReloadResult original = new()
        {
            Success = true,
            LoadedPacks = new List<string> { "warfare-modern", "economy-balanced" },
            Errors = new List<string> { "pack X skipped" }
        };

        string json = JsonConvert.SerializeObject(original);
        ReloadResult? back = JsonConvert.DeserializeObject<ReloadResult>(json);

        back.Should().NotBeNull();
        back!.Success.Should().BeTrue();
        back.LoadedPacks.Should().Equal("warfare-modern", "economy-balanced");
        back.Errors.Should().ContainSingle().Which.Should().Be("pack X skipped");
    }
}
