using DINOForge.Analyzers;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    public class UnsealedConcreteMutableClassAnalyzerTests
    {
        [Fact]
        public void DF1013_HasCorrectId()
        {
            // Arrange
            var analyzer = new UnsealedConcreteMutableClassAnalyzer();

            // Act
            var diagnosticId = UnsealedConcreteMutableClassAnalyzer.DiagnosticId;

            // Assert
            Assert.Equal("DF1013", diagnosticId);
        }

        [Fact]
        public void DF1013_HasInfoSeverity()
        {
            // Arrange
            var analyzer = new UnsealedConcreteMutableClassAnalyzer();

            // Act
            var supportedDiagnostics = analyzer.SupportedDiagnostics;

            // Assert
            Assert.Single(supportedDiagnostics);
            Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Info, supportedDiagnostics[0].DefaultSeverity);
        }

        [Fact]
        public void DF1013_HasDesignCategory()
        {
            // Arrange
            var analyzer = new UnsealedConcreteMutableClassAnalyzer();

            // Act
            var supportedDiagnostics = analyzer.SupportedDiagnostics;

            // Assert
            Assert.Single(supportedDiagnostics);
            Assert.Equal("Design", supportedDiagnostics[0].Category);
        }

        [Fact]
        public void DF1013_HasSuppressionMarker()
        {
            // Arrange
            var analyzer = new UnsealedConcreteMutableClassAnalyzer();
            var supportedDiagnostics = analyzer.SupportedDiagnostics;
            var diagnostic = supportedDiagnostics[0];

            // Act
            var description = diagnostic.Description.ToString();

            // Assert
            // Verify the docstring documents the unsealed-ok suppression marker
            Assert.Contains("unsealed-ok:", description);
        }
    }
}
