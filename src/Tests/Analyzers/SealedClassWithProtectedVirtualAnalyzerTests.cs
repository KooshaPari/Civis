using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class SealedClassWithProtectedVirtualAnalyzerTests
    {
        [Fact]
        public void DF1021_HasCorrectId()
        {
            Assert.Equal("DF1021", DINOForge.Analyzers.SealedClassWithProtectedVirtualAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DF1021_HasWarningSeverity()
        {
            var analyzer = new DINOForge.Analyzers.SealedClassWithProtectedVirtualAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1021_HasDesignCategory()
        {
            var analyzer = new DINOForge.Analyzers.SealedClassWithProtectedVirtualAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal("Design", descriptors[0].Category);
        }

        [Fact]
        public void DF1021_HasSuppressionMarker()
        {
            var analyzer = new DINOForge.Analyzers.SealedClassWithProtectedVirtualAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Description);
            // Verify the analyzer checks for sealed-virtual-ok marker
            Assert.True(descriptors[0].Description?.ToString().Contains("sealed-virtual-ok"));
        }
    }
}
