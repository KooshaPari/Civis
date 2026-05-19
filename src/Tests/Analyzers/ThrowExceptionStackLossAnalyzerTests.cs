using DINOForge.Analyzers;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class ThrowExceptionStackLossAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new ThrowExceptionStackLossAnalyzer();

        [Fact]
        public void DF1012_HasCorrectId()
        {
            var analyzer = GetAnalyzer();
            var diagnostics = analyzer.SupportedDiagnostics;

            Assert.NotEmpty(diagnostics);
            Assert.Equal(ThrowExceptionStackLossAnalyzer.DiagnosticId, diagnostics[0].Id);
            Assert.Equal("DF1012", diagnostics[0].Id);
        }

        [Fact]
        public void DF1012_HasWarningSeverity()
        {
            var analyzer = GetAnalyzer();
            var diagnostics = analyzer.SupportedDiagnostics;

            Assert.NotEmpty(diagnostics);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, diagnostics[0].DefaultSeverity);
        }

        [Fact]
        public void DF1012_HasReliabilityCategory()
        {
            var analyzer = GetAnalyzer();
            var diagnostics = analyzer.SupportedDiagnostics;

            Assert.NotEmpty(diagnostics);
            Assert.Equal("Reliability", diagnostics[0].Category);
        }

        [Fact]
        public void DF1012_HasSuppressionMarker()
        {
            var analyzer = GetAnalyzer();
            var diagnostics = analyzer.SupportedDiagnostics;

            Assert.NotEmpty(diagnostics);
            var description = diagnostics[0].Description.ToString();
            Assert.Contains("rethrow-as-new-ok", description);
        }
    }
}
