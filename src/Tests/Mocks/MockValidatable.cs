#nullable enable
using System;
using DINOForge.SDK.Validation;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// Configurable mock implementation of <see cref="IValidatable"/> for unit testing.
/// Allows injection of custom validation behavior and exception throwing via properties.
/// Tracks validation call counts for behavior verification.
/// </summary>
public class MockValidatable : IValidatable
{
    /// <summary>
    /// Optional callback invoked when <see cref="Validate"/> is called.
    /// If null, no action is taken (default behavior is no-op validation).
    /// </summary>
    public Action? OnValidate { get; set; }

    /// <summary>
    /// If non-null, <see cref="Validate"/> throws this exception instead of returning a result.
    /// </summary>
    public Exception? ThrowOnValidate { get; set; }

    /// <summary>
    /// Number of times <see cref="Validate"/> has been called.
    /// </summary>
    public int ValidateCallCount { get; set; }

    /// <summary>
    /// Performs mock validation. If <see cref="ThrowOnValidate"/> is set, throws that exception.
    /// Otherwise invokes <see cref="OnValidate"/> if set, then returns success.
    /// Increments <see cref="ValidateCallCount"/>.
    /// </summary>
    public ValidationResult Validate()
    {
        ValidateCallCount++;

        if (ThrowOnValidate != null)
            throw ThrowOnValidate;

        OnValidate?.Invoke();

        return ValidationResult.Success();
    }
}
