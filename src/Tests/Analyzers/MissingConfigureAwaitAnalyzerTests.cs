using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers;

public class MissingConfigureAwaitAnalyzerTests
{
    private readonly DiagnosticAnalyzer _analyzer = new DINOForge.Analyzers.MissingConfigureAwaitAnalyzer();

    [Fact]
    public void DF1019_HasCorrectId()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].Id.Should().Be("DF1019");
    }

    [Fact]
    public void DF1019_HasInfoSeverity()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
    }

    [Fact]
    public void DF1019_HasReliabilityCategory()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].Category.Should().Be("Reliability");
    }

    [Fact]
    public void DF1019_HasSuppressionMarker()
    {
        var descriptors = _analyzer.SupportedDiagnostics;
        descriptors.Should().HaveCount(1);
        descriptors[0].Description.ToString().Should().Contain("configureawait-ok:");
    }
}
