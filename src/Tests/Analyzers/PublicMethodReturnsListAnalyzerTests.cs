using System.Collections.Immutable;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class PublicMethodReturnsListAnalyzerTests
    {
        private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.PublicMethodReturnsListAnalyzer();

        [Fact]
        public void DF1027_HasCorrectId()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Id.Should().Be("DF1027");
        }

        [Fact]
        public void DF1027_HasInfoSeverity()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
        }

        [Fact]
        public void DF1027_HasDesignCategory()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors.Should().HaveCount(1);
            descriptors[0].Category.Should().Be("Design");
        }

        [Fact]
        public void DF1027_HasSuppressionMarker()
        {
            var descriptors = _analyzer.SupportedDiagnostics;
            descriptors[0].Description.ToString().Should().Contain("list-return-ok:");
        }
    }
}
