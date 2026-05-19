using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class IDisposableNotImplementedAnalyzerTests
    {
        [Fact]
        public void DF1022_HasCorrectId()
        {
            Assert.Equal("DF1022", DINOForge.Analyzers.IDisposableNotImplementedAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DF1022_HasInfoSeverity()
        {
            var analyzer = new DINOForge.Analyzers.IDisposableNotImplementedAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Info, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1022_HasReliabilityCategory()
        {
            var analyzer = new DINOForge.Analyzers.IDisposableNotImplementedAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void DF1022_HasSuppressionMarker()
        {
            var analyzer = new DINOForge.Analyzers.IDisposableNotImplementedAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Description);
            // Verify the analyzer checks for idisposable-ok marker
            Assert.True(descriptors[0].Description?.ToString().Contains("idisposable-ok"));
        }
    }
}
