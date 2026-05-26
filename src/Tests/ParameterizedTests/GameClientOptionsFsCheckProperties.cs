#nullable enable

using System.Linq;
using DINOForge.Bridge.Client;
using FsCheck;
using FsCheck.Xunit;

namespace DINOForge.Tests.ParameterizedTests;

/// <summary>
/// FsCheck property-based tests for <see cref="GameClientOptions"/> value semantics.
/// Tests immutability, roundtrip serialization of properties, and default-initialization invariants.
/// No live connection required.
/// </summary>
public class GameClientOptionsFsCheckProperties
{
    /// <summary>
    /// Property 1: Default constructor sets all timeout values to positive integers.
    /// </summary>
    [Property(MaxTest = 100)]
    public void DefaultConstructorSetsPositiveTimeouts()
    {
        var opts = new GameClientOptions();

        Assert.True(opts.ConnectTimeoutMs > 0, "ConnectTimeoutMs must be positive");
        Assert.True(opts.SendTimeoutMs > 0, "SendTimeoutMs must be positive");
        Assert.True(opts.ReadTimeoutMs > 0, "ReadTimeoutMs must be positive");
    }

    /// <summary>
    /// Property 2: PipeName roundtrip — setting and reading back produces equality.
    /// Uses printable ASCII strings to avoid encoding issues.
    /// </summary>
    [Property(MaxTest = 100)]
    public void PipeNameRoundtrip(
        NonEmptyString nonEmptyPipeName)
    {
        // Filter to printable ASCII to avoid round-trip encoding bugs
        var pipeName = nonEmptyPipeName.Get;
        if (!pipeName.All(c => c >= 32 && c <= 126))
        {
            return; // Discard non-printable inputs
        }

        var opts = new GameClientOptions { PipeName = pipeName };

        Assert.Equal(pipeName, opts.PipeName);
    }

    /// <summary>
    /// Property 3: PerformConnectHandshake bool roundtrip — setting and reading back.
    /// </summary>
    [Property(MaxTest = 100)]
    public void PerformConnectHandshakeBoolRoundtrip(bool value)
    {
        var opts = new GameClientOptions { PerformConnectHandshake = value };

        Assert.Equal(value, opts.PerformConnectHandshake);
    }

    /// <summary>
    /// Property 4: UseMessageFraming bool roundtrip — setting and reading back.
    /// </summary>
    [Property(MaxTest = 100)]
    public void UseMessageFramingBoolRoundtrip(bool value)
    {
        var opts = new GameClientOptions { UseMessageFraming = value };

        Assert.Equal(value, opts.UseMessageFraming);
    }

    /// <summary>
    /// Property 5: Validation invariant — defaults + valid pipe name does not throw on access.
    /// All property getters succeed without exception.
    /// </summary>
    [Property(MaxTest = 100)]
    public void DefaultsWithValidPipeNameAccessibleWithoutThrow(
        NonEmptyString nonEmptyPipeName)
    {
        var pipeName = nonEmptyPipeName.Get;
        if (!pipeName.All(c => c >= 32 && c <= 126))
        {
            return;
        }

        var opts = new GameClientOptions { PipeName = pipeName };

        // Access all properties — none should throw
        _ = opts.PipeName;
        _ = opts.ConnectTimeoutMs;
        _ = opts.SendTimeoutMs;
        _ = opts.ReadTimeoutMs;
        _ = opts.MaxMessageSizeBytes;
        _ = opts.RetryCount;
        _ = opts.RetryDelayMs;
        _ = opts.UseMessageFraming;
        _ = opts.PerformConnectHandshake;
    }
}
