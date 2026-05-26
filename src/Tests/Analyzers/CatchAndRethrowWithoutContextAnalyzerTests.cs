using DINOForge.Analyzers;
using Xunit;

namespace DINOForge.Tests.Analyzers;

/// <summary>
/// Tests for DF1020: Catch and rethrow without context analyzer.
/// </summary>
public class CatchAndRethrowWithoutContextAnalyzerTests
{
    [Fact]
    public void DF1020_HasCorrectId()
    {
        // Arrange
        var analyzer = new CatchAndRethrowWithoutContextAnalyzer();

        // Act
        var id = CatchAndRethrowWithoutContextAnalyzer.DiagnosticId;

        // Assert
        Assert.Equal("DF1020", id);
        Assert.NotEmpty(analyzer.SupportedDiagnostics);
        Assert.Equal(id, analyzer.SupportedDiagnostics[0].Id);
    }

    [Fact]
    public void DF1020_HasWarningSeverity()
    {
        // Arrange
        var analyzer = new CatchAndRethrowWithoutContextAnalyzer();

        // Act
        var diagnostic = analyzer.SupportedDiagnostics[0];

        // Assert
        Assert.Equal(Microsoft.CodeAnalysis.DiagnosticSeverity.Warning, diagnostic.DefaultSeverity);
    }

    [Fact]
    public void DF1020_HasReliabilityCategory()
    {
        // Arrange
        var analyzer = new CatchAndRethrowWithoutContextAnalyzer();

        // Act
        var diagnostic = analyzer.SupportedDiagnostics[0];

        // Assert
        Assert.Equal("Reliability", diagnostic.Category);
    }

    [Fact]
    public void DF1020_HasSuppressionMarker()
    {
        // Arrange
        var analyzer = new CatchAndRethrowWithoutContextAnalyzer();

        // Act
        var description = analyzer.SupportedDiagnostics[0].Description.ToString();

        // Assert
        Assert.Contains("catch-rethrow-ok:", description);
    }
}
