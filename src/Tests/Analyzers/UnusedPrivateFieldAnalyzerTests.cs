using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class UnusedPrivateFieldAnalyzerTests
    {
        [Fact]
        public void DF1024_HasCorrectId()
        {
            Assert.Equal("DF1024", DINOForge.Analyzers.UnusedPrivateFieldAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DF1024_HasInfoSeverity()
        {
            var analyzer = new DINOForge.Analyzers.UnusedPrivateFieldAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Info, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1024_HasMaintainabilityCategory()
        {
            var analyzer = new DINOForge.Analyzers.UnusedPrivateFieldAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal("Maintainability", descriptors[0].Category);
        }

        [Fact]
        public void DF1024_HasSuppressionMarker()
        {
            var analyzer = new DINOForge.Analyzers.UnusedPrivateFieldAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Description);
            // Verify the analyzer checks for unused-field-ok marker (tested implicitly via the Description + implementation)
            Assert.True(descriptors[0].Description?.ToString().Contains("unused-field-ok"));
        }
    }
}
