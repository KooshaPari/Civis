using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers;

public class PublicMutableFieldAnalyzerTests
{
    private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.PublicMutableFieldAnalyzer();

    [Fact]
    public void DF1018_HasCorrectId()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].Id.Should().Be("DF1018");
    }

    [Fact]
    public void DF1018_HasInfoSeverity()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
    }

    [Fact]
    public void DF1018_HasDesignCategory()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].Category.Should().Be("Design");
    }

    [Fact]
    public void DF1018_HasSuppressionMarker()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].Description.ToString().Should().Contain("public-field-ok:");
    }
}
