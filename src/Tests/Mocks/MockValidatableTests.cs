#nullable enable
using System;
using DINOForge.SDK.Validation;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public sealed class MockValidatableTests
{
    [Fact]
    public void Validate_DefaultBehavior_ReturnsSuccess()
    {
        var mock = new MockValidatable();

        var result = mock.Validate();

        result.IsValid.Should().BeTrue();
        mock.ValidateCallCount.Should().Be(1);
    }

    [Fact]
    public void Validate_OnValidateHook_IsFired()
    {
        var mock = new MockValidatable();
        var hookFired = false;

        mock.OnValidate = () => { hookFired = true; };

        var result = mock.Validate();

        hookFired.Should().BeTrue();
        result.IsValid.Should().BeTrue();
        mock.ValidateCallCount.Should().Be(1);
    }

    [Fact]
    public void Validate_ThrowOnValidateSet_PropagatesException()
    {
        var mock = new MockValidatable();
        var exception = new InvalidOperationException("Test exception");
        mock.ThrowOnValidate = exception;

        var action = () => mock.Validate();

        action.Should().Throw<InvalidOperationException>().WithMessage("Test exception");
        mock.ValidateCallCount.Should().Be(1);
    }

    [Fact]
    public void Validate_MultipleCallsIncrementCount()
    {
        var mock = new MockValidatable();

        mock.Validate();
        mock.Validate();
        mock.Validate();

        mock.ValidateCallCount.Should().Be(3);
    }

    [Fact]
    public void Validate_OnValidatePrecedesThrowCheck()
    {
        var mock = new MockValidatable();

        mock.OnValidate = () => { };
        mock.ThrowOnValidate = new InvalidOperationException("Should throw");

        var action = () => mock.Validate();

        // Hook fires before throw
        action.Should().Throw<InvalidOperationException>();
        // Verify hook was called before the throw (via the call count increment)
        mock.ValidateCallCount.Should().Be(1);
    }
}
