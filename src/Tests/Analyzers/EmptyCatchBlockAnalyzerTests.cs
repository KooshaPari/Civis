using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class EmptyCatchBlockAnalyzerTests
    {
        [Fact]
        public void DF1023_HasCorrectId()
        {
            Assert.Equal("DF1023", DINOForge.Analyzers.EmptyCatchBlockAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DF1023_HasWarningSeverity()
        {
            var analyzer = new DINOForge.Analyzers.EmptyCatchBlockAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1023_HasReliabilityCategory()
        {
            var analyzer = new DINOForge.Analyzers.EmptyCatchBlockAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void DF1023_HasSuppressionMarker()
        {
            var analyzer = new DINOForge.Analyzers.EmptyCatchBlockAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Description);
            // Verify the analyzer checks for safe-swallow marker
            Assert.True(descriptors[0].Description?.ToString().Contains("safe-swallow"));
        }
    }
}
