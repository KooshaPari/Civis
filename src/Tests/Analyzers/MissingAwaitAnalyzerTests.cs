using System.Collections.Immutable;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class MissingAwaitAnalyzerTests
    {
        private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.MissingAwaitAnalyzer();

        [Fact]
        public void DF1017_HasCorrectId()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Id.Should().Be("DF1017");
        }

        [Fact]
        public void DF1017_HasWarningSeverity()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF1017_HasReliabilityCategory()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Category.Should().Be("Reliability");
        }

        [Fact]
        public void DF1017_HasSuppressionMarker()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors[0].Description.ToString().Should().Contain("fire-and-forget-ok:");
        }
    }
}
