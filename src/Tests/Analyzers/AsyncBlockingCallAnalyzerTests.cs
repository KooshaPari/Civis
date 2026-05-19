using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class AsyncBlockingCallAnalyzerTests
    {
        [Fact]
        public void DF1011_HasCorrectId()
        {
            Assert.Equal("DF1011", DINOForge.Analyzers.AsyncBlockingCallAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DF1011_HasWarningSeverity()
        {
            var analyzer = new DINOForge.Analyzers.AsyncBlockingCallAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1011_HasReliabilityCategory()
        {
            var analyzer = new DINOForge.Analyzers.AsyncBlockingCallAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void DF1011_HasSuppressionMarker()
        {
            var analyzer = new DINOForge.Analyzers.AsyncBlockingCallAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Description);
            // Verify the analyzer checks for async-blocking-ok marker (tested implicitly via the Description + implementation)
            Assert.True(descriptors[0].Description?.ToString().Contains("async-blocking-ok"));
        }
    }
}
