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
    /// Tests for <see cref="StringBuilderCapacityAnalyzer"/> (DF0117).
    /// Analyzer has NO suppression marker — substitute negative case is
    /// `new StringBuilder(capacity)` with explicit capacity arg.
    /// </summary>
    public class StringBuilderCapacityAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new StringBuilderCapacityAnalyzer();

        [Fact]
        public void DF0117_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0117");
            StringBuilderCapacityAnalyzer.DiagnosticId.Should().Be("DF0117");
        }

        [Fact]
        public async Task Reports_OnDefaultStringBuilderConstruction()
        {
            const string source = @"
using System.Text;

public class C
{
    public void M()
    {
        var sb = new StringBuilder();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0117");
        }

        [Fact]
        public async Task DoesNotReport_WhenCapacityProvided()
        {
            const string source = @"
using System.Text;

public class C
{
    public void M()
    {
        var sb = new StringBuilder(4096);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenInitialStringProvided()
        {
            const string source = @"
using System.Text;

public class C
{
    public void M()
    {
        var sb = new StringBuilder(""seed"");
    }
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
                MetadataReference.CreateFromFile(typeof(System.Text.StringBuilder).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0117Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new StringBuilderCapacityAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0117").ToImmutableArray();
        }
    }
}
