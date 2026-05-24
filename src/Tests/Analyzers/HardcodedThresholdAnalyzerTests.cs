using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using System.Threading.Tasks;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    [Trait("Category", "Analyzer")]
    public class HardcodedThresholdAnalyzerTests : AnalyzerTestBase<DINOForge.Analyzers.HardcodedThresholdAnalyzer>
    {
        private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.HardcodedThresholdAnalyzer();

        [Fact]
        public Task DoesNotReport_When_LiteralInRangeAttribute()
        {
            // tautological-ok: VerifyAnalyzerAsync(source) with no expected diagnostics asserts zero diagnostics produced
            const string source = @"
using System.ComponentModel.DataAnnotations;
public class Foo
{
    [Range(0, 100)]
    public int X { get; set; }

    [Range(0, 5000)]
    public int Y { get; set; }
}";
            return VerifyAnalyzerAsync(
                source);
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
