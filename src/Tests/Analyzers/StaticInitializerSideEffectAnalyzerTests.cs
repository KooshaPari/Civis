using System.Collections.Immutable;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class StaticInitializerSideEffectAnalyzerTests
    {
        private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.StaticInitializerSideEffectAnalyzer();

        [Fact]
        public void DF1028_HasCorrectId()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Id.Should().Be("DF1028");
        }

        [Fact]
        public void DF1028_HasInfoSeverity()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
        }

        [Fact]
        public void DF1028_HasReliabilityCategory()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Category.Should().Be("Reliability");
        }

        [Fact]
        public void DF1028_HasSuppressionMarker()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors[0].Description.ToString().Should().Contain("static-side-effect-ok:");
        }
    }
}
