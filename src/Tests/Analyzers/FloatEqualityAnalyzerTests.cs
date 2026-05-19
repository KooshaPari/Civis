using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class FloatEqualityAnalyzerTests
    {
        [Fact]
        public void DiagnosticId_IsCorrect()
        {
            Assert.Equal("DF1007", DINOForge.Analyzers.FloatEqualityAnalyzer.DiagnosticId);
        }

        [Fact]
        public void FloatEquality_Title_IsSet()
        {
            var analyzer = new DINOForge.Analyzers.FloatEqualityAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Title);
        }

        [Fact]
        public void FloatEquality_Category_IsReliability()
        {
            var analyzer = new DINOForge.Analyzers.FloatEqualityAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void FloatEquality_Severity_IsWarning()
        {
            var analyzer = new DINOForge.Analyzers.FloatEqualityAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, descriptors[0].DefaultSeverity);
        }
    }
}
