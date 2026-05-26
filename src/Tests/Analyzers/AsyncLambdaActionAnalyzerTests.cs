using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class AsyncLambdaActionAnalyzerTests
    {
        [Fact]
        public void DiagnosticId_IsCorrect()
        {
            Assert.Equal("DF1010", DINOForge.Analyzers.AsyncLambdaActionAnalyzer.DiagnosticId);
        }

        [Fact]
        public void AsyncLambdaAction_Title_IsSet()
        {
            var analyzer = new DINOForge.Analyzers.AsyncLambdaActionAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Single(descriptors);
            Assert.NotNull(descriptors[0].Title);
        }

        [Fact]
        public void AsyncLambdaAction_Category_IsReliability()
        {
            var analyzer = new DINOForge.Analyzers.AsyncLambdaActionAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal("Reliability", descriptors[0].Category);
        }

        [Fact]
        public void AsyncLambdaAction_Severity_IsWarning()
        {
            var analyzer = new DINOForge.Analyzers.AsyncLambdaActionAnalyzer();
            var descriptors = analyzer.SupportedDiagnostics;
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, descriptors[0].DefaultSeverity);
        }
    }
}
