using System;
using System.Collections.Generic;
using System.Linq;
using FluentAssertions;
using FluentAssertions.Collections;
using FluentAssertions.Execution;

namespace DINOForge.Tests.Support;

/// <summary>
/// Custom FluentAssertions extensions for precise collection validation.
/// Pattern #110 — HaveExactCount provides better error messages than open-ended inequalities.
/// </summary>
public static class EnumerableAssertionsExtensions
{
    /// <summary>
    /// Assert that a collection has exactly N items, with a descriptive error message
    /// showing the first 3 items if the count mismatch occurs.
    /// </summary>
    /// <typeparam name="T">Collection element type.</typeparam>
    /// <param name="assertions">The GenericCollectionAssertions to extend.</param>
    /// <param name="expected">Expected number of items.</param>
    /// <param name="because">Reason for the assertion (optional).</param>
    /// <param name="becauseArgs">Arguments for the reason format string.</param>
    /// <returns>AndConstraint for chaining.</returns>
    /// <remarks>
    /// This extension replaces patterns like .HaveCountGreaterThan(N) or .Count.Should().Be(N+1)
    /// with a stricter assertion that proves exact cardinality.
    /// </remarks>
    public static AndConstraint<GenericCollectionAssertions<T>> HaveExactCount<T>(
        this GenericCollectionAssertions<T> assertions,
        int expected,
        string because = "",
        params object[] becauseArgs)
    {
        if (assertions is null) throw new ArgumentNullException(nameof(assertions));
        if (expected < 0) throw new ArgumentException("Expected count cannot be negative.", nameof(expected));

        // Materialize the enumerable only once for count and sample
        var enumerable = assertions.Subject ?? Enumerable.Empty<T>();
        var items = enumerable.ToList();
        var actual = items.Count;

        if (actual != expected)
        {
            // Build a sample of the first 3 items for the error message
            var sample = string.Join(", ", items.Take(3).Select(i => i?.ToString() ?? "<null>"));
            var sampleSuffix = items.Count > 3 ? $", ... ({items.Count} total)" : "";

            var failureMessage = $"Expected exactly {expected} items, but found {actual}. " +
                                $"Sample: [{sample}{sampleSuffix}]";

            Execute.Assertion
                .BecauseOf(because, becauseArgs)
                .FailWith(failureMessage);
        }

        return new AndConstraint<GenericCollectionAssertions<T>>(assertions);
    }
}
