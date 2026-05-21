using System.Collections.Immutable;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Analyzers;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    /// <summary>
    /// Tests for <see cref="EventLifecycleAsymmetryAnalyzer"/> (DF0105).
    /// Supports `// event-lifecycle-ok: &lt;reason&gt;` marker.
    /// </summary>
    public class EventLifecycleAsymmetryAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new EventLifecycleAsymmetryAnalyzer();

        [Fact]
        public void DF0105_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0105");
            EventLifecycleAsymmetryAnalyzer.DiagnosticId.Should().Be("DF0105");
        }

        [Fact]
        public async Task Reports_OnSubscribeWithoutUnsubscribe()
        {
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
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0105");
        }

        [Fact]
        public async Task DoesNotReport_WhenUnsubscribeInDispose()
        {
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
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenSuppressionMarkerPresent()
        {
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
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAnalyzerAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.EventHandler).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0105Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new EventLifecycleAsymmetryAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0105").ToImmutableArray();
        }
    }
}
