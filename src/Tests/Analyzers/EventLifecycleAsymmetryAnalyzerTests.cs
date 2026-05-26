using System.Collections.Immutable;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Analyzers;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    /// <summary>
    /// Tests for <see cref="EventLifecycleAsymmetryAnalyzer"/> (DF0105).
    /// Supports `// event-lifecycle-ok: &lt;reason&gt;` marker.
    /// </summary>
    [Trait("Category", "Analyzer")]
    public class EventLifecycleAsymmetryAnalyzerTests : AnalyzerTestBase<EventLifecycleAsymmetryAnalyzer>
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new EventLifecycleAsymmetryAnalyzer();

        [Fact]
        public void DF0105_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(2);
            diagnostics.Select(d => d.Id).Should().Contain(new[] { "DF0105", "DF0105a" });
            EventLifecycleAsymmetryAnalyzer.DiagnosticId.Should().Be("DF0105");
        }

        [Fact]
        public async Task Reports_OnSubscribeWithoutUnsubscribe()
        {
            // tautological-ok: VerifyAnalyzerAsync throws on unexpected diagnostic count / mismatch — it IS the assertion
            const string source = @"
using System;

public class C
{
    public event EventHandler? E;

    public void Subscribe()
    {
        E += Handler;
    }

    private void Handler(object? s, EventArgs e) { }

    public void Dispose()
    {
        // No -= here
    }
}";
            await VerifyAnalyzerAsync(
                source,
                Diagnostic("DF0105")
                    .WithSpan(10, 9, 10, 21)
                    .WithArguments("E", "Handler"));
        }

        [Fact]
        public async Task DoesNotReport_WhenUnsubscribeInDispose()
        {
            // tautological-ok: VerifyAnalyzerAsync(source) with no expected diagnostics asserts zero diagnostics produced
            const string source = @"
using System;

public class C
{
    public event EventHandler? E;

    public void Subscribe()
    {
        E += Handler;
    }

    private void Handler(object? s, EventArgs e) { }

    public void Dispose()
    {
        E -= Handler;
    }
}";
            await VerifyAnalyzerAsync(source);
        }

        [Fact]
        public async Task DoesNotReport_WhenSuppressionMarkerPresent()
        {
            // tautological-ok: VerifyAnalyzerAsync(source) with no expected diagnostics asserts zero diagnostics produced
            const string source = @"
using System;

public class C
{
    public event EventHandler? E;

    public void Subscribe()
    {
        // event-lifecycle-ok: handler lifetime tied to host
        E += Handler;
    }

    private void Handler(object? s, EventArgs e) { }

    public void Dispose() { }
}";
            await VerifyAnalyzerAsync(source);
        }
    }
}
