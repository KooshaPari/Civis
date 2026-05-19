using System.Collections.Immutable;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class HardcodedThresholdAnalyzerTests
    {
        private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.HardcodedThresholdAnalyzer();

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
