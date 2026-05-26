using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class DictionaryIndexerAnalyzerTests
    {
        [Fact]
        public void DiagnosticId_IsCorrect()
        {
            Assert.Equal("DF1008", DINOForge.Analyzers.DictionaryIndexerAnalyzer.DiagnosticId);
        }

        [Fact]
        public void DictionaryIndexer_Title_IsSet()
        {
            var analyzer = new DINOForge.Analyzers.DictionaryIndexerAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Title);
        }

        [Fact]
        public void DictionaryIndexer_Category_IsReliability()
        {
            var analyzer = new DINOForge.Analyzers.DictionaryIndexerAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void DictionaryIndexer_Severity_IsInfo()
        {
            var analyzer = new DINOForge.Analyzers.DictionaryIndexerAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Info, descriptors[0].DefaultSeverity);
        }
    }
}
