using System.Collections.Immutable;
using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class HardcodedThresholdAnalyzerTests
    {
        private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.HardcodedThresholdAnalyzer();

        private static async Task<ImmutableArray<Diagnostic>> RunAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.ComponentModel.DataAnnotations.RangeAttribute).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF1014Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new DINOForge.Analyzers.HardcodedThresholdAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF1014").ToImmutableArray();
        }

        [Fact]
        public async Task DoesNotReport_When_LiteralInRangeAttribute()
        {
            const string source = @"
using System.ComponentModel.DataAnnotations;
public class Foo
{
    [Range(0, 100)]
    public int X { get; set; }

    [Range(0, 5000)]
    public int Y { get; set; }
}";
            var diagnostics = await RunAsync(source).ConfigureAwait(false);
            diagnostics.Should().BeEmpty(
                "literals inside attribute arguments (Range, etc.) are not user-tunable thresholds");
        }

        [Fact]
        public void DF1014_HasCorrectId()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Id.Should().Be("DF1014");
        }

        [Fact]
        public void DF1014_HasInfoSeverity()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
        }

        [Fact]
        public void DF1014_HasMaintainabilityCategory()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors[0].Category.Should().Be("Maintainability");
        }

        [Fact]
        public void DF1014_HasSuppressionMarker()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors[0].Description.ToString().Should().Contain("threshold-ok");
        }
    }
}
