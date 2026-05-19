using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class EnumParseAnalyzerTests
    {
        [Fact]
        public void DiagnosticId_IsCorrect()
        {
            Assert.Equal("DF1009", DINOForge.Analyzers.EnumParseAnalyzer.DiagnosticId);
        }

        [Fact]
        public void EnumParse_Title_IsSet()
        {
            var analyzer = new DINOForge.Analyzers.EnumParseAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Title);
        }

        [Fact]
        public void EnumParse_Category_IsReliability()
        {
            var analyzer = new DINOForge.Analyzers.EnumParseAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void EnumParse_Severity_IsWarning()
        {
            var analyzer = new DINOForge.Analyzers.EnumParseAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, descriptors[0].DefaultSeverity);
        }
    }
}
