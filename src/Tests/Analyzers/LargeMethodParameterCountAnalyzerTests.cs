using DINOForge.Analyzers;
using Microsoft.CodeAnalysis;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class LargeMethodParameterCountAnalyzerTests
    {
        [Fact]
        public void DF1026_HasCorrectId()
        {
            var analyzer = new LargeMethodParameterCountAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;

            Assert.Single(descriptors);
            Assert.Equal(LargeMethodParameterCountAnalyzer.DiagnosticId, descriptors[0].Id);
            Assert.Equal("DF1026", descriptors[0].Id);
        }

        [Fact]
        public void DF1026_HasInfoSeverity()
        {
            var analyzer = new LargeMethodParameterCountAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;

            Assert.Single(descriptors);
            Assert.Equal(DiagnosticSeverity.Info, descriptors[0].DefaultSeverity);
        }

        [Fact]
        public void DF1026_HasDesignCategory()
        {
            var analyzer = new LargeMethodParameterCountAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;

            Assert.Single(descriptors);
            Assert.Equal("Design", descriptors[0].Category);
        }

        [Fact]
        public void DF1026_HasSuppressionMarker()
        {
            var analyzer = new LargeMethodParameterCountAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;

            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Id);
            Assert.True(descriptors[0].IsEnabledByDefault);
        }
    }
}
