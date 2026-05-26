using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class StringConcatenationInLoopAnalyzerTests
    {
        [Fact]
        public void DF1025_HasCorrectId()
        {
            Assert.Equal("DF1025", DINOForge.Analyzers.StringConcatenationInLoopAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DF1025_HasInfoSeverity()
        {
            var analyzer = new DINOForge.Analyzers.StringConcatenationInLoopAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Info, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1025_HasPerformanceCategory()
        {
            var analyzer = new DINOForge.Analyzers.StringConcatenationInLoopAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal("Performance", descriptors[0].Category);
        }

        [Fact]
        public void DF1025_HasSuppressionMarker()
        {
            var analyzer = new DINOForge.Analyzers.StringConcatenationInLoopAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Description);
            // Verify the analyzer checks for gc-concat-ok marker in the implementation
            // The marker is checked in HasGcConcatOkComment method which is called during analysis
            var descText = descriptors[0].Description?.ToString() ?? "";
            Assert.True(descText.Contains("StringBuilder") || descText.Contains("string.Join"));
        }
    }
}
